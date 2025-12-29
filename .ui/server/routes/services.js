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
    const io = req.app.get('io');
    
    // Emit starting event
    if (io) {
      io.emit('service:starting', { service: containerName });
    }
    
    const result = await dockerService.startContainer(containerName);
    
    // Emit success event
    if (io) {
      io.emit('service:started', { 
        service: containerName,
        running: true
      });
    }
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    // Emit error event
    if (io) {
      io.emit('service:error', { 
        service: containerName,
        error: error.message
      });
    }
    
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
    const io = req.app.get('io');
    
    // Emit stopping event
    if (io) {
      io.emit('service:stopping', { service: containerName });
    }
    
    const result = await dockerService.stopContainer(containerName);
    
    // Emit success event
    if (io) {
      io.emit('service:stopped', { 
        service: containerName,
        running: false
      });
    }
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    // Emit error event
    if (io) {
      io.emit('service:error', { 
        service: containerName,
        error: error.message
      });
    }
    
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
    const io = req.app.get('io');
    
    // Emit restarting event
    if (io) {
      io.emit('service:restarting', { service: containerName });
    }
    
    const result = await dockerService.restartContainer(containerName);
    
    // Emit success event
    if (io) {
      io.emit('service:restarted', { 
        service: containerName,
        running: true
      });
    }
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    // Emit error event
    if (io) {
      io.emit('service:error', { 
        service: containerName,
        error: error.message
      });
    }
    
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
  const { serviceName } = req.params;
  const dockerService = req.app.get('dockerService');
  const io = req.app.get('io');
  
  try {
    const EnvService = (await import('../services/EnvService.js')).default;
    const envService = new EnvService();
    
    // Emit enabling event
    if (io) {
      io.emit('service:enabling', { service: serviceName });
    }
    
    const result = await dockerService.enableService(serviceName, envService);
    
    // Emit success event
    if (io) {
      io.emit('service:enabled', { 
        service: serviceName,
        configured: true,
        running: result.running || false
      });
    }
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    console.error('Enable service error:', error);
    
    // Emit error event
    if (io) {
      io.emit('service:error', { 
        service: serviceName,
        error: error.message
      });
    }
    
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
  const { serviceName } = req.params;
  const dockerService = req.app.get('dockerService');
  const io = req.app.get('io');
  
  try {
    const EnvService = (await import('../services/EnvService.js')).default;
    const envService = new EnvService();
    
    // Emit disabling event
    if (io) {
      io.emit('service:disabling', { service: serviceName });
    }
    
    const result = await dockerService.disableService(serviceName, envService);
    
    // Emit success event
    if (io) {
      io.emit('service:disabled', { 
        service: serviceName,
        configured: false,
        running: false
      });
    }
    
    res.json({
      success: true,
      message: result.message
    });
  } catch (error) {
    console.error('Disable service error:', error);
    
    // Emit error event
    if (io) {
      io.emit('service:error', { 
        service: serviceName,
        error: error.message
      });
    }
    
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

export default router;
