import express from 'express';

const router = express.Router();

/**
 * Open system terminal for container
 */
router.post('/:containerName/open', async (req, res) => {
  try {
    const { containerName } = req.params;
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
      message: error.message
    });
  }
});

export default router;
