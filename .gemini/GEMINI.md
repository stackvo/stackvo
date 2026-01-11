# Stackvo Projesi - Antigravity AI Kuralları

## Genel Kurallar

### Dil Tercihi

- **Tüm açıklama mesajları Türkçe olmalıdır**
- **Tüm artifact dosyaları (Implementation Plan, Walkthrough, Task) Türkçe yazılmalıdır**
- **Commit mesajları İngilizce olmalıdır** (Conventional Commits formatında)
- **Kullanıcıya gönderilen tüm mesajlar Türkçe olmalıdır**

### Proje Yapısı

#### CLI Komutları

- Tüm CLI komutları `./stackvo.sh` ile başlar (root dizinindeki wrapper script)
- Wrapper script, gerçek CLI'yi (`core/cli/stackvo.sh`) çağırır
- Ana komutlar:
  - `generate` - Tüm konfigürasyonları oluşturur
  - `generate projects` - Sadece proje containerlarını oluşturur
  - `generate services` - Sadece servisleri oluşturur
  - `install` - Stackvo'yu kurar ve başlatır
  - `uninstall` - Stackvo'yu tamamen kaldırır
  - `up` - Tüm servisleri başlatır
  - `down` - Tüm servisleri durdurur
  - `restart` - Servisleri yeniden başlatır
  - `pull` - Docker image'larını günceller

#### Dizin Yapısı

```
stackvo/
├── stackvo.sh                          # CLI wrapper script (root entry point)
├── .env.example                        # Örnek konfigürasyon dosyası
├── core/                               # Stackvo çekirdek dosyaları
│   ├── cli/                            # CLI komutları ve kütüphaneler
│   │   ├── stackvo.sh                  # Ana CLI script (gerçek entry point)
│   │   ├── commands/                   # Ana komutlar
│   │   │   ├── generate.sh             # Generate komutu
│   │   │   ├── install.sh              # Install komutu
│   │   │   ├── uninstall.sh            # Uninstall komutu
│   │   │   └── pull.sh                 # Pull komutu
│   │   ├── lib/                        # Yardımcı kütüphaneler
│   │   │   ├── generators/             # Generator modülleri
│   │   │   │   ├── project.sh          # Proje generator
│   │   │   │   ├── compose.sh          # Compose generator
│   │   │   │   ├── traefik.sh          # Traefik generator
│   │   │   │   ├── config.sh           # Config generator
│   │   │   │   ├── tools.sh            # Tools generator
│   │   │   │   ├── stackvo-ui.sh       # UI generator
│   │   │   │   ├── project/            # Proje alt-modülleri
│   │   │   │   └── common/             # Ortak fonksiyonlar
│   │   │   ├── uninstallers/           # Uninstaller modülleri
│   │   │   ├── constants.sh            # Sabitler
│   │   │   ├── logger.sh               # Log fonksiyonları
│   │   │   ├── env-loader.sh           # Environment yükleyici
│   │   │   ├── common.sh               # Ortak yardımcı fonksiyonlar
│   │   │   ├── php-extensions.sh       # PHP extension yönetimi
│   │   │   └── template-processor.sh   # Template işleyici
│   │   └── utils/                      # Yardımcı araçlar
│   ├── ui/                             # Web UI kaynak kodları
│   │   ├── client/                     # Vue.js frontend
│   │   │   ├── src/                    # Kaynak kodlar
│   │   │   ├── public/                 # Statik dosyalar
│   │   │   └── package.json
│   │   ├── server/                     # Node.js backend (Express)
│   │   │   ├── routes/                 # API route'ları
│   │   │   ├── services/               # Backend servisleri
│   │   │   ├── utils/                  # Yardımcı fonksiyonlar
│   │   │   └── index.js                # Server entry point
│   │   ├── dist/                       # Build output (production)
│   │   ├── package.json                # NPM dependencies
│   │   ├── vite.config.js              # Vite konfigürasyonu
│   │   └── README.md                   # UI dokümantasyonu
│   ├── compose/                        # Base docker-compose
│   │   └── stackvo.yml                 # Base compose template
│   └── templates/                      # Template dosyaları
│       ├── servers/                    # Web server şablonları
│       │   ├── nginx/                  # Nginx konfigürasyonları
│       │   ├── apache/                 # Apache konfigürasyonları
│       │   └── caddy/                  # Caddy konfigürasyonları
│       ├── services/                   # Servis şablonları
│       │   ├── mysql/                  # MySQL
│       │   ├── mariadb/                # MariaDB
│       │   ├── postgres/               # PostgreSQL
│       │   ├── redis/                  # Redis
│       │   ├── mongo/                  # MongoDB
│       │   ├── elasticsearch/          # Elasticsearch
│       │   ├── kibana/                 # Kibana
│       │   ├── rabbitmq/               # RabbitMQ
│       │   ├── kafka/                  # Kafka
│       │   ├── memcached/              # Memcached
│       │   ├── mailhog/                # MailHog
│       │   ├── blackfire/              # Blackfire
│       │   └── ...                     # Diğer servisler
│       └── ui/                         # UI şablonları
├── docs/                               # MkDocs dokümantasyonu (gitignore'da)
│   ├── en/                             # İngilizce dokümantasyon
│   │   ├── index.md
│   │   ├── getting-started/
│   │   ├── guides/
│   │   └── references/
│   ├── tr/                             # Türkçe dokümantasyon
│   │   ├── index.md
│   │   ├── getting-started/
│   │   ├── guides/
│   │   └── references/
│   ├── assets/                         # Dokümantasyon varlıkları (gitignore'da)
│   ├── screenshots/                    # Ekran görüntüleri (gitignore'da)
│   ├── overrides/                      # MkDocs tema özelleştirmeleri (gitignore'da)
│   └── scripts/                        # Dokümantasyon scriptleri (gitignore'da)
│       ├── generate-changelog.sh
│       └── release.sh
├── logs/                               # Log dizini (gitignore'da)
│   └── services/                       # Servis log'ları
│       ├── mysql/
│       ├── redis/
│       ├── nginx/
│       └── ...
├── generated/                          # Otomatik oluşturulan dosyalar (gitignore'da)
│   ├── projects/                       # Proje Dockerfile'ları
│   │   └── {proje-adı}/
│   │       └── Dockerfile
│   ├── configs/                        # Web server konfigürasyonları
│   │   ├── {proje-adı}-nginx.conf
│   │   ├── {proje-adı}-apache.conf
│   │   └── {proje-adı}-caddy.conf
│   ├── stackvo.yml                     # Base compose (Traefik)
│   ├── docker-compose.dynamic.yml      # Dinamik servisler
│   └── docker-compose.projects.yml     # Proje containerları
└── projects/                           # Kullanıcı projeleri
    └── {proje-adı}/
        ├── stackvo.json                # Proje konfigürasyonu
        ├── .stackvo/                   # Özel konfigürasyonlar (opsiyonel)
        │   ├── Dockerfile              # Custom Dockerfile
        │   └── nginx.conf              # Custom nginx config
        └── public/                     # Document root (varsayılan)
```

