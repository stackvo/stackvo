import Docker from 'dockerode';
import NodeCache from 'node-cache';

class DockerService {
  constructor() {
    this.docker = new Docker({ socketPath: process.env.DOCKER_SOCKET || '/var/run/docker.sock' });
    this.cache = new NodeCache({ stdTTL: parseInt(process.env.CACHE_TTL) || 5 });
  }

  /**
   * List all Stackvo services (from .env SERVICE_ prefix)
   */
  async listServices() {
    const cacheKey = 'services';
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    const fs = await import('fs/promises');
    const path = await import('path');
    
    // Read .env file to get all SERVICE_ definitions
    const envPath = path.join(process.cwd(), '..', '..', '.env');
    const envContent = await fs.readFile(envPath, 'utf-8');
    
    // Parse SERVICE_*_ENABLE lines
    const serviceRegex = /SERVICE_([A-Z0-9_]+)_ENABLE=(true|false)/g;
    const matches = [...envContent.matchAll(serviceRegex)];
    
    // Parse SERVICE_*_URL lines
    const urlRegex = /SERVICE_([A-Z0-9_]+)_URL=(.+)/g;
    const urlMatches = [...envContent.matchAll(urlRegex)];
    const serviceUrls = {};
    urlMatches.forEach(match => {
      const serviceName = match[1].toLowerCase().replace(/_/g, '-');
      let url = match[2].trim();
      
      // If URL doesn't start with http/https, assume it's just the service name
      // and construct full URL: https://{serviceName}.stackvo.loc
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = `https://${url}.stackvo.loc`;
      }
      
      serviceUrls[serviceName] = url;
    });
    
    // Parse all SERVICE_* variables for credentials
    // Use non-greedy match for service name to correctly parse SERVICE_ACTIVEMQ_ADMIN_USER
    const allServiceVarsRegex = /SERVICE_([A-Z0-9]+?)_([A-Z0-9_]+)=(.+)/g;
    const allMatches = [...envContent.matchAll(allServiceVarsRegex)];
    const serviceCredentials = {};
    
    allMatches.forEach(match => {
      const serviceName = match[1].toLowerCase().replace(/_/g, '-');
      const key = match[2]; // e.g., ROOT_PASSWORD, USER, DATABASE, ADMIN_USER, HOST_PORT_UI
      const value = match[3].trim();
      
      // Skip only ENABLE, VERSION, URL (these are not credentials)
      if (key === 'ENABLE' || key === 'VERSION' || key === 'URL') {
        return;
      }
      
      if (!serviceCredentials[serviceName]) {
        serviceCredentials[serviceName] = {};
      }
      
      serviceCredentials[serviceName][key] = value;
    });
    
    // Get all containers
    const containers = await this.docker.listContainers({ all: true });
    
    // Read /etc/hosts to check DNS configuration
    let hostsContent = '';
    try {
      hostsContent = await fs.readFile('/etc/hosts', 'utf-8');
    } catch (error) {
      console.warn('Could not read /etc/hosts:', error.message);
    }
    
    // Build services list from .env with async port formatting
    const servicesPromises = matches.map(async match => {
      const serviceName = match[1].toLowerCase().replace(/_/g, '-');
      const enabled = match[2] === 'true';
      const containerName = `stackvo-${serviceName}`;
      const url = serviceUrls[serviceName] || null;
      
      // Check if domain is in /etc/hosts
      let dns_configured = false;
      if (url && hostsContent) {
        const domain = url.replace('https://', '').replace('http://', '').split('/')[0];
        dns_configured = hostsContent.includes(domain);
      }
      
      // Find corresponding container
      const container = containers.find(c => 
        c.Names[0] === `/${containerName}` || c.Names[0].includes(containerName)
      );
      
      // Get detailed port info if container exists and is running
      let ports = { ports: {}, ip_address: null, network: null, gateway: null };
      if (container && container.State === 'running') {
        ports = await this.getDetailedPorts(container.Id);
      }
      
      return {
        name: serviceName,
        containerName: containerName,
        enabled: enabled,
        url: url,
        domain: url ? url.replace('https://', '').replace('http://', '').split('/')[0] : null,
        dns_configured: dns_configured,
        status: container ? container.State : 'not created',
        running: container ? container.State === 'running' : false,
        ports: ports,
        image: container ? container.Image : '-',
        created: container ? container.Created : null,
        id: container ? container.Id : null,
        credentials: serviceCredentials[serviceName] || {}
      };
    });
    
