<?php
###################################################################
# Stackvo UI - Control API
# Unified endpoint for service and system-wide container control
###################################################################

require_once __DIR__ . '/../lib/api-base.php';

class ControlApi extends ApiBase
{
    /**
     * Handle control requests
     */
    public function handle()
    {
        // Only accept POST requests
        $this->validateMethod('POST');

        // Get JSON input
        $input = $this->getJsonInput();

        // Determine scope: 'service' or 'system'
        $scope = $input['scope'] ?? 'service';

        if ($scope === 'system') {
            $this->handleSystemControl($input);
        } else {
            $this->handleServiceControl($input);
        }
    }

    /**
     * Handle individual service/container control
     */
    private function handleServiceControl($input)
    {
        // Validate required fields
        $this->validateRequiredFields($input, ['service', 'action']);

        $service = $input['service'];
        $action = $input['action'];

        // Validate action
        if (!in_array($action, ['start', 'stop', 'restart', 'build'])) {
            jsonError('Invalid action. Must be start, stop, restart, or build', 400);
            exit;
        }

        // Sanitize service name (only allow alphanumeric and dash)
        if (!preg_match('/^[a-z0-9-]+$/', $service)) {
            jsonError('Invalid service name', 400);
            exit;
        }

        // Build container name
        // Check if service name already has 'stackvo-' prefix
        if (strpos($service, 'stackvo-') === 0) {
            // Already has prefix (e.g., stackvo-project1, stackvo-mysql)
            $containerName = $service;
        } else {
            // Add prefix (e.g., mysql -> stackvo-mysql, project1 -> stackvo-project1)
            $containerName = 'stackvo-' . $service;
        }

        // Execute docker command
        // Build action uses docker compose, others use docker directly
        if ($action === 'build') {
            // Build action: docker compose up -d --build {projectName}
            $composeFile = '/app/generated/docker-compose.projects.yml';
            $projectName = str_replace('stackvo-', '', $containerName);

            // Wait for Dockerfile to be available (volume mount synchronization)
            $dockerfilePath = '/app/generated/projects/' . $projectName . '/Dockerfile';
            $maxWait = 3000000; // 3 seconds max
            $waited = 0;
            $interval = 200000; // 200ms

            while (!file_exists($dockerfilePath) && $waited < $maxWait) {
                clearstatcache(true, $dockerfilePath);
                usleep($interval);
                $waited += $interval;
            }

            if (!file_exists($dockerfilePath)) {
                Logger::warn('Dockerfile not found after waiting', [
                    'project' => $projectName,
                    'path' => $dockerfilePath,
                    'waited_ms' => $waited / 1000
                ]);

                jsonError('Dockerfile not ready yet. Please try again in a few seconds.', 500);
                exit;
            }

            $command = sprintf(
                'docker compose -p stackvo -f %s up -d --build %s 2>&1',
                escapeshellarg($composeFile),
                escapeshellarg($projectName)
            );

            $output = [];
            $returnCode = 0;
            exec($command, $output, $returnCode);

            // Log Docker command execution
            Logger::logDockerCommand($command, $returnCode, $output);

            if ($returnCode === 0) {
                Logger::info('Container built and started', [
                    'project' => $projectName,
                    'container' => $containerName
                ]);

                $this->sendSuccess([
                    'message' => "Container {$containerName} built and started successfully",
                    'service' => $service,
                    'action' => $action,
                    'output' => implode("\n", $output)
                ]);
            } else {
                Logger::error('Container build failed', [
                    'project' => $projectName,
                    'container' => $containerName,
                    'return_code' => $returnCode,
                    'output' => implode("\n", $output)
                ]);

                jsonError('Failed to build container: ' . implode("\n", $output), 500);
            }
        } else {
            // Normal actions: start, stop, restart
            $command = $this->buildDockerCommand($action, [$containerName]);

            $output = [];
            $returnCode = 0;
            exec($command, $output, $returnCode);

            // Log Docker command execution
            Logger::logDockerCommand($command, $returnCode, $output);

            if ($returnCode === 0) {
                Logger::info("Service {$action} successful", ['service' => $service]);

                $this->sendSuccess([
                    'message' => ucfirst($action) . ' command executed successfully',
                    'service' => $service,
                    'action' => $action
                ]);
            } else {
                Logger::error("Service {$action} failed", [
                    'service' => $service,
                    'action' => $action,
                    'error' => implode("\n", $output)
                ]);

                jsonError('Failed to execute command: ' . implode("\n", $output), 500);
            }
        }
    }

