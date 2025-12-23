# STACKORED PROJECT - ANTIGRAVITY RULES

Bu dosya, Stackored projesi için Antigravity AI asistanının uyması gereken kuralları içerir.

---

## 🎯 PROJE HAKKINDA

Stackored, Docker tabanlı modern bir geliştirme ortamı yönetim sistemidir:

- **Pure Bash** generator sistemi
- **PHP tabanlı** RESTful API backend
- **Vue.js 3 + Vuetify 3** web dashboard
- **Traefik** reverse proxy
- **40+ servis şablonu**

### ⚠️ KRİTİK: LOCAL GELİŞTİRME ORTAMI

**ÇOK ÖNEMLİ**: Stackored, **SADECE LOCAL GELİŞTİRME ORTAMI** için tasarlanmıştır. Production/canlı sunucu DEĞİLDİR!

- ❌ **Production kullanımı için tasarlanmamıştır**
- ❌ **Canlı sunucuda çalıştırılmamalıdır**
- ❌ **Public internet'e açılmamalıdır**
- ✅ **Sadece local geliştirme için kullanılmalıdır**
- ✅ **Tüm planlama ve geliştirme local ortam odaklı olmalıdır**
- ✅ **Güvenlik, performans ve optimizasyon kararları local kullanım için alınmalıdır**

**Antigravity Kuralı**: Stackored ile ilgili her türlü özellik, iyileştirme, planlama ve geliştirme kararı alırken **mutlaka local geliştirme ortamı** olduğu göz önünde bulundurulmalıdır. Production senaryoları için öneriler yapılmamalıdır.

---

## 🌍 DİL TERCİHİ

**ÖNEMLİ**: Antigravity, Stackored projesi ile ilgili tüm yanıtları **TÜRKÇE** olarak vermelidir.

- ✅ Tüm açıklamalar Türkçe olmalı
- ✅ Kod yorumları İngilizce olmalı
- ✅ Commit mesajları Türkçe olmalı
- ✅ Dokümantasyon güncellemeleri Türkçe ve İngilizce olmalı
- ✅ Hata mesajları ve loglar İngilizce olmalı
- ✅ Kullanıcı ile iletişim Türkçe olmalı

**İstisna**: Kod içerisindeki değişken isimleri, fonksiyon isimleri ve teknik terimler İngilizce kalabilir (örn: `generate_nginx_container`, `docker-compose`, `API endpoint`).

---

## 🚫 GIT İŞLEMLERİ KURALLARI

**KRİTİK KURAL**: Antigravity, **ASLA** otomatik olarak `git commit` veya `git push` komutlarını çalıştırmamalıdır.

### Yapılması Gerekenler:

✅ **Commit mesajı önerisi sunmak**:

```bash
# Örnek commit mesajı önerisi
git commit -m "fix: GitHub Pages deployment sorunları düzeltildi

- Minify plugin eklendi
- i18n yapılandırması düzeltildi
- use_directory_urls: false ayarı eklendi"
```

✅ **Değişiklikleri göstermek**:

```bash
git status
git diff
```

✅ **Kullanıcıya commit/push yapması için hatırlatmak**

### Yapılmaması Gerekenler:

❌ **Otomatik commit yapmak**:

```bash
# ASLA YAPMA
git add .
git commit -m "..."
```

❌ **Otomatik push yapmak**:

```bash
# ASLA YAPMA
git push origin main
```

❌ **SafeToAutoRun=true ile git commit/push çalıştırmak**

### İş Akışı:

1. **Değişiklikleri yap** → Kod düzenlemeleri, dosya oluşturma
2. **Değişiklikleri göster** → `git status`, `git diff`
3. **Commit mesajı öner** → Kullanıcıya uygun commit mesajı sun
4. **Kullanıcı commit/push yapar** → Antigravity bekler

**Antigravity Kuralı**: Her çözüm sonrası, kullanıcıya commit mesajı önerisi sun ve commit/push yapması için bilgilendir. Asla otomatik commit veya push yapma.

---

## 📁 PROJE YAPISI VE MİMARİ

### Dizin Yapısı

