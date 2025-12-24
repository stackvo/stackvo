<?php
/**
 * General utility functions
 */

/**
 * Format bytes to human readable format
 * 
 * @param int $bytes Bytes to format
 * @param int $precision Decimal precision
 * @return string Formatted string (e.g., "1.5 MB")
 */
function formatBytes($bytes, $precision = 2) {
    $units = ['B', 'KB', 'MB', 'GB', 'TB'];
    
    for ($i = 0; $bytes > 1024 && $i < count($units) - 1; $i++) {
        $bytes /= 1024;
    }
    
    return round($bytes, $precision) . ' ' . $units[$i];
}

/**
 * Get base directory (works in Docker and locally)
 * 
 * @return string Base directory path
 */
function getBaseDir() {
    static $baseDir = null;
    
    if ($baseDir === null) {
        $baseDir = is_dir('/app') ? '/app' : dirname(__DIR__);
    }
    
    return $baseDir;
}
