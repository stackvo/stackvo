<?php
/**
 * Simple File-Based Cache System
 * 
 * Lightweight caching for local development to reduce
 * Docker command calls and improve performance
 */

class Cache
{
    private static $cacheDir = null;
    private static $enabled = true;
    private static $defaultTTL = 5; // seconds

    /**
     * Initialize cache system
     */
    private static function init()
    {
        if (self::$cacheDir === null) {
            self::$cacheDir = '/tmp/stackvo-cache';
            if (!is_dir(self::$cacheDir)) {
                @mkdir(self::$cacheDir, 0755, true);
            }

            // Get cache settings from environment
            // Note: We can't use getEnvValue() here to avoid circular dependency
            // Cache is DISABLED by default to avoid permission issues in read-only containers
            self::$enabled = false;
            self::$defaultTTL = 5;

            // Try to read .env if it exists
            $envFile = dirname(self::$cacheDir) . '/.env';
            if (file_exists($envFile)) {
                $lines = file($envFile, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
                foreach ($lines as $line) {
                    if (strpos(trim($line), '#') === 0)
                        continue;
                    if (strpos($line, '=') !== false) {
                        list($key, $value) = explode('=', $line, 2);
                        $key = trim($key);
                        $value = trim(trim($value), '"\'');

                        if ($key === 'CACHE_ENABLE') {
                            self::$enabled = $value === 'true';
                        } elseif ($key === 'CACHE_TTL') {
                            self::$defaultTTL = (int) $value;
                        }
                    }
                }
            }
        }
    }

    /**
     * Get a cached value
     * 
     * @param string $key Cache key
     * @param mixed $default Default value if not found or expired
     * @return mixed Cached value or default
     */
    public static function get($key, $default = null)
    {
        self::init();

        if (!self::$enabled) {
            return $default;
        }

        $file = self::getCacheFile($key);

        if (!file_exists($file)) {
            return $default;
        }

        $data = unserialize(file_get_contents($file));

        // Check if expired
        if ($data['expires'] < time()) {
            unlink($file);
            return $default;
        }

        return $data['value'];
    }

    /**
     * Set a cached value
     * 
     * @param string $key Cache key
     * @param mixed $value Value to cache
     * @param int $ttl Time to live in seconds (null = use default)
     */
    public static function set($key, $value, $ttl = null)
    {
        self::init();

        if (!self::$enabled) {
            return;
        }

        if ($ttl === null) {
            $ttl = self::$defaultTTL;
        }

        $file = self::getCacheFile($key);
        $data = [
            'value' => $value,
            'expires' => time() + $ttl,
            'created' => time(),
        ];

        file_put_contents($file, serialize($data), LOCK_EX);
    }

    /**
     * Check if a key exists and is not expired
     * 
     * @param string $key Cache key
     * @return bool True if exists and valid
     */
    public static function has($key)
    {
        return self::get($key) !== null;
    }

    /**
     * Delete a cached value
     * 
     * @param string $key Cache key
     */
    public static function delete($key)
    {
        self::init();

        $file = self::getCacheFile($key);
        if (file_exists($file)) {
            unlink($file);
        }
    }

    /**
     * Clear all cache
     */
    public static function clear()
    {
        self::init();

        $files = glob(self::$cacheDir . '/*.cache');
        if ($files) {
            foreach ($files as $file) {
                unlink($file);
            }
        }
    }

    /**
     * Remember a value (get from cache or execute callback)
     * 
     * This is the most useful method for caching expensive operations:
     * 
     * $status = Cache::remember('container_mysql', function() {
     *     return isContainerRunning('stackvo-mysql');
     * }, 5);
     * 
     * @param string $key Cache key
     * @param callable $callback Callback to execute if not cached
     * @param int $ttl Time to live in seconds
     * @return mixed Cached or fresh value
     */
    public static function remember($key, $callback, $ttl = null)
    {
        $value = self::get($key);

        if ($value !== null) {
            return $value;
        }

        $value = $callback();
        self::set($key, $value, $ttl);

        return $value;
    }

    /**
     * Get cache file path for a key
     * 
     * @param string $key Cache key
     * @return string Cache file path
     */
    private static function getCacheFile($key)
    {
        return self::$cacheDir . '/' . md5($key) . '.cache';
    }

    /**
     * Get cache statistics (for debugging)
     * 
     * @return array Cache stats
     */
    public static function stats()
    {
        self::init();

        $files = glob(self::$cacheDir . '/*.cache');
        $total = count($files);
        $expired = 0;
        $valid = 0;

        foreach ($files as $file) {
            $data = unserialize(file_get_contents($file));
            if ($data['expires'] < time()) {
                $expired++;
            } else {
                $valid++;
            }
        }

        return [
            'enabled' => self::$enabled,
            'total_files' => $total,
            'valid' => $valid,
            'expired' => $expired,
            'directory' => self::$cacheDir,
        ];
    }
}
