# CLI Komutları Referansı

Stackvo CLI komutlarının tam referansı. Bu sayfa, generate, up, down, restart, ps, logs, pull, doctor, install ve uninstall komutlarının syntax'ını, parametrelerini, seçeneklerini ve kullanım örneklerini detaylı olarak açıklamaktadır. Ayrıca environment variables, exit codes ve Docker Compose eşdeğerleri de içermektedir.

## Kurulum

```bash
./core/cli/stackvo.sh install
```

Kurulumdan sonra `stackvo` komutu sistem genelinde kullanılabilir.

---

## Komutlar

### generate

Konfigürasyon dosyalarını üretir.

**Syntax:**
```bash
./core/cli/stackvo.sh generate [MODE] [OPTIONS]
```

**Modes:**
- (boş) - Tümünü üret
- `projects` - Sadece projeleri üret
- `services` - Sadece servisleri üret

**Options:**
- `--uninstall-tools` - Tools konfigürasyonlarını kaldır

**Örnekler:**
```bash
# Tümünü üret
./core/cli/stackvo.sh generate

# Sadece projeleri üret
./core/cli/stackvo.sh generate projects

# Sadece servisleri üret
./core/cli/stackvo.sh generate services

# Tools'u kaldır
./core/cli/stackvo.sh generate --uninstall-tools
```

**Çıktı Dosyaları:**
- `generated/stackvo.yml`
- `generated/docker-compose.dynamic.yml`
- `generated/docker-compose.projects.yml`
- `core/traefik/dynamic/routes.yml`
- `core/generated/configs/*`

---

### up

Servisleri başlatır. Varsayılan olarak minimal mode (sadece core servisler).

**Syntax:**
```bash
./core/cli/stackvo.sh up [MODE_OPTIONS]
```

**Mode Options:**
- (boş) - Minimal mode: Sadece core servisler (Traefik + UI)
- `--all` - Tüm servisleri ve projeleri başlat (eski davranış)
- `--services` - Core + tüm servisleri başlat
- `--projects` - Core + tüm projeleri başlat
- `--profile <name>` - Core + belirli bir profili başlat (birden fazla kullanılabilir)

**Örnekler:**
```bash
# Minimal mode - Sadece Traefik + UI
./core/cli/stackvo.sh up

# Tüm servisleri ve projeleri başlat
./core/cli/stackvo.sh up --all

# Core + tüm servisleri başlat
./core/cli/stackvo.sh up --services

# Core + tüm projeleri başlat
./core/cli/stackvo.sh up --projects

# Core + sadece MySQL başlat
./core/cli/stackvo.sh up --profile mysql

# Core + MySQL ve Redis başlat
./core/cli/stackvo.sh up --profile mysql --profile redis

# Core + belirli bir proje başlat
./core/cli/stackvo.sh up --profile project-myproject
```

**Profile İsimlendirme:**
- Servisler için: `mysql`, `redis`, `postgres`, `mongodb`, vb.
- Projeler için: `project-{proje-adı}` (örn: `project-myproject`)

**Not:** 
- Varsayılan davranış değişti: Artık `up` komutu sadece core servisleri başlatır
- Eski davranış için `--all` parametresini kullanın
- Profile'lar Docker Compose profile özelliğini kullanır

**Eşdeğer Docker Compose:**
```bash
# Minimal mode
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  --profile core up -d

# Belirli profile ile
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  -f generated/docker-compose.projects.yml \
  --profile core --profile mysql up -d
```

---

### down

Tüm servisleri durdurur.

**Syntax:**
```bash
./core/cli/stackvo.sh down [OPTIONS]
```

**Options:**
- `-v, --volumes` - Volume'ları da sil
- `--remove-orphans` - Orphan container'ları kaldır

**Örnekler:**
```bash
# Servisleri durdur
./core/cli/stackvo.sh down

# Volume'larla birlikte durdur
./core/cli/stackvo.sh down -v

# Orphan'ları kaldır
./core/cli/stackvo.sh down --remove-orphans
```

---

### restart

Servisleri yeniden başlatır.

**Syntax:**
```bash
./core/cli/stackvo.sh restart [SERVICE...]
```

**Örnekler:**
```bash
# Tüm servisleri yeniden başlat
./core/cli/stackvo.sh restart

# Belirli servisleri yeniden başlat
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  restart mysql redis
```

---

### ps

Çalışan servisleri listeler.

