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
./stackvo.sh install

# 4. Konfigürasyonu oluşturun
./stackvo.sh generate

# 5. Servisleri başlatın
./stackvo.sh up

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

> **Not:** `./stackvo.sh install` komutunu çalıştırdıktan sonra, her yerden `stackvo` komutunu kullanabilirsiniz:
>
> ```bash
> stackvo up
> stackvo generate
> stackvo logs
> ```

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

## 🛠️ Geliştirme Scriptleri

Bu dizin, Stackvo projesinin changelog yönetimi için kullanılan scriptleri içerir.

### generate-changelog.sh

Git commit geçmişinden otomatik changelog oluşturur.

#### Kullanım

**Manuel Kullanım** (Lokal test için):

```bash
./docs/scripts/generate-changelog.sh [versiyon]
```

**Otomatik Kullanım** (GitHub Actions):

- GitHub'da yeni bir tag oluşturduğunuzda otomatik çalışır
- Workflow: `.github/workflows/changelog.yml`

#### Örnekler

```bash
# Unreleased olarak işaretle
./docs/scripts/generate-changelog.sh

# Belirli versiyon için
./docs/scripts/generate-changelog.sh 1.2.0
```

#### Çıktılar

- `docs/tr/changelog.md` - Türkçe changelog
- `docs/en/changelog.md` - İngilizce changelog

#### Conventional Commits

Script, aşağıdaki commit tiplerini tanır:

- `feat:` → Eklenenler / Added
- `fix:` → Düzeltmeler / Fixed
- `docs:` → Dokümantasyon / Documentation
- `refactor:` → Yeniden Yapılandırma / Refactored
- `perf:` → Performans / Performance
- `test:` → Testler / Tests
- `chore:` → Diğer / Chore

#### GitHub Release İş Akışı

1. **Kodunuzu geliştirin** ve commit edin (Conventional Commits formatında)

   ```bash
   git commit -m "feat: yeni özellik eklendi"
   git commit -m "fix: hata düzeltildi"
   ```

2. **GitHub'da yeni bir release oluşturun**

   - Releases → Draft a new release
   - Tag: `1.2.0` (v prefix olmadan!)
   - Title: `1.2.0`
   - Description: İsteğe bağlı
   - Publish release

3. **GitHub Actions otomatik olarak**:
   - Changelog'u günceller
   - Değişiklikleri commit eder
   - GitHub Release'e changelog ekler

#### Tag Formatı

> [!IMPORTANT]
> Tag oluştururken **"v" prefix kullanmayın**. Doğru format: `1.2.0`, `1.0.5` gibi.

**Doğru**:

- ✅ `1.0.0`
- ✅ `1.2.5`
- ✅ `2.0.0`

**Yanlış**:

- ❌ `v1.0.0`
- ❌ `v1.2.5`

### Notlar

- Bu scriptler dokümantasyon amaçlıdır
- Ana kullanım GitHub Actions üzerinden yapılır
- Manuel kullanım sadece test/geliştirme amaçlıdır
- Tüm commit'ler Conventional Commits formatında olmalıdır

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
