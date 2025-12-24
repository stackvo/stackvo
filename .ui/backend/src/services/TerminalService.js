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
}

export default TerminalService;