**Syntax:**
```bash
./core/cli/stackvo.sh ps [OPTIONS]
```

**Options:**
- `-a, --all` - Tüm container'ları göster (durdurulmuş olanlar dahil)
- `--format` - Çıktı formatı

**Örnekler:**
```bash
# Çalışan servisleri listele
./core/cli/stackvo.sh ps

# Tüm container'ları listele
./core/cli/stackvo.sh ps -a

# Custom format
./core/cli/stackvo.sh ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

**Çıktı:**
```
NAME                      STATUS              PORTS
stackvo-traefik         Up 2 hours          0.0.0.0:80->80/tcp
stackvo-mysql           Up 2 hours          0.0.0.0:3306->3306/tcp
stackvo-redis           Up 2 hours          0.0.0.0:6379->6379/tcp
```

---

### logs

Container loglarını görüntüler.

**Syntax:**
```bash
./core/cli/stackvo.sh logs [OPTIONS] [SERVICE...]
```

**Options:**
- `-f, --follow` - Logları canlı izle
- `--tail=N` - Son N satırı göster
- `--timestamps` - Zaman damgası ekle
- `--since` - Belirli bir zamandan sonraki loglar

**Örnekler:**
```bash
# Tüm logları göster
./core/cli/stackvo.sh logs

# MySQL loglarını izle
./core/cli/stackvo.sh logs -f mysql

# Son 100 satır
./core/cli/stackvo.sh logs --tail=100 mysql

# Zaman damgalı
./core/cli/stackvo.sh logs --timestamps mysql

# Son 1 saatteki loglar
./core/cli/stackvo.sh logs --since=1h mysql
```

---

### pull

Docker image'larını çeker.

**Syntax:**
```bash
./core/cli/stackvo.sh pull [SERVICE...]
```

**Örnekler:**
```bash
# Tüm image'ları çek
./core/cli/stackvo.sh pull

# Belirli image'ları çek
docker compose -f generated/stackvo.yml \
  -f generated/docker-compose.dynamic.yml \
  pull mysql redis
```

---

### doctor

Sistem sağlık kontrolü yapar.

**Syntax:**
```bash
stackvo doctor
```

**Kontroller:**
- ✓ Docker kurulu mu?
- ✓ Docker Compose kurulu mu?
- ✓ Docker daemon çalışıyor mu?
- ✓ Gerekli portlar açık mı?
- ✓ `.env` dosyası var mı?
- ✓ SSL sertifikaları var mı?
- ✓ `generated/` dizini var mı?

**Örnek Çıktı:**
```
✓ Docker is installed (version 24.0.7)
✓ Docker Compose is installed (version 2.23.0)
✓ Docker daemon is running
✓ Port 80 is available
✓ Port 443 is available
✓ .env file exists
✓ SSL certificates found
✓ Generated directory exists

All checks passed!
```

---

### install

Stackvo CLI'yi sisteme kurar.

**Syntax:**
```bash
./core/cli/stackvo.sh install
```

**Ne yapar:**
- `/usr/local/bin/stackvo` sembolik link oluşturur
- CLI'yi sistem genelinde kullanılabilir yapar

**Gereksinimler:**
- Sudo yetkisi

---

### uninstall

Stackvo'u kaldırır.

**Syntax:**
```bash
./core/cli/stackvo.sh uninstall
```

**Ne yapar:**
1. Tüm container'ları durdurur
2. Volume'ları siler (onay ister)
3. Network'ü siler
4. CLI sembolik linkini kaldırır

**Uyarı:** Bu işlem geri alınamaz!

---

## Environment Variables

CLI davranışını kontrol eden environment variable'lar:

### STACKVO_VERBOSE

Detaylı çıktı.

```bash
STACKVO_VERBOSE=true ./core/cli/stackvo.sh generate
```

### STACKVO_DRY_RUN

Komutları çalıştırmadan göster.

```bash
STACKVO_DRY_RUN=true ./core/cli/stackvo.sh generate
```

### ENV_FILE

Farklı .env dosyası kullan.

```bash
ENV_FILE=.env.production ./core/cli/stackvo.sh generate
```

---

## Exit Codes

| Code | Açıklama |
|------|----------|
| 0 | Başarılı |
| 1 | Genel hata |
| 2 | Kullanım hatası |
| 126 | Komut çalıştırılamadı |
| 127 | Komut bulunamadı |
| 130 | Ctrl+C ile iptal edildi |

---

