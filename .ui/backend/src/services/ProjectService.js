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
  async createProject(projectData) {
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
    if (!/^[a-zA-Z0-9\-_]+$/.test(name)) {
      throw new Error('Invalid project name. Only alphanumeric, dash, and underscore allowed');
    }

    const projectsDir = path.join(process.cwd(), '..', '..', 'projects');
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
      const rootDir = path.join(process.cwd(), '..', '..');
      console.log(`Running: ./cli/stackvo.sh generate projects in ${rootDir}`);
      
      const { stdout, stderr } = await execAsync(
        './cli/stackvo.sh generate projects',
        { cwd: rootDir }
      );

      if (stderr) {
        console.error('Generate stderr:', stderr);
      }
      console.log('Generate stdout:', stdout);

      // 6. Build and start container (buildContainer does both: build + docker-compose up)
      console.log(`Building and starting container for project: ${name}`);
      const buildResult = await this.docker.buildContainer(name);
      
      if (!buildResult.success) {
        console.error('Build failed:', buildResult.message);
        throw new Error(`Failed to build container: ${buildResult.message}`);
      }
      
      console.log('Build and start successful:', buildResult.message);

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
    const projectsDir = path.join(process.cwd(), '..', '..', 'projects');
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
    const projectsDir = path.join(process.cwd(), '..', '..', 'projects');
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
      } catch (containerError) {
        console.warn(`Could not remove container ${containerName}:`, containerError.message);
        // Continue even if container doesn't exist
      }

      // 2. Delete project directory
      console.log(`Deleting project directory: ${projectPath}`);
      await fs.rm(projectPath, { recursive: true, force: true });
      console.log(`Project directory deleted: ${projectPath}`);

      // 3. Regenerate docker-compose.projects.yml
      const rootDir = path.join(process.cwd(), '..', '..');
      console.log(`Running: ./cli/stackvo.sh generate projects in ${rootDir}`);
      
      const { stdout, stderr } = await execAsync(
        './cli/stackvo.sh generate projects',
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
