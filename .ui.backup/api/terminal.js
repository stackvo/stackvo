// .ui/api/terminal.js (Express.js)
const express = require('express');
const cors = require('cors');
const { exec } = require('child_process');

const app = express();
const PORT = 3001;

// Middleware
app.use(cors()); // CORS izni (tarayıcıdan istek için)
app.use(express.json()); // JSON body parser

// Terminal açma endpoint'i
app.post('/api/terminal', (req, res) => {
  const { container, clientOS } = req.body;
  
  if (!container) {
    return res.status(400).json({ 
      success: false, 
      message: 'Container name is required' 
    });
  }

  const command = `docker exec -it ${container} bash`;
  
  // İstemciden gelen OS bilgisini kullan (WSL için önemli!)
  const platform = clientOS || process.platform;
  
  console.log('🖥️ Client OS:', clientOS);
  console.log('🔧 Server Platform:', process.platform);
  console.log('✅ Using Platform:', platform);
  
  // İşletim sistemine göre terminal aç
  let terminalCommand;
  
  if (platform === 'windows' || platform === 'win32') {
    // Windows - WSL üzerinden Windows Terminal aç
    // Default WSL distro kullanılır (distro adı belirtilmez)
    terminalCommand = `cmd.exe /c start wt.exe wsl bash -c "${command}"`;
  } else if (platform === 'macos' || platform === 'darwin') {
    // macOS
    terminalCommand = `osascript -e 'tell application "Terminal" to do script "${command}"'`;
  } else {
    // Linux - Önce hangi terminal var kontrol et
    const terminals = [
      `gnome-terminal -- bash -c "${command}; exec bash"`,
      `konsole -e bash -c "${command}; exec bash"`,
      `xfce4-terminal -e "bash -c '${command}; exec bash'"`,
      `xterm -e bash -c "${command}; exec bash"`,
      `alacritty -e bash -c "${command}; exec bash"`,
      `wezterm -e bash -c "${command}; exec bash"`
    ];
    
    terminalCommand = terminals.join(' || ');
  }
  
  console.log('🔧 Platform:', process.platform);
  console.log('📝 Container:', container);
  console.log('💻 Terminal Command:', terminalCommand);
  
  // Terminal komutunu çalıştır
  exec(terminalCommand, (error, _stdout, _stderr) => {
    if (error) {
      console.error('❌ Terminal açma hatası:', error);
      console.error('📋 Error message:', error.message);
      console.error('📋 Error code:', error.code);
      return res.status(500).json({ 
        success: false, 
        message: 'Failed to open terminal',
        error: error.message 
      });
    }
    
    console.log('✅ Terminal başarıyla açıldı');
    res.json({ 
      success: true, 
      message: `Terminal opened for container: ${container}` 
    });
  });
});

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({ status: 'OK', platform: process.platform });
});

// Server başlat
app.listen(PORT, () => {
  console.log(`✅ Stackvo Terminal API running on http://localhost:${PORT}`);
  console.log(`📍 Platform: ${process.platform}`);
});