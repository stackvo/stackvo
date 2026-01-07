import express from 'express';
import { exec } from 'child_process';
import { promisify } from 'util';
import path from 'path';

const router = express.Router();
const execAsync = promisify(exec);

/**
 * POST /api/start-with-profile
 * Start services/projects using Docker Compose profiles
 * 
 * Body:
 *   - mode: 'minimal' | 'services' | 'projects' | 'all' | 'custom'
 *   - profiles: string[] (for custom mode)
 */
router.post('/', async (req, res) => {
  try {
    const { mode = 'minimal', profiles = [] } = req.body;
    
    // Get Stackvo root directory
    const stackvoRoot = process.env.STACKVO_ROOT || path.join(process.cwd(), '..', '..');
    
    // Build profile arguments
    let profileArgs = '';
    
    switch (mode) {
      case 'minimal':
        profileArgs = '--profile core';
        break;
      case 'services':
        profileArgs = '--profile core --profile services';
        break;
      case 'projects':
        profileArgs = '--profile core --profile projects';
        break;
      case 'all':
        profileArgs = '--profile core --profile services --profile projects';
        break;
      case 'custom':
        if (!profiles || profiles.length === 0) {
          return res.status(400).json({
            success: false,
            message: 'Custom mode requires profiles array'
          });
        }
        profileArgs = `--profile core ${profiles.map(p => `--profile ${p}`).join(' ')}`;
        break;
      default:
        return res.status(400).json({
          success: false,
          message: `Invalid mode: ${mode}`
        });
    }
    
    // Build docker-compose command
    const composeFiles = `-f ${stackvoRoot}/generated/stackvo.yml -f ${stackvoRoot}/generated/docker-compose.dynamic.yml -f ${stackvoRoot}/generated/docker-compose.projects.yml`;
    const command = `docker compose ${composeFiles} ${profileArgs} up -d --build --pull=missing --remove-orphans 2>&1`;
    
    console.log(`Starting with profile mode: ${mode}`);
    console.log(`Command: ${command}`);
    
    // Execute command
    const { stdout, stderr } = await execAsync(command, { cwd: stackvoRoot });
    
    // Log output
    if (stderr) {
      console.log('Docker Compose stderr:', stderr);
    }
    if (stdout) {
      console.log('Docker Compose stdout:', stdout);
    }
    
    res.json({
      success: true,
      message: `Services started successfully in ${mode} mode`,
      data: {
        mode,
        profiles: mode === 'custom' ? profiles : null,
        output: stdout
      }
    });
    
  } catch (error) {
    console.error('Start with profile error:', error);
    res.status(500).json({
      success: false,
      message: error.message,
      error: error.stderr || error.stdout || error.message
    });
  }
});

/**
 * POST /api/start-with-profile/service
 * Start a specific service using its profile
 * 
 * Body:
 *   - serviceName: string (e.g., 'mysql', 'redis')
 */
router.post('/service', async (req, res) => {
  try {
    const { serviceName } = req.body;
    
    if (!serviceName) {
      return res.status(400).json({
        success: false,
        message: 'serviceName is required'
      });
    }
    
    // Get Stackvo root directory
    const stackvoRoot = process.env.STACKVO_ROOT || path.join(process.cwd(), '..', '..');
    
    // Build docker-compose command with service profile
    const composeFiles = `-f ${stackvoRoot}/generated/stackvo.yml -f ${stackvoRoot}/generated/docker-compose.dynamic.yml -f ${stackvoRoot}/generated/docker-compose.projects.yml`;
    const command = `docker compose ${composeFiles} --profile core --profile ${serviceName} up -d --build --pull=missing --remove-orphans 2>&1`;
    
    console.log(`Starting service: ${serviceName}`);
    console.log(`Command: ${command}`);
    
    // Execute command
    const { stdout, stderr } = await execAsync(command, { cwd: stackvoRoot });
    
    // Log output
    if (stderr) {
      console.log('Docker Compose stderr:', stderr);
    }
    if (stdout) {
      console.log('Docker Compose stdout:', stdout);
    }
    
    res.json({
      success: true,
      message: `Service ${serviceName} started successfully`,
      data: {
        serviceName,
        output: stdout
      }
    });
    
  } catch (error) {
    console.error('Start service error:', error);
    res.status(500).json({
      success: false,
      message: error.message,
      error: error.stderr || error.stdout || error.message
    });
  }
});

/**
 * POST /api/start-with-profile/project
 * Start a specific project using its profile
 * 
 * Body:
 *   - projectName: string (e.g., 'project1', 'myproject')
 */
router.post('/project', async (req, res) => {
  try {
    const { projectName } = req.body;
    
    if (!projectName) {
      return res.status(400).json({
        success: false,
        message: 'projectName is required'
      });
    }
    
    // Get Stackvo root directory
    const stackvoRoot = process.env.STACKVO_ROOT || path.join(process.cwd(), '..', '..');
    
    // Build docker-compose command with project profile
    const composeFiles = `-f ${stackvoRoot}/generated/stackvo.yml -f ${stackvoRoot}/generated/docker-compose.dynamic.yml -f ${stackvoRoot}/generated/docker-compose.projects.yml`;
    const command = `docker compose ${composeFiles} --profile core --profile project-${projectName} up -d --build --pull=missing --remove-orphans 2>&1`;
    
    console.log(`Starting project: ${projectName}`);
    console.log(`Command: ${command}`);
    
    // Execute command
    const { stdout, stderr } = await execAsync(command, { cwd: stackvoRoot });
    
    // Log output
    if (stderr) {
      console.log('Docker Compose stderr:', stderr);
    }
    if (stdout) {
      console.log('Docker Compose stdout:', stdout);
    }
    
    res.json({
      success: true,
      message: `Project ${projectName} started successfully`,
      data: {
        projectName,
        output: stdout
      }
    });
    
  } catch (error) {
    console.error('Start project error:', error);
    res.status(500).json({
      success: false,
      message: error.message,
      error: error.stderr || error.stdout || error.message
    });
  }
});

export default router;
