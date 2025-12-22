<?php
###################################################################
# Stackvo UI - Supported Languages API
# Returns supported languages and versions from .env
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
Logger::logRequest('/supported-languages.php', 'GET');

try {
    // Get base directory
    $baseDir = Config::get('base_dir');
    $envFile = $baseDir . '/.env';

    if (!file_exists($envFile)) {
        Logger::error('.env file not found', ['path' => $envFile]);
        jsonError('.env file not found');
        exit;
    }

    // Read .env file
    $envContent = file_get_contents($envFile);
    $lines = explode("\n", $envContent);

    $supportedLanguages = [];
    $supportedVersions = [];
    $defaultVersions = [];
    $phpExtensions = [];

    foreach ($lines as $line) {
        $line = trim($line);

        // Skip comments and empty lines
        if (empty($line) || strpos($line, '#') === 0) {
            continue;
        }

        // Parse SUPPORTED_LANGUAGES
        if (strpos($line, 'SUPPORTED_LANGUAGES=') === 0) {
            $value = substr($line, strlen('SUPPORTED_LANGUAGES='));
            $supportedLanguages = array_map('trim', explode(',', $value));
        }

        // Parse SUPPORTED_LANGUAGES_<LANG>_VERSIONS (new format)
        if (preg_match('/^SUPPORTED_LANGUAGES_([A-Z]+)_VERSIONS=(.+)$/', $line, $matches)) {
            $language = strtolower($matches[1]);
            $versions = array_map('trim', explode(',', $matches[2]));
            $supportedVersions[$language] = $versions;
        }

        // Parse SUPPORTED_LANGUAGES_<LANG>_DEFAULT (new format)
        if (preg_match('/^SUPPORTED_LANGUAGES_([A-Z]+)_DEFAULT=(.+)$/', $line, $matches)) {
            $language = strtolower($matches[1]);
            $defaultVersion = trim($matches[2]);
            $defaultVersions[$language] = $defaultVersion;
        }

        // Parse SUPPORTED_LANGUAGES_PHP_EXTENSIONS
        if (strpos($line, 'SUPPORTED_LANGUAGES_PHP_EXTENSIONS=') === 0) {
            $value = substr($line, strlen('SUPPORTED_LANGUAGES_PHP_EXTENSIONS='));
            $phpExtensions = array_map('trim', explode(',', $value));
        }
    }

    // Log response
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/supported-languages.php', 200, $duration);
    Logger::debug('Supported languages loaded', [
        'languages_count' => count($supportedLanguages),
        'versions_count' => count($supportedVersions)
    ]);

    jsonSuccess([
        'languages' => $supportedLanguages,
        'versions' => $supportedVersions,
        'defaults' => $defaultVersions,
        'phpExtensions' => $phpExtensions
    ]);

} catch (Exception $e) {
    $duration = microtime(true) - $startTime;
    Logger::logResponse('/supported-languages.php', 500, $duration);
    Logger::error('Supported languages API error', [
        'message' => $e->getMessage(),
        'file' => $e->getFile(),
        'line' => $e->getLine()
    ]);

    jsonError('Error: ' . $e->getMessage());
}
