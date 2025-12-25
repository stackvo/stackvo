import express from 'express';
import DockerService from '../services/DockerService.js';

const router = express.Router();
const dockerService = new DockerService();

/**
 * GET /api/docker/stats
 * Get Docker system statistics (CPU, Memory, Storage, Network)
 */
router.get('/stats', async (req, res) => {
  try {
    const stats = await dockerService.getDockerStats();
    res.json({
      success: true,
      data: stats
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * GET /api/docker/stats/:containerName
 * Get container statistics
 */
router.get('/stats/:containerName', async (req, res) => {
  try {
    const { containerName } = req.params;
    const stats = await dockerService.getContainerStats(containerName);
    res.json({
      success: true,
      data: stats
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/docker/start-all
 * Start all containers except stackvo-ui and stackvo-traefik
 */
router.post('/start-all', async (req, res) => {
  try {
    const result = await dockerService.startAllContainers();
    res.json({
      success: true,
      data: result,
      message: 'All containers started successfully'
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/docker/stop-all
 * Stop all containers except stackvo-ui and stackvo-traefik
 */
router.post('/stop-all', async (req, res) => {
  try {
    const result = await dockerService.stopAllContainers();
    res.json({
      success: true,
      data: result,
      message: 'All containers stopped successfully'
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

/**
 * POST /api/docker/restart-all
 * Restart all containers except stackvo-ui and stackvo-traefik
 */
router.post('/restart-all', async (req, res) => {
  try {
    const result = await dockerService.restartAllContainers();
    res.json({
      success: true,
      data: result,
      message: 'All containers restarted successfully'
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      message: error.message
    });
  }
});

export default router;