```
stackvo/
├── cli/                    # Bash CLI sistemi
│   ├── stackvo.sh       # Ana CLI giriş noktası
│   ├── commands/          # Komut implementasyonları
│   ├── lib/               # Paylaşılan kütüphaneler
│   │   ├── generators/    # Generator modülleri
│   │   └── uninstallers/  # Uninstaller modülleri
│   └── utils/             # Yardımcı scriptler
├── core/
│   ├── compose/           # Base compose dosyaları
│   └── templates/         # Servis ve sunucu şablonları
│       ├── servers/       # Web sunucu konfigürasyonları
│       ├── services/      # 40+ servis şablonu
│       └── ui/            # UI container şablonları
├── .ui/                   # Web UI (PHP + Vue.js)
│   ├── index.html         # Vue.js 3 SPA
│   ├── api/               # PHP API endpoints
│   ├── lib/               # PHP kütüphaneleri
│   └── config/            # Uygulama konfigürasyonu
├── projects/              # Kullanıcı projeleri
│   └── {project-name}/
│       ├── stackvo.json # Proje konfigürasyonu (ZORUNLU)
│       ├── .stackvo/    # Özel konfigürasyonlar (opsiyonel)
│       └── public/        # Document root
├── generated/             # Otomatik oluşturulan dosyalar
│   ├── stackvo.yml
│   ├── docker-compose.dynamic.yml
│   ├── docker-compose.projects.yml
│   ├── configs/
│   ├── certs/
│   └── traefik/
├── .env                   # Ana konfigürasyon dosyası
└── README.md              # Dokümantasyon
```

---

## 🔧 BASH GENERATOR SİSTEMİ KURALLARI

### 1. **Bash Uyumluluk Kuralları**

**ZORUNLU**: Bash 3.x+ uyumluluğu (macOS için)

```bash
# ❌ KULLANMA - Bash 4+ özellikleri
declare -A assoc_array  # Associative arrays
mapfile -t array        # mapfile komutu

# ✅ KULLAN - Bash 3.x uyumlu
# Indexed arrays kullan
# while read döngüleri kullan
```

### 2. **Template İşleme Kuralları**

**Template Syntax**:

```bash
# Değişken interpolasyonu
{{ VARIABLE_NAME }}  →  ${VARIABLE_NAME}

# Default değerler
{{ VARIABLE_NAME | default('value') }}  →  ${VARIABLE_NAME:-value}
```

**Template Processor** (`cli/lib/template-processor.sh`):

- `render_template()` fonksiyonunu kullan
- `envsubst` ile değişken değiştirme
- `sed` ile syntax dönüşümü

### 3. **Generator Modül Kuralları**

Her generator modülü şu yapıda olmalı:

```bash
#!/bin/bash
###################################################################
# STACKORED {MODULE_NAME} GENERATOR MODULE
# {Description}
###################################################################

generate_{module_name}() {
    log_info "Generating {module_name}..."

    # 1. Dizin oluştur
    mkdir -p "$GENERATED_DIR"

    # 2. Template işle
    render_template "$template_file" > "$output_file"

    # 3. Başarı mesajı
    log_success "Generated {module_name}"
}
```

**Mevcut Generator Modülleri**:

- `compose.sh` - Docker Compose dosyaları
- `project.sh` - Proje container'ları
- `traefik.sh` - Traefik konfigürasyonu
- `tools.sh` - Developer tools
- `config.sh` - Servis konfigürasyonları

### 4. **Logging Kuralları**

```bash
# Kullanılabilir log fonksiyonları (logger.sh)
log_info "Bilgi mesajı"
log_success "Başarı mesajı"
log_warn "Uyarı mesajı"
log_error "Hata mesajı"
```

---

## 🐘 PHP API SİSTEMİ KURALLARI

### 1. **API Endpoint Yapısı**

Her API endpoint şu yapıda olmalı:

```php
<?php
###################################################################
# Stackored UI - {Endpoint Name} API
# {Description}
###################################################################

require_once __DIR__ . '/../lib/api-base.php';
require_once __DIR__ . '/../lib/env.php';
require_once __DIR__ . '/../lib/docker.php';

class {EndpointName}Api extends ApiBase
{
    public function handle()
    {
        // API logic here

        $this->sendSuccess(
            ['data' => $data],
            'Success message',
            ['meta' => 'info']
        );
    }
}

// Run the API
$api = new {EndpointName}Api('/api/{endpoint}.php');
$api->run();
```

### 2. **Response Format Standardı**

