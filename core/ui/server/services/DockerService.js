import Docker from "dockerode";
import NodeCache from "node-cache";
import { exec } from "child_process";
import { promisify } from "util";

const execAsync = promisify(exec);

class DockerService {
  constructor() {
    this.docker = new Docker({
      socketPath: process.env.DOCKER_SOCKET || "/var/run/docker.sock",
    });
    this.cache = new NodeCache({
      stdTTL: parseInt(process.env.CACHE_TTL) || 5,
    });
  }

  /**
   * List all Stackvo services (from .env SERVICE_ prefix)
   */
  async listServices() {
    const cacheKey = "services";
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    const fs = await import("fs/promises");
    const path = await import("path");

    // Read .env file to get all SERVICE_ definitions
    // Use STACKVO_ROOT environment variable or fallback to relative path
    const stackvoRoot = process.env.STACKVO_ROOT || path.join(process.cwd(), "..", "..");
    const envPath = path.join(stackvoRoot, ".env");
    const envContent = await fs.readFile(envPath, "utf-8");

    // Parse SERVICE_*_ENABLE lines
    const serviceRegex = /SERVICE_([A-Z0-9_]+)_ENABLE=(true|false)/g;
    const matches = [...envContent.matchAll(serviceRegex)];

    // Parse SERVICE_*_URL lines
    const urlRegex = /SERVICE_([A-Z0-9_]+)_URL=(.+)/g;
    const urlMatches = [...envContent.matchAll(urlRegex)];
    const serviceUrls = {};
    urlMatches.forEach((match) => {
      const serviceName = match[1].toLowerCase().replace(/_/g, "-");
      let url = match[2].trim();

      // If URL doesn't start with http/https, assume it's just the service name
      // and construct full URL: https://{serviceName}.stackvo.loc
      if (!url.startsWith("http://") && !url.startsWith("https://")) {
        url = `https://${url}.stackvo.loc`;
      }

      serviceUrls[serviceName] = url;
    });

    // Parse all SERVICE_* variables for credentials
    // Use non-greedy match for service name to correctly parse SERVICE_ACTIVEMQ_ADMIN_USER
    const allServiceVarsRegex = /SERVICE_([A-Z0-9]+?)_([A-Z0-9_]+)=(.+)/g;
    const allMatches = [...envContent.matchAll(allServiceVarsRegex)];
    const serviceCredentials = {};

    allMatches.forEach((match) => {
      const serviceName = match[1].toLowerCase().replace(/_/g, "-");
      const key = match[2]; // e.g., ROOT_PASSWORD, USER, DATABASE, ADMIN_USER, HOST_PORT_UI
      const value = match[3].trim();

      // Skip only ENABLE, VERSION, URL (these are not credentials)
      if (key === "ENABLE" || key === "VERSION" || key === "URL") {
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
    let hostsContent = "";
    try {
      hostsContent = await fs.readFile("/etc/hosts", "utf-8");
    } catch (error) {
      console.warn("Could not read /etc/hosts:", error.message);
    }

    // Build services list from .env with async port formatting
    const servicesPromises = matches.map(async (match) => {
      const serviceName = match[1].toLowerCase().replace(/_/g, "-");
      const enabled = match[2] === "true";
      const containerName = `stackvo-${serviceName}`;
      const url = serviceUrls[serviceName] || null;

      // Check if domain is in /etc/hosts
      let dns_configured = false;
      if (url && hostsContent) {
        const domain = url
          .replace("https://", "")
          .replace("http://", "")
          .split("/")[0];
        dns_configured = hostsContent.includes(domain);
      }

      // Find corresponding container
      const container = containers.find(
        (c) =>
          c.Names[0] === `/${containerName}` ||
          c.Names[0].includes(containerName)
      );

      // Get detailed port info if container exists and is running
      let ports = { ports: {}, ip_address: null, network: null, gateway: null };
      if (container && container.State === "running") {
        ports = await this.getDetailedPorts(container.Id);
      }

      // Get version from .env if container doesn't exist
      let imageVersion = "-";
      if (container) {
        imageVersion = container.Image;
      } else {
        // Read version from .env
        const versionKey = `SERVICE_${serviceName
          .toUpperCase()
          .replace(/-/g, "_")}_VERSION`;
        const versionMatch = envContent.match(
          new RegExp(`^${versionKey}=(.+)$`, "m")
        );
        if (versionMatch) {
          // Get service name for image (e.g., redis, mysql, etc.)
          const baseServiceName = serviceName.replace("stackvo-", "");
          imageVersion = `${baseServiceName}:${versionMatch[1]}`;
        }
      }

      return {
        name: serviceName,
        containerName: containerName,
        configured: enabled,  // .env'de SERVICE_*_ENABLE=true olup olmadığı
        url: url,
        domain: url
          ? url.replace("https://", "").replace("http://", "").split("/")[0]
          : null,
        dns_configured: dns_configured,
        status: container ? container.State : "not created",
        running: container ? container.State === "running" : false,
        ports: ports,
        image: imageVersion,
        created: container ? container.Created : null,
        id: container ? container.Id : null,
        credentials: serviceCredentials[serviceName] || {},
      };
    });

    const services = await Promise.all(servicesPromises);

    // Sort services: Running first, then Configured, then Disabled
    services.sort((a, b) => {
      // Running services first
      if (a.running && !b.running) return -1;
      if (!a.running && b.running) return 1;

      // Then configured services
      if (a.configured && !b.configured) return -1;
      if (!a.configured && b.configured) return 1;

      // Finally sort by name
      return a.name.localeCompare(b.name);
    });

    this.cache.set(cacheKey, services);
    return services;
  }

  /**
   * Check if a service is running
   * @param {string} serviceName - Service name (e.g., 'elasticsearch', 'kibana')
   * @returns {Promise<boolean>}
   */
  async isServiceRunning(serviceName) {
    try {
      const containerName = `stackvo-${serviceName}`;
      const containers = await this.docker.listContainers({ all: true });
      
      const container = containers.find(
        (c) => c.Names[0] === `/${containerName}` || c.Names[0].includes(containerName)
      );
      
      return container ? container.State === 'running' : false;
    } catch (error) {
      console.error(`Error checking service ${serviceName}:`, error.message);
      return false;
    }
  }

  /**
   * List all Stackvo tools
   */
  async listTools() {
    const cacheKey = "tools_list";
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    const fs = await import("fs/promises");
    const path = await import("path");

    // Read .env file to get all TOOLS_ definitions
    // Use STACKVO_ROOT environment variable or fallback to relative path
    const stackvoRoot = process.env.STACKVO_ROOT || path.join(process.cwd(), "..", "..");
    const envPath = path.join(stackvoRoot, ".env");
    const envContent = await fs.readFile(envPath, "utf-8");

    // Parse TOOLS_*_ENABLE lines (only lines starting with TOOLS_)
    const toolRegex = /^TOOLS_([A-Z0-9_]+)_ENABLE=(true|false)/gm;
    const matches = [...envContent.matchAll(toolRegex)];

    // Parse TOOLS_*_URL lines
    const urlRegex = /^TOOLS_([A-Z0-9_]+)_URL=(.+)/gm;
    const urlMatches = [...envContent.matchAll(urlRegex)];
    const toolUrls = {};
    urlMatches.forEach((match) => {
      const toolName = match[1].toLowerCase().replace(/_/g, "-");
      let url = match[2].trim();

      // If URL doesn't start with http/https, construct full URL
      if (!url.startsWith("http://") && !url.startsWith("https://")) {
        url = `https://${url}.stackvo.loc`;
      }

      toolUrls[toolName] = url;
    });

    // Parse TOOLS_*_VERSION lines
    const versionRegex = /^TOOLS_([A-Z0-9_]+)_VERSION=(.+)/gm;
    const versionMatches = [...envContent.matchAll(versionRegex)];
    const toolVersions = {};
    versionMatches.forEach((match) => {
      const toolName = match[1].toLowerCase().replace(/_/g, "-");
      toolVersions[toolName] = match[2].trim();
    });

    // Get all containers
    const containers = await this.docker.listContainers({ all: true });

    // Check /etc/hosts for DNS configuration
    let hostsContent = "";
    try {
      hostsContent = await fs.readFile("/etc/hosts", "utf-8");
    } catch (error) {
      // Ignore if can't read /etc/hosts
    }

    const tools = await Promise.all(
      matches.map(async (match) => {
        const toolName = match[1].toLowerCase().replace(/_/g, "-");
        const enabled = match[2] === "true";
        const containerName = `stackvo-${toolName}`;
        const url = toolUrls[toolName] || null;
        const domain = url
          ? url.replace("https://", "").replace("http://", "").split("/")[0]
          : null;

        // Check if domain is in /etc/hosts
        const dns_configured = domain ? hostsContent.includes(domain) : false;

        // Find container
        const container = containers.find((c) =>
          c.Names.some(
            (name) => name === `/${containerName}` || name === containerName
          )
        );

        // Get detailed port info if container exists and is running
        let ports = {
          ports: {},
          ip_address: null,
          network: null,
          gateway: null,
        };
        if (container && container.State === "running") {
          ports = await this.getDetailedPorts(container.Id);
        }

        return {
          name: toolName,
          containerName: containerName,
          configured: enabled,  // .env'de TOOLS_*_ENABLE=true olup olmadığı
          version: toolVersions[toolName] || "latest",
          url: url,
          domain: domain,
          dns_configured: dns_configured,
          status: container ? container.State : "not created",
          running: container ? container.State === "running" : false,
          ports: ports,
          image: container ? container.Image : "-",
          created: container ? container.Created : null,
          id: container ? container.Id : null,
        };
      })
    );

    // Sort: Running first, then Configured, then Disabled
    tools.sort((a, b) => {
      if (a.running && !b.running) return -1;
      if (!a.running && b.running) return 1;
      if (a.configured && !b.configured) return -1;
      if (!a.configured && b.configured) return 1;
      return a.name.localeCompare(b.name);
    });

    this.cache.set(cacheKey, tools);
    return tools;
  }

  /**
   * List all Stackvo projects
   */
  async listProjects() {
    const cacheKey = "projects";
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    const fs = await import("fs/promises");
    const path = await import("path");

    // Get projects directory from environment variable or fallback to relative path
    // PROJECTS_DIR is set in docker-compose.yml to /app/projects
    const projectsDir =
      process.env.PROJECTS_DIR ||
      path.join(process.cwd(), "..", "..", "projects");
    let projectDirs = [];

    try {
      projectDirs = await fs.readdir(projectsDir);
    } catch (error) {
      console.error(
        "Projects directory not found:",
        projectsDir,
        error.message
      );
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
      const configPath = path.join(projectDirPath, "stackvo.json");
      let config = {};

      try {
        const configContent = await fs.readFile(configPath, "utf-8");
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
          error: "Configuration file not found or invalid",
        });
        continue;
      }

      const projectName = config.name || dir;
      const containerName = `stackvo-${projectName}`;

      // Find container for this project
      const container = containers.find((c) =>
        c.Names[0].includes(containerName)
      );

      const running = container ? container.State === "running" : false;
      const containerExists = !!container;

      // SSL and URLs
      const sslEnabled = process.env.SSL_ENABLE === "true";
      const urls = {
        https: config.domain ? `https://${config.domain}` : null,
        http: config.domain ? `http://${config.domain}` : null,
        primary: config.domain
          ? sslEnabled
            ? `https://${config.domain}`
            : `http://${config.domain}`
          : null,
      };

      // Project paths
      const projectPath = {
        container_path: "/var/www/html",
        host_path: `projects/${dir}`,
      };

      // Log paths (if project is running and logs directory exists)
      const webserver = config.webserver || "nginx";
      const webserverPaths = {
        nginx: "/var/log/nginx",
        apache: "/var/log/apache2",
        caddy: "/var/log/caddy",
      };
      const webLogBase = webserverPaths[webserver] || "/var/log/nginx";
      const phpLogBase = `/var/log/${projectName}`;

      const logs = running
        ? {
            web_access: {
              container_path: `${webLogBase}/access.log`,
              host_path: `logs/projects/${projectName}/access.log`,
            },
            web_error: {
              container_path: `${webLogBase}/error.log`,
              host_path: `logs/projects/${projectName}/error.log`,
            },
            php_error: {
              container_path: `${phpLogBase}/php-error.log`,
              host_path: `logs/projects/${projectName}/php-error.log`,
            },
          }
        : null;

      // Check for custom configuration files in .stackvo directory
      const stackvoDir = path.join(projectDirPath, ".stackvo");
      let configuration = {
        type: "default",
        has_custom: false,
        files: [],
      };

      try {
        const stackvoDirExists = await fs.stat(stackvoDir);
        if (stackvoDirExists.isDirectory()) {
          const possibleConfigs = {
            nginx: ["nginx.conf", "default.conf"],
            apache: ["apache.conf", "httpd.conf"],
            caddy: ["Caddyfile"],
            ferron: ["ferron.yaml", "ferron.conf"],
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
            await fs.stat(path.join(stackvoDir, "php.ini"));
            configFiles.push("php.ini");
          } catch (error) {
            // File doesn't exist
          }

          try {
            await fs.stat(path.join(stackvoDir, "php-fpm.conf"));
            configFiles.push("php-fpm.conf");
          } catch (error) {
            // File doesn't exist
          }

          if (configFiles.length > 0) {
            configuration = {
              type: "custom",
              has_custom: true,
              files: configFiles,
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
          const dns = await import("dns/promises");
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
        containerName: container
          ? container.Names[0].replace("/", "")
          : containerName,
        status: container ? container.State : "not created",
        image: container ? container.Image : null,
        created: container ? container.Created : null,
        id: container ? container.Id : null,
        ports:
          container && running
            ? await this.getDetailedPorts(container.Id)
            : { ports: {}, ip_address: null, network: null, gateway: null },
        logs,
        project_path: projectPath,
        containers: {
          main: {
            name: containerName,
            running,
            exists: containerExists,
          },
        },
        configuration,
        error: null,
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
      network: stats.networks,
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
    return ports.map((p) => ({
      private: p.PrivatePort,
      public: p.PublicPort || null,
      type: p.Type,
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
      Object.keys(ports).forEach((key) => {
        const portBindings = ports[key];
        if (portBindings && portBindings.length > 0) {
          formattedPorts[key] = {
            docker_port: key,
            host_ip: portBindings[0].HostIp || "0.0.0.0",
            host_port: portBindings[0].HostPort,
            exposed: true,
          };
        } else {
          formattedPorts[key] = {
            docker_port: key,
            exposed: false,
          };
        }
      });

      return {
        ports: formattedPorts,
        ip_address: networkSettings.IPAddress || null,
        network: Object.keys(networkSettings.Networks)[0] || null,
        gateway:
          networkSettings.Gateway ||
          networkSettings.Networks[Object.keys(networkSettings.Networks)[0]]
            ?.Gateway ||
          null,
      };
    } catch (error) {
      return { ports: {}, ip_address: null, network: null, gateway: null };
    }
  }

  /**
   * Calculate CPU usage percentage
   */
  calculateCPUPercent(stats) {
    const cpuDelta =
      stats.cpu_stats.cpu_usage.total_usage -
      stats.precpu_stats.cpu_usage.total_usage;
    const systemDelta =
      stats.cpu_stats.system_cpu_usage - stats.precpu_stats.system_cpu_usage;
    const cpuCount = stats.cpu_stats.online_cpus || 1;

    if (systemDelta === 0) return "0.00";
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
      percent,
    };
  }

  /**
   * Build project containers with realtime progress streaming
   * @param {string} projectName - Project name
   * @param {Object} io - Socket.io instance (optional)
   */
  async buildContainer(projectName, io = null) {
    const path = await import("path");
    const { spawn } = await import("child_process");

    try {
      // Use container path for compose file (we're running inside container)
      const containerRootDir = process.env.STACKVO_ROOT || "/app";
      const composeFile = path.join(
        containerRootDir,
        "generated",
        "docker-compose.projects.yml"
      );

      console.log(`Building container for project: ${projectName}`);
      console.log(`Using compose file: ${composeFile}`);

      // Emit start event
      if (io) {
        io.emit("build:start", { project: projectName });
      }

      // Build with streaming output
      const buildProcess = spawn("docker-compose", [
        "-f",
        composeFile,
        "build",
        projectName,
      ]);

      let buildOutput = "";
      let buildError = "";

      // Stream stdout
      buildProcess.stdout.on("data", (data) => {
        const output = data.toString();
        buildOutput += output;
        console.log(output);
        
        if (io) {
          io.emit("build:progress", {
            project: projectName,
            output: output,
            type: "stdout",
          });
        }
      });

      // Stream stderr
      buildProcess.stderr.on("data", (data) => {
        const output = data.toString();
        buildError += output;
        console.error(output);
        
        if (io) {
          io.emit("build:progress", {
            project: projectName,
            output: output,
            type: "stderr",
          });
        }
      });

      // Wait for build completion
      const buildExitCode = await new Promise((resolve) => {
        buildProcess.on("close", (code) => {
          resolve(code);
        });
      });

      if (buildExitCode !== 0) {
        throw new Error(
          `Build failed with exit code ${buildExitCode}: ${buildError}`
        );
      }

      console.log(`Build successful, creating container for: ${projectName}`);

      // Emit progress for container start
      if (io) {
        io.emit("build:progress", {
          project: projectName,
          output: "Build successful! Starting container...\n",
          type: "info",
        });
      }

      // Create and start container with docker-compose up
      const upProcess = spawn("docker-compose", [
        "-f",
        composeFile,
        "up",
        "-d",
        "--no-build",
        projectName,
      ]);

      let upOutput = "";
      let upError = "";

      upProcess.stdout.on("data", (data) => {
        const output = data.toString();
        upOutput += output;
        console.log(output);
        
        if (io) {
          io.emit("build:progress", {
            project: projectName,
            output: output,
            type: "stdout",
          });
        }
      });

      upProcess.stderr.on("data", (data) => {
        const output = data.toString();
        upError += output;
        console.error(output);
        
        if (io) {
          io.emit("build:progress", {
            project: projectName,
            output: output,
            type: "stderr",
          });
        }
      });

      // Wait for up completion
      const upExitCode = await new Promise((resolve) => {
        upProcess.on("close", (code) => {
          resolve(code);
        });
      });

      if (upExitCode !== 0) {
        throw new Error(
          `Container start failed with exit code ${upExitCode}: ${upError}`
        );
      }

      // Emit success
      if (io) {
        io.emit("build:success", {
          project: projectName,
          message: `Container built and started successfully`,
        });
      }

      return {
        success: true,
        message: `Container built and started successfully for ${projectName}`,
        output: buildOutput + "\n" + upOutput,
      };
    } catch (error) {
      console.error("Build error:", error);

      // Emit error
      if (io) {
        io.emit("build:error", {
          project: projectName,
          error: error.message,
        });
      }

      return {
        success: false,
        message: `Failed to build container: ${error.message}`,
        output: error.message,
      };
    }
  }

  /**
   * Check if project containers are built
   */
  async isProjectBuilt(projectName) {
    try {
      const containers = await this.docker.listContainers({ all: true });
      const container = containers.find((c) =>
        c.Names.some((name) => name.includes(`stackvo-${projectName}`))
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
    const sizes = ["Bytes", "KB", "MB", "GB"];
    if (bytes === 0) return "0 Bytes";
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round((bytes / Math.pow(1024, i)) * 100) / 100 + " " + sizes[i];
  }

  /**
   * Enable a service
   * @param {string} serviceName - Service name
   * @param {Object} envService - EnvService instance
   * @param {Object} io - Socket.io instance
   * @returns {Promise<Object>}
   */
  async enableService(serviceName, envService, io = null) {
    const { execAsync } = await import("../utils/exec.js");
    const path = await import("path");
    const fs = await import("fs/promises");

    const rootDir = process.env.STACKVO_ROOT || path.join(process.cwd(), "..", "..");

    // Helper to emit progress
    const emitProgress = (step, status, message) => {
      if (io) {
        io.emit('service:progress', { 
          service: serviceName, 
          step, 
          status, 
          message 
        });
      }
    };

    try {
      emitProgress('dependency', 'running', 'Checking dependencies...');
      
      // 0. Check and auto-start required dependencies
      const configPath = path.join(process.cwd(), 'config', 'serviceDependencies.json');
      let dependencies = {};
      try {
        const configData = await fs.readFile(configPath, 'utf-8');
        dependencies = JSON.parse(configData);
      } catch (error) {
        // If config file doesn't exist, continue without dependencies
        console.log('No service dependencies configured');
      }

      const serviceDeps = dependencies[serviceName] || { required: [], optional: [] };
      
      // Auto-start required dependencies
      if (serviceDeps.required && serviceDeps.required.length > 0) {
        console.log(`Checking required dependencies for ${serviceName}:`, serviceDeps.required);
        
        for (const dep of serviceDeps.required) {
          const isRunning = await this.isServiceRunning(dep);
          if (!isRunning) {
            console.log(`Auto-starting required dependency: ${dep}`);
            emitProgress('dependency', 'running', `Auto-starting dependency: ${dep}...`);
            try {
              // Recursively enable the dependency
              await this.enableService(dep, envService, io);
              console.log(`Successfully started dependency: ${dep}`);
            } catch (depError) {
              console.error(`Failed to start dependency ${dep}:`, depError.message);
              throw new Error(`Cannot enable ${serviceName}: required dependency ${dep} failed to start`);
            }
          } else {
            console.log(`Dependency ${dep} is already running`);
          }
        }
      }
      emitProgress('dependency', 'done', 'Dependencies checked');

      // 1. Create log directory for the service
      const logDir = path.join(rootDir, "logs", "services", serviceName);
      try {
        await fs.mkdir(logDir, { recursive: true });
        console.log(`Created log directory: ${logDir}`);
        
        // Set permissions to 777 so container can write logs
        try {
          await fs.chmod(logDir, 0o777);
          console.log(`Set log directory permissions: ${logDir}`);
        } catch (chmodError) {
          console.warn(`Could not set log directory permissions: ${chmodError.message}`);
        }
      } catch (mkdirError) {
        console.warn(`Failed to create log directory: ${mkdirError.message}`);
        // Continue anyway - some services might not need log directories
      }

      // 2. Update .env file
      emitProgress('env', 'running', 'Updating configuration...');
      await envService.updateServiceEnable(serviceName, true);
      console.log(
        `Updated .env: SERVICE_${serviceName.toUpperCase()}_ENABLE=true`
      );

      // 2.5. Verify .env file was updated (ensure file is flushed to disk)
      try {
        const envContent = await fs.readFile(path.join(rootDir, '.env'), 'utf-8');
        const envVar = `SERVICE_${serviceName.toUpperCase()}_ENABLE`;
        if (!envContent.includes(`${envVar}=true`)) {
          throw new Error(`.env file not updated correctly: ${envVar}=true not found`);
        }
        console.log(`Verified .env file contains: ${envVar}=true`);
      } catch (verifyError) {
        console.error('Failed to verify .env update:', verifyError.message);
        throw new Error(`Failed to verify .env update: ${verifyError.message}`);
      }
      emitProgress('env', 'done', 'Configuration updated');

      // 3. Regenerate docker-compose.dynamic.yml
      emitProgress('generate', 'running', 'Generating Docker Compose files...');
      const cliScript = path.join(rootDir, 'core', 'cli', 'stackvo.sh');
      console.log(`Running: ${cliScript} generate services`);

      const { stdout, stderr } = await execAsync(
        `${cliScript} generate services`
      );

      if (stderr) {
        console.error("Generate stderr:", stderr);
      }
      console.log("Generate stdout:", stdout);
      emitProgress('generate', 'done', 'Docker Compose files generated');

      // 4. Remove existing container(s) if they exist (to apply new volume mounts)
      // This ensures fresh containers with correct volume paths
      emitProgress('container', 'running', 'Preparing container...');
      const containerName = `stackvo-${serviceName}`;
      try {
        console.log(`Removing existing container: ${containerName}`);
        await execAsync(`docker rm -f ${containerName} 2>/dev/null || true`, { cwd: rootDir });
        console.log(`Removed existing container: ${containerName}`);
        
        // Special case: Kafka also has zookeeper container
        if (serviceName === 'kafka') {
          console.log(`Removing zookeeper container`);
          await execAsync(`docker rm -f stackvo-zookeeper 2>/dev/null || true`, { cwd: rootDir });
          console.log(`Removed zookeeper container`);
        }
      } catch (rmError) {
        // Ignore error if container doesn't exist
        console.log(`No existing container to remove or removal failed: ${rmError.message}`);
      }

      // 5. Build and start the service with all compose files
      // Note: Use --profile to enable services with profiles
      // IMPORTANT: Use HOST_STACKVO_ROOT for cwd because docker-compose runs on host via Docker socket
      // and paths in compose files are relative to the working directory
      emitProgress('container', 'running', 'Building and starting container...');
      console.log(`Building and starting service: ${serviceName}`);
      console.log(`Using working directory: ${rootDir}`); // Using container path for execution context
      const upResult = await execAsync(
        `docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml --profile ${serviceName} up -d --build ${serviceName} 2>&1`,
        { cwd: rootDir }
      );

      console.log("Service started:", upResult.stdout);
      emitProgress('container', 'done', 'Container started successfully');

      // 6. Clear cache
      this.cache.flushAll();
      console.log("Cache cleared");

      return {
        success: true,
        message: `Service "${serviceName}" enabled and started successfully`,
        running: true,
      };
    } catch (error) {
      console.error("Enable service error:", error);

      // Rollback: Disable the service in .env
      try {
        await envService.updateServiceEnable(serviceName, false);
        console.log(`Rolled back .env: SERVICE_${serviceName.toUpperCase()}_ENABLE=false`);
      } catch (rollbackError) {
        console.error("Rollback failed:", rollbackError.message);
      }

      throw new Error(`Failed to enable service: ${error.message}`);
    }
  }

  /**
   * Disable a service
   * @param {string} serviceName - Service name
   * @param {Object} envService - EnvService instance
   * @param {Object} io - Socket.io instance
   * @returns {Promise<Object>}
   */
  async disableService(serviceName, envService, io = null) {
    const { execAsync } = await import("../utils/exec.js");
    const path = await import("path");
    const fs = await import("fs/promises");

    const rootDir = process.env.STACKVO_ROOT || path.join(process.cwd(), "..", "..");

    // Helper to emit progress
    const emitProgress = (step, status, message) => {
      if (io) {
        io.emit('service:progress', { 
          service: serviceName, 
          step, 
          status, 
          message 
        });
      }
    };

    try {
      const containerName = `stackvo-${serviceName}`;
      let imageToRemove = null;

      // 1. Get image ID from container BEFORE removing it
      try {
        const container = this.docker.getContainer(containerName);
        const containerInfo = await container.inspect();
        imageToRemove = containerInfo.Image; // This is the image ID
        console.log(`Found image from container: ${imageToRemove}`);
      } catch (inspectError) {
        console.log(`Could not inspect container for image: ${inspectError.message}`);
      }

      // 2. Stop and remove container
      emitProgress('container', 'running', 'Stopping and removing container...');
      try {
        const container = this.docker.getContainer(containerName);

        // Stop container if running
        try {
          await container.stop();
          console.log(`Container ${containerName} stopped`);
        } catch (stopError) {
          console.log(
            `Container ${containerName} already stopped or not running`
          );
        }

        // Remove container
        await container.remove();
        console.log(`Container ${containerName} removed`);
      } catch (containerError) {
        console.warn(
          `Could not remove container ${containerName}:`,
          containerError.message
        );
        // Continue even if container doesn't exist
      }

      // Special case: Kafka also has zookeeper container
      if (serviceName === 'kafka') {
        try {
          const zookeeperContainer = this.docker.getContainer('stackvo-zookeeper');
          
          try {
            await zookeeperContainer.stop();
            console.log(`Zookeeper container stopped`);
          } catch (stopError) {
            console.log(`Zookeeper container already stopped or not running`);
          }
          
          await zookeeperContainer.remove();
          console.log(`Zookeeper container removed`);
        } catch (zookeeperError) {
          console.warn(
            `Could not remove zookeeper container:`,
            zookeeperError.message
          );
          // Continue even if container doesn't exist
        }
      }
      emitProgress('container', 'done', 'Container removed');

      // 3. Remove Docker image
      try {
        // If we got image ID from container, remove it
        if (imageToRemove) {
          try {
            const image = this.docker.getImage(imageToRemove);
            await image.remove({ force: true });
            console.log(`Docker image ${imageToRemove} removed`);
          } catch (imageError) {
            console.warn(
              `Could not remove image ${imageToRemove}:`,
              imageError.message
            );
          }
        } else {
          // Fallback: Try to find image by service name pattern
          const images = await this.docker.listImages({
            filters: { reference: [`*${serviceName}*`] },
          });

          if (images.length > 0) {
            for (const imageInfo of images) {
              const imageName = imageInfo.RepoTags?.[0] || imageInfo.Id;
              try {
                const image = this.docker.getImage(imageName);
                await image.remove({ force: true });
                console.log(`Docker image ${imageName} removed`);
              } catch (imageError) {
                console.warn(
                  `Could not remove image ${imageName}:`,
                  imageError.message
                );
              }
            }
          } else {
            console.log(`No image found for service: ${serviceName}`);
          }
        }
      } catch (imageError) {
        console.warn(
          `Error removing images for ${serviceName}:`,
          imageError.message
        );
        // Continue even if image removal fails
      }

      // 3. Remove Docker volumes (AGGRESSIVE CLEANUP)
      try {
        const volumeList = await this.docker.listVolumes({
          filters: { name: [`stackvo-${serviceName}`] },
        });

        if (volumeList.Volumes && volumeList.Volumes.length > 0) {
          for (const volumeInfo of volumeList.Volumes) {
            try {
              const volume = this.docker.getVolume(volumeInfo.Name);
              await volume.remove({ force: true });
              console.log(`Docker volume ${volumeInfo.Name} removed`);
            } catch (volumeError) {
              console.warn(
                `Could not remove volume ${volumeInfo.Name}:`,
                volumeError.message
              );
            }
          }
        } else {
          console.log(`No volumes found for service: ${serviceName}`);
        }
      } catch (volumeError) {
        console.warn(
          `Error removing volumes for ${serviceName}:`,
          volumeError.message
        );
        // Continue even if volume removal fails
      }

      // 4. Remove log directory (AGGRESSIVE CLEANUP)
      try {
        const logDir = path.join(rootDir, "logs", "services", serviceName);
        try {
          await fs.rm(logDir, { recursive: true, force: true });
          console.log(`Log directory removed: ${logDir}`);
        } catch (rmError) {
          console.warn(`Could not remove log directory: ${rmError.message}`);
        }
      } catch (logError) {
        console.warn(
          `Error removing log directory for ${serviceName}:`,
          logError.message
        );
        // Continue even if log removal fails
      }

      // 5. Update .env file
      emitProgress('env', 'running', 'Updating configuration...');
      await envService.updateServiceEnable(serviceName, false);
      console.log(
        `Updated .env: SERVICE_${serviceName.toUpperCase()}_ENABLE=false`
      );
      emitProgress('env', 'done', 'Configuration updated');

      // 6. Regenerate docker-compose.dynamic.yml
      emitProgress('generate', 'running', 'Generating Docker Compose files...');
      const cliScript = path.join(rootDir, 'core', 'cli', 'stackvo.sh');
      console.log(`Running: ${cliScript} generate services`);
      console.log(`Using working directory: ${rootDir}`);

      const { stdout, stderr } = await execAsync(
        `${cliScript} generate services`,
        { cwd: rootDir }
      );

      if (stderr) {
        console.error("Generate stderr:", stderr);
      }
      console.log("Generate stdout:", stdout);
      emitProgress('generate', 'done', 'Docker Compose files generated');

      // 7. Clear cache
      this.cache.flushAll();
      console.log("Cache cleared");

      return {
        success: true,
        message: `Service "${serviceName}" disabled and completely removed`,
        running: false,
      };
    } catch (error) {
      console.error("Disable service error:", error);
      throw new Error(`Failed to disable service: ${error.message}`);
    }
  }

  /**
   * Enable a tool (requires tools container rebuild)
   * @param {string} toolName - Tool name
   * @param {Object} envService - EnvService instance
   * @returns {Promise<Object>}
   */
  async enableTool(toolName, envService) {
    const { execAsync } = await import("../utils/exec.js");
    const path = await import("path");

    try {
      const containerName = "stackvo-tools";
      const rootDir = process.env.STACKVO_ROOT || path.join(process.cwd(), "..", "..");

      // 1. Update .env file
      await envService.updateToolEnable(toolName, true);
      console.log(`Updated .env: TOOLS_${toolName.toUpperCase()}_ENABLE=true`);

      // 2. Regenerate templates
      const cliScript = path.join(rootDir, 'core', 'cli', 'stackvo.sh');
      console.log(`Running: ${cliScript} generate`);

      try {
        const { stdout: genStdout, stderr: genStderr } = await execAsync(
          `${cliScript} generate`
        );

        if (genStderr) {
          console.error("Generate stderr:", genStderr);
        }
        console.log("Generate stdout:", genStdout);
      } catch (genError) {
        // Ignore hosts file errors - they are not critical
        if (genError.message && genError.message.includes('Failed to update') && genError.message.includes('hosts file')) {
          console.warn('Hosts file update failed (expected in container), continuing...');
        } else {
          throw genError;
        }
      }

      // 3. Stop and remove tools container (if running)
      try {
        const container = this.docker.getContainer(containerName);

        try {
          await container.stop();
          console.log(`Container ${containerName} stopped`);
        } catch (stopError) {
          console.log(`Container ${containerName} already stopped`);
        }

        await container.remove();
        console.log(`Container ${containerName} removed`);
      } catch (containerError) {
        console.warn(
          `Could not remove container ${containerName}:`,
          containerError.message
        );
      }

      // 4. Rebuild tools container
      // Note: Template path is fixed to ./tools so context resolves correctly inside generated dir
      console.log("Rebuilding tools container...");
      const { stdout: buildStdout } = await execAsync(
        "docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml build stackvo-tools",
        { cwd: rootDir }
      );
      console.log("Build stdout:", buildStdout);

      // 5. Start tools container
      console.log("Starting tools container...");
      const { stdout: upStdout } = await execAsync(
        "docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml up -d stackvo-tools",
        { cwd: rootDir }
      );
      console.log("Up stdout:", upStdout);

      // 6. Clear cache
      this.cache.flushAll();
      console.log("Cache cleared");

      return {
        success: true,
        message: `Tool "${toolName}" enabled successfully`,
      };
    } catch (error) {
      console.error("Enable tool error:", error);

      // Rollback: Disable the tool in .env
      try {
        await envService.updateToolEnable(toolName, false);
        console.log(`Rolled back .env: TOOLS_${toolName.toUpperCase()}_ENABLE=false`);
      } catch (rollbackError) {
        console.error("Rollback failed:", rollbackError.message);
      }

      throw new Error(`Failed to enable tool: ${error.message}`);
    }
  }

  /**
   * Disable a tool (requires tools container rebuild)
   * @param {string} toolName - Tool name
   * @param {Object} envService - EnvService instance
   * @returns {Promise<Object>}
   */
  async disableTool(toolName, envService) {
    const { execAsync } = await import("../utils/exec.js");
    const path = await import("path");

    try {
      const containerName = "stackvo-tools";
      const rootDir = process.env.STACKVO_ROOT || path.join(process.cwd(), "..", "..");

      // 1. Update .env file
      await envService.updateToolEnable(toolName, false);
      console.log(`Updated .env: TOOLS_${toolName.toUpperCase()}_ENABLE=false`);

      // 2. Regenerate templates
      const cliScript = path.join(rootDir, 'core', 'cli', 'stackvo.sh');
      console.log(`Running: ${cliScript} generate`);

      try {
        const { stdout: genStdout, stderr: genStderr } = await execAsync(
          `${cliScript} generate`
        );

        if (genStderr) {
          console.error("Generate stderr:", genStderr);
        }
        console.log("Generate stdout:", genStdout);
      } catch (genError) {
        // Ignore hosts file errors - they are not critical
        if (genError.message && genError.message.includes('Failed to update') && genError.message.includes('hosts file')) {
          console.warn('Hosts file update failed (expected in container), continuing...');
        } else {
          throw genError;
        }
      }

      // 3. Stop and remove tools container (if running)
      try {
        const container = this.docker.getContainer(containerName);

        try {
          await container.stop();
          console.log(`Container ${containerName} stopped`);
        } catch (stopError) {
          console.log(`Container ${containerName} already stopped`);
        }

        await container.remove();
        console.log(`Container ${containerName} removed`);
      } catch (containerError) {
        console.warn(
          `Could not remove container ${containerName}:`,
          containerError.message
        );
      }

      // 4. Checking if any other tools enabled. If so, rebuild and start.
      // For now, simpler approach: rebuild and start if tools service is still in compose file.
      // If no tools enabled, "generate" should have removed stackvo-tools from docker-compose.dynamic.yml
      // However, we can try to build/up anyway. If service missing, it might warn/error or ignore.
      
      // Let's check environment variable from process? No, EnvService updated file but process.env not reloaded here.
      // But we can blindly run compose up provided generate was correct.
      
      console.log("Rebuilding and restarting tools container (if still enabled)...");
      try {
          const { stdout: buildStdout } = await execAsync(
            "docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml build stackvo-tools",
            { cwd: rootDir }
          );
          console.log("Build stdout:", buildStdout);
          
          const { stdout: upStdout } = await execAsync(
            "docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml up -d stackvo-tools",
            { cwd: rootDir }
          );
          console.log("Up stdout:", upStdout);
      } catch (composeError) {
          // If no services enabled, this might fail or warn.
          // If the service 'stackvo-tools' is removed from dynamic yaml, it will say "no such service: stackvo-tools"
          console.log("Compose command output (might be empty/error if no tools enabled):", composeError.message);
      }

      // 5. Clear cache
      this.cache.flushAll();
      console.log("Cache cleared");

      return {
        success: true,
        message: `Tool "${toolName}" disabled successfully`,
      };
    } catch (error) {
      console.error("Disable tool error:", error);
      throw new Error(`Failed to disable tool: ${error.message}`);
    }
  }

  /**
   * Get Docker system stats (CPU, Memory, Storage, Network)
   */
  async getDockerStats() {
    const cacheKey = "docker_stats";
    const cached = this.cache.get(cacheKey);
    if (cached) return cached;

    try {
      const [cpu, memory, storage, network] = await Promise.all([
        this.getCPUStats(),
        this.getMemoryStats(),
        this.getStorageStats(),
        this.getNetworkStats(),
      ]);

      const stats = {
        cpu,
        memory,
        storage,
        network,
        timestamp: Date.now(),
      };

      // Cache for 2 seconds
      this.cache.set(cacheKey, stats, 2);

      return stats;
    } catch (error) {
      console.error("Get Docker stats error:", error);
      throw error;
    }
  }

  /**
   * Get CPU stats from /proc/stat
   */
  async getCPUStats() {
    try {
      const fs = await import("fs/promises");
      const stat = await fs.readFile("/proc/stat", "utf-8");
      const cpuLine = stat.split("\n")[0];
      const values = cpuLine.split(/\s+/).slice(1).map(Number);

      const [user, nice, system, idle, iowait, irq, softirq] = values;
      const total = user + nice + system + idle + iowait + irq + softirq;
      const used = total - idle;

      return {
        percent: ((used / total) * 100).toFixed(1),
        breakdown: {
          user: ((user / total) * 100).toFixed(1),
          nice: ((nice / total) * 100).toFixed(1),
          system: ((system / total) * 100).toFixed(1),
          idle: ((idle / total) * 100).toFixed(1),
        },
      };
    } catch (error) {
      console.error("Get CPU stats error:", error);
      return {
        percent: "0.0",
        breakdown: { user: "0.0", nice: "0.0", system: "0.0", idle: "100.0" },
      };
    }
  }

  /**
   * Get Memory stats from /proc/meminfo
   */
  async getMemoryStats() {
    try {
      const fs = await import("fs/promises");
      const meminfo = await fs.readFile("/proc/meminfo", "utf-8");

      const getVal = (key) => {
        const match = meminfo.match(new RegExp(`${key}:\\s+(\\d+)`));
        return match ? parseInt(match[1]) : 0;
      };

      const total = getVal("MemTotal");
      const available = getVal("MemAvailable");
      const used = total - available;

      return {
        total: (total / 1024 / 1024).toFixed(2), // GB
        used: (used / 1024 / 1024).toFixed(2),
        available: (available / 1024 / 1024).toFixed(2),
        percent: ((used / total) * 100).toFixed(1),
      };
    } catch (error) {
      console.error("Get memory stats error:", error);
      return {
        total: "0.00",
        used: "0.00",
        available: "0.00",
        percent: "0.0",
      };
    }
  }

  /**
   * Get Storage stats from df command
   */
  async getStorageStats() {
    try {
      const { promisify } = await import("util");
      const { exec } = await import("child_process");
      const execAsync = promisify(exec);

      // Get disk usage for /var/lib/docker or fallback to /
      // Use grep to skip header line
      const { stdout } = await execAsync(
        'df -BG /var/lib/docker 2>/dev/null | grep -v "^Filesystem" || df -BG / | grep -v "^Filesystem"'
      );
      const lines = stdout.trim().split("\n");
      const lastLine = lines[lines.length - 1];
      const parts = lastLine.trim().split(/\s+/);

      return {
        total: parts[1].replace("G", ""),
        used: parts[2].replace("G", ""),
        available: parts[3].replace("G", ""),
        percent: parts[4].replace("%", ""),
      };
    } catch (error) {
      console.error("Get storage stats error:", error);
      return {
        total: "0",
        used: "0",
        available: "0",
        percent: "0",
      };
    }
  }

  /**
   * Get Network stats from /proc/net/dev
   */
  async getNetworkStats() {
    try {
      const fs = await import("fs/promises");
      const netdev = await fs.readFile("/proc/net/dev", "utf-8");

      let totalRx = 0;
      let totalTx = 0;

      netdev
        .split("\n")
        .slice(2)
        .forEach((line) => {
          if (line.trim() && !line.includes("lo:")) {
            // Skip loopback
            const parts = line.trim().split(/\s+/);
            totalRx += parseInt(parts[1]) || 0;
            totalTx += parseInt(parts[9]) || 0;
          }
        });

      // Calculate speed (MB/s) if we have previous values
      let rxSpeed = 0;
      let txSpeed = 0;

      if (this.previousNetworkStats) {
        const timeDelta =
          (Date.now() - this.previousNetworkStats.timestamp) / 1000; // seconds
        const rxDelta = totalRx - this.previousNetworkStats.totalRx;
        const txDelta = totalTx - this.previousNetworkStats.totalTx;

        rxSpeed = (rxDelta / timeDelta / 1024 / 1024).toFixed(2); // MB/s
        txSpeed = (txDelta / timeDelta / 1024 / 1024).toFixed(2); // MB/s
      }

      // Store current values for next calculation
      this.previousNetworkStats = {
        totalRx,
        totalTx,
        timestamp: Date.now(),
      };

      return {
        rx: (totalRx / 1024 / 1024 / 1024).toFixed(2), // Total GB
        tx: (totalTx / 1024 / 1024 / 1024).toFixed(2), // Total GB
        rxSpeed: rxSpeed < 0 ? "0.00" : rxSpeed, // MB/s (prevent negative)
        txSpeed: txSpeed < 0 ? "0.00" : txSpeed, // MB/s (prevent negative)
      };
    } catch (error) {
      console.error("Get network stats error:", error);
      return {
        rx: "0.00",
        tx: "0.00",
        rxSpeed: "0.00",
        txSpeed: "0.00",
      };
    }
  }

  /**
   * Start all containers except stackvo-ui and stackvo-traefik
   */
  async startAllContainers() {
    try {
      // Get all containers
      const { stdout } = await execAsync('docker ps -a --format "{{.Names}}"');
      const containers = stdout
        .trim()
        .split("\n")
        .filter((name) => {
          // Exclude UI and Traefik containers
          return name && name !== "stackvo-ui" && name !== "stackvo-traefik";
        });

      const results = [];
      for (const containerName of containers) {
        try {
          await execAsync(`docker start ${containerName}`);
          results.push({ container: containerName, status: "started" });
        } catch (error) {
          results.push({
            container: containerName,
            status: "failed",
            error: error.message,
          });
        }
      }

      return {
        total: containers.length,
        results,
      };
    } catch (error) {
      throw new Error(`Failed to start containers: ${error.message}`);
    }
  }

  /**
   * Stop all containers except stackvo-ui and stackvo-traefik
   */
  async stopAllContainers() {
    try {
      // Get all running containers
      const { stdout } = await execAsync('docker ps --format "{{.Names}}"');
      const containers = stdout
        .trim()
        .split("\n")
        .filter((name) => {
          // Exclude UI and Traefik containers
          return name && name !== "stackvo-ui" && name !== "stackvo-traefik";
        });

      const results = [];
      for (const containerName of containers) {
        try {
          await execAsync(`docker stop ${containerName}`);
          results.push({ container: containerName, status: "stopped" });
        } catch (error) {
          results.push({
            container: containerName,
            status: "failed",
            error: error.message,
          });
        }
      }

      return {
        total: containers.length,
        results,
      };
    } catch (error) {
      throw new Error(`Failed to stop containers: ${error.message}`);
    }
  }

  /**
   * Restart all containers except stackvo-ui and stackvo-traefik
   */
  async restartAllContainers() {
    try {
      // Get all containers
      const { stdout } = await execAsync('docker ps -a --format "{{.Names}}"');
      const containers = stdout
        .trim()
        .split("\n")
        .filter((name) => {
          // Exclude UI and Traefik containers
          return name && name !== "stackvo-ui" && name !== "stackvo-traefik";
        });

      const results = [];
      for (const containerName of containers) {
        try {
          await execAsync(`docker restart ${containerName}`);
          results.push({ container: containerName, status: "restarted" });
        } catch (error) {
          results.push({
            container: containerName,
            status: "failed",
            error: error.message,
          });
        }
      }

      return {
        total: containers.length,
        results,
      };
    } catch (error) {
      throw new Error(`Failed to restart containers: ${error.message}`);
    }
  }
}

export default DockerService;
