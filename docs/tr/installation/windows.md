---
title: Windows Kurulumu
description: Windows 10/11'de WSL2 ile Stackvo kurulumu
---

# Windows Kurulumu

Windows'ta Stackvo, **WSL2 (Windows Subsystem for Linux 2)** üzerinde çalışır. Bu kılavuz, Windows 10 ve Windows 11'de WSL2 kurulumu, Docker Desktop yapılandırması ve Stackvo'un WSL2 içinde çalıştırılması için gereken tüm adımları detaylı olarak anlatmaktadır. WSL2 sayesinde Linux benzeri bir deneyim elde edersiniz.

---

!!! tip "Sistem Gereksinimlerini Kontrol Ettiniz mi?"
    Kuruluma başlamadan önce [Sistem Gereksinimleri](../started/requirements.md) sayfasını kontrol edin.

!!! warning "WSL2 Gerekli"
    Windows 10 Pro/Enterprise (Build 19041+) veya Windows 11 gereklidir.

---

## WSL2 Kurulumu

### Otomatik Kurulum (Önerilir)

**PowerShell'i Yönetici Olarak Açın:**

```powershell
# WSL2'yi kur (tek komut)
wsl --install

# Bilgisayarı yeniden başlatın
Restart-Computer
```

!!! success "Tek Komut!"
    Bu komut WSL2, Ubuntu ve tüm gereksinimleri otomatik kurar.

### İlk Başlatma

1. Başlat menüsünden "Ubuntu" açın
2. Kullanıcı adı girin (küçük harf, boşluksuz)
3. Şifre belirleyin (2 kez)

---

## Docker Desktop Kurulumu

### 1. Docker Desktop İndirme

[Docker Desktop for Windows](https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe) indirin.

### 2. Kurulum

1. İndirilen `.exe` dosyasını çalıştırın
2. **"Use WSL 2 instead of Hyper-V"** seçeneğini işaretleyin
3. "Install" tıklayın
4. Kurulum tamamlandığında bilgisayarı yeniden başlatın

### 3. Docker Desktop Ayarları

**Settings** → **General:**
- ✅ Use the WSL 2 based engine

**Settings** → **Resources** → **WSL Integration:**
- ✅ Enable integration with my default WSL distro
- ✅ Ubuntu-22.04

**Settings** → **Resources:**

| Kaynak | Minimum | Önerilen |
|--------|---------|----------|
| **CPU** | 2 cores | 4 cores |
| **Memory** | 4 GB | 8 GB |
| **Disk** | 30 GB | 50 GB |

**Apply & Restart**

---

## Stackvo Kurulumu (WSL2 İçinde)

### WSL2'ye Giriş

```powershell
# PowerShell'den WSL2'ye geç
wsl
```

### Sistem Güncellemesi

```bash
# Ubuntu içinde
sudo apt update && sudo apt upgrade -y
sudo apt install -y git curl jq
```

### Stackvo Kurulumu

Docker Desktop kurulumu tamamlandıktan sonra Stackvo'u kurmak için [Hızlı Başlangıç](../started/quick-start.md) sayfasını takip edin.

**Özet:**

```bash
# WSL2 içinde
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Konfigürasyon
cp .env.example .env

# Başlat
./stackvo.sh generate
./stackvo.sh up
```

### Hosts Dosyası (Windows)

**Windows'ta** Notepad'i yönetici olarak açın:

```
C:\Windows\System32\drivers\etc\hosts
```

Ekleyin:

```
127.0.0.1  stackvo.loc
127.0.0.1  traefik.stackvo.loc
```

---

## Kurulum Doğrulama

### WSL2 Kontrolü

```powershell
# PowerShell'de
wsl --list --verbose

# Çıktı:
#   NAME            STATE           VERSION
# * Ubuntu-22.04    Running         2
```

### Docker Kontrolü

```bash
# WSL2 içinde
docker --version
docker compose version
docker ps
```

### Web UI Kontrolü

**Windows tarayıcısında** açın:

- **Stackvo Dashboard:** https://stackvo.loc
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Kurulum Tamamlandı!"
    [Hızlı Başlangıç](../started/quick-start.md) sayfasına geçerek ilk projenizi oluşturabilirsiniz.

---

## Yaygın Sorunlar

### WSL2 Başlamıyor

**Hata:** `WslRegisterDistribution failed with error: 0x80370102`

**Çözüm:** BIOS'ta Virtualization aktif olmalı

```powershell
# Hyper-V'yi aktif et
Enable-WindowsOptionalFeature -Online -FeatureName Microsoft-Hyper-V -All
Restart-Computer
```

### Docker Daemon Bağlanamıyor

**Çözüm:**
1. Docker Desktop çalışıyor mu kontrol edin
2. Settings → Resources → WSL Integration aktif mi?
3. WSL2'yi yeniden başlatın: `wsl --shutdown` → `wsl`

### Port Çakışması

```powershell
# Windows'ta çakışan servisi bul
netstat -ano | findstr :80

# Process'i durdur
taskkill /PID <PID> /F
```