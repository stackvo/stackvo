import fs from 'fs/promises';
import path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

class ProjectService {
  constructor(dockerService) {
    this.docker = dockerService;
  }

  /**
   * Create a new project
   * @param {Object} projectData - Project configuration
   * @returns {Promise<Object>} Created project info
   */
  async createProject(projectData, io = null) {
    const { 
      name, 
      domain, 
      runtime, 
      version, 
      webserver, 
      document_root, 
      extensions 
    } = projectData;

    // Validate required fields
    if (!name || !runtime || !version) {
      throw new Error('Missing required fields: name, runtime, version');
    }

    // Validate project name
    if (!/^[a-zA-Z0-9\-_.]+$/.test(name)) {
      throw new Error('Invalid project name. Alphanumeric, dash, underscore, and dot allowed');
    }

    // Use PROJECTS_DIR environment variable or fallback to relative path
    const projectsDir = process.env.PROJECTS_DIR || path.join(process.cwd(), '..', '..', 'projects');
    const projectPath = path.join(projectsDir, name);

    // Check if project already exists
    try {
      await fs.access(projectPath);
      throw new Error(`Project "${name}" already exists`);
    } catch (error) {
      if (error.code !== 'ENOENT') {
        throw error;
      }
    }

    try {
      // 1. Create project directory
      await fs.mkdir(projectPath, { recursive: true });

      // 2. Create stackvo.json
      const config = {
        name,
        domain: domain || `${name}.loc`,
        webserver: webserver || 'nginx',
        document_root: document_root || 'public'
      };

      // Add runtime-specific config
      if (runtime === 'php') {
        config.php = {
          version,
          extensions: extensions || ['pdo', 'pdo_mysql', 'mysqli']
        };
      } else if (runtime === 'nodejs') {
        config.nodejs = { version };
      } else if (runtime === 'python') {
        config.python = { version };
      } else if (runtime === 'ruby') {
        config.ruby = { version };
      } else if (runtime === 'golang') {
        config.golang = { version };
      }

      await fs.writeFile(
        path.join(projectPath, 'stackvo.json'),
        JSON.stringify(config, null, 2)
      );

      // 3. Create document root directory
      const docRootPath = path.join(projectPath, document_root || 'public');
      await fs.mkdir(docRootPath, { recursive: true });

      // 3.5. Create .stackvo directory for build files
      const stackvoDir = path.join(projectPath, '.stackvo');
      await fs.mkdir(stackvoDir, { recursive: true });

      // 4. Create index file
      let indexFile, indexContent;
      
      if (runtime === 'php') {
        indexFile = 'index.php';
        indexContent = `<?php
/**
 * ${name} - Welcome Page
 */
?>
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to ${name}</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .info { background: #f4f4f4; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>🎉 Welcome to ${name}!</h1>
    <div class="info">
        <p><strong>Project:</strong> ${name}</p>
        <p><strong>Domain:</strong> ${config.domain}</p>
        <p><strong>PHP Version:</strong> <?php echo phpversion(); ?></p>
        <p><strong>Document Root:</strong> ${document_root || 'public'}</p>
    </div>
    <p>Your project is ready! Start building something amazing.</p>
</body>
</html>
`;
      } else {
        indexFile = 'index.html';
        indexContent = `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Welcome to ${name}</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        h1 { color: #333; }
        .info { background: #f4f4f4; padding: 15px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>🎉 Welcome to ${name}!</h1>
    <div class="info">
        <p><strong>Project:</strong> ${name}</p>
        <p><strong>Domain:</strong> ${config.domain}</p>
        <p><strong>Runtime:</strong> ${runtime} ${version}</p>
        <p><strong>Document Root:</strong> ${document_root || 'public'}</p>
    </div>
    <p>Your project is ready! Start building something amazing.</p>
</body>
</html>
`;
      }

      await fs.writeFile(
        path.join(docRootPath, indexFile),
        indexContent
      );

      // 5. Run generate command
      // Use STACKVO_ROOT environment variable or fallback to relative path
      const rootDir = process.env.STACKVO_ROOT || path.join(process.cwd(), '..', '..');
      const cliScript = path.join(rootDir, 'core', 'cli', 'stackvo.sh');
      console.log(`Running: ${cliScript} generate projects`);
      
      const { stdout, stderr } = await execAsync(
        `${cliScript} generate projects`,
        { cwd: rootDir }
      );

      if (stderr) {
        console.error('Generate stderr:', stderr);
      }
      console.log('Generate stdout:', stdout);

      // 5.5. Fix file ownership (UI container runs as root, host user needs access)
      // Get host user ID from environment or use default 1000
      const hostUid = process.env.HOST_UID || '1000';
      const hostGid = process.env.HOST_GID || '1000';
      
      try {
        console.log(`Fixing file ownership for ${projectPath} to ${hostUid}:${hostGid}`);
        const { stdout: _chownStdout, stderr: chownStderr } = await execAsync(
          `chown -R ${hostUid}:${hostGid} ${projectPath}`,
          { cwd: rootDir }
        );
        
        if (chownStderr) {
          console.warn('Chown stderr:', chownStderr);
        }
        console.log('File ownership fixed successfully');
      } catch (chownError) {
        console.error('Failed to fix file ownership:', chownError.message);
        // Don't throw error, just log warning - project can still work
      }

      // 6. Build and start container in background
      console.log(`Building and starting container for project: ${name} (Background)`);
      
      // Don't await build - let it run in background and emit socket events
      this.docker.buildContainer(name, io).catch(buildError => {
        console.error('Background build failed:', buildError);
        if (io) {
          io.emit('project:error', { 
            project: name, 
            error: `Build failed: ${buildError.message}` 
          });
        }
      });
      
      console.log('Project definitions created, build started in background');

      return {
        name,
        domain: config.domain,
        path: `projects/${name}`,
        runtime,
        version,
        webserver: config.webserver
      };
    } catch (error) {
      // Cleanup on error
      try {
        await fs.rm(projectPath, { recursive: true, force: true });
      } catch (cleanupError) {
        console.error('Cleanup error:', cleanupError);
      }
      throw error;
    }
  }

