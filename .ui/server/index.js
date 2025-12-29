import express from 'express';
import { createServer } from 'http';
import { Server } from 'socket.io';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';

// Routes
import servicesRouter from './routes/services.js';
import projectsRouter from './routes/projects.js';
import dockerRouter from './routes/docker.js';
import envRouter from './routes/env.js';
import toolsRouter from './routes/tools.js';
import terminalRouter from './routes/terminal.js';
import supportedLanguagesRouter from './routes/supported-languages.js';
import startWithProfileRouter from './routes/start-with-profile.js';

// Services
import TerminalService from './services/TerminalService.js';
import DockerService from './services/DockerService.js';

// ESM __dirname alternative
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Load environment variables from STACKVO_ROOT/.env
const stackvoRoot = process.env.STACKVO_ROOT || '/app';
dotenv.config({ path: `${stackvoRoot}/.env` });

const app = express();
const httpServer = createServer(app);
const io = new Server(httpServer, {
  cors: {
    origin: '*',
    methods: ['GET', 'POST']
  }
});

// Middleware
app.use(helmet({
  contentSecurityPolicy: false
}));
app.use(cors());
app.use(compression());
app.use(express.json());

// Logging middleware
app.use((req, res, next) => {
  console.log(`${new Date().toISOString()} - ${req.method} ${req.path}`);
  next();
});

// API Routes
app.use('/api/services', servicesRouter);
app.use('/api/projects', projectsRouter);
app.use('/api/docker', dockerRouter);
app.use('/api/env', envRouter);
app.use('/api/tools', toolsRouter);
app.use('/api/terminal', terminalRouter);
app.use('/api/supported-languages', supportedLanguagesRouter);
app.use('/api/start-with-profile', startWithProfileRouter);

// Make Socket.io available to routes
app.set('io', io);

// Terminal WebSocket and Service
const terminalService = new TerminalService(io);
app.set('terminalService', terminalService);

// Docker Service
const dockerService = new DockerService();
app.set('dockerService', dockerService);

// Serve static files in production
if (process.env.NODE_ENV === 'production') {
  const distPath = path.join(__dirname, '../dist');
  app.use(express.static(distPath));
  
  app.get('*', (req, res) => {
    res.sendFile(path.join(distPath, 'index.html'));
  });
}

// Health check
app.get('/health', (req, res) => {
  res.json({ 
    status: 'ok',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
    environment: process.env.NODE_ENV || 'development'
  });
});

// Error handler
app.use((err, req, res, _next) => {
  console.error('Error:', err);
  res.status(500).json({
    success: false,
    message: err.message || 'Internal server error'
  });
});

const PORT = process.env.PORT || 3000;
httpServer.listen(PORT, () => {
  console.log(`🚀 Stackvo API running on http://localhost:${PORT}`);
  console.log(`📡 WebSocket server ready`);
  console.log(`🌍 Environment: ${process.env.NODE_ENV || 'development'}`);
});
