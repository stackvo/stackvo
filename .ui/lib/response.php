<?php
/**
 * HTTP response utilities
 */

/**
 * Send JSON response and exit
 * 
 * @param array $data Response data
 * @param int $statusCode HTTP status code
 */
function jsonResponse($data, $statusCode = 200) {
    http_response_code($statusCode);
    header('Content-Type: application/json');
    header('Access-Control-Allow-Origin: *');
    echo json_encode($data, JSON_PRETTY_PRINT);
    exit;
}

/**
 * Send JSON error response and exit
 * 
 * @param string $message Error message
 * @param int $statusCode HTTP status code
 */
function jsonError($message, $statusCode = 500) {
    jsonResponse([
        'success' => false,
        'message' => $message
    ], $statusCode);
}

/**
 * Send JSON success response and exit
 * 
 * @param array $data Response data (will be merged with success:true)
 */
function jsonSuccess($data) {
    jsonResponse(array_merge(['success' => true], $data));
}

/**
 * Set CORS headers
 */
function setCorsHeaders() {
    header('Access-Control-Allow-Origin: *');
    header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
    header('Access-Control-Allow-Headers: Content-Type');
}

/**
 * Handle OPTIONS preflight request
 */
function handlePreflight() {
    if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
        setCorsHeaders();
        http_response_code(200);
        exit;
    }
}