```php
// Başarılı response
{
    "success": true,
    "data": { ... },
    "message": "Operation successful",
    "meta": { "count": 10 }
}

// Hata response
{
    "success": false,
    "message": "Error message",
    "error": "Detailed error"
}
```

### 3. **Docker Integration Kuralları**

**Kullanılabilir Fonksiyonlar** (`.ui/lib/docker.php`):

```php
// Container durumu kontrolü
isContainerRunning($containerName)  // bool

// Port mapping bilgisi
getContainerPorts($containerName)   // array
// Returns: ['ports' => [...], 'ip_address' => '...', 'network' => '...', 'gateway' => '...']

// Network bilgileri
getContainerIP($containerName)      // string|null
getContainerNetwork($containerName) // string|null
getContainerGateway($containerName) // string|null
```

### 4. **Caching Kuralları**

```php
// Cache kullanımı (lib/cache.php)
Cache::remember(
    'cache_key',
    function() {
        // Expensive operation
        return $result;
    },
    $ttl_seconds  // 5-10 saniye önerilen
);
```

**Cache TTL Standartları**:

- Container status: 5 saniye
- Port mappings: 10 saniye
- Docker stats: 2 saniye (real-time)

---

## 🎨 VUE.JS WEB UI KURALLARI

### 1. **Component Yapısı**

```javascript
// Vue 3 Composition API kullan (CDN üzerinden)
const { createApp, ref, computed, onMounted } = Vue;
const { createVuetify } = Vuetify;

// Reactive state
const services = ref([]);
const loading = ref(false);

// Computed properties
const runningServicesCount = computed(
  () => services.value.filter((s) => s.running).length
);

// Lifecycle hooks
onMounted(() => {
  loadServices();
});
```

### 2. **API Çağrı Standardı**

```javascript
async function loadServices() {
  loading.value = true;
  try {
    const response = await fetch("/api/services.php");
    const data = await response.json();

    if (data.success) {
      services.value = data.data.services;
    } else {
      console.error("Error:", data.message);
    }
  } catch (error) {
    console.error("Fetch error:", error);
  } finally {
    loading.value = false;
  }
}
```

### 3. **Auto-Refresh Kuralları**

```javascript
// Farklı interval'ler kullan
setInterval(loadDockerStats, 2000); // 2 saniye - Real-time stats
setInterval(loadServices, 5000); // 5 saniye - Services
setInterval(loadProjects, 10000); // 10 saniye - Projects
```

### 4. **Theme Persistence**

```javascript
// LocalStorage kullan
function toggleTheme() {
  const newTheme = theme.global.current.value.dark ? "light" : "dark";
  theme.global.name.value = newTheme;
  localStorage.setItem("stackvo-theme", newTheme);
}

// Sayfa yüklendiğinde
const savedTheme = localStorage.getItem("stackvo-theme") || "dark";
theme.global.name.value = savedTheme;
```

---

## 📝 NAMING CONVENTIONS (İSİMLENDİRME KURALLARI)

### 1. **Container İsimleri**

```bash
# Pattern
stackvo-{service}              # Servisler için
stackvo-{project}-{type}       # Projeler için

# Örnekler
stackvo-mysql
stackvo-postgres
stackvo-project1-web
stackvo-project1-php
stackvo-traefik
stackvo-tools
```

### 2. **Network İsimleri**

```bash
# Tek network kullan
stackvo-net  # Tüm container'lar bu network'te
```

### 3. **Volume İsimleri**

```bash
# Pattern
stackvo-{service}-data

# Örnekler
stackvo-mysql-data
stackvo-postgres-data
stackvo-redis-data
```

### 4. **Domain İsimleri**

```bash
# Pattern
{service}.{TLD_SUFFIX}    # Servisler için
{project}.{TLD_SUFFIX}    # Projeler için (veya custom domain)

# Örnekler (TLD_SUFFIX=stackvo.loc)
traefik.stackvo.loc
adminer.stackvo.loc
rabbitmq.stackvo.loc
project1.loc              # Custom domain
```

### 5. **Environment Variable İsimleri**

```bash
# Service enable flags
SERVICE_{UPPERCASE}_ENABLE=true

# Service configuration
SERVICE_{UPPERCASE}_{PARAM}=value

# Constants (constants.sh)
CONST_{NAME}=value

# Defaults
DEFAULT_{NAME}=value

# Örnekler
SERVICE_MYSQL_ENABLE=true
SERVICE_MYSQL_VERSION=8.0
SERVICE_MYSQL_ROOT_PASSWORD=root
CONST_DEFAULT_PHP_VERSION=8.2
DEFAULT_WEBSERVER=nginx
```

