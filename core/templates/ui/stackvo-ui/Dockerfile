###################################################################
# STACKVO UI DOCKERFILE
# Multi-stage build: Node.js build + Nginx serve
###################################################################

# Stage 1: Build Vue.js frontend
FROM node:20-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy frontend package files
COPY .ui/frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy frontend source
COPY .ui/frontend/ ./

# Build for production
RUN npm run build

# Stage 2: Setup Node.js backend
FROM node:20-alpine AS backend-setup

# Install build dependencies for native modules (node-pty)
RUN apk add --no-cache python3 make g++

WORKDIR /app/backend

# Copy backend package files
COPY .ui/backend/package*.json ./

# Install production dependencies only
RUN npm ci --only=production

# Copy backend source
COPY .ui/backend/ ./

# Stage 3: Final Nginx + Node.js runtime
FROM node:20-alpine

# Install nginx, Docker CLI, and bash
RUN apk add --no-cache nginx docker-cli docker-cli-compose bash

# Create necessary directories
RUN mkdir -p /var/log/nginx /usr/share/nginx/html /opt/backend

# Copy Nginx configuration
COPY core/templates/ui/stackvo-ui/nginx.conf /etc/nginx/http.d/default.conf

# Copy built frontend from builder stage
COPY --from=frontend-builder /app/frontend/dist /usr/share/nginx/html/frontend

# Copy backend from backend-setup stage to /opt/backend
COPY --from=backend-setup /app/backend /opt/backend

# Create startup script
RUN echo '#!/bin/sh' > /start.sh && \
    echo '# Fix docker socket permissions' >> /start.sh && \
    echo 'if [ -S /var/run/docker.sock ]; then' >> /start.sh && \
    echo '  chmod 666 /var/run/docker.sock' >> /start.sh && \
    echo 'fi' >> /start.sh && \
    echo '# Start Node.js backend in background' >> /start.sh && \
    echo 'cd /opt/backend && node src/server.js &' >> /start.sh && \
    echo '# Start Nginx in foreground' >> /start.sh && \
    echo 'nginx -g "daemon off;"' >> /start.sh && \
    chmod +x /start.sh

WORKDIR /app

EXPOSE 80

CMD ["/start.sh"]
