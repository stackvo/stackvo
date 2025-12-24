<?php
/**
 * Docker Stats API Endpoint
 * Returns real-time Docker container statistics for monitoring
 */

header('Content-Type: application/json');

// Function to execute shell command and return output
function executeCommand($command)
{
    $output = [];
    $returnVar = 0;
    exec($command . ' 2>&1', $output, $returnVar);
    return [
        'success' => $returnVar === 0,
        'output' => implode("\n", $output),
        'lines' => $output
    ];
}

// Function to parse Docker stats output
function getDockerStats()
{
    // Get stats for all containers with JSON format
    $result = executeCommand('docker stats --no-stream --format "{{json .}}"');

    if (!$result['success']) {
        return [
            'success' => false,
            'error' => 'Failed to get Docker stats',
            'details' => $result['output']
        ];
    }

    $containers = [];
    $totalCpu = 0;
    $totalMemUsedMB = 0; // Store in MB for better precision
    $totalNetInputBytes = 0;
    $totalNetOutputBytes = 0;
    $containerCount = 0;

    // Get system memory info
    $systemMemResult = executeCommand('free -m | grep Mem');
    $systemMemTotal = 16000; // Default 16GB
    if ($systemMemResult['success'] && !empty($systemMemResult['lines'])) {
        $memLine = preg_split('/\s+/', trim($systemMemResult['lines'][0]));
        if (isset($memLine[1])) {
            $systemMemTotal = floatval($memLine[1]); // Total memory in MB
        }
    }

    foreach ($result['lines'] as $line) {
        if (empty($line))
            continue;

        $stat = json_decode($line, true);
        if (!$stat)
            continue;

        // Sadece stackvo- ile başlayan containerları işle
        if (
            strpos($stat['Name'], 'stackvo-') === false &&
            strpos($stat['Name'], 'stackvo_') === false
        ) {
            continue;
        }

        // Bu containerı say
        $containerCount++;

        // Parse CPU percentage
        $cpuPercent = floatval(str_replace('%', '', $stat['CPUPerc']));

        // Parse memory (format: "123.4MiB / 1.5GiB")
        $memParts = explode(' / ', $stat['MemUsage']);
        $memUsedBytes = parseMemoryValue($memParts[0] ?? '0');
        $memUsedMB = $memUsedBytes / 1024 / 1024; // Convert to MB

        // Parse network IO (format: "1.23MB / 456kB")
        $netIO = $stat['NetIO'] ?? '0B / 0B';
        $netParts = explode(' / ', $netIO);
        $netInputBytes = parseMemoryValue($netParts[0] ?? '0B');
        $netOutputBytes = parseMemoryValue($netParts[1] ?? '0B');

        $containers[] = [
            'name' => $stat['Name'],
            'cpu' => $cpuPercent,
            'memory_used_mb' => round($memUsedMB, 2),
            'memory_percent' => floatval(str_replace('%', '', $stat['MemPerc'])),
            'net_input_bytes' => $netInputBytes,
            'net_output_bytes' => $netOutputBytes
        ];

        $totalCpu += $cpuPercent;
        $totalMemUsedMB += $memUsedMB;
        $totalNetInputBytes += $netInputBytes;
        $totalNetOutputBytes += $netOutputBytes;
    }

    // Get disk usage
    $diskStats = getDiskStats();

    return [
        'success' => true,
        'timestamp' => time(),
        'containers' => $containers,
        'aggregate' => [
            'cpu_total' => round($totalCpu, 2),
            'cpu_average' => $containerCount > 0 ? round($totalCpu / $containerCount, 2) : 0,
            'memory_used_mb' => round($totalMemUsedMB, 2),
            'memory_used_gb' => round($totalMemUsedMB / 1024, 2),
            'memory_total_mb' => $systemMemTotal,
            'memory_total_gb' => round($systemMemTotal / 1024, 2),
            'memory_percent' => $systemMemTotal > 0 ? round(($totalMemUsedMB / $systemMemTotal) * 100, 2) : 0,
            'container_count' => $containerCount,
            'network_input_mb' => round($totalNetInputBytes / 1024 / 1024, 2),
            'network_output_mb' => round($totalNetOutputBytes / 1024 / 1024, 2),
            'network_input_gb' => round($totalNetInputBytes / 1024 / 1024 / 1024, 3),
            'network_output_gb' => round($totalNetOutputBytes / 1024 / 1024 / 1024, 3)
        ],
        'disk' => $diskStats
    ];
}

