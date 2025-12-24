<?php
###################################################################
# Stackvo UI - Services API
# Returns services from core/templates/services with .env status
###################################################################

require_once __DIR__ . '/../lib/api-base.php';
require_once __DIR__ . '/../lib/env.php';
require_once __DIR__ . '/../lib/docker.php';
require_once __DIR__ . '/../lib/network.php';
require_once __DIR__ . '/../lib/utils.php';

class ServicesApi extends ApiBase
{
    /**
     * Get service logs with size information
     */
    private function getServiceLogs($serviceName, $baseDir)
    {
        $hostLogPath = $baseDir . '/' . Config::get('logs_dir') . '/' . $serviceName;

        // Convention-based log paths with exceptions for non-standard services
        // Most services follow /var/log/{service} pattern
        $logPathExceptions = [
            'activemq' => '/opt/apache-activemq/data',
            'tomcat' => '/usr/local/tomcat/logs',
            'postgres' => '/var/lib/postgresql/data/log',
            'postgresql' => '/var/lib/postgresql/data/log',
        ];

        $containerBasePath = $logPathExceptions[strtolower($serviceName)] ?? '/var/log/' . $serviceName;

        // Check if log directory exists
        if (!is_dir($hostLogPath)) {
            return null;
        }

        // Common log file patterns
        $possibleLogFiles = array_merge(
            [$serviceName . '.log'],
            ['error.log', 'access.log', 'main.log', 'slow.log']
        );

        $foundLogFile = null;
        $foundFileName = null;
        foreach ($possibleLogFiles as $fileName) {
            $file = $hostLogPath . '/' . $fileName;
            if (file_exists($file)) {
                $foundLogFile = $file;
                $foundFileName = $fileName;
                break;
            }
        }

        if (!$foundLogFile) {
            // Return directory paths if no specific log file found
            return [
                'container_path' => $containerBasePath,
                'host_path' => 'logs/' . $serviceName,
                'size' => null
            ];
        }

        // Get file size
        $size = filesize($foundLogFile);
        $sizeFormatted = formatBytes($size);

        // Build paths
        $containerPath = $containerBasePath . '/' . $foundFileName;
        $hostPath = 'logs/' . $serviceName . '/' . $foundFileName;

        return [
            'container_path' => $containerPath,
            'host_path' => $hostPath,
            'size' => $sizeFormatted
        ];
    }

    /**
     * Handle API request
     */
    public function handle()
    {
        $baseDir = Config::get('base_dir');
        $servicesDir = $baseDir . '/' . Config::get('services_dir');

        // Scan services directory
        $services = [];

        if (is_dir($servicesDir)) {
            $servicesList = scandir($servicesDir);

            foreach ($servicesList as $service) {
                if ($service === '.' || $service === '..') {
                    continue;
                }

                $servicePath = $servicesDir . '/' . $service;
                if (!is_dir($servicePath)) {
                    continue;
                }

                $serviceName = $service;
                $serviceUpper = strtoupper($serviceName);

                // Get service configuration from .env
                $enabled = getEnvValue('SERVICE_' . $serviceUpper . '_ENABLE', 'false') === 'true';
                $version = getEnvValue('SERVICE_' . $serviceUpper . '_VERSION', '');
                $url = getEnvValue('SERVICE_' . $serviceUpper . '_URL', '');
                $port = getEnvValue('HOST_PORT_' . $serviceUpper, '');

                // Check if container is running
                $containerName = Config::get('container_prefix') . $serviceName;
                $running = isContainerRunning($containerName);

                // Build URL if exists
                $fullUrl = '';
                $domain = '';
                $dnsConfigured = false;
                if (!empty($url)) {
                    $tldSuffix = getEnvValue('DEFAULT_TLD_SUFFIX', 'stackvo.loc');
                    $sslEnable = getEnvValue('SSL_ENABLE', 'true') === 'true';
                    $protocol = $sslEnable ? 'https' : 'http';
                    $domain = $url . '.' . $tldSuffix;
                    $fullUrl = $protocol . '://' . $domain;
                    $dnsConfigured = isDomainConfigured($domain);
                }

                // Get port mappings if container is running
                $ports = [];
                if ($running) {
                    $ports = getContainerPorts($containerName);
                }

                // Get logs
                $logs = $this->getServiceLogs($serviceName, $baseDir);

                // Get credentials from .env
                $credentials = [];

                // Common credential patterns
                $credentialKeys = [
                    'USER',
                    'USERNAME',
                    'ADMIN_USER',
                    'DEFAULT_USER',
                    'INITDB_ROOT_USERNAME',
                    'PASSWORD',
                    'PASS',
                    'ADMIN_PASSWORD',
                    'DEFAULT_PASS',
                    'ROOT_PASSWORD',
                    'INITDB_ROOT_PASSWORD',
                    'DB_PASSWORD',
                    'DATABASE',
                    'DB',
                    'DB_NAME',
                    'INITDB_DATABASE',
                    'PORT',
                    'HOST_PORT_OPENWIRE',
                    'HOST_PORT_AMQP',
                    'HOST_PORT_STOMP',
                    'HOST_PORT_MQTT',
                    'HOST_PORT_WS',
                    'HOST_PORT_UI',
                    'REDIS_PORT',
                    'POSTGRES_PORT'
                ];

                foreach ($credentialKeys as $key) {
                    $envKey = 'SERVICE_' . $serviceUpper . '_' . $key;
                    $value = getEnvValue($envKey, null);

                    if ($value !== null && $value !== '') {
                        // Format the key for display (remove SERVICE_ prefix and service name)
                        $displayKey = str_replace('SERVICE_' . $serviceUpper . '_', '', $envKey);
                        $credentials[$displayKey] = $value;
                    }
                }

                $services[] = [
                    'name' => $serviceName,
                    'enabled' => $enabled,
                    'running' => $running,
                    'version' => $version,
                    'port' => $port,
                    'url' => $fullUrl,
                    'domain' => $domain,
                    'dns_configured' => $dnsConfigured,
                    'ports' => $ports,
                    'logs' => $logs,
                    'credentials' => $credentials,
                ];
            }
        }

        // Sort services alphabetically
        usort($services, function ($a, $b) {
            return strcmp($a['name'], $b['name']);
        });

        // Send success response
        $this->sendSuccess(
            ['services' => $services],
            'Services loaded',
            ['count' => count($services)]
        );
    }
}

// Run the API
$api = new ServicesApi('/api/services.php');
$api->run();