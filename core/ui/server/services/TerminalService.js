import pty from 'node-pty';

class TerminalService {
  constructor(io) {
    this.io = io;
    this.terminals = new Map();
    this.setupSocketHandlers();
  }

  setupSocketHandlers() {
    this.io.on('connection', (socket) => {
      console.log('Terminal client connected:', socket.id);

      // Create terminal session
      socket.on('create-terminal', (data) => {
        const { containerName, cols, rows } = data;
        
        try {
          // Spawn docker exec process
          const ptyProcess = pty.spawn('docker', [
            'exec', '-it', containerName, 'bash', '-l'
          ], {
            name: 'xterm-color',
            cols: cols || 80,
            rows: rows || 30,
            cwd: process.env.HOME,
            env: process.env
          });

          // Store terminal process
          this.terminals.set(socket.id, ptyProcess);

          // Send terminal output to client
          ptyProcess.onData((data) => {
            socket.emit('terminal-output', data);
          });

          // Handle terminal exit
          ptyProcess.onExit(({ exitCode }) => {
            socket.emit('terminal-closed', { exitCode });
            this.terminals.delete(socket.id);
            console.log(`Terminal closed for ${containerName}, exit code: ${exitCode}`);
          });

          socket.emit('terminal-ready');
          console.log(`Terminal created for container: ${containerName}`);
        } catch (error) {
          console.error('Terminal creation error:', error);
          socket.emit('terminal-error', { message: error.message });
        }
      });

      // Handle terminal input from client
      socket.on('terminal-input', (data) => {
        const ptyProcess = this.terminals.get(socket.id);
        if (ptyProcess) {
          ptyProcess.write(data);
        }
      });

      // Handle terminal resize
      socket.on('terminal-resize', (data) => {
        const ptyProcess = this.terminals.get(socket.id);
        if (ptyProcess) {
          ptyProcess.resize(data.cols, data.rows);
        }
      });

      // Handle disconnect
      socket.on('disconnect', () => {
        const ptyProcess = this.terminals.get(socket.id);
        if (ptyProcess) {
          ptyProcess.kill();
          this.terminals.delete(socket.id);
        }
        console.log('Terminal client disconnected:', socket.id);
      });
    });
  }

  /**
   * Open system terminal with docker exec command
   * @param {string} containerName - Container name to exec into
   * @returns {Promise<void>}
   */
  async openSystemTerminal(containerName) {
    const { spawn, execSync } = await import('child_process');
    const os = await import('os');
    const fs = await import('fs');
    
    const platform = os.platform();
    const command = `docker exec -it ${containerName} bash -l`;
    
    let terminalCmd;
    let terminalArgs;
    
    // Check if running in WSL
    let isWSL = false;
    if (platform === 'linux') {
      try {
        const procVersion = fs.readFileSync('/proc/version', 'utf8').toLowerCase();
        isWSL = procVersion.includes('microsoft') || procVersion.includes('wsl');
      } catch (error) {
        // Not WSL
      }
    }
    
    // Detect terminal emulator based on OS
    if (isWSL) {
      // WSL - use PowerShell to open Windows Terminal without security warning
      console.log('Detected WSL environment, using PowerShell to open Windows terminal');
      
      // Use PowerShell to start Windows Terminal
      // This avoids the LxLaunch.exe security warning
      terminalCmd = 'powershell.exe';
      terminalArgs = [
        '-NoProfile',
        '-Command',
        `Start-Process wt -ArgumentList 'wsl -e bash -c \\"${command}\\"'`
      ];
    } else if (platform === 'darwin') {
      // macOS - use Terminal.app
      terminalCmd = 'osascript';
      terminalArgs = [
        '-e',
        `tell application "Terminal" to do script "${command}"`
      ];
    } else if (platform === 'win32') {
      // Windows - use cmd or wt (Windows Terminal)
      terminalCmd = 'cmd';
      terminalArgs = ['/c', 'start', 'cmd', '/k', command];
    } else {
      // Linux - try common terminal emulators
      const terminals = [
        'gnome-terminal',
        'konsole',
        'xfce4-terminal',
        'xterm',
        'alacritty',
        'wezterm'
      ];
      
      // Find available terminal
      for (const term of terminals) {
        try {
          execSync(`which ${term}`, { stdio: 'ignore' });
          terminalCmd = term;
          
          // Set args based on terminal
          if (term === 'gnome-terminal') {
            terminalArgs = ['--', 'bash', '-c', command];
          } else if (term === 'konsole') {
            terminalArgs = ['-e', 'bash', '-c', command];
          } else if (term === 'xfce4-terminal') {
            terminalArgs = ['-e', `bash -c "${command}"`];
          } else {
            terminalArgs = ['-e', 'bash', '-c', command];
          }
          break;
        } catch (error) {
          continue;
        }
      }
      
      if (!terminalCmd) {
        throw new Error('No terminal emulator found. Please install gnome-terminal, konsole, xfce4-terminal, xterm, alacritty, or wezterm.');
      }
    }
    
    // Spawn terminal process
    const terminal = spawn(terminalCmd, terminalArgs, {
      detached: true,
      stdio: 'ignore'
    });
    
    terminal.unref();
    
    console.log(`Opened system terminal for container: ${containerName} (WSL: ${isWSL}, Platform: ${platform})`);
  }
}

export default TerminalService;
