<?php
/**
 * Docker container management utilities
 */

// Load cache for performance
require_once __DIR__ . '/cache.php';

/**
 * Check if a Docker container is running (with caching)
 * 
 * @param string $containerName Container name
 * @return bool True if running, false otherwise
 */
function isContainerRunning($containerName) {
    return Cache::remember(
        "container_running_{$containerName}",
        function() use ($containerName) {
            $output = [];
            $returnCode = 0;
            exec(
                sprintf("docker inspect -f '{{.State.Running}}' %s 2>/dev/null", escapeshellarg($containerName)),
                $output,
                $returnCode
            );
            return $returnCode === 0 && isset($output[0]) && trim($output[0]) === 'true';
        },
        5 // 5 second cache
    );
}

/**
 * Get container port mappings and network information (with caching)
 * 
 * @param string $containerName Container name
 * @return array Port and network information
 */
function getContainerPorts($containerName) {
    return Cache::remember(
        "container_ports_{$containerName}",
        function() use ($containerName) {
            return [
                'ports' => getPortMappings($containerName),
                'ip_address' => getContainerIP($containerName),
                'network' => getContainerNetwork($containerName),
                'gateway' => getContainerGateway($containerName),
            ];
        },
        10 // 10 second cache
    );
}

/**
 * Get port mappings for a container
 * 
 * @param string $containerName Container name
 * @return array Port mappings
 */
function getPortMappings($containerName) {
    $ports = [];
    $output = [];
    $returnCode = 0;
    
    exec(
        sprintf('docker inspect -f \'{{json .NetworkSettings.Ports}}\' %s 2>/dev/null', escapeshellarg($containerName)),
        $output,
        $returnCode
    );
    
    if ($returnCode === 0 && !empty($output[0])) {
        $portData = json_decode($output[0], true);
        if ($portData) {
            foreach ($portData as $dockerPort => $hostBindings) {
                if ($hostBindings === null) {
                    $ports[$dockerPort] = [
                        'docker_port' => rtrim($dockerPort, '/tcp'),
                        'host_ip' => null,
                        'host_port' => null,
                        'exposed' => false
                    ];
                } else {
                    $binding = $hostBindings[0];
                    $ports[$dockerPort] = [
                        'docker_port' => rtrim($dockerPort, '/tcp'),
                        'host_ip' => $binding['HostIp'] === '0.0.0.0' ? '0.0.0.0' : $binding['HostIp'],
                        'host_port' => $binding['HostPort'],
                        'exposed' => true
                    ];
                }
            }
        }
    }
    
    return $ports;
}

/**
 * Get container IP address
 * 
 * @param string $containerName Container name
 * @return string|null IP address or null
 */
function getContainerIP($containerName) {
    $networks = getContainerNetworks($containerName);
    if (!empty($networks)) {
        $networkData = reset($networks);
        return $networkData['IPAddress'] ?? null;
    }
    return null;
}

/**
 * Get container network name
 * 
 * @param string $containerName Container name
 * @return string|null Network name or null
 */
function getContainerNetwork($containerName) {
    $networks = getContainerNetworks($containerName);
    if (!empty($networks)) {
        return key($networks);
    }
    return null;
}

/**
 * Get container gateway
 * 
 * @param string $containerName Container name
 * @return string|null Gateway IP or null
 */
function getContainerGateway($containerName) {
    $networks = getContainerNetworks($containerName);
    if (!empty($networks)) {
        $networkData = reset($networks);
        return $networkData['Gateway'] ?? null;
    }
    return null;
}

/**
 * Get all networks for a container (internal helper with caching)
 * 
 * @param string $containerName Container name
 * @return array Networks data
 */
function getContainerNetworks($containerName) {
    static $cache = [];
    
    if (isset($cache[$containerName])) {
        return $cache[$containerName];
    }
    
    $output = [];
    $returnCode = 0;
    exec(
        sprintf('docker inspect -f \'{{json .NetworkSettings.Networks}}\' %s 2>/dev/null', escapeshellarg($containerName)),
        $output,
        $returnCode
    );
    
    if ($returnCode === 0 && !empty($output[0])) {
        $networks = json_decode($output[0], true);
        if ($networks && is_array($networks)) {
            $cache[$containerName] = $networks;
            return $networks;
        }
    }
    
    $cache[$containerName] = [];
    return [];
}
