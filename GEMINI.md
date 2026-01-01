# Stackvo Projesi - Antigravity AI Kuralları

## Genel Kurallar

### Dil Tercihi

- **Tüm açıklama mesajları Türkçe olmalıdır**
- **Tüm artifact dosyaları (Implementation Plan, Walkthrough, Task) Türkçe yazılmalıdır**
- **Kod içi yorumlar Türkçe olmalıdır**
- **Kullanıcıya gönderilen tüm mesajlar Türkçe olmalıdır**
- **Git commit mesajları İngilizce olmalıdır**

### Git Commit Mesajları

**KRİTİK KURAL**: Her geliştirme/düzeltme sonrasında mutlaka Git commit mesajı hazırlanmalıdır.

**Format**: GitHub Desktop için optimize edilmiş format kullanılmalıdır:

- **Title (Kısa Başlık)**: Maksimum 50 karakter, conventional commits formatında
- **Description (Detaylı Açıklama)**: Çok satırlı, değişiklikleri detaylı açıklayan

**Conventional Commits Prefix'leri**:

- `feat:` - Yeni özellik
- `fix:` - Bug düzeltmesi
- `docs:` - Dokümantasyon değişikliği
- `refactor:` - Kod yeniden yapılandırma
- `perf:` - Performans iyileştirmesi
- `test:` - Test ekleme/düzeltme
- `chore:` - Build/config değişiklikleri
- `style:` - Kod formatı değişiklikleri

**Örnek Format**:

```
Title:
fix: invalid YAML generation when projects directory missing

Description:
Fixed docker-compose.projects.yml generating invalid YAML format when
projects/ directory doesn't exist in fresh install scenario.

Changes:
- Use empty mapping (services: {}) instead of empty block
- Add project counter to handle edge cases
- Recreate file with valid YAML if no valid projects found

Impact:
- Fresh install now starts core services (Traefik + UI) automatically
- docker-compose.projects.yml always generates valid YAML
- Edge cases (empty dir, invalid projects) properly handled

Files Modified:
- cli/lib/generators/project.sh
```

**Kullanım**: Her geliştirme sonunda kullanıcıya şu format ile sunulmalıdır:

```markdown
## Git Commit Mesajı

**Title:**
```

[kısa başlık]

```

**Description:**
```

[detaylı açıklama]

```

```

---

## Proje Yapısı

### CLI Komutları

- Tüm CLI komutları `./cli/stackvo.sh` ile başlar
- Ana komutlar:
  - `generate` - Tüm konfigürasyonları oluşturur
  - `generate projects` - Sadece proje containerlarını oluşturur
  - `generate services` - Sadece servisleri oluşturur
  - `up` - Tüm servisleri başlatır
  - `down` - Tüm servisleri durdurur
  - `restart` - Servisleri yeniden başlatır

### Dizin Yapısı

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

---

**Not**: Bu kurallar Stackvo projesinin dinamik yapısına uygun şekilde tasarlanmıştır. Tüm geliştirmeler bu kurallara uygun olarak yapılmalıdır.
