<?php
/**
 * Tools API - Lists all TOOLS_ prefixed environment variables
 * Returns tools configuration from .env file
 */

header('Content-Type: application/json');

// Load environment variables from .env file
function loadEnvFile($filePath)
{
    if (!file_exists($filePath)) {
        return [];
    }

    $lines = file($filePath, FILE_IGNORE_NEW_LINES | FILE_SKIP_EMPTY_LINES);
    $env = [];

    foreach ($lines as $line) {
        // Skip comments and empty lines
        if (empty(trim($line)) || strpos(trim($line), '#') === 0) {
            continue;
        }

        // Parse KEY=VALUE
        if (strpos($line, '=') !== false) {
            list($key, $value) = explode('=', $line, 2);
            $key = trim($key);
            $value = trim($value);
            $env[$key] = $value;
        }
    }

    return $env;
}

// Get tools from environment
function getTools()
{
    // In Docker container, the project root is mounted at /app
    $envFile = '/app/.env';
    $env = loadEnvFile($envFile);

    $tools = [];
    $toolNames = [];

    // Find all TOOLS_ prefixed variables
    foreach ($env as $key => $value) {
        if (strpos($key, 'TOOLS_') === 0) {
            // Extract tool name (e.g., TOOLS_ADMINER_ENABLE -> ADMINER)
            $parts = explode('_', $key);
            if (count($parts) >= 3) {
                $toolName = $parts[1]; // Get the tool name part

                if (!isset($toolNames[$toolName])) {
                    $toolNames[$toolName] = [];
                }

                // Determine the property type
                $property = implode('_', array_slice($parts, 2));
                $toolNames[$toolName][$property] = $value;
            }
        }
    }

    // Build tools array
    foreach ($toolNames as $name => $properties) {
        $enabled = isset($properties['ENABLE']) ?
            (strtolower($properties['ENABLE']) === 'true') : false;

        $url = isset($properties['URL']) ? $properties['URL'] : '';
        $domain = $env['DEFAULT_TLD_SUFFIX'] ?? 'stackvo.loc';
        $fullUrl = $url ? "https://{$url}.{$domain}" : '';

        $version = isset($properties['VERSION']) ? $properties['VERSION'] : '-';

        $tools[] = [
            'name' => ucfirst(strtolower($name)),
            'enabled' => $enabled,
            'url' => $fullUrl,
            'raw_url' => $url,
            'version' => $version,
            'properties' => $properties
        ];
    }

    // Sort by name
    usort($tools, function ($a, $b) {
        return strcmp($a['name'], $b['name']);
    });

    return $tools;
}

try {
    $envFile = '/app/.env';
    $env = loadEnvFile($envFile);
    $tools = getTools();

    // Count TOOLS_ prefixed variables for debugging
    $toolsVarCount = 0;
    foreach ($env as $key => $value) {
        if (strpos($key, 'TOOLS_') === 0) {
            $toolsVarCount++;
        }
    }

    echo json_encode([
        'success' => true,
        'tools' => $tools,
        'count' => count($tools),
        'debug' => [
            'env_file' => $envFile,
            'env_file_exists' => file_exists($envFile),
            'total_env_vars' => count($env),
            'tools_vars_found' => $toolsVarCount
        ]
    ], JSON_PRETTY_PRINT);

} catch (Exception $e) {
    http_response_code(500);
    echo json_encode([
        'success' => false,
        'error' => $e->getMessage()
    ], JSON_PRETTY_PRINT);
}