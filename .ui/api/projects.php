<?php
###################################################################
# Stackvo UI - Projects API
# Returns projects from projects directory with stackvo.json
###################################################################

// Load shared libraries
require_once __DIR__ . '/../lib/config.php';
require_once __DIR__ . '/../lib/env.php';
require_once __DIR__ . '/../lib/docker.php';
require_once __DIR__ . '/../lib/network.php';
require_once __DIR__ . '/../lib/response.php';
require_once __DIR__ . '/../lib/utils.php';
require_once __DIR__ . '/../lib/logger.php';

// Load configuration
Config::load('app');

setCorsHeaders();

// Start request tracking
$startTime = microtime(true);
Logger::logRequest('/projects.php', 'GET');

$baseDir = Config::get('base_dir');
$projectsDir = $baseDir . '/' . Config::get('projects_dir');

// Function to get project logs
function getProjectLogs($projectName, $webserver, $baseDir)
{
    $logsDir = $baseDir . '/' . Config::get('logs_dir') . '/projects/' . $projectName;

    // Convention-based webserver log paths
    $webserverPaths = [
        'nginx' => '/var/log/nginx',
        'apache' => '/var/log/apache2',
        'caddy' => '/var/log/caddy',
        'ferron' => '/var/log/ferron',
    ];

    $webLogBase = $webserverPaths[$webserver] ?? '/var/log/nginx';

    // PHP container logs - standard path
    $phpLogBase = '/var/log/' . $projectName;

    // Check if logs directory exists
    if (!is_dir($logsDir)) {
        return null;
    }

    $logs = [];

    // Check for web access log (nginx/apache)
    $webAccessLog = $logsDir . '/access.log';
    if (file_exists($webAccessLog)) {
        $logs['web_access'] = [
            'container_path' => $webLogBase . '/access.log',
            'host_path' => 'logs/projects/' . $projectName . '/access.log'
        ];
    }

    // Check for web error log
    $webErrorLog = $logsDir . '/error.log';
    if (file_exists($webErrorLog)) {
        $logs['web_error'] = [
            'container_path' => $webLogBase . '/error.log',
            'host_path' => 'logs/projects/' . $projectName . '/error.log'
        ];
    }

    // Check for PHP error log
    $phpErrorLog = $logsDir . '/php-error.log';
    if (file_exists($phpErrorLog)) {
        $logs['php_error'] = [
            'container_path' => $phpLogBase . '/php-error.log',
            'host_path' => 'logs/projects/' . $projectName . '/php-error.log'
        ];
    }

    return !empty($logs) ? $logs : null;
}

// Function to check project configuration
function getProjectConfiguration($projectPath, $webserver)
{
    $stackvoDir = $projectPath . '/.stackvo';

    // Check if .stackvo directory exists
    if (!is_dir($stackvoDir)) {
        return [
            'type' => 'default',
            'has_custom' => false,
            'files' => []
        ];
    }

    $configFiles = [];

    // Check for common config files based on webserver type
    $possibleConfigs = [
        'nginx' => ['nginx.conf', 'default.conf'],
        'apache' => ['apache.conf', 'httpd.conf'],
        'caddy' => ['Caddyfile'],
        'ferron' => ['ferron.yaml', 'ferron.conf']
    ];

    // Check webserver-specific configs
    if (isset($possibleConfigs[$webserver])) {
        foreach ($possibleConfigs[$webserver] as $configFile) {
            if (file_exists($stackvoDir . '/' . $configFile)) {
                $configFiles[] = $configFile;
            }
        }
    }

    // Check for PHP configs
    if (file_exists($stackvoDir . '/php.ini')) {
        $configFiles[] = 'php.ini';
    }
    if (file_exists($stackvoDir . '/php-fpm.conf')) {
        $configFiles[] = 'php-fpm.conf';
    }

    return [
        'type' => !empty($configFiles) ? 'custom' : 'default',
        'has_custom' => !empty($configFiles),
        'files' => $configFiles
    ];
}

$projects = [];

