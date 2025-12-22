<?php
/**
 * Configuration Management
 * 
 * Simple configuration loader with dot notation support
 */

class Config {
    private static $config = [];
    private static $loaded = [];
    
    /**
     * Load a configuration file
     * 
     * @param string $name Config file name (without .php extension)
     * @throws RuntimeException if config file not found
     */
    public static function load($name) {
        if (isset(self::$loaded[$name])) {
            return; // Already loaded
        }
        
        $configFile = __DIR__ . "/../config/{$name}.php";
        if (!file_exists($configFile)) {
            throw new RuntimeException("Config file not found: {$name}");
        }
        
        $config = require $configFile;
        if (!is_array($config)) {
            throw new RuntimeException("Config file must return an array: {$name}");
        }
        
        self::$config = array_merge(self::$config, $config);
        self::$loaded[$name] = true;
    }
    
    /**
     * Get a configuration value
     * 
     * Supports dot notation for nested values:
     * Config::get('service_paths.nginx') => '/var/log/nginx'
     * 
     * @param string $key Config key (supports dot notation)
     * @param mixed $default Default value if key not found
     * @return mixed Config value or default
     */
    public static function get($key, $default = null) {
        // Support dot notation (e.g., 'service_paths.nginx')
        $keys = explode('.', $key);
        $value = self::$config;
        
        foreach ($keys as $k) {
            if (!isset($value[$k])) {
                return $default;
            }
            $value = $value[$k];
        }
        
        return $value;
    }
    
    /**
     * Check if a configuration key exists
     * 
     * @param string $key Config key (supports dot notation)
     * @return bool True if key exists
     */
    public static function has($key) {
        $keys = explode('.', $key);
        $value = self::$config;
        
        foreach ($keys as $k) {
            if (!isset($value[$k])) {
                return false;
            }
            $value = $value[$k];
        }
        
        return true;
    }
    
    /**
     * Get all configuration values
     * 
     * @return array All config values
     */
    public static function all() {
        return self::$config;
    }
    
    /**
     * Set a configuration value (useful for testing)
     * 
     * @param string $key Config key
     * @param mixed $value Config value
     */
    public static function set($key, $value) {
        self::$config[$key] = $value;
    }
    
    /**
     * Clear all loaded configuration (useful for testing)
     */
    public static function clear() {
        self::$config = [];
        self::$loaded = [];
    }
}