// Function to parse memory values (handles MiB, GiB, etc.)
function parseMemoryValue($value)
{
    $value = trim($value);

    // Check units from longest to shortest to avoid partial matches
    // e.g., "MiB" contains "B", so check "MiB" before "B"
    $units = [
        'TiB' => 1024 * 1024 * 1024 * 1024,
        'GiB' => 1024 * 1024 * 1024,
        'MiB' => 1024 * 1024,
        'KiB' => 1024,
        'TB' => 1000 * 1000 * 1000 * 1000,
        'GB' => 1000 * 1000 * 1000,
        'MB' => 1000 * 1000,
        'KB' => 1000,
        'B' => 1
    ];

    foreach ($units as $unit => $multiplier) {
        if (strpos($value, $unit) !== false) {
            return floatval($value) * $multiplier;
        }
    }

    return floatval($value);
}

// Function to get disk usage statistics
function getDiskStats()
{
    // Get Docker system disk usage
    $result = executeCommand('docker system df --format "{{json .}}"');

    if (!$result['success'] || empty($result['lines'])) {
        return [
            'total_gb' => 0,
            'used_gb' => 0,
            'available_gb' => 0,
            'percent' => 0
        ];
    }

    $totalBytes = 0;
    $reclaimableBytes = 0;

    // Parse each line of docker system df output
    foreach ($result['lines'] as $line) {
        if (empty($line))
            continue;

        $data = json_decode($line, true);
        if (!$data)
            continue;

        // Parse Size (e.g., "1.5GB", "500MB")
        if (isset($data['Size'])) {
            $totalBytes += parseDockerSize($data['Size']);
        }

        // Parse Reclaimable (e.g., "500MB (33%)")
        if (isset($data['Reclaimable'])) {
            $reclaimableStr = preg_replace('/\s*\(.*\)/', '', $data['Reclaimable']); // Remove percentage
            $reclaimableBytes += parseDockerSize($reclaimableStr);
        }
    }

    // Get system total disk for reference
    $systemDiskResult = executeCommand('df -BG / | tail -1');
    $systemTotalGB = 100; // Default fallback
    if ($systemDiskResult['success'] && !empty($systemDiskResult['lines'])) {
        $parts = preg_split('/\s+/', trim($systemDiskResult['lines'][0]));
        if (isset($parts[1])) {
            $systemTotalGB = floatval(str_replace('G', '', $parts[1]));
        }
    }

    // Convert to GB
    $totalGB = round($totalBytes / 1024 / 1024 / 1024, 2);
    $reclaimableGB = round($reclaimableBytes / 1024 / 1024 / 1024, 2);
    $usedGB = round($totalGB - $reclaimableGB, 2);
    $percent = $systemTotalGB > 0 ? round(($totalGB / $systemTotalGB) * 100, 2) : 0;

    return [
        'total_gb' => $systemTotalGB,
        'used_gb' => $totalGB,
        'available_gb' => round($systemTotalGB - $totalGB, 2),
        'percent' => $percent,
        'docker_used_gb' => $totalGB,
        'docker_reclaimable_gb' => $reclaimableGB
    ];
}

// Function to parse Docker size strings (e.g., "1.5GB", "500MB", "1.2kB")
function parseDockerSize($sizeStr)
{
    $sizeStr = trim($sizeStr);

    // Handle "0B" or empty
    if (empty($sizeStr) || $sizeStr === '0B') {
        return 0;
    }

    $units = [
        'TB' => 1024 * 1024 * 1024 * 1024,
        'GB' => 1024 * 1024 * 1024,
        'MB' => 1024 * 1024,
        'kB' => 1024,
        'B' => 1
    ];

    foreach ($units as $unit => $multiplier) {
        if (stripos($sizeStr, $unit) !== false) {
            return floatval($sizeStr) * $multiplier;
        }
    }

    return floatval($sizeStr);
}

// Main execution
try {
    $stats = getDockerStats();
    echo json_encode($stats, JSON_PRETTY_PRINT);
} catch (Exception $e) {
    echo json_encode([
        'success' => false,
        'error' => $e->getMessage()
    ], JSON_PRETTY_PRINT);
}
