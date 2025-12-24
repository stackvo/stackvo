import express from 'express';
import { readFileSync } from 'fs';
import { resolve } from 'path';

const router = express.Router();

/**
 * GET /api/env
 * Get environment variables from .env file
 */
router.get('/', async (req, res) => {
  try {
    const envPath = resolve(process.cwd(), '../../.env');
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

    res.json({
      success: true,
      data: { env: envVars }
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

export default router;
