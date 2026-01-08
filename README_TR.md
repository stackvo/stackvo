<div align="center">

# 🚀 StackVo

**Modern LAMP ve MEAN Stack'leri Sunan Docker Tabanlı Yerel Geliştirme Ortamı**

![Status](https://img.shields.io/badge/status-active-success.svg)
![Release](https://img.shields.io/github/v/release/stackvo/stackvo)
![GitHub Issues](https://img.shields.io/github/issues/stackvo/stackvo)
![GitHub Closed Issues](https://img.shields.io/github/issues-closed/stackvo/stackvo)
![GitHub Pull Requests](https://img.shields.io/github/issues-pr/stackvo/stackvo)
![GitHub Contributors](https://img.shields.io/github/contributors/stackvo/stackvo)
![Security](https://img.shields.io/badge/security-policy-success?logo=security&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

![Docker](https://img.shields.io/badge/Docker-Required-2496ED?logo=docker&logoColor=white)
![Bash](https://img.shields.io/badge/Bash-3.x+-4EAA25?logo=gnubash&logoColor=white)
![Traefik](https://img.shields.io/badge/Traefik-Reverse_Proxy-24A1C1?logo=traefikproxy&logoColor=white)

[🇬🇧 English](README.md) |
[🇹🇷 Türkçe](README_TR.md)

</div>

## 📖 Hakkında

**Stackvo**, modern web geliştirme projeleriniz için Docker tabanlı, tamamen özelleştirilebilir ve modüler bir geliştirme ortamı yönetim sistemidir. Pure Bash ile yazılmış generator sistemi sayesinde 40+ servisi tek komutla yönetebilirsiniz.

### ✨ Öne Çıkan Özellikler

- 🐳 **40+ Hazır Servis** - MySQL, PostgreSQL, MongoDB, Redis, RabbitMQ ve daha fazlası
- 🌐 **Multi-Language Desteği** - PHP, Node.js, Python, Go, Ruby, Rust (6 dil)
- 🔧 **3 Web Server Seçeneği** - Nginx, Apache, Caddy
- 🎯 **Pure Bash Generator** - Bash 3.x+ uyumlu, macOS ve Linux desteği
- 🔒 **Traefik Reverse Proxy** - Otomatik SSL/TLS, routing ve load balancing
- 🎨 **Modern Web UI** - Vue.js 3 + Vuetify 3 ile real-time monitoring
- 📦 **Tek Network Mimarisi** - Tüm servisler stackvo-net üzerinde
- 🚀 **Modüler Yapı** - .env ile servisleri kolayca aktif/pasif edin
- 🔄 **Dinamik Konfigürasyon** - Otomatik Docker Compose ve Traefik routing
- ⚡ **Zero-Config** - Varsayılan ayarlarla hemen çalışır

---

## 🚀 Hızlı Başlangıç

### Gereksinimler

**Sistem Gereksinimleri:**

- **Docker:** 20.10+ (macOS/Windows'ta Docker Desktop, Linux'ta Docker Engine)
- **Docker Compose:** 2.0+ (v2 plugin formatı - `docker compose` komutu, `docker-compose` değil)
- **Bash:** 3.2+ (macOS ve Linux'ta varsayılan olarak yüklü, Windows'ta WSL2 veya Git Bash kullanın)
- **RAM:** Minimum 4GB, önerilen 8GB+
- **Disk Alanı:** 10GB+ boş alan

**Desteklenen İşletim Sistemleri:**

- ✅ **macOS** 10.15+ (Catalina veya sonrası) - Intel & Apple Silicon
- ✅ **Linux** - Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch Linux
- ✅ **Windows** 10/11 ile WSL2 (WSL içinde Ubuntu 20.04+)

**Desteklenmeyen:**

- ❌ Native Windows (WSL2 olmadan)
- ❌ macOS < 10.15
- ❌ Docker Compose v1 (kullanımdan kaldırıldı)

### Kurulum

**Adım 1: Projeyi Klonlama ve Kurulum**

```bash
# Projeyi klonlayın
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# Environment dosyasını kopyalayın
cp .env.example .env
```

**Adım 2: CLI Kurulumu**

```bash
# Stackvo CLI'yi global olarak kurun
./stackvo.sh install

# Kurulumu doğrulayın
stackvo --help
```

**Adım 3: Konfigürasyon Oluşturma**

```bash
# Tüm konfigürasyonları oluşturun
stackvo generate

# Bu komut şunları oluşturur:
# - generated/stackvo.yml (Traefik + UI)
# - generated/docker-compose.dynamic.yml (Servisler)
# - generated/docker-compose.projects.yml (Projeler)
```

**Adım 4: Servisleri Başlatma**

```bash
# Core servisleri başlatın (Traefik + UI)
stackvo up

# Servislerin başlamasını bekleyin (~30 saniye)
# Durumu kontrol edin
stackvo ps
```

**Adım 5: Hosts Dosyası Ayarı**

```bash
# Stackvo UI domain'ini hosts dosyasına ekleyin
echo "127.0.0.1  stackvo.loc" | sudo tee -a /etc/hosts
```

**Adım 6: Web UI'ya Erişim**

Tarayıcınızda şu adresi açın: **https://stackvo.loc**

> **Not:** Development ortamında self-signed sertifika kullandığımız için SSL uyarısı göreceksiniz. "Gelişmiş" → "Siteye git" seçeneklerini kullanarak devam edebilirsiniz.

### İlk Projenizi Oluşturun

```bash
# Proje klasörü oluşturun
mkdir -p projects/myproject/public

# stackvo.json dosyası oluşturun
cat > projects/myproject/stackvo.json <<'EOF'
{
  "name": "myproject",
  "domain": "myproject.loc",
  "php": {
    "version": "8.2",
    "extensions": ["pdo", "pdo_mysql", "mbstring"]
  },
  "webserver": "nginx",
  "document_root": "public"
}
EOF

# Test dosyası oluşturun
echo "<?php phpinfo();" > projects/myproject/public/index.php

# Konfigürasyonu yeniden oluşturun
./stackvo.sh generate

# Servisleri yeniden başlatın
./stackvo.sh restart

# Hosts dosyasına ekleyin
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts

# Tarayıcıda açın: https://myproject.loc
```

---

## 📚 Temel Komutlar

```bash
# Kurulum ve Konfigürasyon
./stackvo.sh install               # CLI'yi sisteme kur
./stackvo.sh generate              # Tüm konfigürasyonları üret
./stackvo.sh generate projects     # Sadece projeleri üret
./stackvo.sh generate services     # Sadece servisleri üret

# Container Yönetimi
./stackvo.sh up                    # Core servisleri başlat (minimal)
./stackvo.sh up --all              # Tüm servisleri ve projeleri başlat
./stackvo.sh up --services         # Core + tüm servisleri başlat
./stackvo.sh up --projects         # Core + tüm projeleri başlat
./stackvo.sh up --profile mysql    # Core + MySQL başlat
./stackvo.sh down                  # Tüm servisleri durdur
./stackvo.sh restart               # Tüm servisleri yeniden başlat
./stackvo.sh ps                    # Çalışan servisleri listele

# Loglar ve Diğer
./stackvo.sh logs                  # Tüm logları izle
./stackvo.sh logs mysql            # Belirli servis logunu izle
./stackvo.sh pull                  # Docker image'larını çek
./stackvo.sh uninstall             # Stackvo'u kaldır
```

---

## 🛠️ Desteklenen Servisler

| Kategori                | Adet | Servisler                                      |
| ----------------------- | ---- | ---------------------------------------------- |
| **Veritabanları**       | 5    | MySQL, MariaDB, PostgreSQL, MongoDB, Cassandra |
| **Cache Sistemleri**    | 2    | Redis, Memcached                               |
| **Message Queues**      | 2    | RabbitMQ, Kafka                                |
| **Arama ve İndeksleme** | 2    | Elasticsearch, Kibana                          |
| **Monitoring**          | 1    | Grafana                                        |
| **Developer Tools**     | 2    | MailHog, Blackfire                             |

> **Toplam 14 servis** • Detaylı bilgi için: [Servisler Dokümantasyonu](docs/tr/references/services.md)

---

## 🎨 Web UI Dashboard

Stackvo, Vue.js 3 ve Vuetify 3 ile geliştirilmiş modern bir web arayüzü sunar:

- **Real-time Monitoring** - CPU, Memory, Storage, Network
- **Services Management** - Start/Stop/Restart, Port mappings, Logs
- **Projects Management** - Proje oluşturma, silme, konfigürasyon
- **Tools Access** - Adminer, PhpMyAdmin, RabbitMQ UI ve daha fazlası

**Erişim:** `https://stackvo.loc`

### 📸 Ekran Görüntüleri

<table>
  <tr>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/1-Dashboard.png?raw=true" alt="Dashboard" />
      <p align="center"><b>Dashboard</b></p>
    </td>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/2-Projects-list.png?raw=true" alt="Projeler Listesi" />
      <p align="center"><b>Projeler Listesi</b></p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/3-Projects-detail.png?raw=true" alt="Proje Detayı" />
      <p align="center"><b>Proje Detayı</b></p>
    </td>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/4-Projects-new.png?raw=true" alt="Yeni Proje" />
      <p align="center"><b>Yeni Proje</b></p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/5-Services-list.png?raw=true" alt="Servisler Listesi" />
      <p align="center"><b>Servisler Listesi</b></p>
    </td>
    <td width="50%">
      <img src="https://github.com/stackvo/stackvo/blob/main/docs/screenshots/6-Services-detail.png?raw=true" alt="Servis Detayı" />
      <p align="center"><b>Servis Detayı</b></p>
    </td>
  </tr>
</table>

---

## 📖 Dokümantasyon

Detaylı dokümantasyon için [docs](docs/tr) dizinini ziyaret edin:

- **[Başlangıç](docs/tr/started/introduction.md)** - Stackvo'a giriş ve temel kavramlar
- **[Kurulum](docs/tr/installation/index.md)** - Detaylı kurulum kılavuzu
- **[Hızlı Başlangıç](docs/tr/started/quick-start.md)** - İlk projenizi oluşturun
- **[Konfigürasyon](docs/tr/configuration/index.md)** - .env ve stackvo.json ayarları
- **[CLI Referansı](docs/tr/references/cli.md)** - Tüm CLI komutları
- **[Servisler](docs/tr/references/services.md)** - Desteklenen tüm servisler
- **[Mimari](docs/tr/concepts/architecture.md)** - Sistem mimarisi ve tasarım
- **[Sorun Giderme](docs/tr/community/troubleshooting.md)** - Sık karşılaşılan sorunlar

---

## 🤝 Katkıda Bulunma

Stackvo açık kaynaklı bir projedir ve katkılarınızı bekliyoruz!

Kod standartları, commit mesaj formatı ve changelog generation workflow dahil detaylı katkı kılavuzu için [Katkıda Bulunma Rehberi](CONTRIBUTING.md)'ni inceleyin.

### Hızlı Katkı Adımları

1. Bu repository'yi fork edin
2. Feature branch'i oluşturun (`git checkout -b feature/amazing-feature`)
3. Değişikliklerinizi commit edin (`git commit -m 'feat: add amazing feature'`)
4. Branch'inizi push edin (`git push origin feature/amazing-feature`)
5. Pull Request oluşturun

---

## 📝 Lisans

Bu proje MIT lisansı altında lisanslanmıştır. Detaylar için [LICENSE.md](LICENSE.md) dosyasına bakın.

---

## 🔗 Bağlantılar

- **Dokümantasyon:** [stackvo.github.io/stackvo](https://stackvo.github.io/stackvo/)
- **GitHub:** [github.com/stackvo/stackvo](https://github.com/stackvo/stackvo)
- **Issues:** [github.com/stackvo/stackvo/issues](https://github.com/stackvo/stackvo/issues)
- **Discussions:** [github.com/stackvo/stackvo/discussions](https://github.com/stackvo/stackvo/discussions)
- **Changelog:** [CHANGELOG.md](docs/tr/changelog.md)

---

## 💬 Destek

Sorularınız veya sorunlarınız için:

- 📖 [Dokümantasyon](docs/tr) sayfalarını inceleyin
- 🐛 [Issue](https://github.com/stackvo/stackvo/issues) açın
- 💬 [Discussions](https://github.com/stackvo/stackvo/discussions) bölümünde soru sorun
- 📧 [Destek Kılavuzu](docs/tr/community/support.md)'nu okuyun

---
