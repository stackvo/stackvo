import fs from 'fs/promises';
import path from 'path';

class EnvService {
  constructor() {
    // Use STACKVO_ROOT environment variable or fallback to relative path
    const stackvoRoot = process.env.STACKVO_ROOT || path.join(process.cwd(), '..', '..');
    this.envPath = path.join(stackvoRoot, '.env');
  }

  /**
   * Update SERVICE_X_ENABLE value in .env file
   * @param {string} serviceName - Service name (e.g., 'redis', 'mysql')
   * @param {boolean} enabled - Enable or disable
   * @returns {Promise<void>}
   */
  async updateServiceEnable(serviceName, enabled) {
    try {
      // Read .env file
      let envContent = await fs.readFile(this.envPath, 'utf-8');
      
      // Convert service name to uppercase for env variable
      const serviceUpper = serviceName.toUpperCase();
      const envVar = `SERVICE_${serviceUpper}_ENABLE`;
      const newValue = enabled ? 'true' : 'false';
      
      // Create regex to find the line
      const regex = new RegExp(`^${envVar}=.*$`, 'gm');
      
      // Check if variable exists
      if (regex.test(envContent)) {
        // Update existing line
        envContent = envContent.replace(regex, `${envVar}=${newValue}`);
      } else {
        // Variable doesn't exist, add it
        envContent += `\n${envVar}=${newValue}\n`;
      }
      
      // Write back to file
      await fs.writeFile(this.envPath, envContent, 'utf-8');
      
      console.log(`Updated ${envVar}=${newValue} in .env`);
    } catch (error) {
      console.error('Error updating .env:', error);
      throw new Error(`Failed to update .env: ${error.message}`);
    }
  }

  /**
   * Get all environment variables
   * @returns {Promise<Object>}
   */
  async getAllEnv() {
    try {
      const envContent = await fs.readFile(this.envPath, 'utf-8');
      const env = {};
      
      envContent.split('\n').forEach(line => {
        const trimmed = line.trim();
        if (trimmed && !trimmed.startsWith('#')) {
          const [key, ...valueParts] = trimmed.split('=');
          if (key) {
            env[key] = valueParts.join('=');
          }
        }
      });
      
      return env;
    } catch (error) {
      console.error('Error reading .env:', error);
      throw new Error(`Failed to read .env: ${error.message}`);
    }
  }

  /**
   * Update TOOLS_X_ENABLE value in .env file
   * @param {string} toolName - Tool name (e.g., 'phpmyadmin', 'adminer')
   * @param {boolean} enabled - Enable or disable
   * @returns {Promise<void>}
   */
  async updateToolEnable(toolName, enabled) {
    try {
      // Read .env file
      let envContent = await fs.readFile(this.envPath, 'utf-8');
      
      // Convert tool name to uppercase for env variable
      const toolUpper = toolName.toUpperCase();
      const envVar = `TOOLS_${toolUpper}_ENABLE`;
      const newValue = enabled ? 'true' : 'false';
      
      // Create regex to find the line
      const regex = new RegExp(`^${envVar}=.*$`, 'gm');
      
      // Check if variable exists
      if (regex.test(envContent)) {
        // Update existing line
        envContent = envContent.replace(regex, `${envVar}=${newValue}`);
      } else {
        // Variable doesn't exist, add it
        envContent += `\n${envVar}=${newValue}\n`;
      }
      
      // Write back to file
      await fs.writeFile(this.envPath, envContent, 'utf-8');
      
      console.log(`Updated ${envVar}=${newValue} in .env`);
    } catch (error) {
      console.error('Error updating .env:', error);
      throw new Error(`Failed to update .env: ${error.message}`);
    }
  }
}

export default EnvService;
