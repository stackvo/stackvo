import express from 'express';
import DockerService from '../services/DockerService.js';

const router = express.Router();
const dockerService = new DockerService();

/**
 * GET /api/services
 * List all Stackvo services
 */
router.get('/', async (req, res) => {
  try {
    const services = await dockerService.listServices();
    res.json({
      success: true,
      data: { services },
      meta: { count: services.length }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/services/:containerName/start
 * Start a service container
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
 * POST /api/services/:containerName/stop
 * Stop a service container
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
 * POST /api/services/:containerName/restart
 * Restart a service container
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
 * Enable a service
 */
router.post('/:serviceName/enable', async (req, res) => {
  try {
    const { serviceName } = req.params;
    const dockerService = req.app.get('dockerService');
    const EnvService = (await import('../services/EnvService.js')).default;
    const envService = new EnvService();
    
    const result = await dockerService.enableService(serviceName, envService);
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    console.error('Enable service error:', error);
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * Disable a service
 */
router.post('/:serviceName/disable', async (req, res) => {
  try {
    const { serviceName } = req.params;
    const dockerService = req.app.get('dockerService');
    const EnvService = (await import('../services/EnvService.js')).default;
    const envService = new EnvService();
    
    const result = await dockerService.disableService(serviceName, envService);
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    console.error('Disable service error:', error);
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

export default router;
