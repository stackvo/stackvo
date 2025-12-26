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
    const fullContainerName = `stackvo-${containerName}`;
    const result = await dockerService.startContainer(fullContainerName);
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
 * POST /api/projects/:containerName/stop
 * Stop a project container
 */
router.post('/:containerName/stop', async (req, res) => {
  try {
    const { containerName } = req.params;
    const fullContainerName = `stackvo-${containerName}`;
    const result = await dockerService.stopContainer(fullContainerName);
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
 * POST /api/projects/:containerName/restart
 * Restart a project container
 */
router.post('/:containerName/restart', async (req, res) => {
  try {
    const { containerName } = req.params;
    const fullContainerName = `stackvo-${containerName}`;
    const result = await dockerService.restartContainer(fullContainerName);
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
 * Build project containers
 */
router.post('/:projectName/build', async (req, res) => {
  try {
    const { projectName } = req.params;
    const result = await projectService.buildProject(projectName);
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
 * Create new project
 */
router.post('/create', async (req, res) => {
  try {
    const projectData = req.body;
    
    // Validation
    if (!projectData.name || !projectData.runtime || !projectData.version) {
      return res.status(400).json({
        success: false,
        message: 'Missing required fields: name, runtime, version'
      });
    }
    
    // Create project
    const result = await projectService.createProject(projectData);
    
    res.json({
      success: true,
      message: 'Project created successfully',
      project: result
    });
  } catch (error) {
    console.error('Create project error:', error);
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
    
    const result = await projectService.deleteProject(name);
    
    res.json({
      success: true,
      message: result.message,
      project: { name: result.projectName }
    });
  } catch (error) {
    console.error('Delete project error:', error);
    res.status(500).json({
      success: false,
      message: error.message || 'Failed to delete project'
    });
  }
});

export default router;
