import express from 'express';
import DockerService from '../services/DockerService.js';

const router = express.Router();

/**
 * GET /api/services
 * List all Stackvo services
 */
router.get('/', async (req, res) => {
  try {
    const dockerService = req.app.get('dockerService');
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
  const { containerName } = req.params;
  const dockerService = req.app.get('dockerService');
  const io = req.app.get('io');
  
  try {
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
  const { containerName } = req.params;
  const dockerService = req.app.get('dockerService');
  const io = req.app.get('io');

  try {
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
  const { containerName } = req.params;
  const dockerService = req.app.get('dockerService');
  const io = req.app.get('io');

  try {
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
    
    const result = await dockerService.enableService(serviceName, envService, io);
    
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
    
    const result = await dockerService.disableService(serviceName, envService, io);
    
    // Emit success event
    if (io) {
      console.log(`SocketEmit [${serviceName}]: service:disabled`);
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

/**
 * GET /api/services/:serviceName/dependencies
 * Get service dependencies and their status
 */
router.get('/:serviceName/dependencies', async (req, res) => {
  try {
    const { serviceName } = req.params;
    
    // Load service dependencies configuration
    const fs = await import('fs/promises');
    const path = await import('path');
    const configPath = path.join(process.cwd(), 'config', 'serviceDependencies.json');
    
    let dependencies = {};
    try {
      const configData = await fs.readFile(configPath, 'utf-8');
      dependencies = JSON.parse(configData);
    } catch (error) {
      // If config file doesn't exist, return empty dependencies
      console.warn('serviceDependencies.json not found, using empty dependencies');
    }
    
    const serviceDeps = dependencies[serviceName] || { required: [], optional: [], internal: [] };
    
    // Check which dependencies are already running
    const status = await Promise.all([
      ...serviceDeps.required.map(async (dep) => ({
        name: dep,
        type: 'required',
        running: await dockerService.isServiceRunning(dep)
      })),
      ...serviceDeps.optional.map(async (dep) => ({
        name: dep,
        type: 'optional',
        running: await dockerService.isServiceRunning(dep)
      }))
    ]);
    
    const hasUnmetDependencies = status.some(d => d.type === 'required' && !d.running);
    
    res.json({
      success: true,
      data: {
        service: serviceName,
        description: serviceDeps.description || '',
        dependencies: status,
        hasUnmetDependencies,
        internal: serviceDeps.internal || []
      }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

export default router;