try {
    // Check if projects directory exists
    if (!is_dir($projectsDir)) {
        echo json_encode([
            'success' => false,
            'message' => 'Projects directory not found',
            'projects' => []
        ]);
        exit;
    }

    // Scan projects directory
    $directories = array_diff(scandir($projectsDir), ['.', '..']);

    foreach ($directories as $dir) {
        $projectPath = $projectsDir . '/' . $dir;

        // Skip if not a directory
        if (!is_dir($projectPath)) {
            continue;
        }

        $configFile = $projectPath . '/stackvo.json';

        // Check if stackvo.json exists
        if (!file_exists($configFile)) {
            // Add project with minimal info if config is missing
            $projects[] = [
                'name' => $dir,
                'domain' => null,
                'php' => null,
                'webserver' => null,
                'document_root' => null,
                'error' => 'Configuration file not found'
            ];
            continue;
        }

        // Read and parse stackvo.json
        $configContent = file_get_contents($configFile);
        $config = json_decode($configContent, true);

        if (json_last_error() !== JSON_ERROR_NONE) {
            // Add project with error info if JSON is invalid
            $projects[] = [
                'name' => $dir,
                'domain' => null,
                'php' => null,
                'webserver' => null,
                'document_root' => null,
                'error' => 'Invalid JSON: ' . json_last_error_msg()
            ];
            continue;
        }

        // Check container status for this project (single container)
        $projectName = $config['name'] ?? $dir;
        $containerPrefix = Config::get('container_prefix'); // stackvo-
        $containerName = $containerPrefix . $projectName; // stackvo-project1

        // Check if container is running
        $containerRunning = false;

        $containerOutput = [];
        $containerReturnCode = 0;
        exec(sprintf('docker inspect -f "{{.State.Running}}" %s 2>/dev/null', escapeshellarg($containerName)), $containerOutput, $containerReturnCode);
        if ($containerReturnCode === 0 && isset($containerOutput[0]) && $containerOutput[0] === 'true') {
            $containerRunning = true;
        }

        // Project is running if container is running
        $running = $containerRunning;

        // Check if domain is configured in DNS/hosts
        $domain = $config['domain'] ?? null;
        $dnsConfigured = isDomainConfigured($domain);

        // Get SSL status from environment
        $sslEnabled = getEnvValue('SSL_ENABLE', 'true') === 'true';

        // Build URLs
        $urls = [
            'https' => $domain ? 'https://' . $domain : null,
            'http' => $domain ? 'http://' . $domain : null,
            'primary' => $domain ? ($sslEnabled ? 'https://' . $domain : 'http://' . $domain) : null
        ];

        // Get port mappings for container
        $containerPorts = [];
        if ($containerRunning) {
            $containerPorts = getContainerPorts($containerName);
        }

        // Get project logs
        $webserver = $config['webserver'] ?? 'nginx';
        $logs = getProjectLogs($projectName, $webserver, $baseDir);

        // Get project configuration info
        $configuration = getProjectConfiguration($projectPath, $webserver);

        // Use container port information
        $ports = $containerRunning ? $containerPorts : [];

        // Build project path info
        $projectPathInfo = [
            'container_path' => '/var/www/html',
            'host_path' => str_replace($baseDir . '/', '', $projectPath)
        ];

        // Add project with full configuration
        // Support multiple runtime languages: php, nodejs, python, ruby, golang
        $projects[] = [
            'name' => $projectName,
            'domain' => $domain,
            'dns_configured' => $dnsConfigured,
            'ssl_enabled' => $sslEnabled,
            'urls' => $urls,
            'php' => $config['php'] ?? null,
            'nodejs' => $config['nodejs'] ?? null,
            'python' => $config['python'] ?? null,
            'ruby' => $config['ruby'] ?? null,
            'golang' => $config['golang'] ?? null,
            'webserver' => $config['webserver'] ?? null,
            'document_root' => $config['document_root'] ?? null,
            'running' => $running,
            'ports' => $ports,
            'logs' => $logs,
            'configuration' => $configuration,
            'project_path' => $projectPathInfo,
            'containers' => [
                'main' => array_merge([
                    'name' => $containerName,  // stackvo-project1
                    'running' => $containerRunning
                ], $containerPorts)
            ],
            'error' => null
        ];
    }

    // Sort projects by name
    usort($projects, function ($a, $b) {
        return strcmp($a['name'], $b['name']);
    });

    // Log response
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/projects.php', 200, $duration);
    Logger::debug('Projects loaded', ['count' => count($projects)]);

    jsonSuccess([
        'projects' => $projects,
        'count' => count($projects)
    ]);

} catch (Exception $e) {
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/projects.php', 500, $duration);
    Logger::error('Projects API error', [
        'message' => $e->getMessage(),
        'file' => $e->getFile(),
        'line' => $e->getLine()
    ]);

    jsonError('Error: ' . $e->getMessage());
}
