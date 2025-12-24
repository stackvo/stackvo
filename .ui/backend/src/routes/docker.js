import express from 'express';
import DockerService from '../services/DockerService.js';

const router = express.Router();
const dockerService = new DockerService();

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

export default router;
