---
title: Linux Kurulumu
description: Ubuntu, Debian, CentOS, Arch ve diğer Linux dağıtımlarında Stackvo kurulumu
---

# Linux Kurulumu

Linux, Stackvo için en iyi performansı ve en sorunsuz deneyimi sunar. Bu kılavuz, Ubuntu, Debian, CentOS, Rocky Linux, Arch ve diğer popüler Linux dağıtımlarında Docker ve Stackvo kurulumunu adım adım anlatmaktadır. Native Docker desteği sayesinde Windows ve macOS'a göre daha hızlı ve verimli çalışır.

!!! tip "Sistem Gereksinimlerini Kontrol Ettiniz mi?"
    Kuruluma başlamadan önce [Sistem Gereksinimleri](../started/requirements.md) sayfasını kontrol edin.

---

## Docker Kurulumu

### 1. Sistem Güncellemesi

```bash
# Ubuntu/Debian
sudo apt update && sudo apt upgrade -y

# CentOS/RHEL
sudo yum update -y

# Rocky/Alma
sudo dnf update -y

# Arch/Manjaro
sudo pacman -Syu
```

### 2. Docker Kurulumu

=== "Ubuntu/Debian"

    ```bash
    # Gerekli paketler
    sudo apt install -y apt-transport-https ca-certificates curl gnupg git jq
    
    # Docker GPG key
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | \
      sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
    
    # Docker repository
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] \
      https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | \
      sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    
    # Docker kur
    sudo apt update
    sudo apt install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    ```

=== "CentOS/RHEL"

    ```bash
    # Gerekli paketler
    sudo yum install -y yum-utils git jq
    
    # Docker repository
    sudo yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
    
    # Docker kur
    sudo yum install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    ```

=== "Rocky/Alma"

    ```bash
    # Gerekli paketler
    sudo dnf install -y dnf-plugins-core git jq
    
    # Docker repository
    sudo dnf config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
    
    # Docker kur
    sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
    ```

=== "Arch/Manjaro"

    ```bash
    # Docker kur (repository'de mevcut)
    sudo pacman -S docker docker-compose git jq
    ```

### 3. Docker Servisini Başlat

```bash
sudo systemctl start docker
sudo systemctl enable docker
```

### 4. Kullanıcı İzinleri

```bash
# Docker grubuna ekle
sudo usermod -aG docker $USER

# Oturumu yenile
newgrp docker
```

### 5. Docker Doğrulama

```bash
# Versiyon kontrolü
docker --version
docker compose version

# Test
docker run hello-world
```

---

## Stackvo Kurulumu

Docker kurulumu tamamlandıktan sonra Stackvo'u kurmak için [Hızlı Başlangıç](../started/quick-start.md) sayfasını takip edin.

**Özet:**

```bash
# Repository klonla
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Konfigürasyon
cp .env.example .env

# Başlat
./core/cli/stackvo.sh generate
./core/cli/stackvo.sh up
```

---

## Kurulum Doğrulama

```bash
# Container durumu
docker ps

# Web UI'yı aç
```

Tarayıcınızda:

- **Stackvo Dashboard:** https://stackvo.loc
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Kurulum Tamamlandı!"
    [Hızlı Başlangıç](../started/quick-start.md) sayfasına geçerek ilk projenizi oluşturabilirsiniz.

---

## Yaygın Sorunlar

### Permission Denied

```bash
# Docker grubuna ekle
sudo usermod -aG docker $USER
newgrp docker
```

### Port Çakışması

```bash
# Çakışan servisi durdur
sudo systemctl stop apache2
sudo systemctl stop nginx
```