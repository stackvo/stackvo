# Stackvo UI - Node.js + Express.js + Vue.js

Modern web-based dashboard for Stackvo Docker development environment.

## 📁 Dizin Yapısı

```
.ui/
├── backend/                 # Node.js + Express.js API
│   ├── src/
│   │   ├── server.js       # Ana server
│   │   ├── routes/         # API endpoints
│   │   ├── services/       # Business logic
│   │   ├── middleware/     # Express middleware
│   │   └── utils/          # Yardımcı fonksiyonlar
│   ├── package.json
│   └── .env.example
│
└── frontend/                # Vue.js 3 + Vuetify 3 SPA
    ├── src/
    │   ├── main.js         # Entry point
    │   ├── App.vue         # Ana component
    │   ├── router/         # Vue Router
    │   ├── stores/         # Pinia stores
    │   ├── views/          # Sayfa component'leri
    │   ├── components/     # Reusable component'ler
    │   └── plugins/        # Vuetify config
    ├── public/
    ├── index.html
    ├── package.json
    └── vite.config.js
```

## 🚀 Kurulum

### Backend

```bash
cd .ui/backend

# .env dosyası oluştur
cp .env.example .env

# Bağımlılıkları kur
npm install

# Development mode
npm run dev

# Production mode
npm start
```

### Frontend

```bash
cd .ui/frontend

# Bağımlılıkları kur
npm install

# Development server (HMR ile)
npm run dev

# Production build
npm run build

# Production preview
npm run preview
```

## 🔌 API Endpoints

### Services

- `GET /api/services` - Tüm servisleri listele
- `POST /api/services/:containerName/start` - Servisi başlat
- `POST /api/services/:containerName/stop` - Servisi durdur
- `POST /api/services/:containerName/restart` - Servisi yeniden başlat

### Projects

- `GET /api/projects` - Tüm projeleri listele
- `POST /api/projects/:containerName/start` - Projeyi başlat
- `POST /api/projects/:containerName/stop` - Projeyi durdur
- `POST /api/projects/:containerName/restart` - Projeyi yeniden başlat

### Docker

- `GET /api/docker/stats/:containerName` - Container istatistikleri

### Environment

- `GET /api/env` - Environment variables

### WebSocket

- `socket.io` - Terminal emulation için WebSocket bağlantısı

## 🛠️ Teknolojiler

### Backend

- **Node.js** 18+
- **Express.js** 4.x - Web framework
- **Socket.io** 4.x - WebSocket server
- **dockerode** 4.x - Docker API client
- **node-pty** 1.x - Terminal emulation
- **node-cache** 5.x - In-memory caching

### Frontend

- **Vue.js** 3.4+ - Progressive framework
- **Vuetify** 3.5+ - Material Design component framework
- **Vue Router** 4.x - Routing
- **Pinia** 2.x - State management
- **Axios** 1.x - HTTP client
- **Socket.io Client** 4.x - WebSocket client
- **xterm.js** 5.x - Terminal emulator
- **Vite** 5.x - Build tool

## 📝 Development

### Backend Development

```bash
cd .ui/backend
npm run dev  # nodemon ile auto-reload
```

### Frontend Development

```bash
cd .ui/frontend
npm run dev  # Vite dev server (http://localhost:5173)
```

API proxy otomatik olarak `http://localhost:3000`'e yönlendirilir.

## 🔄 Migration from PHP

Bu proje, önceki PHP backend'den Node.js + Express.js'e geçiş yapılarak oluşturulmuştur.

**Avantajlar:**

- ⚡ 3-5x daha hızlı Docker API çağrıları (dockerode)
- 📡 Real-time updates (WebSocket)
- 🎯 Tek process (terminal entegrasyonu built-in)
- 🚀 Modern development (HMR, TypeScript desteği)
- 💾 Daha iyi caching ve performans

## 📦 Production Build

```bash
# Frontend build
cd .ui/frontend
npm run build  # dist/ klasörüne build edilir

# Backend production
cd .ui/backend
NODE_ENV=production npm start
```

## 🔐 Environment Variables

### Backend (.ui/backend/.env)

```env
PORT=3000
NODE_ENV=development
DOCKER_SOCKET=/var/run/docker.sock
CACHE_TTL=5
```

## 🐛 Troubleshooting

### npm install hatası (WSL/Windows)

Eğer WSL üzerinde Windows npm kullanıyorsanız UNC path hatası alabilirsiniz:

```bash
# Linux native Node.js kur
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Tekrar dene
npm install
```

### Docker socket erişim hatası

```bash
# Docker socket permission
sudo chmod 666 /var/run/docker.sock
```

## 📄 License

MIT
