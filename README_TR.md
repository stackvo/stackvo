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

- Docker 20.10+
- Docker Compose 2.0+
- Bash 3.2+
- 4GB+ RAM
- 10GB+ Disk alanı

### Kurulum

```bash
# 1. Projeyi klonlayın
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# 2. Environment dosyasını kopyalayın
cp .env.example .env

# 3. CLI'yi kurun
./cli/stackvo.sh install

# 4. Konfigürasyonu oluşturun
./cli/stackvo.sh generate

# 5. Servisleri başlatın
./cli/stackvo.sh up

# 6. Hosts dosyasını güncelleyin
echo "127.0.0.1  stackvo.loc" | sudo tee -a /etc/hosts

# 7. Web UI'ya erişin
# https://stackvo.loc
```

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
./cli/stackvo.sh generate

# Servisleri yeniden başlatın
./cli/stackvo.sh restart

# Hosts dosyasına ekleyin
echo "127.0.0.1  myproject.loc" | sudo tee -a /etc/hosts

# Tarayıcıda açın: https://myproject.loc
```

---

## 📚 Temel Komutlar

```bash
# Kurulum ve Konfigürasyon
./cli/stackvo.sh install               # CLI'yi sisteme kur
./cli/stackvo.sh generate              # Tüm konfigürasyonları üret
./cli/stackvo.sh generate projects     # Sadece projeleri üret
./cli/stackvo.sh generate services     # Sadece servisleri üret

# Container Yönetimi
./cli/stackvo.sh up                    # Tüm servisleri başlat
./cli/stackvo.sh down                  # Tüm servisleri durdur
./cli/stackvo.sh restart               # Tüm servisleri yeniden başlat
./cli/stackvo.sh ps                    # Çalışan servisleri listele

# Loglar ve Diğer
./cli/stackvo.sh logs                  # Tüm logları izle
./cli/stackvo.sh logs mysql            # Belirli servis logunu izle
./cli/stackvo.sh pull                  # Docker image'larını çek
./cli/stackvo.sh doctor                # Sistem sağlık kontrolü
./cli/stackvo.sh uninstall             # Stackvo'u kaldır
```

---

## 🛠️ Desteklenen Servisler

| Kategori                | Adet | Servisler                                                                      |
| ----------------------- | ---- | ------------------------------------------------------------------------------ |
| **Veritabanları**       | 8    | MySQL, MariaDB, PostgreSQL, MongoDB, Cassandra, Percona, CouchDB, Couchbase    |
| **Cache Sistemleri**    | 2    | Redis, Memcached                                                               |
| **Message Queues**      | 4    | RabbitMQ, Apache ActiveMQ, Kafka, NATS                                         |
| **Arama ve İndeksleme** | 4    | Elasticsearch, Kibana, Meilisearch, Solr                                       |
| **Monitoring ve QA**    | 5    | Grafana, Netdata, SonarQube, Sentry, Logstash                                  |
| **Developer Tools**     | 8    | Adminer, PhpMyAdmin, PhpPgAdmin, PhpMongo, MailHog, Ngrok, Selenium, Blackfire |
| **Application Servers** | 2    | Tomcat, Kong API Gateway                                                       |

> **Toplam 33+ servis** • Detaylı bilgi için: [Servisler Dokümantasyonu](docs/tr/references/services.md)

---

## 🎨 Web UI Dashboard

Stackvo, Vue.js 3 ve Vuetify 3 ile geliştirilmiş modern bir web arayüzü sunar:

- **Real-time Monitoring** - CPU, Memory, Storage, Network
- **Services Management** - Start/Stop/Restart, Port mappings, Logs
- **Projects Management** - Proje oluşturma, silme, konfigürasyon
- **Tools Access** - Adminer, PhpMyAdmin, RabbitMQ UI ve daha fazlası

**Erişim:** `https://stackvo.loc`

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

1. Bu repository'yi fork edin
2. Feature branch'i oluşturun (`git checkout -b feature/amazing-feature`)
3. Değişikliklerinizi commit edin (`git commit -m 'feat: add amazing feature'`)
4. Branch'inizi push edin (`git push origin feature/amazing-feature`)
5. Pull Request oluşturun

Detaylı bilgi için [Katkıda Bulunma Kılavuzu](docs/tr/community/contributing.md)'nu inceleyin.

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
