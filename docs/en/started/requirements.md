---
title: Sistem Gereksinimleri
description: Comprehensive guide to system requirements for running Stackvo smoothly. Includes minimum, recommended, and professional hardware specifications, supported operating systems (Linux, macOS, Windows), Docker and Docker Compose versions, network ports, and system verification details.
---

# Sistem Gereksinimleri

Stackvo'u sorunsuz ve verimli bir şekilde çalıştırabilmek için sisteminizin belirli donanım ve yazılım gereksinimlerini karşılaması gerekmektedir. Bu sayfa, minimum, önerilen ve profesyonel kullanım için gereken tüm sistem gereksinimlerini, desteklenen işletim sistemlerini, Docker versiyonlarını ve network ayarlarını detaylı olarak açıklamaktadır.

---

## Donanım Gereksinimleri

### Minimum Gereksinimler

!!! warning "Minimum Konfigürasyon"
Bu konfigürasyon ile sadece temel servisleri çalıştırabilirsiniz.

| Bileşen      | Minimum | Açıklama                 |
| ------------ | ------- | ------------------------ |
| **CPU**      | 2 Core  | Dual-core işlemci        |
| **RAM**      | 4 GB    | Sistem + Docker için     |
| **Disk**     | 20 GB   | Boş disk alanı           |
| **İnternet** | Var     | İlk kurulum için gerekli |

**Çalıştırılabilir Servisler:**

- MySQL veya PostgreSQL (1 adet)
- Redis
- Nginx
- 1-2 küçük proje

### Önerilen Gereksinimler

!!! success "Önerilen Konfigürasyon"
Rahat geliştirme için önerilen konfigürasyon.

| Bileşen      | Önerilen | Açıklama              |
| ------------ | -------- | --------------------- |
| **CPU**      | 4 Core   | Quad-core işlemci     |
| **RAM**      | 8 GB     | Çoklu servis için     |
| **Disk**     | 50 GB    | SSD önerilir          |
| **İnternet** | Hızlı    | İmaj indirmeleri için |

**Çalıştırılabilir Servisler:**

- 10-15 servis eşzamanlı
- 3-5 orta ölçekli proje
- Monitoring araçları

### Profesyonel Gereksinimler

!!! tip "Profesyonel Konfigürasyon"
Tüm servisleri ve çoklu projeleri rahatça çalıştırın.

| Bileşen      | Profesyonel | Açıklama           |
| ------------ | ----------- | ------------------ |
| **CPU**      | 8+ Core     | Multi-core işlemci |
| **RAM**      | 16+ GB      | Tüm servisler için |
| **Disk**     | 100+ GB     | NVMe SSD önerilir  |
| **İnternet** | Çok Hızlı   | Fiber bağlantı     |

**Çalıştırılabilir Servisler:**

- 40+ servis eşzamanlı
- 10+ proje
- Tüm monitoring ve logging araçları

---

## İşletim Sistemi Gereksinimleri

### Linux

!!! success "En İyi Performans"
    Linux, Docker için en iyi performansı sunar.

| Dağıtım | Minimum Versiyon | Kernel |
|---------|------------------|--------|
| **Ubuntu** | 20.04 LTS+ | 4.4+ |
| **Debian** | 10+ | 4.4+ |
| **CentOS/RHEL** | 7+ | 3.10+ |
| **Rocky/Alma** | 8+ | 3.10+ |
| **Arch/Manjaro** | Rolling | 5.0+ |
| **Fedora** | 35+ | 5.0+ |

### macOS

!!! info "Docker Desktop Gerekli"
    macOS'ta Docker Desktop kullanılması gerekmektedir.

| Versiyon | Chip Desteği |
|----------|--------------|
| **macOS 12+** (Monterey, Ventura, Sonoma) | Intel x86_64, Apple Silicon (M1/M2/M3) |

**Not:** Apple Silicon için Rosetta 2 gerekebilir.

### Windows

!!! warning "WSL2 Zorunlu"
    Windows'ta WSL2 (Windows Subsystem for Linux 2) kullanılması zorunludur.

| Versiyon | Gereksinim |
|----------|------------|
| **Windows 10 Pro/Enterprise** | Build 19041+ |
| **Windows 11 Pro/Enterprise** | Tüm versiyonlar |

**Gereksinimler:** WSL2 aktif + Ubuntu 20.04+ WSL dağıtımı + Docker Desktop 4.0+

---

## Docker Gereksinimleri

### Docker Engine

!!! danger "Kritik Gereksinim"
Docker Engine kurulu olmalıdır!

**Minimum Versiyon:**

```bash
Docker Engine: 20.10.0+
```

**Önerilen Versiyon:**

```bash
Docker Engine: 24.0.0+
```

**Kontrol:**

```bash
docker --version
# Çıktı: Docker version 24.0.7, build afdd53b
```

### Docker Compose

!!! danger "Kritik Gereksinim"
Docker Compose kurulu olmalıdır!

**Minimum Versiyon:**

```bash
Docker Compose: 2.0.0+
```

**Önerilen Versiyon:**

```bash
Docker Compose: 2.20.0+
```

**Kontrol:**

```bash
docker compose version
# Çıktı: Docker Compose version v2.23.0
```

!!! warning "Eski Versiyon Uyarısı"
`docker-compose` (v1.x) yerine `docker compose` (v2.x) kullanılmalıdır!

---

## Network Gereksinimleri

### Kritik Portlar

Stackvo'un çalışması için gerekli portlar:

| Port | Servis | Açıklama |
|------|--------|----------|
| **80** | Traefik | HTTP |
| **443** | Traefik | HTTPS |
| **8080** | Traefik Dashboard | Yönetim paneli |

!!! warning "Port Çakışması"
    Bu portlar başka bir uygulama tarafından kullanılmamalıdır!

**Port Kontrolü:**
```bash
# Linux/macOS
sudo lsof -i :80
sudo lsof -i :443

# Windows (PowerShell)
netstat -ano | findstr :80
```

### İnternet Bağlantısı

- **İlk Kurulum:** Docker imajları için ~5-10 GB indirme
- **Normal Kullanım:** Opsiyonel (sadece güncellemeler için)

---

## Yazılım Gereksinimleri

### Zorunlu Yazılımlar

```bash
# Bash 4.0+
bash --version

# Git 2.0+
git --version

# Curl 7.0+
curl --version

# jq 1.5+ (JSON parser)
jq --version
```

### Opsiyonel Araçlar

- **IDE:** VS Code, PhpStorm, WebStorm
- **Terminal:** htop, ncdu, lazydocker

---

## Sistem Kontrolü

Stackvo, sistem gereksinimlerini otomatik kontrol eden bir script sağlar:

```bash
cd stackvo
./cli/check-requirements.sh
```

**Örnek Çıktı:**

```
✅ İşletim Sistemi: Ubuntu 22.04 LTS
✅ Docker Engine: 24.0.7
✅ Docker Compose: 2.23.0
✅ Bash: 5.1.16
✅ Git: 2.34.1
✅ RAM: 16 GB (Yeterli)
✅ Disk: 120 GB boş (Yeterli)
⚠️  Port 80: Kullanımda (Apache çalışıyor)

Toplam: 8/9 kontrol başarılı
```

!!! tip "Kuruluma Hazır mısınız?"
    Tüm kontroller başarılıysa [Kurulum](../installation/index.md) sayfasına geçebilirsiniz.