    const services = await Promise.all(servicesPromises);

    // Sort services: Running first, then Enabled, then Disabled
    services.sort((a, b) => {
      // Running services first
      if (a.running && !b.running) return -1;
      if (!a.running && b.running) return 1;
      
      // Then enabled services
      if (a.enabled && !b.enabled) return -1;
      if (!a.enabled && b.enabled) return 1;
      
      // Finally sort by name
      return a.name.localeCompare(b.name);
    });

    this.cache.set(cacheKey, services);
    return services;
  }

  /**
   * List all Stackvo tools
   */
  async listTools() {
    const cacheKey = 'tools_list';
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    const fs = await import('fs/promises');
    const path = await import('path');
    
    // Read .env file to get all TOOLS_ definitions
    const envPath = path.join(process.cwd(), '..', '..', '.env');
    const envContent = await fs.readFile(envPath, 'utf-8');
    
    // Parse TOOLS_*_ENABLE lines (only lines starting with TOOLS_)
    const toolRegex = /^TOOLS_([A-Z0-9_]+)_ENABLE=(true|false)/gm;
    const matches = [...envContent.matchAll(toolRegex)];
    
    // Parse TOOLS_*_URL lines
    const urlRegex = /^TOOLS_([A-Z0-9_]+)_URL=(.+)/gm;
    const urlMatches = [...envContent.matchAll(urlRegex)];
    const toolUrls = {};
    urlMatches.forEach(match => {
      const toolName = match[1].toLowerCase().replace(/_/g, '-');
      let url = match[2].trim();
      
      // If URL doesn't start with http/https, construct full URL
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        url = `https://${url}.stackvo.loc`;
      }
      
      toolUrls[toolName] = url;
    });
    
    // Parse TOOLS_*_VERSION lines
    const versionRegex = /^TOOLS_([A-Z0-9_]+)_VERSION=(.+)/gm;
    const versionMatches = [...envContent.matchAll(versionRegex)];
    const toolVersions = {};
    versionMatches.forEach(match => {
      const toolName = match[1].toLowerCase().replace(/_/g, '-');
      toolVersions[toolName] = match[2].trim();
    });
    
    // Get all containers
    const containers = await this.docker.listContainers({ all: true });
    
    // Check /etc/hosts for DNS configuration
    let hostsContent = '';
    try {
      hostsContent = await fs.readFile('/etc/hosts', 'utf-8');
    } catch (error) {
      // Ignore if can't read /etc/hosts
    }
    
    const tools = await Promise.all(matches.map(async match => {
      const toolName = match[1].toLowerCase().replace(/_/g, '-');
      const enabled = match[2] === 'true';
      const containerName = `stackvo-${toolName}`;
      const url = toolUrls[toolName] || null;
      const domain = url ? url.replace('https://', '').replace('http://', '').split('/')[0] : null;
      
      // Check if domain is in /etc/hosts
      const dns_configured = domain ? hostsContent.includes(domain) : false;
      
      // Find container
      const container = containers.find(c => 
        c.Names.some(name => name === `/${containerName}` || name === containerName)
      );
      
      // Get detailed port info if container exists and is running
      let ports = { ports: {}, ip_address: null, network: null, gateway: null };
      if (container && container.State === 'running') {
        ports = await this.getDetailedPorts(container.Id);
      }
      
      return {
        name: toolName,
        containerName: containerName,
        enabled: enabled,
        version: toolVersions[toolName] || 'latest',
        url: url,
        domain: domain,
        dns_configured: dns_configured,
        status: container ? container.State : 'not created',
        running: container ? container.State === 'running' : false,
        ports: ports,
        image: container ? container.Image : '-',
        created: container ? container.Created : null,
        id: container ? container.Id : null
      };
    }));
    
    // Sort: Running first, then Enabled, then Disabled
    tools.sort((a, b) => {
      if (a.running && !b.running) return -1;
      if (!a.running && b.running) return 1;
      if (a.enabled && !b.enabled) return -1;
      if (!a.enabled && b.enabled) return 1;
      return a.name.localeCompare(b.name);
    });

    this.cache.set(cacheKey, tools);
    return tools;
  }

  /**
   * List all Stackvo projects
   */
  async listProjects() {
    const cacheKey = 'projects';
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    const fs = await import('fs/promises');
    const path = await import('path');
    
    // Get project directories - backend runs in .ui/backend, projects are 2 levels up
    const projectsDir = path.join(process.cwd(), '..', '..', 'projects');
    let projectDirs = [];
    
    try {
      projectDirs = await fs.readdir(projectsDir);
    } catch (error) {
      console.error('Projects directory not found:', projectsDir, error.message);
      return [];
    }

    // Get all containers
    const containers = await this.docker.listContainers({ all: true });
    
    const projects = [];

    for (const dir of projectDirs) {
      const projectDirPath = path.join(projectsDir, dir);
      
      // Check if it's a directory
      try {
        const stat = await fs.stat(projectDirPath);
        if (!stat.isDirectory()) continue;
      } catch (error) {
        continue;
      }

      // Read stackvo.json
      const configPath = path.join(projectDirPath, 'stackvo.json');
      let config = {};
      
      try {
        const configContent = await fs.readFile(configPath, 'utf-8');
        config = JSON.parse(configContent);
      } catch (error) {
        // If stackvo.json doesn't exist or is invalid, skip or use minimal info
        projects.push({
          name: dir,
          domain: null,
          php: null,
          webserver: null,
          document_root: null,
          running: false,
          container_exists: false,
          error: 'Configuration file not found or invalid'
        });
        continue;
      }

      const projectName = config.name || dir;
      const containerName = `stackvo-${projectName}`;

      // Find container for this project
      const container = containers.find(c => c.Names[0].includes(containerName));
      
      const running = container ? container.State === 'running' : false;
      const containerExists = !!container;

      // SSL and URLs
      const sslEnabled = process.env.SSL_ENABLE === 'true';
      const urls = {
        https: config.domain ? `https://${config.domain}` : null,
        http: config.domain ? `http://${config.domain}` : null,
        primary: config.domain ? (sslEnabled ? `https://${config.domain}` : `http://${config.domain}`) : null
      };

      // Project paths
      const projectPath = {
        container_path: '/var/www/html',
        host_path: `projects/${dir}`
      };

      // Log paths (if project is running and logs directory exists)
      const webserver = config.webserver || 'nginx';
      const webserverPaths = {
        'nginx': '/var/log/nginx',
        'apache': '/var/log/apache2',
        'caddy': '/var/log/caddy'
      };
      const webLogBase = webserverPaths[webserver] || '/var/log/nginx';
      const phpLogBase = `/var/log/${projectName}`;

      const logs = running ? {
        web_access: {
          container_path: `${webLogBase}/access.log`,
          host_path: `logs/projects/${projectName}/access.log`
        },
        web_error: {
          container_path: `${webLogBase}/error.log`,
          host_path: `logs/projects/${projectName}/error.log`
        },
        php_error: {
          container_path: `${phpLogBase}/php-error.log`,
          host_path: `logs/projects/${projectName}/php-error.log`
        }
      } : null;

      // Check for custom configuration files in .stackvo directory
      const stackvoDir = path.join(projectDirPath, '.stackvo');
      let configuration = {
        type: 'default',
        has_custom: false,
        files: []
      };

      try {
        const stackvoDirExists = await fs.stat(stackvoDir);
        if (stackvoDirExists.isDirectory()) {
          const possibleConfigs = {
            'nginx': ['nginx.conf', 'default.conf'],
            'apache': ['apache.conf', 'httpd.conf'],
            'caddy': ['Caddyfile'],
            'ferron': ['ferron.yaml', 'ferron.conf']
          };

          const configFiles = [];

          // Check webserver-specific configs
          if (possibleConfigs[webserver]) {
            for (const configFile of possibleConfigs[webserver]) {
              try {
                await fs.stat(path.join(stackvoDir, configFile));
                configFiles.push(configFile);
              } catch (error) {
                // File doesn't exist, continue
              }
            }
          }

          // Check for PHP configs
          try {
            await fs.stat(path.join(stackvoDir, 'php.ini'));
            configFiles.push('php.ini');
          } catch (error) {
            // File doesn't exist
          }

          try {
            await fs.stat(path.join(stackvoDir, 'php-fpm.conf'));
            configFiles.push('php-fpm.conf');
          } catch (error) {
            // File doesn't exist
          }

          if (configFiles.length > 0) {
            configuration = {
              type: 'custom',
              has_custom: true,
              files: configFiles
            };
          }
        }
      } catch (error) {
        // .stackvo directory doesn't exist, use default
      }

      // Check if domain is configured in DNS/hosts
      let dnsConfigured = false;
      if (config.domain) {
        try {
          const dns = await import('dns/promises');
          await dns.lookup(config.domain);
          dnsConfigured = true;
        } catch (error) {
          // Domain not configured in DNS/hosts
          dnsConfigured = false;
        }
      }

      projects.push({
        name: projectName,
        domain: config.domain || null,
        dns_configured: dnsConfigured,
        ssl_enabled: sslEnabled,
        urls,
        php: config.php || null,
        nodejs: config.nodejs || null,
        python: config.python || null,
        ruby: config.ruby || null,
        golang: config.golang || null,
        webserver: config.webserver || null,
        document_root: config.document_root || null,
        running,
        container_exists: containerExists,
        containerName: container ? container.Names[0].replace('/', '') : containerName,
        status: container ? container.State : 'not created',
        image: container ? container.Image : null,
        created: container ? container.Created : null,
        id: container ? container.Id : null,
        ports: container && running ? await this.getDetailedPorts(container.Id) : { ports: {}, ip_address: null, network: null, gateway: null },
        logs,
        project_path: projectPath,
        containers: {
          main: {
            name: containerName,
            running,
            exists: containerExists
          }
        },
        configuration,
        error: null
      });
    }

    // Sort by running status first (running first), then by name
    projects.sort((a, b) => {
      // First sort by running status (running projects first)
      if (a.running && !b.running) return -1;
      if (!a.running && b.running) return 1;
      
      // Then sort alphabetically by name
      return a.name.localeCompare(b.name);
    });

    this.cache.set(cacheKey, projects);
    return projects;
  }

  /**
   * Get container statistics
   */
  async getContainerStats(containerName) {
    const container = this.docker.getContainer(containerName);
    const stats = await container.stats({ stream: false });
    
    return {
      cpu: this.calculateCPUPercent(stats),
      memory: this.calculateMemoryUsage(stats),
      network: stats.networks
    };
  }

  /**
   * Start a container
   */
  async startContainer(containerName) {
    const container = this.docker.getContainer(containerName);
    await container.start();
    this.cache.flushAll();
    return { success: true, message: `Container ${containerName} started` };
  }

  /**
   * Stop a container
   */
  async stopContainer(containerName) {
    const container = this.docker.getContainer(containerName);
    await container.stop();
    this.cache.flushAll();
    return { success: true, message: `Container ${containerName} stopped` };
  }

  /**
   * Restart a container
   */
  async restartContainer(containerName) {
    const container = this.docker.getContainer(containerName);
    await container.restart();
    this.cache.flushAll();
    return { success: true, message: `Container ${containerName} restarted` };
  }

  /**
   * Format port mappings (simple format for backward compatibility)
   */
  formatPorts(ports) {
    if (!ports) return [];
    return ports.map(p => ({
      private: p.PrivatePort,
      public: p.PublicPort || null,
      type: p.Type
    }));
  }

  /**
   * Get detailed port mappings with network info (old UI format)
   */
  async getDetailedPorts(containerId) {
    try {
      const container = this.docker.getContainer(containerId);
      const inspect = await container.inspect();
      
      const networkSettings = inspect.NetworkSettings;
      const ports = inspect.NetworkSettings.Ports || {};
      
      // Format ports object
      const formattedPorts = {};
      Object.keys(ports).forEach(key => {
        const portBindings = ports[key];
        if (portBindings && portBindings.length > 0) {
          formattedPorts[key] = {
            docker_port: key,
            host_ip: portBindings[0].HostIp || '0.0.0.0',
            host_port: portBindings[0].HostPort,
            exposed: true
          };
        } else {
          formattedPorts[key] = {
            docker_port: key,
            exposed: false
          };
        }
      });
      
      return {
        ports: formattedPorts,
        ip_address: networkSettings.IPAddress || null,
        network: Object.keys(networkSettings.Networks)[0] || null,
        gateway: networkSettings.Gateway || (networkSettings.Networks[Object.keys(networkSettings.Networks)[0]]?.Gateway) || null
      };
    } catch (error) {
      return { ports: {}, ip_address: null, network: null, gateway: null };
    }
  }

  /**
   * Calculate CPU usage percentage
   */
  calculateCPUPercent(stats) {
    const cpuDelta = stats.cpu_stats.cpu_usage.total_usage - 
                     stats.precpu_stats.cpu_usage.total_usage;
    const systemDelta = stats.cpu_stats.system_cpu_usage - 
                        stats.precpu_stats.system_cpu_usage;
    const cpuCount = stats.cpu_stats.online_cpus || 1;
    
    if (systemDelta === 0) return '0.00';
    return ((cpuDelta / systemDelta) * cpuCount * 100).toFixed(2);
  }

  /**
   * Calculate memory usage
   */
  calculateMemoryUsage(stats) {
    const used = stats.memory_stats.usage || 0;
    const limit = stats.memory_stats.limit || 1;
    const percent = ((used / limit) * 100).toFixed(2);
    
    return {
      used: this.formatBytes(used),
      limit: this.formatBytes(limit),
      percent
    };
  }

  /**
   * Build project containers
   */
  async buildContainer(projectName) {
    const { execSync } = await import('child_process');
    const path = await import('path');
    
    try {
      const rootDir = path.join(process.cwd(), '..', '..');
      const composeFile = path.join(rootDir, 'generated', 'docker-compose.projects.yml');
      
      console.log(`Building container for project: ${projectName}`);
      
      // Stackvo uses single-container architecture
      const buildCommand = `docker-compose -f ${composeFile} build ${projectName}`;
      
      const buildOutput = execSync(buildCommand, {
        cwd: rootDir,
        encoding: 'utf-8',
        stdio: 'pipe'
      });
      
      console.log(`Build successful, creating container for: ${projectName}`);
      
      // Create and start container with docker-compose up
      const upCommand = `docker-compose -f ${composeFile} up -d --no-build ${projectName}`;
      
      const upOutput = execSync(upCommand, {
        cwd: rootDir,
        encoding: 'utf-8',
        stdio: 'pipe'
      });
      
      return {
        success: true,
        message: `Container built and started successfully for ${projectName}`,
        output: buildOutput + '\n' + upOutput
      };
    } catch (error) {
      console.error('Build error:', error);
      return {
        success: false,
        message: `Failed to build container: ${error.message}`,
        output: error.stdout || error.stderr || error.message
      };
    }
  }

  /**
   * Check if project containers are built
   */
  async isProjectBuilt(projectName) {
    try {
      const containers = await this.docker.listContainers({ all: true });
      const container = containers.find(c => 
        c.Names.some(name => name.includes(`stackvo-${projectName}`))
      );
      
      return !!container;
    } catch (error) {
      return false;
    }
  }

  /**
   * Format bytes to human readable
   */
  formatBytes(bytes) {
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 Bytes';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  }
}

export default DockerService;