  /**
   * Build project containers
   * @param {string} projectName - Project name
   * @returns {Promise<Object>} Build result
   */
  async buildProject(projectName) {
    // Use PROJECTS_DIR environment variable or fallback to relative path
    const projectsDir = process.env.PROJECTS_DIR || path.join(process.cwd(), '..', '..', 'projects');
    const projectPath = path.join(projectsDir, projectName);

    // Check if project exists
    try {
      await fs.access(projectPath);
    } catch (error) {
      throw new Error(`Project "${projectName}" does not exist`);
    }

    // Build and start container (buildContainer does both)
    const buildResult = await this.docker.buildContainer(projectName);
    
    if (!buildResult.success) {
      throw new Error(buildResult.message);
    }

    return {
      success: true,
      message: buildResult.message,
      projectName
    };
  }

  /**
   * Delete a project
   * @param {string} projectName - Project name
   * @returns {Promise<Object>} Deletion result
   */
  async deleteProject(projectName) {
    // Use PROJECTS_DIR environment variable or fallback to relative path
    const projectsDir = process.env.PROJECTS_DIR || path.join(process.cwd(), '..', '..', 'projects');
    const projectPath = path.join(projectsDir, projectName);

    // Check if project exists
    try {
      await fs.access(projectPath);
    } catch (error) {
      throw new Error(`Project "${projectName}" does not exist`);
    }

    try {
      // 1. Stop and remove container
      const containerName = `stackvo-${projectName}`;
      try {
        const container = this.docker.docker.getContainer(containerName);
        
        // Stop container if running
        try {
          await container.stop();
          console.log(`Container ${containerName} stopped`);
        } catch (stopError) {
          // Container might already be stopped
          console.log(`Container ${containerName} already stopped or not running`);
        }
        
        // Remove container
        await container.remove();
        console.log(`Container ${containerName} removed`);
        
        // Remove Docker image
        const imageName = `stackvo-${projectName}:latest`;
        try {
          const image = this.docker.docker.getImage(imageName);
          await image.remove({ force: true });
          console.log(`Docker image ${imageName} removed`);
        } catch (imageError) {
          console.warn(`Could not remove image ${imageName}:`, imageError.message);
          // Continue even if image doesn't exist
        }
      } catch (containerError) {
        console.warn(`Could not remove container ${containerName}:`, containerError.message);
        // Continue even if container doesn't exist
      }

      // 2. Delete project directory
      console.log(`Deleting project directory: ${projectPath}`);
      await fs.rm(projectPath, { recursive: true, force: true });
      console.log(`Project directory deleted: ${projectPath}`);

      // 3. Regenerate docker-compose.projects.yml
      // Use STACKVO_ROOT environment variable or fallback to relative path
      const rootDir = process.env.STACKVO_ROOT || path.join(process.cwd(), '..', '..');
      const cliScript = path.join(rootDir, 'core', 'cli', 'stackvo.sh');
      console.log(`Running: ${cliScript} generate projects`);
      
      const { stdout, stderr } = await execAsync(
        `${cliScript} generate projects`,
        { cwd: rootDir }
      );

      if (stderr) {
        console.error('Generate stderr:', stderr);
      }
      console.log('Generate stdout:', stdout);

      // 4. Clear cache to force reload
      this.docker.cache.flushAll();
      console.log('Cache cleared');

      return {
        success: true,
        message: `Project "${projectName}" deleted successfully`,
        projectName
      };
    } catch (error) {
      throw new Error(`Failed to delete project: ${error.message}`);
    }
  }
}

export default ProjectService;
