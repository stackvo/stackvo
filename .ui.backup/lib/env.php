<?php
/**
 * Environment variable management utilities
 */

/**
 * Get value from .env file with caching
 * 
 * @param string $key Environment variable key
 * @param string $default Default value if not found
 * @return string Environment variable value
 */
function getEnvValue($key, $default = '') {
    static $envCache = null;
    static $envFile = null;
    
    // Initialize on first call
    if ($envFile === null) {
        $baseDir = is_dir('/app') ? '/app' : dirname(__DIR__);
        $envFile = $baseDir . '/.env';
    }
    
    // Load .env file into cache once
    if ($envCache === null) {
        $envCache = [];
        if (file_exists($envFile)) {
            $lines = file($envFile, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
            foreach ($lines as $line) {
                // Skip comments
                if (strpos(trim($line), '#') === 0) {
                    continue;
                }
                
                // Parse key=value
                if (strpos($line, '=') !== false) {
                    list($envKey, $envValue) = explode('=', $line, 2);
                    $envKey = trim($envKey);
                    $envValue = trim($envValue);
                    $envValue = trim($envValue, '"\'');
                    $envCache[$envKey] = $envValue;
                }
            }
        }
    }
    
    return $envCache[$key] ?? $default;
}
