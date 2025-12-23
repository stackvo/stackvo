<?php
###################################################################
# Stackvo UI - Create Project API
# Creates a new project with stackvo.json configuration
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
Logger::logRequest('/create-project.php', 'POST');

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
    $requiredFields = ['name', 'runtime', 'webserver', 'version', 'document_root'];
    foreach ($requiredFields as $field) {
        if (!isset($data[$field]) || empty($data[$field])) {
            Logger::error('Missing required field', ['field' => $field]);
            jsonError("Missing required field: {$field}");
            exit;
        }
    }

    // Sanitize project name (only alphanumeric, dash, underscore)
    $projectName = preg_replace('/[^a-zA-Z0-9\-_]/', '', $data['name']);
    if (empty($projectName)) {
        Logger::error('Invalid project name', ['name' => $data['name']]);
        jsonError('Invalid project name. Use only alphanumeric characters, dash, and underscore.');
        exit;
    }

    // Create domain from project name
    $domain = strtolower($projectName) . '.loc';

    // Build stackvo.json configuration
    $config = [
        'name' => $projectName,
        'domain' => $domain
    ];

    // Add runtime-specific configuration
    $runtime = $data['runtime'];
    switch ($runtime) {
        case 'php':
            $config['php'] = [
                'version' => $data['version'],
                'extensions' => $data['extensions'] ?? ['pdo', 'pdo_mysql', 'mysqli', 'gd', 'curl', 'zip', 'mbstring']
            ];
            break;
        case 'node':
            $config['nodejs'] = [
                'version' => $data['version']
            ];
            break;
        case 'go':
            $config['golang'] = [
                'version' => $data['version']
            ];
            break;
        case 'python':
            $config['python'] = [
                'version' => $data['version']
            ];
            break;
        case 'ruby':
            $config['ruby'] = [
                'version' => $data['version']
            ];
            break;
        default:
            Logger::error('Unsupported runtime', ['runtime' => $runtime]);
            jsonError("Unsupported runtime: {$runtime}");
            exit;
    }

    // Add webserver and document_root
    $config['webserver'] = $data['webserver'];
    $config['document_root'] = $data['document_root'];

    // Get base directory
    $baseDir = Config::get('base_dir');
    $projectsDir = $baseDir . '/' . Config::get('projects_dir');
    $projectPath = $projectsDir . '/' . $projectName;

    // Check if project already exists
    if (file_exists($projectPath)) {
        Logger::error('Project already exists', ['name' => $projectName]);
        jsonError("Project '{$projectName}' already exists");
        exit;
    }

    // Create project directory
    if (!mkdir($projectPath, 0755, true)) {
        Logger::error('Failed to create project directory', ['path' => $projectPath]);
        jsonError('Failed to create project directory');
        exit;
    }

    // Create document root directory
    $documentRoot = $data['document_root'];
    $documentRootPath = $projectPath . '/' . $documentRoot;
    if (!mkdir($documentRootPath, 0755, true)) {
        Logger::error('Failed to create document root directory', ['path' => $documentRootPath]);
        jsonError('Failed to create document root directory');
        exit;
    }

    // Create .stackvo directory for custom configurations
    $stackvoDir = $projectPath . '/.stackvo';
    if (!mkdir($stackvoDir, 0755, true)) {
        Logger::error('Failed to create .stackvo directory', ['path' => $stackvoDir]);
        jsonError('Failed to create .stackvo directory');
        exit;
    }

    // Create index.php in document root
    $indexContent = "<?php\nphpinfo();\n";
    if (file_put_contents($documentRootPath . '/index.php', $indexContent) === false) {
        Logger::error('Failed to create index.php', ['path' => $documentRootPath . '/index.php']);
        jsonError('Failed to create index.php');
        exit;
    }

    // Write stackvo.json configuration file
    $configFilePath = $projectPath . '/stackvo.json';
    if (file_put_contents($configFilePath, json_encode($config, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES)) === false) {
        Logger::error('Failed to write stackvo.json', ['path' => $configFilePath]);
        jsonError('Failed to write stackvo.json');
        exit;
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
                    $generatorMessage = 'Bash is not installed in the UI container. Please rebuild the container with: docker compose -f generated/docker-compose.dynamic.yml build stackvo-ui --no-cache && docker compose -f generated/docker-compose.dynamic.yml up -d stackvo-ui';
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
    // If generator was successful, deploy the project containers
    if ($generatorSuccess) {
        try {
            Logger::info('Deploying project containers');
            
            // Simplified deployment: use docker build + docker run directly
            // This bypasses permission issues with docker-compose
            $projectDockerfile = $actualBaseDir . '/generated/projects/' . $projectName;
            $projectPath = $actualBaseDir . '/projects/' . $projectName;
            $containerName = 'stackvo-' . $projectName;
            $imageName = 'stackvo-' . $projectName . ':latest';
            $network = 'stackvo-net';
            
            // Step 1: Build the image
            $buildCommand = "docker build -t " . escapeshellarg($imageName) . 
                          " " . escapeshellarg($projectDockerfile) . " 2>&1";
            
            $buildOutput = [];
            $buildReturnCode = 0;
            @exec($buildCommand, $buildOutput, $buildReturnCode);
            
            if ($buildReturnCode !== 0) {
                Logger::warn('Docker build failed', [
                    'project' => $projectName,
                    'return_code' => $buildReturnCode,
                    'output' => implode("\n", $buildOutput)
                ]);
                $generatorMessage .= "\nWarning: Docker build failed. Run './cli/stackvo.sh up' manually.";
            } else {
                Logger::info('Docker build successful', ['project' => $projectName]);
                
                // Step 2: Stop and remove existing container if exists
                @exec("docker stop " . escapeshellarg($containerName) . " 2>&1", $stopOutput);
                @exec("docker rm " . escapeshellarg($containerName) . " 2>&1", $rmOutput);
                
                // Step 3: Run the container
                $runCommand = "docker run -d " .
                            "--name " . escapeshellarg($containerName) . " " .
                            "--network " . escapeshellarg($network) . " " .
                            "--restart unless-stopped " .
                            "-v " . escapeshellarg($projectPath) . ":/var/www/html " .
                            "-l 'com.docker.compose.project=stackvo' " .
                            "-l 'com.docker.compose.service=" . $projectName . "' " .
                            "-l 'traefik.enable=true' " .
                            "-l 'traefik.http.routers." . $projectName . ".rule=Host(`" . $domain . "`)' " .
                            "-l 'traefik.http.routers." . $projectName . ".entrypoints=websecure' " .
                            "-l 'traefik.http.routers." . $projectName . ".tls=true' " .
                            "-l 'traefik.http.services." . $projectName . ".loadbalancer.server.port=80' " .
                            escapeshellarg($imageName) . " 2>&1";
                
                $runOutput = [];
                $runReturnCode = 0;
                @exec($runCommand, $runOutput, $runReturnCode);
                
                if ($runReturnCode === 0) {
                    Logger::info('Container started successfully', [
                        'project' => $projectName,
                        'container' => $containerName
                    ]);
                    $generatorMessage .= "\nProject container built and started successfully";
                } else {
                    Logger::warn('Container start failed', [
                        'project' => $projectName,
                        'return_code' => $runReturnCode,
                        'output' => implode("\n", $runOutput)
                    ]);
                    $generatorMessage .= "\nWarning: Container build succeeded but start failed. Run './cli/stackvo.sh up' manually.";
                }
            }
        } catch (Exception $deployException) {
            Logger::error('Project deployment exception', [
                'message' => $deployException->getMessage()
            ]);
        }
    }

    // Log success
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/create-project.php', 200, $duration);
    Logger::info('Project created successfully', [
        'name' => $projectName,
        'domain' => $domain,
        'runtime' => $runtime,
        'webserver' => $data['webserver'],
        'generator_executed' => $generatorSuccess
    ]);

    // Return success response
    jsonSuccess([
        'message' => 'Project created successfully' . ($generatorSuccess ? ' and docker-compose updated' : ' (docker-compose update pending)'),
        'project' => [
            'name' => $projectName,
            'domain' => $domain,
            'path' => 'projects/' . $projectName,
            'config' => $config
        ],
        'generator' => [
            'executed' => $generatorSuccess,
            'message' => $generatorSuccess ? 'Docker Compose configuration updated' : 'Generator execution failed, please run manually'
        ]
    ]);

} catch (Exception $e) {
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/create-project.php', 500, $duration);
    Logger::error('Create project API error', [
        'message' => $e->getMessage(),
        'file' => $e->getFile(),
        'line' => $e->getLine()
    ]);

    jsonError('Error: ' . $e->getMessage());
}
