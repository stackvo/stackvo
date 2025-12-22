---
title: macOS Kurulumu
description: macOS'ta Stackvo kurulumu - Intel ve Apple Silicon (M1/M2/M3) desteği
---

# macOS Kurulumu

macOS'ta Stackvo kurulumu için Docker Desktop kullanılır. Bu kılavuz, hem Intel işlemcili hem de Apple Silicon (M1/M2/M3) Mac bilgisayarlarda Docker Desktop kurulumu, sistem ayarları ve Stackvo yapılandırmasını detaylı olarak açıklamaktadır. Homebrew ile kolay kurulum seçenekleri de mevcuttur.

!!! tip "Sistem Gereksinimlerini Kontrol Ettiniz mi?"
    Kuruluma başlamadan önce [Sistem Gereksinimleri](../started/requirements.md) sayfasını kontrol edin.

---

## Docker Desktop Kurulumu

### 1. Docker Desktop İndirme

=== "Apple Silicon (M1/M2/M3)"

    ```bash
    # Tarayıcıda açın:
    https://desktop.docker.com/mac/main/arm64/Docker.dmg

    # Veya Homebrew ile (önerilir):
    brew install --cask docker
    ```

=== "Intel"

    ```bash
    # Tarayıcıda açın:
    https://desktop.docker.com/mac/main/amd64/Docker.dmg

    # Veya Homebrew ile (önerilir):
    brew install --cask docker
    ```

### 2. Docker Desktop Kurulumu

1. DMG dosyasını açın
2. Docker.app'i Applications'a sürükleyin
3. Applications klasöründen Docker'ı başlatın
4. İlk açılışta admin şifresi isteyecek

!!! warning "İlk Başlatma"
    Docker Desktop ilk başlatmada birkaç dakika sürebilir.

### 3. Docker Desktop Ayarları

Docker Desktop açıldıktan sonra:

**Settings** (⚙️) → **Resources**

| Kaynak | Minimum | Önerilen |
|--------|---------|----------|
| **CPU** | 2 cores | 4 cores |
| **Memory** | 4 GB | 8 GB |
| **Disk** | 20 GB | 50 GB |

**Apply & Restart** butonuna tıklayın.

### 4. Docker Doğrulama

```bash
# Versiyon kontrolü
docker --version
docker compose version

# Test
docker run hello-world
```

---

## Homebrew ile Kurulum (Önerilir)

```bash
# Homebrew kurulu değilse:
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Docker Desktop + Gerekli araçlar
brew install --cask docker
brew install git jq curl

# Docker Desktop'ı başlat
open -a Docker
```

---

## Stackvo Kurulumu

Docker Desktop kurulumu tamamlandıktan sonra Stackvo'u kurmak için [Hızlı Başlangıç](../started/quick-start.md) sayfasını takip edin.

**Özet:**

```bash
# Repository klonla
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Konfigürasyon
cp .env.example .env

# Başlat
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

---

## Kurulum Doğrulama

```bash
# Container durumu
docker ps
```

Tarayıcınızda:

- **Stackvo Dashboard:** https://stackvo.loc
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Kurulum Tamamlandı!"
    [Hızlı Başlangıç](../started/quick-start.md) sayfasına geçerek ilk projenizi oluşturabilirsiniz.

---

## Yaygın Sorunlar

### Docker Desktop Başlamıyor

```bash
# Docker Desktop'ı tamamen kapat
pkill -SIGHUP -f Docker

# Yeniden başlat
open -a Docker
```

### Rosetta 2 Gerekli (Apple Silicon)

```bash
# Rosetta 2 kur
softwareupdate --install-rosetta --agree-to-license
```

### Port Çakışması

```bash
# Çakışan process'i bul ve durdur
sudo lsof -i :80
sudo kill -9 <PID>
```