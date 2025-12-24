<?php
###################################################################
# Stackvo UI - Terminal API
# Opens a terminal window and connects to a Docker container via bash
###################################################################

require_once __DIR__ . '/../lib/api-base.php';
require_once __DIR__ . '/../lib/docker.php';

class TerminalApi extends ApiBase
{
    /**
     * Handle terminal opening request
     */
    public function handle()
    {
        // Only accept POST requests
        $this->validateMethod('POST');

        // Get JSON input
        $input = $this->getJsonInput();

        // Validate required fields
        $this->validateRequiredFields($input, ['container']);

        $containerName = $input['container'];

        // Validate container name (only allow alphanumeric, dash, and underscore)
        if (!preg_match('/^[a-zA-Z0-9_-]+$/', $containerName)) {
            jsonError('Invalid container name', 400);
            exit;
        }

        // Check if container exists and is running
        if (!isContainerRunning($containerName)) {
            Logger::warn('Terminal open failed: Container not running', [
                'container' => $containerName
            ]);

            jsonError("Container '{$containerName}' is not running", 400);
            exit;
        }

        // Build the docker exec command
        $dockerCommand = sprintf(
            'docker exec -it %s bash',
            escapeshellarg($containerName)
        );

        // Build the AppleScript to open Terminal.app
        $terminalScript = sprintf(
            'tell application "Terminal"
    activate
    do script "%s"
end tell',
            addslashes($dockerCommand)
        );

        // IMPORTANT: We're running inside a Docker container (stackvo-ui)
        // osascript is a macOS command and doesn't exist in the container
        // Solution: Create a temporary script on the host and execute it via volume mount

        // Generate unique script name
        $scriptName = 'open-terminal-' . uniqid() . '.sh';
        $containerScriptPath = '/usr/share/nginx/html/cache/' . $scriptName;
        $hostScriptPath = '/Users/fahrettinaksoy/Desktop/stackvo/stackvo/.ui/cache/' . $scriptName;

        // Create the bash script that will run on the host
        $bashScript = sprintf(
            '#!/bin/bash
osascript -e %s
',
            escapeshellarg($terminalScript)
        );

        // Write the script to the cache directory (mounted from host)
        if (!file_put_contents($containerScriptPath, $bashScript)) {
            Logger::error('Failed to create terminal script', [
                'script_path' => $containerScriptPath
            ]);

            jsonError('Failed to create terminal script', 500);
            exit;
        }

        // Make the script executable
        chmod($containerScriptPath, 0755);

        // Execute the script on the host
        // The script is accessible from host via volume mount
        $hostCommand = sprintf(
            'bash %s 2>&1',
            escapeshellarg($hostScriptPath)
        );

        // Execute the command
        $output = [];
        $returnCode = 0;
        exec($hostCommand, $output, $returnCode);

        // Clean up the script file
        @unlink($containerScriptPath);

        // Log the terminal opening
        Logger::logDockerCommand($hostCommand, $returnCode, $output);

        if ($returnCode === 0) {
            Logger::info('Terminal opened successfully', [
                'container' => $containerName
            ]);

            $this->sendSuccess([
                'message' => "Terminal opened for container '{$containerName}'",
                'container' => $containerName
            ]);
        } else {
            Logger::error('Failed to open terminal', [
                'container' => $containerName,
                'return_code' => $returnCode,
                'output' => implode("\n", $output)
            ]);

            jsonError(
                'Failed to open terminal: ' . implode("\n", $output),
                500
            );
        }
    }
}

// Run the API
$api = new TerminalApi('/api/terminal.php');
$api->run();
