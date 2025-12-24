<?php
/**
 * Logging System for Local Development
 * 
 * Simple file-based logger for debugging and error tracking
 * in local development environment
 */

class Logger {
    const LEVEL_DEBUG = 'DEBUG';
    const LEVEL_INFO = 'INFO';
    const LEVEL_WARNING = 'WARNING';
    const LEVEL_ERROR = 'ERROR';
    
    private static $logFile = null;
    private static $minLevel = self::LEVEL_DEBUG;
    private static $enabled = true; // Enabled with writable volume mount
    
    /**
     * Initialize logger
     */
    private static function init() {
        if (self::$logFile === null) {
            $logDir = __DIR__ . '/../logs';
            
            // Only try to create if we can write
            if (is_writable(dirname($logDir)) || is_dir($logDir)) {
                if (!is_dir($logDir)) {
                    @mkdir($logDir, 0755, true);
                }
                self::$logFile = $logDir . '/app.log';
            }
            
            // Try to read .env for settings
            $envFile = dirname($logDir) . '/.env';
            if (file_exists($envFile)) {
                $lines = file($envFile, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
                foreach ($lines as $line) {
                    if (strpos(trim($line), '#') === 0) continue;
                    if (strpos($line, '=') !== false) {
                        list($key, $value) = explode('=', $line, 2);
                        $key = trim($key);
                        $value = trim(trim($value), '"\'');
                        
                        if ($key === 'LOG_LEVEL') {
                            self::$minLevel = $value;
                        } elseif ($key === 'LOG_ENABLE') {
                            self::$enabled = $value === 'true';
                        }
                    }
                }
            }
        }
    }
    
    /**
     * Log a debug message (for development debugging)
     * 
     * @param string $message Log message
     * @param array $context Additional context data
     */
    public static function debug($message, $context = []) {
        self::log(self::LEVEL_DEBUG, $message, $context);
    }
    
    /**
     * Log an info message (general information)
     * 
     * @param string $message Log message
     * @param array $context Additional context data
     */
    public static function info($message, $context = []) {
        self::log(self::LEVEL_INFO, $message, $context);
    }
    
    /**
     * Log a warning message (potential issues)
     * 
     * @param string $message Log message
     * @param array $context Additional context data
     */
    public static function warning($message, $context = []) {
        self::log(self::LEVEL_WARNING, $message, $context);
    }
    
    /**
     * Alias for warning() - shorter version
     * 
     * @param string $message Log message
     * @param array $context Additional context data
     */
    public static function warn($message, $context = []) {
        self::warning($message, $context);
    }
    
    /**
     * Log an error message (failures and exceptions)
     * 
     * @param string $message Log message
     * @param array $context Additional context data
     */
    public static function error($message, $context = []) {
        self::log(self::LEVEL_ERROR, $message, $context);
    }
    
    /**
     * Internal log method
     * 
     * @param string $level Log level
     * @param string $message Log message
     * @param array $context Additional context data
     */
    private static function log($level, $message, $context = []) {
        self::init();
        
        if (!self::$enabled) {
            return;
        }
        
        // Check if we should log this level
        $levels = [
            self::LEVEL_DEBUG => 0,
            self::LEVEL_INFO => 1,
            self::LEVEL_WARNING => 2,
            self::LEVEL_ERROR => 3,
        ];
        
        if ($levels[$level] < $levels[self::$minLevel]) {
            return;
        }
        
        // Format log entry
        $timestamp = date('Y-m-d H:i:s');
        $contextStr = !empty($context) ? ' ' . json_encode($context, JSON_UNESCAPED_SLASHES) : '';
        $entry = "[{$timestamp}] [{$level}] {$message}{$contextStr}\n";
        
        // Write to log file (only if we have a writable file)
        if (self::$logFile && is_writable(dirname(self::$logFile))) {
            @file_put_contents(self::$logFile, $entry, FILE_APPEND | LOCK_EX);
        }
    }
    
    /**
     * Log API request (for debugging API calls)
     * 
     * @param string $endpoint API endpoint
     * @param string $method HTTP method
     * @param array $data Request data
     */
    public static function logRequest($endpoint, $method, $data = []) {
        self::info("API Request: {$method} {$endpoint}", [
            'ip' => $_SERVER['REMOTE_ADDR'] ?? 'unknown',
            'user_agent' => $_SERVER['HTTP_USER_AGENT'] ?? 'unknown',
            'data' => $data,
        ]);
    }
    
    /**
     * Log API response (for performance tracking)
     * 
     * @param string $endpoint API endpoint
     * @param int $statusCode HTTP status code
     * @param float $duration Request duration in seconds
     */
    public static function logResponse($endpoint, $statusCode, $duration) {
        $durationMs = round($duration * 1000, 2);
        $level = $statusCode >= 500 ? 'ERROR' : ($statusCode >= 400 ? 'WARNING' : 'INFO');
        
        self::log($level, "API Response: {$endpoint}", [
            'status' => $statusCode,
            'duration_ms' => $durationMs,
        ]);
    }
    
    /**
     * Log Docker command execution (for debugging container operations)
     * 
     * @param string $command Docker command
     * @param int $returnCode Command return code
     * @param array $output Command output
     */
    public static function logDockerCommand($command, $returnCode, $output = []) {
        $level = $returnCode === 0 ? 'DEBUG' : 'ERROR';
        self::log($level, "Docker command executed", [
            'command' => $command,
            'return_code' => $returnCode,
            'output' => $returnCode !== 0 ? implode("\n", $output) : null,
        ]);
    }
    
    /**
     * Clear log file (useful for testing)
     */
    public static function clear() {
        self::init();
        if (file_exists(self::$logFile)) {
            file_put_contents(self::$logFile, '');
        }
    }
    
    /**
     * Get log file path
     * 
     * @return string Log file path
     */
    public static function getLogFile() {
        self::init();
        return self::$logFile;
    }
}