    /**
     * Handle system-wide control (all containers)
     */
    private function handleSystemControl($input)
    {
        // Increase execution time for stopping many containers
        set_time_limit(120);

        // Validate required fields
        $this->validateRequiredFields($input, ['action']);

        $action = $input['action'];

        // Validate action (using 'command' terminology for system-wide)
        $validActions = ['up', 'down', 'restart', 'start', 'stop'];
        if (!in_array($action, $validActions)) {
            jsonError('Invalid action. Must be up, down, restart, start, or stop', 400);
            exit;
        }

        // Normalize action names
        $actionMap = [
            'up' => 'start',
            'down' => 'stop',
            'restart' => 'restart',
            'start' => 'start',
            'stop' => 'stop'
        ];
        $dockerAction = $actionMap[$action];

        // Get all stackvo and project containers
        $output = [];
        $returnCode = 0;

        // Get list of all stackvo containers
        exec('docker ps -a --filter "name=stackvo-" --format "{{.Names}}"', $stackvoContainers, $returnCode);

        if ($returnCode !== 0) {
            jsonError('Failed to get container list', 500);
            exit;
        }

        // Get list of all project containers (project*-web and project*-php)
        exec('docker ps -a --filter "name=project" --format "{{.Names}}"', $projectContainers, $returnCode);

        // Combine both lists
        $containerList = array_merge($stackvoContainers, $projectContainers ?? []);

        if (empty($containerList)) {
            $this->sendSuccess([
                'message' => 'No stackvo containers found',
                'action' => $action,
                'affected_containers' => 0,
                'containers' => []
            ]);
            return;
        }

        // Exclude UI and Traefik from down and restart commands to keep UI accessible
        if (in_array($action, ['down', 'stop', 'restart'])) {
            $containerList = array_filter($containerList, function ($container) {
                return !in_array($container, ['stackvo-ui', 'stackvo-traefik']);
            });

            if (empty($containerList)) {
                $this->sendSuccess([
                    'message' => 'All containers ' . ($action === 'down' || $action === 'stop' ? 'stopped' : 'restarted') . ' (UI and Traefik kept running)',
                    'action' => $action,
                    'affected_containers' => 0,
                    'containers' => [],
                    'note' => 'stackvo-ui and stackvo-traefik are kept running to maintain UI access'
                ]);
                return;
            }
        }

        // Build and execute docker command
        $command = $this->buildDockerCommand($dockerAction, $containerList);

        $output = [];
        $returnCode = 0;
        exec($command, $output, $returnCode);

        // Log Docker command execution
        Logger::logDockerCommand($command, $returnCode, $output);

        if ($returnCode === 0) {
            Logger::info("System {$action} successful", [
                'affected_containers' => count($containerList)
            ]);

            $this->sendSuccess([
                'message' => ucfirst($action) . ' command executed successfully on all containers',
                'action' => $action,
                'affected_containers' => count($containerList),
                'containers' => $containerList
            ]);
        } else {
            Logger::error("System {$action} failed", [
                'action' => $action,
                'error' => implode("\n", $output)
            ]);

            jsonError('Failed to execute ' . $action . ' command: ' . implode("\n", $output), 500);
        }
    }

    /**
     * Build docker command for action
     * @param string $action Docker action (start, stop, restart)
     * @param array $containers List of container names
     * @return string Docker command
     */
    private function buildDockerCommand($action, $containers)
    {
        $containerNames = implode(' ', array_map('escapeshellarg', $containers));
        return "docker {$action} {$containerNames} 2>&1";
    }
}

// Run the API
$api = new ControlApi('/api/control.php');
$api->run();