### 6. **Dosya İsimleri**

```bash
# Template dosyaları
docker-compose.{service}.tpl
{service}.conf.tpl

# Generated dosyalar
stackvo.yml
docker-compose.dynamic.yml
docker-compose.projects.yml

# Config dosyaları
stackvo.json           # Proje konfigürasyonu
nginx.conf              # Web server config
php.ini                 # PHP config
```

---

## 🔐 KONFİGÜRASYON KURALLARI

### 1. **`.env` Dosyası Yapısı**

```bash
###################################################################
# SECTION NAME
###################################################################
VARIABLE_NAME=value
ANOTHER_VARIABLE=value

# Boş satır ile ayır
```

**Önemli Bölümler**:

- Traefik Settings
- Default Project Settings
- Docker Network
- Service Toggles (40+ servis)
- Supported Languages

### 2. **`stackvo.json` Yapısı**

```json
{
  "name": "project-name",
  "domain": "project.loc",
  "php": {
    "version": "8.2",
    "extensions": ["pdo", "pdo_mysql", "mysqli"]
  },
  "nodejs": {
    "version": "14.23"
  },
  "python": {
    "version": "3.14"
  },
  "golang": {
    "version": "1.23"
  },
  "ruby": {
    "version": "3.3"
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

**Zorunlu Alanlar**:

- `name` - Proje adı
- `domain` - Domain adı

**Opsiyonel Alanlar**:

- `php`, `nodejs`, `python`, `golang`, `ruby` - Runtime'lar
- `webserver` - nginx/apache/caddy/ferron (default: nginx)
- `document_root` - Document root (default: public)

### 3. **Custom Config Dosyaları**

Proje dizininde `.stackvo/` klasörü oluştur:

```
projects/myproject/
├── stackvo.json
├── .stackvo/
│   ├── nginx.conf       # Custom Nginx config
│   ├── apache.conf      # Custom Apache config
│   ├── Caddyfile        # Custom Caddy config
│   ├── ferron.yaml      # Custom Ferron config
│   ├── php.ini          # Custom PHP config
│   └── php-fpm.conf     # Custom PHP-FPM config
└── public/
    └── index.php
```

**Öncelik Sırası**:

1. `.stackvo/{config}` - Önce özel config ara
2. `{config}` - Proje root'unda ara
3. `core/templates/servers/{webserver}/` - Template kullan

---

## 🚀 YENİ SERVİS EKLEME KURALLARI

### 1. **Template Oluşturma**

```bash
# Dizin oluştur
mkdir -p core/templates/services/{service-name}

# Template dosyası oluştur
touch core/templates/services/{service-name}/docker-compose.{service-name}.tpl
```

### 2. **Template İçeriği**

```yaml
###################################################################
# STACKORED {SERVICE_NAME} COMPOSE TEMPLATE
###################################################################

services:
  { service-name }:
    image: "{service-image}:{{ SERVICE_{UPPERCASE}_VERSION }}"
    container_name: "stackvo-{service-name}"
    restart: unless-stopped

    environment:
      ENV_VAR: "{{ SERVICE_{UPPERCASE}_ENV_VAR | default('default-value') }}"

    volumes:
      - stackvo-{service-name}-data:/data/path
      - ./logs/{service-name}:/var/log/{service-name}

    ports:
      - "{{ HOST_PORT_{UPPERCASE} | default('default-port') }}:{container-port}"

    networks:
      - "{{ DOCKER_DEFAULT_NETWORK }}"

volumes:
  stackvo-{service-name}-data:
```

### 3. **`.env` Değişkenleri Ekle**

```bash
# {Service Name}
SERVICE_{UPPERCASE}_ENABLE=false
SERVICE_{UPPERCASE}_VERSION=latest
SERVICE_{UPPERCASE}_ENV_VAR=value
```

### 4. **Generator'a Ekle**

`cli/lib/generators/compose.sh` dosyasında:

```bash
local services=(
    # ... existing services
    "SERVICE_{UPPERCASE}_ENABLE:services/{service-name}/docker-compose.{service-name}.tpl"
)
```

### 5. **Traefik Route Ekle (Eğer Web UI varsa)**

`cli/lib/generators/traefik.sh` dosyasında:

```bash
# Router ekle
add_router_if_enabled "SERVICE_{UPPERCASE}_ENABLE" "{service-name}" "SERVICE_{UPPERCASE}_URL"

