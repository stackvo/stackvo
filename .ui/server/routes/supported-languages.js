import express from 'express';
import { readFileSync } from 'fs';

const router = express.Router();

/**
 * GET /api/supported-languages
 * Get supported languages, versions, defaults and extensions from .env file
 */
router.get('/', async (req, res) => {
  try {
    const stackvoRoot = process.env.STACKVO_ROOT || '/app';
    const envPath = `${stackvoRoot}/.env`;
    const envContent = readFileSync(envPath, 'utf-8');
    
    // Parse .env file
    const envVars = {};
    envContent.split('\n').forEach(line => {
      line = line.trim();
      if (line && !line.startsWith('#')) {
        const [key, ...valueParts] = line.split('=');
        if (key) {
          envVars[key.trim()] = valueParts.join('=').trim();
        }
      }
    });

    // Parse SUPPORTED_LANGUAGES
    const languages = envVars.SUPPORTED_LANGUAGES 
      ? envVars.SUPPORTED_LANGUAGES.split(',').map(l => l.trim())
      : [];

    // Parse versions for each language
    const versions = {};
    const defaults = {};
    const extensions = {};

    languages.forEach(lang => {
      const langUpper = lang.toUpperCase();
      
      // Get versions
      const versionsKey = `SUPPORTED_LANGUAGES_${langUpper}_VERSIONS`;
      if (envVars[versionsKey]) {
        versions[lang] = envVars[versionsKey].split(',').map(v => v.trim());
      }
      
      // Get default version
      const defaultKey = `SUPPORTED_LANGUAGES_${langUpper}_DEFAULT`;
      if (envVars[defaultKey]) {
        defaults[lang] = envVars[defaultKey].trim();
      }
      
      // Get extensions (only for PHP)
      if (lang === 'php') {
        const extensionsKey = `SUPPORTED_LANGUAGES_${langUpper}_EXTENSIONS`;
        if (envVars[extensionsKey]) {
          extensions[lang] = envVars[extensionsKey].split(',').map(e => e.trim());
        }
        
        // Get default extensions for PHP
        const extensionsDefaultKey = `SUPPORTED_LANGUAGES_${langUpper}_EXTENSIONS_DEFAULT`;
        if (envVars[extensionsDefaultKey]) {
          defaults[`${lang}_extensions`] = envVars[extensionsDefaultKey].split(',').map(e => e.trim());
        }
      }
    });

    // Parse webservers
    const webservers = envVars.SUPPORTED_WEBSERVERS 
      ? envVars.SUPPORTED_WEBSERVERS.split(',').map(w => w.trim())
      : [];
    const defaultWebserver = envVars.SUPPORTED_WEBSERVERS_DEFAULT 
      ? envVars.SUPPORTED_WEBSERVERS_DEFAULT.trim()
      : 'nginx';

    res.json({
      success: true,
      data: {
        languages,
        versions,
        defaults,
        extensions,
        webservers,
        defaultWebserver
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
