# Stackvo Projesi - Antigravity AI Kuralları

## Genel Kurallar

### Dil Tercihi

- **Tüm açıklama mesajları Türkçe olmalıdır**
- **Tüm artifact dosyaları (Implementation Plan, Walkthrough, Task) Türkçe yazılmalıdır**
- **Commit mesajları Türkçe olmalıdır**
- **Kod içi yorumlar Türkçe olmalıdır**
- **Kullanıcıya gönderilen tüm mesajlar Türkçe olmalıdır**

### Proje Yapısı

#### CLI Komutları

- Tüm CLI komutları `./cli/stackvo.sh` ile başlar
- Ana komutlar:
  - `generate` - Tüm konfigürasyonları oluşturur
  - `generate projects` - Sadece proje containerlarını oluşturur
  - `generate services` - Sadece servisleri oluşturur
  - `up` - Tüm servisleri başlatır
  - `down` - Tüm servisleri durdurur
  - `restart` - Servisleri yeniden başlatır

#### Dizin Yapısı

```
stackvo/
├── cli/                    # CLI komutları ve kütüphaneler
│   ├── commands/          # Ana komutlar (generate.sh, up.sh, etc.)
│   └── lib/               # Yardımcı kütüphaneler
│       ├── generators/    # Generator modülleri
│       ├── constants.sh   # Sabitler
│       ├── logger.sh      # Log fonksiyonları
│       └── env-loader.sh  # Environment yükleyici
├── core/
│   ├── compose/           # Base docker-compose dosyaları
│   └── templates/         # Template dosyaları
│       ├── servers/       # Web server şablonları (nginx, apache, caddy)
│       ├── services/      # Servis şablonları (mysql, redis, etc.)
│       └── ui/            # UI şablonları
├── generated/             # Otomatik oluşturulan dosyalar (gitignore'da)
│   ├── projects/          # Proje Dockerfile'ları
│   ├── configs/           # Nginx/Apache konfigürasyonları
│   ├── stackvo.yml        # Base compose
│   ├── docker-compose.dynamic.yml    # Dinamik servisler
│   └── docker-compose.projects.yml   # Proje containerları
├── projects/              # Kullanıcı projeleri
│   └── {proje-adı}/
│       ├── stackvo.json   # Proje konfigürasyonu
│       └── public/        # Document root
└── .env                   # Ana konfigürasyon dosyası
```

### Kod Yazım Kuralları

#### Bash Script Kuralları

1. **Fonksiyon Başlıkları**: Her fonksiyon öncesi Türkçe açıklama bloğu ekle

   ```bash
   ##
   # Proje containerlarını oluşturur
   #
   # Parametreler:
   #   $1 - Proje adı
   #   $2 - PHP versiyonu
   ##
   ```

2. **Log Mesajları**: Türkçe log mesajları kullan

   ```bash
   log_info "Proje containerları oluşturuluyor..."
   log_success "Dockerfile başarıyla oluşturuldu"
   log_error "Konfigürasyon dosyası bulunamadı"
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

### Docker Compose Yapısı

#### Compose Dosyaları

Stackvo 3 ayrı compose dosyası kullanır:

1. `generated/stackvo.yml` - Base (Traefik)
2. `generated/docker-compose.dynamic.yml` - Dinamik servisler (.env'den)
3. `generated/docker-compose.projects.yml` - Proje containerları

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

```
feat: PHP extension desteği eklendi
fix: curl bağımlılığı düzeltildi
docs: Türkçe dokümantasyon güncellendi
refactor: Generator fonksiyonları yeniden yapılandırıldı
```

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

---

**Not**: Bu kurallar Stackvo projesinin dinamik yapısına uygun şekilde tasarlanmıştır. Tüm geliştirmeler bu kurallara uygun olarak yapılmalıdır.