### Kod Yazım Kuralları

#### Bash Script Kuralları

1. **Fonksiyon Başlıkları**: Her fonksiyon öncesi İngilizce açıklama bloğu ekle

   ```bash
   ##
   # Generate project containers
   #
   # Parameters:
   #   $1 - Project name
   #   $2 - PHP version
   ##
   ```

2. **Log Mesajları**: İngilizce log mesajları kullan

   ```bash
   log_info "Generating project containers..."
   log_success "Dockerfile created successfully"
   log_error "Configuration file not found"
   ```

3. **Değişken İsimlendirme**: İngilizce snake_case kullan
   ```bash
   local project_name=$1
   local php_version=$2
   local extensions=$3
   ```

#### PHP Extension Desteği

- `stackvo.json` dosyasındaki `extensions` alanı dinamik Dockerfile oluşturur
- Her extension için gerekli sistem bağımlılıkları otomatik yüklenir
- Desteklenen extension'lar ve bağımlılıkları:
  - `gd` → libpng-dev, libjpeg-dev, libfreetype6-dev
  - `zip` → libzip-dev
  - `curl` → libcurl4-openssl-dev
  - `mbstring` → libonig-dev
  - `pgsql`, `pdo_pgsql` → libpq-dev
  - `intl` → libicu-dev
  - `soap` → libxml2-dev

#### Compose Dosyaları

Stackvo 3 ayrı compose dosyası kullanır:

1. `generated/stackvo.yml` - Base (Traefik)
2. `generated/docker-compose.dynamic.yml` - Dinamik servisler (.env'den)
3. `generated/docker-compose.projects.yml` - Proje containerları

#### Docker Compose Profiles

**KRİTİK KURAL**: Tüm servis template'leri Docker Compose **profiles** kullanır!

**Etkilenen Servisler (14 adet):**

- blackfire, mailhog, cassandra, kafka, redis, mongo, mysql, mariadb, kibana, postgres, grafana, memcached, elasticsearch, rabbitmq

**Profile Yapısı:**

```yaml
services:
  blackfire:
    profiles: ["services", "blackfire"] # Her servis kendi profile'ına sahip
    # ...
```

**Doğru Kullanım:**

```bash
# ✅ DOĞRU - Profile ile başlatma
docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml --profile blackfire up -d blackfire

# ❌ YANLIŞ - Profile olmadan (çalışmaz!)
docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml up -d blackfire
```

**UI Backend'de Kullanım:**

```javascript
// enableService fonksiyonunda
const upResult = await execAsync(
  `docker-compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml --profile ${serviceName} up -d --build ${serviceName} 2>&1`,
  { cwd: rootDir }
);
```

**Neden Profile Kullanılıyor:**

- ✅ Servisleri gruplandırma (core, services, tools)
- ✅ Seçici başlatma (sadece istenen servisler)
- ✅ Resource yönetimi (gereksiz container'lar başlamaz)

**Antigravity Kuralı**: Servis enable/disable işlemlerinde **MUTLAKA** `--profile {serviceName}` parametresi kullanılmalıdır. Aksi halde Docker Compose servisi bulamaz ve hata verir.

---

#### Servis Template Kuralları

**KRİTİK**: Aşağıdaki kurallar tüm servis template'leri için geçerlidir. Bu kurallar sayesinde fresh install'da bile tüm servisler sorunsuz çalışır.

##### 1. depends_on Kullanımı

**KURAL**: Servis template'lerinde `depends_on` kullanılmamalıdır.

**Neden**: Local development ortamında servisler birbirinden bağımsız çalışabilmeli. Kullanıcı istediği servisi enable edebilmeli.

**İstisna**: Aynı template içinde tanımlı internal servisler (örn: Sentry → sentry-redis, sentry-postgres)

**Etkilenen Servisler**:

- ✅ Kibana (Elasticsearch bağımlılığı kaldırıldı)
- ✅ Logstash (Elasticsearch bağımlılığı kaldırıldı)
- ✅ Kafka (Zookeeper bağımlılığı kaldırıldı - aynı template içinde)
- ✅ Sentry (Redis/Postgres bağımlılığı kaldırıldı - aynı template içinde)

**Yanlış**:

```yaml
services:
  kibana:
    depends_on:
      - elasticsearch # ❌ Elasticsearch enable değilse hata verir
```

**Doğru**:

```yaml
services:
  kibana:
    environment:
      ELASTICSEARCH_HOSTS: "http://stackvo-elasticsearch:9200"
    # depends_on yok - Elasticsearch olmadan da başlar
```

##### 2. Log Volume Mount Kuralları

**KURAL**: Servis log'ları `logs/services/{service-name}/` dizinine mount edilmelidir.

**Dizin Yapısı**:

```
stackvo/
└── logs/                   # Log dizini (gitignore'da)
    └── services/           # Servis log'ları
        ├── mysql/
        ├── redis/
        ├── nginx/
        └── ...
```

**Doğru Kullanım**:

```yaml
volumes:
  - ../logs/services/mysql:/var/log/mysql
  - ../logs/services/nginx:/var/log/nginx
```

**Not**: Docker Compose relative path'leri compose dosyasının bulunduğu dizine göre resolve eder. `generated/*.yml` dosyaları için `../logs` kullanılmalıdır (bir üst dizin).

##### 3. Config Dosya Yolu Kuralları

**KURAL**: Template'lerde config dosya yolları `../core/templates/services/{service}/` ile başlamalıdır.

**Neden**: Docker Compose `generated/` dizininden çalıştığı için relative path bir üst dizinden başlamalı.

**Etkilenen Servisler**:

- ✅ Logstash (config yolu düzeltildi)
- ✅ Kong (config mount kaldırıldı)

**Doğru**:

```yaml
volumes:
  - ../core/templates/services/logstash/logstash.conf:/usr/share/logstash/pipeline/logstash.conf:ro
```

**Yanlış**:

```yaml
volumes:
  - ./core/templates/services/logstash/logstash.conf:/usr/share/logstash/pipeline/logstash.conf:ro # ❌ Yol bulunamaz
  - ./core/templates/monitoring/logstash/logstash.conf:/usr/share/logstash/pipeline/logstash.conf:ro # ❌ Yanlış dizin
```

##### 4. Minimal Konfigürasyon Kuralı

**KURAL**: Servisler minimal konfigürasyonla çalışmalıdır. Credentials olmadan da container ayakta kalabilmeli.

**Neden**: Local development ortamı - production değil. Kullanıcı servisi test edebilmeli.

**Etkilenen Servisler**:

- ✅ Kong (declarative config kaldırıldı - DB-less modda minimal çalışıyor)
- ✅ Blackfire (credentials opsiyonel - restart olabilir ama crash etmez)
- ✅ Logstash (Elasticsearch output kaldırıldı - sadece stdout kullanıyor)

**Örnek - Kong**:

```yaml
environment:
  KONG_DATABASE: "off" # DB-less mod
  # KONG_DECLARATIVE_CONFIG yok - minimal konfigürasyon
```

**Örnek - Logstash**:

```conf
output {
  # Minimal config - stdout only
  # For production, add Elasticsearch output with proper configuration
  stdout {
    codec => rubydebug
  }
}
```

**Antigravity Kuralı**: Yeni servis template'i oluştururken bu 4 kurala mutlaka uyulmalıdır. Aksi halde fresh install'da sorunlar yaşanır.

---

#### Container İsimlendirme

- Format: `stackvo-{proje-adı}-{servis-türü}`
- Örnekler:
  - `stackvo-php-laravel-php`
  - `stackvo-php-laravel-web`
  - `stackvo-mysql`
  - `stackvo-redis`

### stackvo.json Yapısı

```json
{
  "name": "proje-adi",
  "domain": "proje-adi.loc",
  "php": {
    "version": "8.2",
    "extensions": [
      "pdo",
      "pdo_mysql",
      "mysqli",
      "gd",
      "curl",
      "zip",
      "mbstring"
    ]
  },
  "webserver": "nginx",
  "document_root": "public"
}
```

**Önemli Alanlar:**

- `name`: Proje adı (container isimleri için kullanılır)
- `domain`: Traefik routing için domain
- `php.version`: PHP versiyonu (8.0, 8.1, 8.2, 8.3, 8.4)
- `php.extensions`: Kurulacak PHP extension'ları (artık kullanılıyor!)
- `webserver`: nginx, apache, caddy
- `document_root`: Web root dizini (varsayılan: public)

### Generator Sistemi

#### Generator Modülleri

- `cli/lib/generators/project.sh` - Proje containerları
- `cli/lib/generators/compose.sh` - Docker Compose dosyaları
- `cli/lib/generators/traefik.sh` - Traefik konfigürasyonu
- `cli/lib/generators/config.sh` - Servis konfigürasyonları
- `cli/lib/generators/tools.sh` - Tools container

#### Dockerfile Oluşturma

1. `parse_project_config()` - stackvo.json'dan bilgileri çıkarır
2. `generate_php_dockerfile()` - Extension'lara göre Dockerfile oluşturur
3. `generate_php_container()` - docker-compose.yml'e build context ekler

### Environment Variables

#### Önemli .env Değişkenleri

```bash
# Varsayılan Ayarlar
DEFAULT_PHP_VERSION=8.2
DEFAULT_WEBSERVER=nginx
DEFAULT_DOCUMENT_ROOT=public

# Docker Network
DOCKER_DEFAULT_NETWORK=stackvo-net

# SSL
SSL_ENABLE=true
REDIRECT_TO_HTTPS=true

# Servis Ayarları
SERVICE_MYSQL_ENABLE=false
SERVICE_MARIADB_ENABLE=true
SERVICE_REDIS_ENABLE=false
```

### Geliştirme Kuralları

#### Yeni Feature Ekleme

1. **Planning**: Implementation plan oluştur (Türkçe)
2. **Implementation**: Kodu yaz, Türkçe yorumlar ekle
3. **Testing**: Test et ve doğrula
4. **Documentation**: Walkthrough oluştur (Türkçe)

#### Commit Mesajları

**Format:** [Conventional Commits](https://www.conventionalcommits.org/) standardı kullanılır.

```
<type>: <description>

[optional body]

[optional footer]
```

**Tipler:**

- `feat:` - Yeni özellik
- `fix:` - Hata düzeltmesi
- `docs:` - Dokümantasyon değişiklikleri
- `refactor:` - Kod yeniden yapılandırma
- `perf:` - Performans iyileştirmeleri
- `test:` - Test ekleme veya güncelleme
- `chore:` - Bakım işleri

**Örnekler:**

```bash
# İyi commit mesajları (İngilizce)
feat: add PostgreSQL 15 support
fix: resolve Traefik SSL certificate error
docs: update installation guide for macOS
refactor: extract common generator functions
perf: optimize Docker image build time

# Kötü commit mesajları
update readme
fixed bug
improvements
```

**Not:** Commit mesajları **İngilizce** yazılmalıdır (CONTRIBUTING.md standardına uygun).

### Artifact Kuralları

#### Implementation Plan (Türkçe)

```markdown
# [Özellik Adı] - Uygulama Planı

## Genel Bakış

[Özelliğin açıklaması]

## Kullanıcı Onayı Gerekli

> [!IMPORTANT] > **Breaking Change**: [Değişiklik açıklaması]

## Önerilen Değişiklikler

### [Bileşen Adı]

[Değişiklik detayları]

## Doğrulama Planı

[Test adımları]
```

#### Walkthrough (Türkçe)

```markdown
# [Özellik Adı] - Uygulama Özeti

## Genel Bakış

[Yapılan değişikliklerin özeti]

## Yapılan Değişiklikler

[Detaylı değişiklik listesi]

## Test Sonuçları

[Test sonuçları ve doğrulama]

## Kullanım Örneği

[Örnekler]
```

#### Task (Türkçe)

```markdown
# [Özellik Adı] - Görev Listesi

## Planlama

- [x] Proje yapısı analizi
- [x] Tasarım

## Uygulama

- [/] Kod yazımı
- [ ] Test

## Doğrulama

- [ ] Manuel test
- [ ] Otomatik test
```

### Özel Durumlar

#### Custom Dockerfile

Eğer proje dizininde `.stackvo/Dockerfile` varsa, o kullanılır (öncelik sırası):

1. `{proje}/.stackvo/Dockerfile`
2. `generated/projects/{proje}/Dockerfile` (otomatik oluşturulan)

#### Custom Nginx Config

Öncelik sırası:

1. `{proje}/.stackvo/nginx.conf`
2. `{proje}/nginx.conf`
3. `generated/configs/{proje}-nginx.conf` (otomatik oluşturulan)

### Hata Ayıklama

#### Log Dosyaları

- Generator logs: `core/generator.log`
- Container logs: `logs/projects/{proje}/`

#### Yaygın Sorunlar

1. **Extension kurulumu başarısız**: Sistem bağımlılığı eksik olabilir
2. **Build yavaş**: İlk build 3-5 dakika sürer, sonrası cache'den hızlı
3. **Network hatası**: `docker network create stackvo-net` çalıştır

### Best Practices

1. **Her zaman `./cli/stackvo.sh generate` çalıştır** - Değişikliklerden sonra
2. **Extension değişikliklerinde `--no-cache` kullan** - Temiz build için
3. **Türkçe mesajlar kullan** - Kullanıcı deneyimi için
4. **Kod yorumları ekle** - Bakım kolaylığı için
5. **Test et** - Her değişiklikten sonra
6. **UI değişikliklerinden sonra build et** - `.ui/client` veya `.ui/server` dizinlerinde değişiklik yapıldığında mutlaka aşağıdaki komutu çalıştır:
   ```bash
   docker compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml build stackvo-ui && docker compose -f generated/stackvo.yml -f generated/docker-compose.dynamic.yml up -d stackvo-ui
   ```

---

**Not**: Bu kurallar Stackvo projesinin dinamik yapısına uygun şekilde tasarlanmıştır. Tüm geliştirmeler bu kurallara uygun olarak yapılmalıdır.