# Service ekle
add_service_if_enabled "SERVICE_{UPPERCASE}_ENABLE" "{service-name}" "{port}"
```

---

## 🌐 YENİ WEB SERVER EKLEME KURALLARI

### 1. **Template Oluşturma**

```bash
# Dizin oluştur
mkdir -p core/templates/servers/{webserver-name}

# Config template oluştur
touch core/templates/servers/{webserver-name}/default.conf
```

### 2. **Project Generator'a Ekle**

`cli/lib/generators/project.sh` dosyasında:

```bash
generate_{webserver}_container() {
    local project_name=$1
    local project_path=$2
    local project_domain=$3
    local document_root=$4
    local host_project_path=$5
    local host_logs_path=$6
    local host_generated_configs_dir=$7

    # Config path belirleme
    local config_mount=""
    if [ -f "$project_path/.stackvo/{webserver}.conf" ]; then
        config_mount="      - ${host_project_path}/.stackvo/{webserver}.conf:/etc/{webserver}/conf.d/default.conf:ro"
    else
        # Template kullan
        mkdir -p "$GENERATED_CONFIGS_DIR"
        local template_file="$ROOT_DIR/core/templates/servers/{webserver}/default.conf"
        local generated_config="$GENERATED_CONFIGS_DIR/${project_name}-{webserver}.conf"

        sed "s/{{PROJECT_NAME}}/${project_name}/g" "$template_file" > "$generated_config"
        config_mount="      - ${host_generated_configs_dir}/${project_name}-{webserver}.conf:/etc/{webserver}/conf.d/default.conf:ro"
    fi

    # Container definition
    cat <<EOF
  ${project_name}-web:
    image: "{webserver-image}:latest"
    container_name: "stackvo-${project_name}-web"
    restart: unless-stopped

    volumes:
      - ${host_project_path}:/var/www/html
      - ${host_logs_path}:/var/log/{webserver}
$config_mount

    networks:
      - ${DOCKER_DEFAULT_NETWORK:-stackvo-net}

    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.${project_name}.rule=Host(\`${project_domain}\`)"
      - "traefik.http.routers.${project_name}.entrypoints=websecure"
      - "traefik.http.routers.${project_name}.tls=true"
      - "traefik.http.services.${project_name}.loadbalancer.server.port=80"

    depends_on:
      - ${project_name}-php

EOF
}
```

### 3. **`generate_web_container()` Fonksiyonuna Ekle**

```bash
case "$web_server" in
    # ... existing cases
    {webserver-name})
        generate_{webserver}_container "$project_name" "$project_path" "$project_domain" "$document_root" "$host_project_path" "$host_logs_path" "$host_generated_configs_dir"
        ;;
esac
```

---

## 🔍 YENİ API ENDPOINT EKLEME KURALLARI

### 1. **Endpoint Dosyası Oluşturma**

```bash
touch .ui/api/{endpoint-name}.php
```

### 2. **Endpoint İçeriği**

```php
<?php
###################################################################
# Stackored UI - {Endpoint Name} API
# {Description}
###################################################################

require_once __DIR__ . '/../lib/api-base.php';
require_once __DIR__ . '/../lib/env.php';
require_once __DIR__ . '/../lib/docker.php';

class {EndpointName}Api extends ApiBase
{
    /**
     * Handle API request
     */
    public function handle()
    {
        try {
            // 1. Validate input
            $param = $_GET['param'] ?? null;
            if (!$param) {
                $this->sendError('Parameter required');
                return;
            }

            // 2. Process request
            $result = $this->processRequest($param);

            // 3. Send response
            $this->sendSuccess(
                ['data' => $result],
                'Operation successful',
                ['count' => count($result)]
            );

        } catch (Exception $e) {
            $this->sendError('Error: ' . $e->getMessage());
        }
    }

    /**
     * Process the request
     */
    private function processRequest($param)
    {
        // Implementation
        return [];
    }
}

// Run the API
$api = new {EndpointName}Api('/api/{endpoint-name}.php');
$api->run();
```

### 3. **Vue.js'te Kullanım**

```javascript
// API fonksiyonu ekle
async function load{EndpointName}() {
    loading.value = true
    try {
        const response = await fetch('/api/{endpoint-name}.php?param=value')
        const data = await response.json()

        if (data.success) {
            // Handle success
            console.log(data.data)
        } else {
            console.error('Error:', data.message)
        }
    } catch (error) {
        console.error('Fetch error:', error)
    } finally {
        loading.value = false
    }
}
```

---

## 🐛 HATA AYIKLAMA KURALLARI

### 1. **Bash Script Debugging**

```bash
# Debug mode aktif et
set -x  # Her komutu yazdır
set -e  # Hata durumunda dur
set -o pipefail  # Pipe'da hata kontrolü

# Veya hepsini birlikte
set -xeo pipefail
```

### 2. **PHP Error Logging**

```php
// Logger kullan (lib/logger.php)
Logger::debug('Debug message', ['data' => $data]);
Logger::info('Info message');
Logger::warning('Warning message');
Logger::error('Error message', ['error' => $e->getMessage()]);
```

### 3. **JavaScript Console Logging**

```javascript
// Detaylı log
console.log("Services loaded:", services.value);
console.error("Error loading services:", error);

// Vuetify dev tools kullan
// Vue DevTools browser extension yükle
```

### 4. **Docker Debugging**

```bash
# Container loglarını izle
docker logs -f stackvo-{service}

# Container içine gir
docker exec -it stackvo-{service} bash

# Network kontrolü
docker network inspect stackvo-net

# Volume kontrolü
docker volume inspect stackvo-{service}-data
```

---

## 📊 PERFORMANS KURALLARI

### 1. **Caching Stratejisi**

```php
// Pahalı işlemleri cache'le
Cache::remember('expensive_operation', function() {
    // Expensive Docker API call
    return $result;
}, 10);  // 10 saniye TTL
```

### 2. **Lazy Loading**

```javascript
// Sadece görünür tab'ı yükle
<div v-show="currentView === 'services'">
    <!-- Services content -->
</div>
```

### 3. **Debouncing**

```javascript
// Search input için debounce kullan
let searchTimeout;
function onSearchInput(value) {
  clearTimeout(searchTimeout);
  searchTimeout = setTimeout(() => {
    performSearch(value);
  }, 300);
}
```

---

## 🔒 GÜVENLİK KURALLARI

### 1. **Input Validation**

```php
// PHP'de
$containerName = escapeshellarg($_POST['container'] ?? '');
exec("docker inspect $containerName", $output, $returnCode);

// Regex ile validate et
if (!preg_match('/^[a-z0-9-]+$/', $containerName)) {
    throw new Exception('Invalid container name');
}
```

### 2. **CORS Headers**

```php
// Her API endpoint'te CORS header'ları ekle
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');
```

### 3. **Environment Variables**

```bash
# Hassas bilgileri .env'de sakla
# .gitignore'a .env ekle
echo ".env" >> .gitignore

# Varsayılan değerler kullan
MYSQL_ROOT_PASSWORD=${MYSQL_ROOT_PASSWORD:-root}
```

---

## 📚 DOKÜMANTASYON KURALLARI

### 1. **Bash Function Documentation**

```bash
##
# Function description
#
# Arguments:
#   $1 - First argument description
#   $2 - Second argument description
#
# Returns:
#   0 - Success
#   1 - Error
##
function_name() {
    # Implementation
}
```

### 2. **PHP DocBlocks**

```php
/**
 * Function description
 *
 * @param string $param Parameter description
 * @return array Result description
 * @throws Exception When something goes wrong
 */
function functionName($param) {
    // Implementation
}
```

### 3. **README Güncellemeleri**

Yeni özellik eklendiğinde README.md'yi güncelle:

- Özellik listesine ekle
- Kullanım örneği ekle
- Konfigürasyon bilgisi ekle

---

## 🧪 TEST KURALLARI

### 1. **Generator Test**

```bash
# Test et
./cli/stackvo.sh generate

# Oluşturulan dosyaları kontrol et
ls -la generated/

# Syntax kontrolü
docker compose -f generated/stackvo.yml config
docker compose -f generated/docker-compose.dynamic.yml config
docker compose -f generated/docker-compose.projects.yml config
```

### 2. **API Test**

```bash
# cURL ile test et
curl http://localhost/api/services.php

# JSON formatını kontrol et
curl http://localhost/api/services.php | jq .
```

### 3. **UI Test**

```javascript
// Browser console'da test et
await loadServices();
console.log(services.value);

await loadProjects();
console.log(projects.value);
```

---

## 🎯 ÖNEMLİ HATIRLATMALAR

### ✅ YAPILMASI GEREKENLER

1. **Her zaman `.env` dosyasını kontrol et** - Tüm konfigürasyon buradan
2. **Template syntax'ını doğru kullan** - `{{ VAR }}` veya `{{ VAR | default('value') }}`
3. **Naming convention'lara uy** - `stackvo-{service}`, `SERVICE_{UPPERCASE}_ENABLE`
4. **Log fonksiyonlarını kullan** - `log_info`, `log_success`, `log_warn`, `log_error`
5. **Bash 3.x uyumluluğunu koru** - macOS için kritik
6. **Cache kullan** - Docker API çağrıları pahalı
7. **Error handling ekle** - Try-catch, return code kontrolü
8. **CORS header'ları ekle** - API endpoint'lerinde
9. **Input validation yap** - Güvenlik için kritik
10. **Dokümantasyon güncelle** - Yeni özellik eklendiğinde

### ❌ YAPILMAMASI GEREKENLER

1. **Bash 4+ özellikleri kullanma** - Associative arrays, mapfile
2. **Hardcoded değerler kullanma** - Her şey `.env`'den gelmeli
3. **Farklı network'ler oluşturma** - Tek network: `stackvo-net`
4. **Container isimlerini değiştirme** - Pattern: `stackvo-{service}`
5. **Template syntax'ını bozma** - `{{ VAR }}` formatını koru
6. **Cache'siz Docker API çağrısı yapma** - Performans sorunu
7. **CORS header'larını unutma** - API çalışmaz
8. **Input validation atlama** - Güvenlik riski
9. **Error handling atlama** - Kullanıcı deneyimi kötü
10. **README güncellemeden özellik ekleme** - Dokümantasyon eksik kalır

---

## 🚀 HIZLI REFERANS

### Sık Kullanılan Komutlar

```bash
# Generator çalıştır
./cli/stackvo.sh generate

# Servisleri başlat
./cli/stackvo.sh up

# Servisleri durdur
./cli/stackvo.sh down

# Logları izle
./cli/stackvo.sh logs

# Durum kontrolü
./cli/stackvo.sh ps
```

### Sık Kullanılan Dosya Yolları

```bash
# Ana konfigürasyon
.env

# Generator modülleri
cli/lib/generators/

# Servis şablonları
core/templates/services/

# API endpoints
.ui/api/

# Web UI
.ui/index.html

# Oluşturulan dosyalar
generated/
```

### Sık Kullanılan API Endpoints

```bash
# Servisler
GET /api/services.php

# Projeler
GET /api/projects.php

# Docker stats
GET /api/docker-stats.php

# Container kontrolü
POST /api/control.php

# Proje oluştur
POST /api/create-project.php

# Proje sil
POST /api/delete-project.php
```

---

## 📖 EK KAYNAKLAR

### Resmi Dokümantasyon

- Docker Compose: https://docs.docker.com/compose/
- Traefik: https://doc.traefik.io/traefik/
- Vue.js 3: https://vuejs.org/
- Vuetify 3: https://vuetifyjs.com/

### Proje Dokümantasyonu

- README.md - Ana dokümantasyon (1480 satır)
- stackvo_analysis.md - Detaylı kod analizi

---

## SON NOTLAR

Bu kurallar, Stackored projesinin tutarlılığını ve kalitesini korumak için tasarlanmıştır. Yeni özellik eklerken veya mevcut kodu değiştirirken bu kurallara uyulması kritik öneme sahiptir.

**Proje Felsefesi**:

- ✅ Basitlik (Convention over Configuration)
- ✅ Modülerlik (Her şey ayrı modül)
- ✅ Esneklik (Kolay özelleştirme)
- ✅ Performans (Caching, lazy loading)
- ✅ Güvenlik (Input validation, CORS)
- ✅ Dokümantasyon (Her şey dokümante)

**Hedef**: Production-ready, enterprise-grade, kullanımı kolay bir geliştirme ortamı yönetim sistemi.
