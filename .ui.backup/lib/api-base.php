<?php
/**
 * Base class for API endpoints
 * Provides common functionality for request handling, logging, and error management
 */

abstract class ApiBase
{
    protected $startTime;
    protected $endpoint;
    protected $method;

    /**
     * Constructor
     * @param string $endpoint Endpoint path (e.g., '/api.php')
     */
    public function __construct($endpoint)
    {
        $this->endpoint = $endpoint;
        $this->method = $_SERVER['REQUEST_METHOD'] ?? 'GET';
        $this->initialize();
    }

    /**
     * Initialize API: load dependencies, set CORS, start logging
     */
    protected function initialize()
    {
        // Load shared libraries
        require_once __DIR__ . '/config.php';
        require_once __DIR__ . '/response.php';
        require_once __DIR__ . '/logger.php';

        // Load configuration
        Config::load('app');

        // Set CORS headers
        setCorsHeaders();

        // Handle preflight requests
        if ($this->method === 'OPTIONS') {
            handlePreflight();
        }

        // Start request tracking
        $this->startTime = microtime(true);
        Logger::logRequest($this->endpoint, $this->method);
    }

    /**
     * Validate HTTP method
     * @param string|array $allowedMethods Single method or array of allowed methods
     */
    protected function validateMethod($allowedMethods)
    {
        $allowedMethods = (array) $allowedMethods;

        if (!in_array($this->method, $allowedMethods)) {
            jsonError('Method not allowed', 405);
            exit;
        }
    }

    /**
     * Get JSON input from request body
     * @return array Decoded JSON data
     */
    protected function getJsonInput()
    {
        $input = file_get_contents('php://input');
        $data = json_decode($input, true);

        if (json_last_error() !== JSON_ERROR_NONE) {
            Logger::error('Invalid JSON input', ['error' => json_last_error_msg()]);
            jsonError('Invalid JSON input: ' . json_last_error_msg());
            exit;
        }

        return $data;
    }

    /**
     * Validate required fields in data
     * @param array $data Input data
     * @param array $requiredFields List of required field names
     */
    protected function validateRequiredFields($data, $requiredFields)
    {
        foreach ($requiredFields as $field) {
            if (!isset($data[$field]) || empty($data[$field])) {
                Logger::error('Missing required field', ['field' => $field]);
                jsonError("Missing required field: {$field}");
                exit;
            }
        }
    }

    /**
     * Handle errors with consistent logging and response
     * @param Exception $e Exception object
     */
    protected function handleError(Exception $e)
    {
        $duration = microtime(true) - $this->startTime;
        Logger::logResponse($this->endpoint, 500, $duration);
        Logger::error($this->endpoint . ' error', [
            'message' => $e->getMessage(),
            'file' => $e->getFile(),
            'line' => $e->getLine()
        ]);

        jsonError('Error: ' . $e->getMessage());
    }

    /**
     * Send success response with consistent logging
     * @param array $data Response data
     * @param string|null $debugMessage Optional debug message
     * @param array $debugContext Optional debug context
     */
    protected function sendSuccess($data, $debugMessage = null, $debugContext = [])
    {
        $duration = microtime(true) - $this->startTime;
        Logger::logResponse($this->endpoint, 200, $duration);

        if ($debugMessage) {
            Logger::debug($debugMessage, $debugContext);
        }

        jsonSuccess($data);
    }

    /**
     * Execute the API logic
     * Must be implemented by child classes
     */
    abstract public function handle();

    /**
     * Run the API
     * Wrapper that calls handle() with error handling
     */
    public function run()
    {
        try {
            $this->handle();
        } catch (Exception $e) {
            $this->handleError($e);
        }
    }
}
