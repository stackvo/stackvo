<?php
/**
 * Application Configuration
 * 
 * Core application constants that never change
 * For configurable values, use .env file
 */

return [
    // Runtime paths (calculated at runtime)
    'base_dir' => is_dir('/app') ? '/app' : dirname(__DIR__, 2),

    // Application constants (never change)
    'container_prefix' => 'stackvo-',
    'excluded_containers' => ['stackvo-ui', 'stackvo-traefik'],

    // Directory paths
    'projects_dir' => 'projects',
    'services_dir' => 'core/templates/services',
    'logs_dir' => 'logs',
];
