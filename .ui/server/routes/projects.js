import express from 'express';
import DockerService from '../services/DockerService.js';
import ProjectService from '../services/ProjectService.js';

const router = express.Router();
const dockerService = new DockerService();
const projectService = new ProjectService(dockerService);

/**
 * GET /api/projects
 * List all Stackvo projects
 */
router.get('/', async (req, res) => {
  try {
    const projects = await dockerService.listProjects();
    res.json({
      success: true,
      data: { projects },
      meta: { count: projects.length }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/projects/:containerName/start
 * Start a project container
 */
router.post('/:containerName/start', async (req, res) => {
  try {
    const { containerName } = req.params;
    const io = req.app.get('io');
    const fullContainerName = `stackvo-${containerName}`;
    
    // Emit starting event
    if (io) {
      io.emit('project:starting', { project: containerName });
    }
    
    const result = await dockerService.startContainer(fullContainerName);
    
    // Emit success event
    if (io) {
      io.emit('project:started', { 
        project: containerName,
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
      io.emit('project:error', { 
        project: containerName,
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
 * POST /api/projects/:containerName/stop
 * Stop a project container
 */
router.post('/:containerName/stop', async (req, res) => {
  try {
    const { containerName } = req.params;
    const io = req.app.get('io');
    const fullContainerName = `stackvo-${containerName}`;
    
    // Emit stopping event
    if (io) {
      io.emit('project:stopping', { project: containerName });
    }
    
    const result = await dockerService.stopContainer(fullContainerName);
    
    // Emit success event
    if (io) {
      io.emit('project:stopped', { 
        project: containerName,
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
      io.emit('project:error', { 
        project: containerName,
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
 * POST /api/projects/:containerName/restart
 * Restart a project container
 */
router.post('/:containerName/restart', async (req, res) => {
  try {
    const { containerName } = req.params;
    const io = req.app.get('io');
    const fullContainerName = `stackvo-${containerName}`;
    
    // Emit restarting event
    if (io) {
      io.emit('project:restarting', { project: containerName });
    }
    
    const result = await dockerService.restartContainer(fullContainerName);
    
    // Emit success event
    if (io) {
      io.emit('project:restarted', { 
        project: containerName,
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
      io.emit('project:error', { 
        project: containerName,
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
 * Build project containers (with realtime WebSocket progress)
 */
router.post('/:projectName/build', async (req, res) => {
  try {
    const { projectName } = req.params;
    const io = req.app.get('io');  // Get Socket.io instance
    
    // Start build in background (don't wait)
    dockerService.buildContainer(projectName, io).catch(error => {
      console.error(`Build error for ${projectName}:`, error);
    });
    
    // Immediately return 202 Accepted
    res.status(202).json({
      success: true,
      message: `Build started for ${projectName}. Listen to WebSocket events for progress.`,
      data: { projectName }
    });
    
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * Create new project
 */
router.post('/create', async (req, res) => {
  try {
    const projectData = req.body;
    const io = req.app.get('io');
    
    // Validation
    if (!projectData.name || !projectData.runtime || !projectData.version) {
      return res.status(400).json({
        success: false,
        message: 'Missing required fields: name, runtime, version'
      });
    }
    
    // Emit creating event
    if (io) {
      io.emit('project:creating', { project: projectData.name });
    }
    
    // Create project
    const result = await projectService.createProject(projectData);
    
    // Emit success event
    if (io) {
      io.emit('project:created', { 
        project: projectData.name
      });
    }
    
    res.json({
      success: true,
      message: 'Project created successfully',
      project: result
    });
  } catch (error) {
    console.error('Create project error:', error);
    
    // Emit error event
    if (io) {
      io.emit('project:error', { 
        project: projectData?.name || 'unknown',
        error: error.message
      });
    }
    
    res.status(500).json({
      success: false,
      message: error.message || 'Failed to create project'
    });
  }
});

/**
 * DELETE /api/projects/:name
 * Delete a project
 */
router.delete('/:name', async (req, res) => {
  try {
    const { name } = req.params;
    const io = req.app.get('io');
    
    // Emit deleting event
    if (io) {
      io.emit('project:deleting', { project: name });
    }
    
    const result = await projectService.deleteProject(name);
    
    // Emit success event
    if (io) {
      io.emit('project:deleted', { 
        project: name
      });
    }
    
    res.json({
      success: true,
      message: result.message,
      project: { name: result.projectName }
    });
  } catch (error) {
    console.error('Delete project error:', error);
    
    // Emit error event
    if (io) {
      io.emit('project:error', { 
        project: name,
        error: error.message
      });
    }
    
    res.status(500).json({
      success: false,
      message: error.message || 'Failed to delete project'
    });
  }
});

export default router;
