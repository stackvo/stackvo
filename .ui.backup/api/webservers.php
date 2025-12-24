<?php
###################################################################
# Stackvo UI - Webservers API
# Returns available webservers from core/templates/servers
###################################################################

// Load shared libraries
require_once __DIR__ . '/../lib/config.php';
require_once __DIR__ . '/../lib/response.php';
require_once __DIR__ . '/../lib/logger.php';

// Load configuration
Config::load('app');

setCorsHeaders();

// Start request tracking
$startTime = microtime(true);
Logger::logRequest('/webservers.php', 'GET');

try {
    $baseDir = Config::get('base_dir');
    $serversDir = $baseDir . '/core/templates/servers';
    
    // Check if servers directory exists
    if (!is_dir($serversDir)) {
        Logger::error('Servers directory not found', ['path' => $serversDir]);
        jsonError('Servers directory not found');
        exit;
    }
    
    // Scan servers directory
    $directories = array_diff(scandir($serversDir), ['.', '..']);
    $webservers = [];
    
    foreach ($directories as $dir) {
        $serverPath = $serversDir . '/' . $dir;
        
        // Only include directories
        if (is_dir($serverPath)) {
            $webservers[] = $dir;
        }
    }
    
    // Sort alphabetically
    sort($webservers);
    
    // Log response
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/webservers.php', 200, $duration);
    Logger::debug('Webservers loaded', ['count' => count($webservers)]);
    
    jsonSuccess([
        'webservers' => $webservers,
        'count' => count($webservers)
    ]);
    
} catch (Exception $e) {
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/webservers.php', 500, $duration);
    Logger::error('Webservers API error', [
        'message' => $e->getMessage(),
        'file' => $e->getFile(),
        'line' => $e->getLine()
    ]);
    
    jsonError('Error: ' . $e->getMessage());
}
