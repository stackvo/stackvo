import express from 'express';

const router = express.Router();

/**
 * Open system terminal for container
 * Note: This feature is not available when running in a container.
 * Users should use: docker exec -it {containerName} bash
 */
router.post('/:containerName/open', async (req, res) => {
  try {
    const { containerName } = req.params;
    
    // Check if running in container
    const fs = await import('fs');
    const isContainer = fs.existsSync('/.dockerenv') || fs.existsSync('/run/.containerenv');
    
    if (isContainer) {
      // Running in container - cannot open host terminal
      return res.status(400).json({
        success: false,
        message: 'Terminal cannot be opened from containerized environment',
        hint: `Please run this command on your host machine: docker exec -it ${containerName} bash`
      });
    }
    
    // If not in container, try to open system terminal
    const terminalService = req.app.get('terminalService');
    
    if (!terminalService) {
      return res.status(500).json({
        success: false,
        message: 'Terminal service not available'
      });
    }
    
    await terminalService.openSystemTerminal(containerName);
    
    res.json({
      success: true,
      message: `Terminal opened for container: ${containerName}`
    });
  } catch (error) {
    console.error('Terminal open error:', error);
    res.status(500).json({
      success: false,
      message: error.message,
      hint: `Please run this command on your host machine: docker exec -it ${req.params.containerName} bash`
    });
  }
});

export default router;
