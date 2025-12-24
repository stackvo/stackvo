<?php
###################################################################
# Stackvo UI - Delete Project API
# Deletes a project directory
###################################################################

// Load shared libraries
require_once __DIR__ . '/../lib/config.php';
require_once __DIR__ . '/../lib/response.php';
require_once __DIR__ . '/../lib/logger.php';

// Load configuration
Config::load('app');

setCorsHeaders();

// Only accept POST requests
if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    jsonError('Method not allowed', 405);
    exit;
}

// Start request tracking
$startTime = microtime(true);
Logger::logRequest('/delete-project.php', 'POST');

try {
    // Get JSON input
    $input = file_get_contents('php://input');
    $data = json_decode($input, true);

    if (json_last_error() !== JSON_ERROR_NONE) {
        Logger::error('Invalid JSON input', ['error' => json_last_error_msg()]);
        jsonError('Invalid JSON input: ' . json_last_error_msg());
        exit;
    }

    // Validate required fields
    if (!isset($data['name']) || empty($data['name'])) {
        Logger::error('Missing project name');
        jsonError('Project name is required');
        exit;
    }

    // Sanitize project name (only alphanumeric, dash, underscore)
    $projectName = preg_replace('/[^a-zA-Z0-9\-_]/', '', $data['name']);
    if (empty($projectName) || $projectName !== $data['name']) {
        Logger::error('Invalid project name', ['name' => $data['name']]);
        jsonError('Invalid project name. Use only alphanumeric characters, dash, and underscore.');
        exit;
    }

    // Get base directory
    $baseDir = Config::get('base_dir');
    $projectsDir = $baseDir . '/' . Config::get('projects_dir');
    $projectPath = $projectsDir . '/' . $projectName;

    // Security: Ensure the path is within projects directory
    $realProjectsDir = realpath($projectsDir);
    $realProjectPath = realpath($projectPath);

    if (!$realProjectPath || strpos($realProjectPath, $realProjectsDir) !== 0) {
        Logger::error('Invalid project path', [
            'name' => $projectName,
            'path' => $projectPath,
            'real_path' => $realProjectPath
        ]);
        jsonError('Invalid project path');
        exit;
    }

    // Check if project exists
    if (!file_exists($projectPath) || !is_dir($projectPath)) {
        Logger::error('Project not found', ['name' => $projectName, 'path' => $projectPath]);
        jsonError("Project '{$projectName}' not found");
        exit;
    }

    // Delete project directory recursively
    function deleteDirectory($dir)
    {
        if (!file_exists($dir)) {
            return true;
        }

        if (!is_dir($dir)) {
            return unlink($dir);
        }

        foreach (scandir($dir) as $item) {
            if ($item == '.' || $item == '..') {
                continue;
            }

            if (!deleteDirectory($dir . DIRECTORY_SEPARATOR . $item)) {
                return false;
            }
        }

        return rmdir($dir);
    }

    if (!deleteDirectory($projectPath)) {
        Logger::error('Failed to delete project directory', ['path' => $projectPath]);
        jsonError('Failed to delete project directory');
        exit;
    }

    // Stop and remove project containers
    $containerCleanupSuccess = false;
    $containerCleanupMessage = '';

    try {
        Logger::info('Stopping and removing project containers');

        // Stop and remove PHP container
        $phpContainerName = "stackvo-{$projectName}-php";
        $stopPhpCommand = "docker stop {$phpContainerName} 2>&1";
        $removePhpCommand = "docker rm {$phpContainerName} 2>&1";

        @exec($stopPhpCommand, $phpStopOutput, $phpStopReturnCode);
        @exec($removePhpCommand, $phpRemoveOutput, $phpRemoveReturnCode);

        // Stop and remove Web container
        $webContainerName = "stackvo-{$projectName}-web";
        $stopWebCommand = "docker stop {$webContainerName} 2>&1";
        $removeWebCommand = "docker rm {$webContainerName} 2>&1";

        @exec($stopWebCommand, $webStopOutput, $webStopReturnCode);
        @exec($removeWebCommand, $webRemoveOutput, $webRemoveReturnCode);

        $containerCleanupSuccess = true;
        $containerCleanupMessage = "Containers stopped and removed successfully";

        Logger::info('Project containers removed', [
            'project' => $projectName,
            'php_container' => $phpContainerName,
            'web_container' => $webContainerName
        ]);
    } catch (Exception $containerException) {
        Logger::warn('Container cleanup failed', [
            'message' => $containerException->getMessage()
        ]);
        $containerCleanupMessage = 'Container cleanup failed: ' . $containerException->getMessage();
    }

    // Run generator to update docker-compose.projects.yml
    $generatorSuccess = false;
    $generatorMessage = '';

    try {
        // Check if exec is available
        $disabledFunctions = explode(',', ini_get('disable_functions'));
        $disabledFunctions = array_map('trim', $disabledFunctions);

        if (in_array('exec', $disabledFunctions)) {
            Logger::warn('exec() function is disabled, skipping generator');
            $generatorMessage = 'exec() function is disabled. Please run: ./stackvo generate projects';
        } else {
            Logger::info('Running generator to update docker-compose configuration');

            // Detect if running in container (check for /app directory)
            $containerBaseDir = '/app';
            $isContainer = file_exists($containerBaseDir . '/cli/commands/generate.sh');
            $actualBaseDir = $isContainer ? $containerBaseDir : $baseDir;

            $generatorScript = $actualBaseDir . '/cli/commands/generate.sh';

            // Check if generator script exists
            if (!file_exists($generatorScript)) {
                Logger::warn('Generator script not found', ['path' => $generatorScript, 'is_container' => $isContainer]);
                $generatorMessage = 'Generator script not found at: ' . $generatorScript;
            } else {
                // Check if bash is available (required for generate.sh)
                $bashCheck = [];
                $bashReturnCode = 0;
                @exec('bash --version 2>&1', $bashCheck, $bashReturnCode);

                if ($bashReturnCode !== 0) {
                    // Bash not found - this means container needs rebuild
                    Logger::error('Bash not found in container', [
                        'is_container' => $isContainer,
                        'bash_check_output' => implode("\n", $bashCheck)
                    ]);
                    $generatorMessage = 'Bash is not installed in the UI container. Please rebuild the container.';
                } else {
                    // Bash is available, proceed with generation
                    $shell = 'bash';
                    $generatorCommand = "cd " . escapeshellarg($actualBaseDir) . " && " . $shell . " " . escapeshellarg($generatorScript) . " projects 2>&1";

                    Logger::info('Executing generator', [
                        'command' => $generatorCommand,
                        'is_container' => $isContainer,
                        'base_dir' => $actualBaseDir
                    ]);

                    $generatorOutput = [];
                    $generatorReturnCode = 0;
                    @exec($generatorCommand, $generatorOutput, $generatorReturnCode);

                    $generatorSuccess = ($generatorReturnCode === 0);
                    $generatorMessage = implode("\n", $generatorOutput);

                    if ($generatorSuccess) {
                        Logger::info('Generator executed successfully', ['output' => $generatorMessage]);
                    } else {
                        Logger::warn('Generator execution failed', [
                            'return_code' => $generatorReturnCode,
                            'output' => $generatorMessage
                        ]);
                    }
                }
            }
        }
    } catch (Exception $genException) {
        Logger::error('Generator execution exception', [
            'message' => $genException->getMessage()
        ]);
        $generatorMessage = 'Generator execution failed: ' . $genException->getMessage();
    }

    // Log success
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/delete-project.php', 200, $duration);
    Logger::info('Project deleted successfully', [
        'name' => $projectName,
        'path' => $projectPath,
        'containers_removed' => $containerCleanupSuccess,
        'generator_executed' => $generatorSuccess
    ]);

    jsonSuccess([
        'message' => 'Project deleted successfully' . ($generatorSuccess ? ' and docker-compose updated' : ' (docker-compose update pending)'),
        'project' => [
            'name' => $projectName,
            'path' => 'projects/' . $projectName
        ],
        'containers' => [
            'removed' => $containerCleanupSuccess,
            'message' => $containerCleanupMessage
        ],
        'generator' => [
            'executed' => $generatorSuccess,
            'message' => $generatorSuccess ? 'Docker Compose configuration updated' : 'Generator execution failed, please run manually'
        ]
    ]);

} catch (Exception $e) {
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/delete-project.php', 500, $duration);
    Logger::error('Delete project API error', [
        'message' => $e->getMessage(),
        'file' => $e->getFile(),
        'line' => $e->getLine()
    ]);

    jsonError('Error: ' . $e->getMessage());
}
