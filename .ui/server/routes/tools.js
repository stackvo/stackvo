import express from 'express';
import DockerService from '../services/DockerService.js';

const router = express.Router();
const dockerService = new DockerService();

/**
 * GET /api/tools
 * List all Stackvo tools
 */
router.get('/', async (req, res) => {
  try {
    const tools = await dockerService.listTools();
    res.json({
      success: true,
      data: { tools },
      meta: { count: tools.length }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/tools/:containerName/start
 * Start a tool container
 */
router.post('/:containerName/start', async (req, res) => {
  try {
    const { containerName } = req.params;
    const result = await dockerService.startContainer(containerName);
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/tools/:containerName/stop
 * Stop a tool container
 */
router.post('/:containerName/stop', async (req, res) => {
  try {
    const { containerName } = req.params;
    const result = await dockerService.stopContainer(containerName);
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/tools/:containerName/restart
 * Restart a tool container
 */
router.post('/:containerName/restart', async (req, res) => {
  try {
    const { containerName } = req.params;
    const result = await dockerService.restartContainer(containerName);
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/tools/:toolName/enable
 * Enable a tool (requires container rebuild)
 */
router.post('/:toolName/enable', async (req, res) => {
  try {
    const { toolName } = req.params;
    const dockerService = req.app.get('dockerService');
    const EnvService = (await import('../services/EnvService.js')).default;
    const envService = new EnvService();
    
    const result = await dockerService.enableTool(toolName, envService);
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    console.error('Enable tool error:', error);
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/tools/:toolName/disable
 * Disable a tool (requires container rebuild)
 */
router.post('/:toolName/disable', async (req, res) => {
  try {
    const { toolName } = req.params;
    const dockerService = req.app.get('dockerService');
    const EnvService = (await import('../services/EnvService.js')).default;
    const envService = new EnvService();
    
    const result = await dockerService.disableTool(toolName, envService);
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    console.error('Disable tool error:', error);
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

export default router;
