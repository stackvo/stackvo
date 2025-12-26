###################################################################
# STACKVO UI DOCKERFILE
# Multi-stage build: Node.js build + Nginx serve
###################################################################

# Stage 1: Install dependencies and build frontend
FROM node:20-alpine AS builder

# Install build dependencies for native modules (node-pty)
RUN apk add --no-cache python3 make g++

WORKDIR /app

# Copy package files
COPY .ui/package*.json ./

# Install all dependencies (including devDependencies for build)
RUN npm install

# Copy source code
COPY .ui/ ./

# Build frontend for production
RUN npm run build

# Stage 2: Production runtime
FROM node:20-alpine

# Install nginx, Docker CLI, bash, perl, and gettext
RUN apk add --no-cache nginx docker-cli docker-cli-compose bash perl gettext

# Create necessary directories
RUN mkdir -p /var/log/nginx /usr/share/nginx/html /opt/app

# Copy Nginx configuration
COPY generated/ui/nginx.conf /etc/nginx/http.d/default.conf

# Copy built frontend from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html/frontend

# Copy backend and node_modules from builder stage
COPY --from=builder /app/server /opt/app/server
COPY --from=builder /app/node_modules /opt/app/node_modules

# Create startup script
RUN echo '#!/bin/sh' > /start.sh && \
    echo '# Fix docker socket permissions' >> /start.sh && \
    echo 'if [ -S /var/run/docker.sock ]; then' >> /start.sh && \
    echo '  chmod 666 /var/run/docker.sock' >> /start.sh && \
    echo 'fi' >> /start.sh && \
    echo '# Start Node.js backend in background' >> /start.sh && \
    echo 'cd /opt/app && NODE_ENV=production node server/index.js &' >> /start.sh && \
    echo '# Start Nginx in foreground' >> /start.sh && \
    echo 'nginx -g "daemon off;"' >> /start.sh && \
    chmod +x /start.sh

WORKDIR /opt/app

EXPOSE 80

CMD ["/start.sh"]
