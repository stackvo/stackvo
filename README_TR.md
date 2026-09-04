<div align="center">

<!-- LOGO — docs/images/logo.png (512×512) buraya gelecek
     docs/images/logo.png (512×512) buraya gelecek
     <img src="docs/images/logo.png" alt="StackVo" width="120"> -->

# StackVo

**Docker tabanlı yerel geliştirme ortamlarını yöneten masaüstü uygulaması.**

Her projeye kendi PHP sürümü, kendi veritabanı, kendi alan adı ve kendi HTTPS sertifikası.
Terminalde `docker compose` yazmadan, makinenize tek bir PHP kurmadan.

[![CI](https://img.shields.io/github/actions/workflow/status/stackvo/stackvo/ci.yml?branch=main&style=flat-square&logo=github&label=CI)](https://github.com/stackvo/stackvo/actions/workflows/ci.yml)
[![Nightly](https://img.shields.io/github/actions/workflow/status/stackvo/stackvo/nightly.yml?branch=main&style=flat-square&logo=github&label=nightly)](https://github.com/stackvo/stackvo/actions/workflows/nightly.yml)
[![Release](https://img.shields.io/github/v/release/stackvo/stackvo?style=flat-square&sort=semver&display_name=tag&label=release)](https://github.com/stackvo/stackvo/releases)
[![Downloads](https://img.shields.io/github/downloads/stackvo/stackvo/total?style=flat-square&label=downloads)](https://github.com/stackvo/stackvo/releases)
[![License](https://img.shields.io/github/license/stackvo/stackvo?style=flat-square&label=license)](LICENSE)

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey?style=flat-square)](#2-kurulum)
[![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-stable-CE422B?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-42B883?style=flat-square&logo=vuedotjs&logoColor=white)](https://vuejs.org)
[![Node.js](https://img.shields.io/badge/Node.js-%E2%89%A522-5FA04E?style=flat-square&logo=nodedotjs&logoColor=white)](.nvmrc)
[![Docker](https://img.shields.io/badge/Docker-gerekli-2496ED?style=flat-square&logo=docker&logoColor=white)](#1-gereksinimler)

[![Issues](https://img.shields.io/github/issues/stackvo/stackvo?style=flat-square&label=issues)](https://github.com/stackvo/stackvo/issues)
[![Pull requests](https://img.shields.io/github/issues-pr/stackvo/stackvo?style=flat-square&label=PRs)](https://github.com/stackvo/stackvo/pulls)
[![Contributors](https://img.shields.io/github/contributors/stackvo/stackvo?style=flat-square)](https://github.com/stackvo/stackvo/graphs/contributors)
[![Last commit](https://img.shields.io/github/last-commit/stackvo/stackvo?style=flat-square)](https://github.com/stackvo/stackvo/commits/main)
[![Davranış kuralları](https://img.shields.io/badge/Contributor%20Covenant-uyarlandi-4baaaa?style=flat-square)](CODE_OF_CONDUCT.md)
[![PR'lar hos geldiniz](https://img.shields.io/badge/PR-hos%20geldiniz-brightgreen?style=flat-square)](CONTRIBUTING.md)
[![Stars](https://img.shields.io/github/stars/stackvo/stackvo?style=flat-square)](https://github.com/stackvo/stackvo/stargazers)

**Türkçe** &nbsp;·&nbsp; [English](README.md)

[Hızlı başlangıç](#hızlı-başlangıç-5-dakika) ·
[Özellikler](#neden-stackvo) ·
[Örnekler](#örneklerle-kullanım) ·
[CLI](#terminalden-kullanım-stackvo) ·
[Mimari](#mimari) ·
[Karşılaştırma](#benzer-araçlarla-karşılaştırma)

</div>

<img src="docs/screenshots/dashboard.png" alt="StackVo paneli: sağlık, imajlar, CPU, bellek ve disk" width="100%">

---

## İçindekiler

- [StackVo nedir?](#stackvo-nedir)
- [Neden StackVo?](#neden-stackvo)
- [Hızlı başlangıç (5 dakika)](#hızlı-başlangıç-5-dakika)
- [Ekran görüntüleri](#ekran-görüntüleri)
- [Temel kavramlar](#temel-kavramlar)
- [Örneklerle kullanım](#örneklerle-kullanım)
- [Terminalden kullanım (`stackvo`)](#terminalden-kullanım-stackvo)
- [Yapay zekâ asistanlarıyla kullanım (MCP)](#yapay-zekâ-asistanlarıyla-kullanım-mcp)
- [Başka bir araçtan geçiş](#başka-bir-araçtan-geçiş)
- [Desteklenen yığın](#desteklenen-yığın)
- [Mimari](#mimari)
- [Yapılandırma](#yapılandırma)
- [Güvenlik ve gizlilik](#güvenlik-ve-gizlilik)
- [Benzer araçlarla karşılaştırma](#benzer-araçlarla-karşılaştırma)
- [Kaynaktan derleme ve geliştirme](#kaynaktan-derleme-ve-geliştirme)
- [Durum ve yol haritası](#durum-ve-yol-haritası)
- [Sık sorulan sorular](#sık-sorulan-sorular)
- [Katkı, destek ve lisans](#katkı-destek-ve-lisans)

---

## StackVo nedir?

StackVo, bilgisayarınızdaki **yerel geliştirme ortamını** yöneten bir masaüstü
uygulamasıdır. Her proje için gereken web sunucusunu, PHP (veya Node, Python,
Go, Ruby, Rust) sürümünü, veritabanını ve yardımcı servisleri **Docker
konteynerleri** olarak ayağa kaldırır; siz sadece pencereden bir düğmeye
basarsınız.

Basit bir örnekle:

```text
Eski yöntem                                  StackVo ile
─────────────────────────────────────────    ────────────────────────────────
brew install php@8.1                          Yeni proje → PHP 8.4 seç
sonra diğer proje PHP 8.4 istiyor…            Diğer proje → PHP 8.1 seç
MySQL 8.0 kurulu, proje 5.7 istiyor…          Her projeye kendi MySQL sürümü
/etc/hosts dosyasını elle düzenle             Alan adı otomatik: shop.loc
Sertifika uyarısına "devam et" de             HTTPS otomatik ve güvenilir
```

Üç cümlede özet:

1. **Her proje bir konteynerdir.** İki proje aynı makinede iki farklı PHP
   sürümü, iki farklı veritabanı ve iki farklı ortam değişkeni setiyle
   çakışmadan çalışır.
2. **Uygulama makinenizde çalışır, konteynerin içinde değil.** Docker kapalıysa
   size söyler ve başlatmayı teklif eder; tüm yığını durdurabilir; gerçek CPU
   ve bellek değerlerini okur.
3. **Aynı çekirdek üç yüzeyden sürülür:** pencere, `stackvo` komut satırı ve
   yapay zekâ asistanları için MCP sunucusu. Üçü de aynı sözleşmeyi kullanır.

> **Kimin için?** Aynı anda birden fazla PHP/Laravel/WordPress/Node projesi
> taşıyan; "bende çalışıyordu" sorununu yaşamak istemeyen; ekipçe aynı ortamı
> paylaşmak isteyen geliştiriciler için.

---

## Neden StackVo?

| Özellik | Ne işe yarar? |
|---------|----------------|
| **Proje başına izole ortam** | PHP 5.6–8.5, Node, Python, Go, Ruby, Rust — projeler birbirinin sürümünü bozmaz. |
| **Otomatik HTTPS ve alan adı** | `shop.loc` gibi bir adres, mkcert ile güvenilir sertifika, Traefik ile yönlendirme. Tarayıcı uyarısı yok. |
| **Gerçek masaüstü uygulaması** | Tauri 2 + Rust. Kurulum ~27 MB, CLI dahil. Web arayüzü değil; Docker kapalıyken de açılır. |
| **30+ hazır servis** | MySQL, MariaDB, PostgreSQL, MongoDB, Redis, Valkey, RabbitMQ, Kafka, Elasticsearch, MinIO, Grafana, phpMyAdmin… |
| **Git dalı başına tam ortam** | Her worktree kendi alan adını **ve kendi veritabanını** alır. Bulut "preview environment" fikri, yerelde ve ücretsiz. |
| **"Bu istek neden yavaştı?"** | Profiler + sorgu günlüğü + `dump()` çıktıları tek bir zaman ekseninde; kaydedilen isteği tek tıkla tekrar gönderme. |
| **Uygulama içi mail kutusu** | Gönderilen e-postalar yakalanır, uygulamada okunur; tarayıcı sekmesi gerekmez. |
| **İsimli veritabanı anlık görüntüleri** | Migration öncesi `snapshot`, sonra tek tıkla geri dön. Zamanlanmış da alınabilir. |
| **Monorepo tek proje olarak** | `api/` Go, `web/` Next.js, `worker/` Python — tek giriş, tek başlatma, tek sertifika. |
| **9 sağlayıcılı genel tünel** | Cloudflare, ngrok, Tailscale, zrok, Pinggy, localtunnel, localhost.run, LocalXpose. |
| **Yapay zekâ asistanları için MCP** | 38 araç; yazma yetkisi açık bir bayrağın ve süre sınırının arkasında. |
| **7 araçtan içe aktarma** | XAMPP, Laragon, MAMP, Valet, Sail, Herd, DDEV — projelerinizi olduğu gibi getirir. |
| **Maliyeti ölçer** | "`shop` bugün 4,2 GB·saat tuttu ve 38 dakika CPU kullandı." Docker'ın bedelini söyleyen tek araç. |
| **Türkçe arayüz** | Uygulama ve yardım metinleri Türkçe/İngilizce; erişilebilirlik beyanı EN 301 549 biçiminde. |

---

## Hızlı başlangıç (5 dakika)

### 1) Gereksinimler

| Gereksinim | Ayrıntı |
|------------|---------|
| **Docker** | macOS/Windows'ta Docker Desktop, Linux'ta Docker Engine. **Podman, Colima ve OrbStack** da tanınır (Podman'ın rootless soketi öncelikli aranır). |
| **İşletim sistemi** | macOS 10.15+, Windows 10+, Linux (x86_64 / aarch64) |
| **Disk** | Uygulama ~27 MB + Docker imajları (birkaç GB) |

> Docker çalışmıyorsa uygulama yine de açılır, durumu raporlar ve başlatmayı
> teklif eder — konteyner içinde çalışan bir panelin asla yapamayacağı şey.

### 2) Kurulum

Her sürüm için üç platformda altı yükleyici üretilir:

| Platform | Biçim | Not |
|----------|-------|-----|
| macOS | `.dmg` | Apple Silicon ve Intel |
| Windows | `.msi`, `.exe` (NSIS) | x64 ve ARM64 |
| Linux | `.deb`, `.rpm`, `.AppImage` | x86_64 ve aarch64 |

> **Şu an yayımlanmış bir sürüm yok.** Bugün tek yol
> [kaynaktan derlemek](#kaynaktan-derleme-ve-geliştirme). İlk etiket
> yayımlandığında bu bölüm indirme bağlantılarıyla güncellenecek.

<details>
<summary><b>İmzasız bir derlemeyi açmak</b> (macOS "hasarlı" diyorsa buraya bakın)</summary>

Sürümler kod imzalı değil; bu bir eksiklik değil bilinçli bir karar (bkz.
[SSS](#sık-sorulan-sorular)). İlk açılışta işletim sistemi uyarır:

- **macOS** — "StackVo is damaged and can't be opened" mesajı aslında karantina
  özniteliğiyle ilgilidir. Çözüm: uygulamaya sağ tık → **Aç** → yine **Aç**.
  Ya da terminalden:

  ```sh
  xattr -dr com.apple.quarantine /Applications/StackVo.app
  ```

- **Windows** — SmartScreen "Windows protected your PC" der. **More info**
  **Run anyway**.
- **Linux** — Ek bir adım yok. AppImage için `chmod +x` yeterlidir.

Her sürümün yanında `SHA256SUMS-<hedef>.txt` yayımlanır; uygulamanın kendi
güncelleyicisi ise güncelleme bildirimini **minisign** imzasıyla doğrular.

</details>

### 3) İlk çalışma sihirbazı

Uygulamayı ilk açtığınızda tek soru sorulur: **çalışma alanı (workspace)
nerede olsun?**

```text
~/.stackvo/                 ← varsayılan; STACKVO_ROOT ile değiştirilebilir
├── .env                    yığının ayarları (servisler, portlar, TLD)
├── generated/              üretilen compose/Dockerfile dosyaları — silinebilir
├── certs/                  mkcert sertifikaları
└── projects/
    └── shop/
        ├── stackvo.json    projenin manifest'i — elle düzenlenen tek dosya
        └── Dockerfile      üretilir
```

Boş bir klasör gösterin; uygulama gerekli her şeyi kendisi yazar. Elinizde
mevcut bir StackVo çalışma alanı varsa olduğu gibi kullanılır, hiçbir dosyası
değiştirilmez.

### 4) İlk projeniz

Her şey pencereden yürür. **Projeler → Yeni proje** bir sihirbaz açar:

| Adım | Ne sorulur |
|------|------------|
| 1 | Proje adı ve klasörü — alan adı addan türetilir (`shop` → `shop.loc`) |
| 2 | Çerçeve: Laravel, WordPress, Symfony, düz PHP, Node… |
| 3 | Çalışma zamanı ve sürümü (PHP 8.4, Node 22…), web sunucusu (nginx, caddy…) |
| 4 | İstediğiniz servisler: MySQL, Redis, Mailpit… |

**Oluştur**'a bastığınızda arka planda olan biten şudur:

```text
stackvo.json  ──►  üretici (Rust)  ──►  Dockerfile + docker-compose parçası + Traefik yönlendirmesi
                                          │
                                          ├─ konteyner ayağa kalkar
                                          ├─ /etc/hosts'a bir satır (önce fark gösterilir, tek yetkili yazma)
                                          └─ mkcert sertifikası üretilir  →  https://shop.loc
```

İlerleme, tahmin çubuğu olarak değil, **gerçek çıktı akışı** olarak gösterilir:
imaj kurulurken Docker'ın yazdığı satırları olduğu gibi görürsünüz. Bittiğinde
proje listesindeki satırın alan adına tıklayın — tarayıcı açılır, sertifika
uyarısı çıkmaz.

<img src="docs/screenshots/project-new.png" alt="Yeni proje çekmecesi: ad, alan adı, runtime ve PHP yapılandırması" width="100%">

### 5) Günlük kullanım — nerede ne var

Sol taraftaki yedi sayfa uygulamanın tamamıdır:

| Sayfa | Ne için |
|-------|---------|
| **Panel** | Makinenin durumu: CPU, bellek, disk, ağ, çalışan projeler, Docker'ın sağlığı |
| **Projeler** | Projelerin listesi; satır üzerinden başlat / durdur / yeniden kur, alan adını aç |
| **Katalog** | Servis kurma ve sürüm seçme; aynı servisin iki örneğini yan yana çalıştırma |
| **Loglar** | Bütün projelerin günlükleri tek yerde, canlı |
| **Dump'lar** | Uygulamanızın `dump()` / `dd()` çıktıları — tarayıcıya basılmadan |
| **Mail** | Uygulamanın gönderdiği e-postalar; HTML önizleme, arama, bağlantı denetimi |
| **Ayarlar** | Alan adı, sertifika, PHP, tanılama, yedekler, yapay zekâ asistanları |

Bir projenin adına tıkladığınızda **proje sayfası** açılır: 45 panel, her biri tek
bir konuya ait — Genel bakış, Servisler, Günlükler, Terminal, Xdebug, Profilleyici,
Neden yavaş, Snapshot'lar, Worktree'ler, Paylaş, Üretim imajı, Manifest… Her
panelin sağ üstünde bir **?** düğmesi vardır ve o panelin ne yaptığını kendi
diliyle anlatır.

**Bir şey ters giderse:** **Ayarlar → Tanılama** içindeki *Doctor*, neyin bozuk
olduğunu ve nasıl düzeleceğini satır satır söyler; çoğu bulgunun yanında düzelten
bir düğme vardır.

> Terminali sevenler için: aşağıdaki işlerin tamamının bir
> [`stackvo` komutu karşılığı](#terminalden-kullanım-stackvo) da var. Ama
> uygulamayı kullanmak için gerekli değildir — CLI, betikler ve CI için oradadır.

---

## Ekran görüntüleri

<table>
  <tr><td width="25%" valign="top"><a href="docs/screenshots/dashboard.png"><img src="docs/screenshots/dashboard.png" alt="Panel"></a><br><sub><b>Panel</b><br>Sağlık, maliyet, makine</sub></td><td width="25%" valign="top"><a href="docs/screenshots/projects.png"><img src="docs/screenshots/projects.png" alt="Projeler"></a><br><sub><b>Projeler</b><br>Her proje ve durumu</sub></td><td width="25%" valign="top"><a href="docs/screenshots/project-detail.png"><img src="docs/screenshots/project-detail.png" alt="Proje detayı"></a><br><sub><b>Proje detayı</b><br>Bir proje ne yapıyor</sub></td><td width="25%" valign="top"><a href="docs/screenshots/project-new.png"><img src="docs/screenshots/project-new.png" alt="Yeni proje"></a><br><sub><b>Yeni proje</b><br>Ad, runtime, PHP</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="docs/screenshots/market.png"><img src="docs/screenshots/market.png" alt="Katalog"></a><br><sub><b>Katalog</b><br>Paketler ve sürümler</sub></td><td width="25%" valign="top"><a href="docs/screenshots/market-service-detail.png"><img src="docs/screenshots/market-service-detail.png" alt="Servis detayı"></a><br><sub><b>Servis detayı</b><br>Servise nasıl erişilir</sub></td><td width="25%" valign="top"><a href="docs/screenshots/project-detail-debugging.png"><img src="docs/screenshots/project-detail-debugging.png" alt="Hata ayıklama"></a><br><sub><b>Hata ayıklama</b><br>Xdebug, profil, dump</sub></td><td width="25%" valign="top"><a href="docs/screenshots/project-detail-terminal.png"><img src="docs/screenshots/project-detail-terminal.png" alt="Terminal"></a><br><sub><b>Terminal</b><br>Konteynerde bir kabuk</sub></td></tr>
  <tr><td width="25%" valign="top"><a href="docs/screenshots/mail.png"><img src="docs/screenshots/mail.png" alt="Mail"></a><br><sub><b>Mail</b><br>Projelerin gönderdikleri</sub></td><td width="25%" valign="top"><a href="docs/screenshots/logs.png"><img src="docs/screenshots/logs.png" alt="Günlükler"></a><br><sub><b>Günlükler</b><br>Uygulama ve sunucu</sub></td><td width="25%" valign="top"><a href="docs/screenshots/settings.png"><img src="docs/screenshots/settings.png" alt="Görünüm"></a><br><sub><b>Görünüm</b><br>Tema, yarıçap, yoğunluk</sub></td><td width="25%" valign="top"><a href="docs/screenshots/settings-doctor.png"><img src="docs/screenshots/settings-doctor.png" alt="Doctor"></a><br><sub><b>Doctor</b><br>Ne bozuk, adıyla</sub></td></tr>
</table>

**[Bütün ekranlar, tek sayfada →](docs/screenshots/README.md)** — otuz dokuz
görüntü: her sayfa, proje detayının on bölümü, ayarların on yedi paneli, kendi
adresi olmayan dört ekran, ve tarayıcının çekemediği iki ekran.

Hepsi elle değil, `npm run screenshots` ile çekiliyor: 1600x1000@2x, ve
Playwright takımının sahnelediği sınırın karşısında — yani arayüz değişince
hepsi yeniden çekiliyor ve hiçbiri "o an pencere ne kadarsa" değil. Eksik olan
iki ekran da aynı yoldan geliyor: dal başına worktree ortamı diğer her şey gibi
sınırda sahneleniyor; bir terminal programı olan `stackvo tui` ise onu çizen
kodun bastığı tek bir kare olarak, hücre hücre çiziliyor.

---

## Temel kavramlar

Beş kavramı anlarsanız uygulamanın tamamını anlarsınız.

### 1. Çalışma alanı (workspace)

Uygulamanın yönettiği tek klasör. İçinde ayarlar (`.env`), üretilen dosyalar
(`generated/`) ve projeler bulunur. Veritabanı yoktur — **durum diskteki
klasörün kendisidir**. `generated/` klasörünü istediğiniz an silebilirsiniz;
gerektiğinde yeniden üretilir.

### 2. Manifest — `stackvo.json`

Bir projenin ne olduğunu anlatan, **elle düzenlemesi beklenen tek dosya**.
Depoya commit'lenir, böylece takım arkadaşınız aynı ortamı alır.

```json
{
  "name": "shop",
  "framework": "laravel",
  "php": { "version": "8.4", "extensions": ["redis", "intl", "gd"] },
  "server": "nginx",
  "domain": "shop.loc",
  "services": ["mysql", "redis", "mailpit"],
  "commands": {
    "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Arama dizinini yeniden kur" }
  }
}
```

### 3. Üretim (generation)

Compose dosyaları ve Dockerfile **her seferinde manifest'ten yeniden üretilir**,
asla yerinde düzenlenmez. Bu yüzden üretilen dosyaya elle dokunmak yerine
manifest'i değiştirirsiniz. **Ayarlar → Çalışma alanı → Üretici** paneli,
diskteki üretilmiş dosyaların hâlâ üreticinin yazacağı şeyle aynı olup olmadığını
söyler — elle düzenlenmiş bir dosyayı bu yakalar.

Üretici Rust'tır ve artık hiçbir şeyin yanında koşmuyor: port, gerçek veriye
karşı 28 fikstürün hepsinde bayt bayt eşitliğe ulaştı ve Bash motoru aynı
değişiklikte emekliye ayrıldı. Yani üç değil, iki davranış var:

| Kip | Davranış |
|-----|----------|
| `rust` | Üretir ve yazar. **Varsayılan**, ve yazan tek kip. |
| `verify` | Yazmadan üretir, ve diskte olanla arasındaki farkı raporlar. |
| `bash` | Emekli. Eski bir çağıranın ayrıştırma hatası yerine bir cümle alması için enum'da duruyor. |

### 4. Servisler ve paketler

MySQL, Redis, Elasticsearch gibi servisler **paket kataloğundan** gelir. Katalog
imzalıdır: `registry.json` minisign ile, her manifest sha256 ile, her dosya
ayrıca sha256 ile doğrulanır. Aynı anda **MySQL 8.0 ve 8.4** gibi iki örneği
yan yana çalıştırabilirsiniz.

### 5. Üç yüzey, tek çekirdek

```text
                   ┌──────────────────────────────┐
   Pencere ────────►│                              │
   (Vue 3)          │   Rust çekirdek              │──► Docker / Compose
   stackvo CLI ────►│   (130 modül, 348 komut)     │──► Traefik · mkcert · hosts
   stackvo-mcp ────►│                              │──► Dosya sistemi (workspace)
   (AI asistanı)    └──────────────────────────────┘
```

Üçü de `contracts/ipc.json` sözleşmesine bakar. Sözleşmede olmayan bir komutu
uygulayan CLI kodu **derlenmez** — bu bir alışkanlık değil, testle zorlanan bir
kural.

---

## Örneklerle kullanım

Bu bölümdeki her şey **pencereden** yapılır. Manifest (`stackvo.json`) örnekleri,
"aynı şey dosyadan da yapılabilir" diyen ikinci yoldur — zorunlu değildir.

### PHP sürümünü değiştirmek

**Proje → Genel bakış → PHP sürümü** açılır listesinden seçin. Uygulama imajın
yeniden kurulması gerektiğini söyler ve tek düğmeyle kurar.

Aynı şey depoya işlenen manifest'ten de yapılabilir; dosyayı kaydettiğinizde
uygulama değişikliği fark eder ve size sorar:

```jsonc
// projects/shop/stackvo.json
"php": { "version": "8.1" }   // 8.4 → 8.1
```

### Bir servis açmak

**Katalog** sayfası kurulabilir servisleri listeler. **Redis → Kur** → sürümü
seçin. Ardından **Proje → Servisler** panelinden işaretleyin; bağlantı dizesi
aynı panelde durur, parola tıklayınca görünür.

İki proje iki farklı Redis sürümü isterse ikisi de çalışır: servisler tekil
kurulum değil, **örnek** (instance) olarak kurulur.

### Alan adı ve HTTPS

İlk projede kendiliğinden halledilir. Elle bakmak isterseniz:

- **Ayarlar → HTTPS sertifikası** — tek joker sertifika paneli, servisleri ve
  tüm projeleri kapsar; yeniden üretme düğmesi buradadır.
- **Ayarlar → Alan adı** — TLD'yi (`.loc`) değiştirin, `/etc/hosts` satırlarını
  görün. hosts yazımı **tek bir yetkili çağrıdır** ve öncesinde farkı gösterir.

### Xdebug ile hata ayıklama

**Proje → Xdebug** panelindeki anahtarı açın. Panel ayrıca IDE'nizin ihtiyaç
duyduğu portu ve yol eşlemesini (`/var/www/html` ↔ proje klasörü) yazar ve
"dinleyen bir şey var mı" sorusunu cevaplar — "breakpoint neden çalışmıyor"un
cevabı tek ekranda.

### "Bu istek neden yavaştı?"

**Proje → Neden yavaş** panelinde **Kaydet**'e basın, sayfayı tarayıcıda açın,
kaydı durdurun. Listeden bir isteğe tıkladığınızda üç kaynak yan yana gelir:

- örnekleyici profilleyicinin gördüğü fonksiyonlar,
- veritabanına gerçekten sorulan sorgular (aynı sorgunun 40 kez sorulması dâhil),
- uygulamanızın kendi `dump()` çıktıları.

Aynı panelde **Tekrar gönder** düğmesi vardır: kaydedilen isteği profilleyici
açıkken yeniden yollar ve iki ölçümü yan yana koyar — performans işinin en sık
tekrarlanan döngüsü ("değişikliğim işe yaradı mı") dört adım yerine tek tık.

<img src="docs/screenshots/project-detail-debugging.png" alt="Hata ayıklama bölümü: Xdebug, profil çıkarıcı ve dump yakalayıcı" width="100%">

### E-postaları görmek

**Mail** sayfası, uygulamanın gönderdiği e-postaları yakalar ve **uygulamanın
içinde** gösterir: HTML önizleme, bağlantı denetimi, arama. Tarayıcı sekmesi
gerekmez.

### Veritabanı anlık görüntüsü

**Proje → Snapshot'lar** → ad verin → **Al**. Geri yükleme aynı paneldedir ve
onay ister. **Ayarlar → Yedekler** ise bunu zamanlanmış hale getirir: saatten
değil son snapshot'tan ölçer, yani üç gün kapalı kalan bir dizüstü üç değil bir
snapshot borçludur.

### Git dalı başına tam ortam

**Proje → Worktree'ler** → dalı seçin → **Oluştur**. Yeni dal kendi alan adını
(`feature-checkout.shop.loc`), **kendi veritabanını** ve kendi ortam
değişkenlerini alır.

Veritabanı gerçekten ayrıdır: o veritabanına özel bir kullanıcı ile açılır, yani
dal türediği veritabanına erişemez. İşiniz bitince aynı panelden tek düğmeyle
silinir.

Panelin arkasında `worktree.rs` ve yedi komut var; aynı komutlar CLI'dan ve MCP
sunucusundan da çağrılabiliyor — bulutun "preview environment" diye sattığı şey,
yerelde ve ücretsiz.

### Projeyi internete açmak

**Proje → Paylaş** panelinden bir sağlayıcı seçip başlatın. Dokuz sağlayıcı
desteklenir: Cloudflare (anonim ve adlı), ngrok, Tailscale, zrok, Pinggy,
localtunnel, localhost.run, LocalXpose. Sabit adres tutan sağlayıcılarda tünelin
önüne parola koyan bir koruma vardır.

### Monorepo'yu tek proje gibi yönetmek

**Proje → Bu deponun geri kalanı** paneli, deponun diğer klasörlerini kendi
çalışma zamanlarıyla birer bileşen olarak ekler. Manifest'teki karşılığı:

```json
{
  "name": "platform",
  "components": [
    { "name": "api",    "path": "api",    "runtime": "go",     "port": 8080 },
    { "name": "web",    "path": "web",    "runtime": "nodejs", "port": 3000 },
    { "name": "worker", "path": "worker", "runtime": "python" }
  ]
}
```

Tek giriş, tek başlatma, tek sertifika. Her bileşen kendi Dockerfile'ını, kendi
compose servisini ve kendi Traefik yönlendirmesini alır; hiçbiri host portu
açamaz.

### Takım arkadaşınıza yığını devretmek

`.env` commit'lenmez (içinde parolalar vardır), dolayısıyla depoyu klonlayan kişi
"hangi servisler açık, hangi sürümde?" bilgisini alamaz. Bu cümleyi depoya
koyabileceğiniz yer `stackvo.preset.json`:

```json
{
  "services": { "mysql": { "enabled": true, "version": "8.4" },
                "redis": { "enabled": true, "version": "7.2" } },
  "settings": { "DEFAULT_TLD_SUFFIX": "loc" }
}
```

Projeyi açtığınızda **Gereksinimler** kartı "bir hazır ayar var" der ve
uygularsanız neyin değişeceğini **önce plan olarak gösterir** — başkasının
klonuyla gelen bir dosya, siz bir sayfayı açtınız diye yığınınızı değiştirmemeli.
Hazır ayar hiçbir zaman parola taşıyamaz; şeması buna izin vermez.

### Kendi komutlarınızı tanımlamak

Bir projeye özel komut manifest'te durur ve **Proje → Komutlar** panelinde düğme
olarak görünür:

```json
"commands": {
  "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Arama dizinini yeniden kur" }
}
```

Her projede kullandığınız komutlar için **Ayarlar → Makine komutları** var; aynı
şema, çalışma alanının kökündeki tek bir `commands.json` dosyası. Bir id'yi proje
de tanımlarsa **proje kazanır** ve panel hangi satırın hangi dosyadan geldiğini
söyler. Komutlar **yalnızca projenin konteynerinde** çalışır; host'a ulaşan bir
biçim yoktur.

### Üretim imajını buradan kurmak

**Proje → Üretim imajı** paneli planı gösterir, imajı kurar, kaydeder ve bir
kayıt defterine gönderir. Yerel geliştirme ortamı, dağıttığınız imajı da kurar.

Arkasında `release.rs` ve yedi IPC komutu var — plan, kur, kaydet, yükle, reçete,
gönderim planı, gönder — yani aynı adımlar yalnız bu panelden değil, CLI'dan ve
bir asistandan da çağrılabiliyor.

### Devcontainer dışa aktarımı

**Proje → Devcontainer → Dışa aktar**: konteynerin *içinde* çalışmayı tercih eden
takım arkadaşları için bir `.devcontainer` üretir — böylece burada kurulan proje
bir bulut ortamında da açılabilir.

---

## Terminalden kullanım (`stackvo`)

> **Bu bölüm isteğe bağlıdır.** Yukarıdaki her iş pencereden yapılır; CLI, aynı
> çekirdeğin ikinci bir yüzüdür ve asıl işe yaradığı yerler bellidir: betikler,
> CI adımları, `cd` ettiğiniz projede hızlıca bir komut çalıştırmak ve pencere
> açmadan bir soruya cevap almak.

`stackvo` ve `stackvo-mcp` **uygulamanın içinde gelir**; ayrıca indirmeniz
gerekmez. **Ayarlar → Araçlar** paneli aynı işi düğmelerle yapar.

```bash
stackvo path-install      # uygulamanın kendi klasörüne bağlar ve PATH'e ekler
stackvo tools             # ne nerede, hepsinin durumu
stackvo path-remove       # eklenen satırı geri alır (yedek alınmıştı)
```

### Sık kullanılan komutlar

```bash
stackvo status                        # projeler ve servisler
stackvo up shop / down shop           # başlat / durdur
stackvo restart shop
stackvo logs shop --follow            # canlı günlük
stackvo open shop                     # tarayıcıda aç
stackvo doctor --json | jq '.ports[] | select(.state != "ok")'
stackvo tui                           # tam ekran terminal arayüzü
```

### Projenin konteynerinde komut çalıştırma

Proje klasörüne `cd` yapıp yazmanız yeterli:

```bash
stackvo php -v            # makinede PHP olmasa bile projenin PHP'si
stackvo artisan migrate --force
stackvo composer install
stackvo npm run build
stackvo wp plugin list    # ayrıca console, rails, bundle, yarn, pnpm
stackvo python -V         # ve ruby, go, cargo, bun, deno
stackvo shell             # konteynerde etkileşimli kabuk
stackvo exec <program>    # başka her şey
```

Komut adından sonrası **olduğu gibi** aktarılır, çıkış kodu **aynen döner** —
bu yüzden `stackvo artisan test` bir CI betiğinde anlamlıdır.

### Betikler için sözleşme

| Kural | Ayrıntı |
|-------|---------|
| `--json` | **Her** komutta var. Ekrandaki tablo da bu değerden çizilir, ikisi ayrışamaz. |
| stdout / stderr | Cevap stdout'ta, anlatım stderr'de. Hata durumunda stdout boş kalır. |
| Çıkış kodları | `3` = bu makinede kurulum yok · `4` = Docker çalışmıyor · `2` = hatalı komut satırı · `1` = diğer · `127` = projede olmayan çalıştırıcı |
| Bilinmeyen bayrak | **Hata** — sessizce yok sayılmaz. |
| Tamamlama | bash, zsh, fish, PowerShell. Komut listesi tek yerde tutulduğu için yeni komut anında tamamlanır. |

---

## Yapay zekâ asistanlarıyla kullanım (MCP)

`stackvo-mcp`, uygulamanın sürdüğü çekirdek üzerinde bir **MCP sunucusudur**.
Asistan "shop.loc neden açılmıyor?" sorusunu ön kontrol raporundan, hosts
dosyasından, sertifikanın SAN listesinden ve konteynerin son yüz satırından
cevaplayabilir — pencere açık olmasa bile.

**Ayarlar → Yapay zekâ asistanları** makinedeki sekiz istemciyi listeler ve tek
tıkla kaydeder:

Claude Code · Claude Desktop · Cursor · Windsurf · VS Code · Gemini CLI ·
Codex · Zed

Her istemcinin kendi yapılandırma dosyası okunur, **tek bir `stackvo` girdisi**
eklenir ve geri yazılır; diğer sunucularınız korunur, önce `.stackvo-backup`
yedeği alınır.

### Yetkilendirme — varsayılan salt okunur

Yetkiyi **Ayarlar → Yapay zekâ asistanları** panelindeki anahtarlarla verirsiniz;
panel, kaydettiğiniz satırın hangi cümleye karşılık geldiğini de yazar —
*"bu asistan `shop` projesini yeniden başlatabilir, önümüzdeki yarım saat
boyunca"*.

| Ayar | Etkisi |
|------|--------|
| *(varsayılan)* | Yalnızca okuma. 38 aracın 26'sı görünür. |
| **Yazmaya izin ver** | 12 değiştirici araç eklenir — `stack_down` dâhil, yani yığının tamamını durdurabilir. |
| **Projeyle sınırla** | Sunucu tek projeye bağlanır; hiçbir projenin sınırlayamadığı sekiz araç **hiç sunulmaz**. |
| **Süre sınırı** | Yazma yarısı belirtilen süre sonunda kendiliğinden kapanır. |
| **Tek tek araç seçimi** | Dördü de fazlaysa yalnızca adını verdiğiniz araçlar açılır. |

**Bayrağı geçmeden önce bu listeyi okuyun.** 38 aracın 12'si bir şeyi değiştirir
ve yalnızca **Yazmaya izin ver** ile görünür: `xdebug_set`,
`certificates_reissue`, `project_start`, `project_stop`, `stack_up`,
`stack_down`, `generate`, `project_restart`, `service_start`, `service_stop`,
`service_restart`, `snapshot_take`. Bu, asistana yalnız Xdebug'ı açıp kapatma
değil, **yığının tamamını durdurma** ve her projenin bağlı olduğu ortak bir
servisi durdurma yetkisi verir. Her araç `readOnlyHint` / `destructiveHint` ile
işaretlidir, yani istemci hiç görmediği bir araç için onay isteyebilir.

**Ya da sınırlı: araç araç, proje proje.** Anahtarların CLI karşılığı var:
`--project=shop` sunucuyu tek projeye bağlar, `--for=30m` yazma yarısını
kendiliğinden kapatır. Proje sınırı altında yazan araçlardan geriye yalnızca
bir projenin sınırlayabildikleri kalır — `xdebug_set`, `project_start`,
`project_stop`, `project_restart` — hiçbir projenin sınırlamadıkları, `stack_down`
dâhil, hiç sunulmaz. `stack_down`'ı yine de sunan bir sınır, uygulamadığı bir
sınırı bildiriyor olurdu; bu hiç sınır olmamasından kötüdür. Okumaları da aynı
ölçüde sınırlar: bir projeyi *adlandıran* hiçbir araç, kapsam dışındaki bir
proje için cevap vermez. Bu bir bilgi yalıtımı değildir ve öyle anlatılmıyor —
makine geneli araçlar cevap vermeye devam eder, çünkü onlar bir projeyle değil
makineyle ilgilidir.

**Sunulmayan:** değiştiren yüzeyin geri kalanı. 344 komutun 69'u bir `AppHandle`
alır, çünkü ilerlemeyi Tauri'nin olay sistemi üzerinden bildirirler ve bir stdio
alt süreci içine olay yayabileceği bir uygulamaya sahip değildir. Bunu ayırmak
kendi başına bir yeniden yapılandırma; aksini iddia etmek `project_build`'i
duyurup çağrıldığında düşmesine izin vermek olurdu.

Sınırlar:

- **Hiçbir araç parola döndürmez**; şemada `reveal` / `password` / `secret` /
  `token` alanı olmadığını bir test doğrular.
- **Snapshot geri yükleme bilerek araç değildir.** Almak araçtır (dosya ekler,
  hiçbir şeyi değiştirmez); canlı satırların üzerine veri yazmak uygulamanın
  kendi onayına aittir.
- Yapılan her yazma çağrısı — **reddedilenler dâhil** — denetim kaydına yazılır,
  çoğu "bunu geri almak ne demek" bilgisiyle birlikte; böylece Ayarlar'dan tek
  tıkla geri alınabilir.

### Asistana *ne zaman* kullanacağını söylemek

**Ayarlar → AI kuralları**, asistanın zaten okuduğu talimat dosyasına gerekli
satırları yazar:

| Dosya | Okuyan |
|-------|--------|
| `CLAUDE.md` | Claude Code |
| `AGENTS.md` | Codex, Zed |
| `.cursor/rules/stackvo.mdc` | Cursor |
| `.github/instructions/stackvo.instructions.md` | VS Code, Copilot |
| `.windsurf/rules/stackvo.md` | Windsurf |
| `GEMINI.md` | Gemini CLI |

Yalnızca `<!-- stackvo:rules:begin -->` ile `<!-- stackvo:rules:end -->` arası
yazılır; dosyanın geri kalanı bayt bayt korunur.

---

## Başka bir araçtan geçiş

StackVo **yedi** yerel ortamdan içe aktarma yapar — bu kategorideki en geniş
liste:

| Kaynak | Bulunan | Kaynak | Bulunan |
|--------|---------|--------|---------|
| **XAMPP** | `htdocs` | **Laravel Sail** | `docker-compose.yml` |
| **Laragon** | `www` | **Laravel Herd** | site listesi |
| **MAMP** | `htdocs` | **DDEV** | `.ddev/config.yaml` |
| **Laravel Valet** | park/link edilmiş siteler | | |

Nasıl çalışır:

1. Makinede kurulu olanları bulur ve ne bulduğunu **gösterir**.
2. Varsayılan olarak **kopyalar**; taşımasını isterseniz siz söylersiniz.
3. **Geldiği kuruluma tek bayt yazmaz** — PATH düzenlemesi yok, servis kapatma
   yok. Vazgeçerseniz geri alınacak bir şey yoktur.

---

## Desteklenen yığın

### Diller ve sürümler

| Dil | Sürümler | Varsayılan |
|-----|----------|------------|
| **PHP** | 5.6 · 7.0–7.4 · 8.0–8.5 | 8.4 |
| **Node.js** | 16 · 18 · 20 · 21 · 22 · 23 | 22 |
| **Python** | 2.7 · 3.5–3.14 | 3.14 |
| **Go** | 1.11–1.23 | 1.23 |
| **Ruby** | 2.4–3.3 | 3.3 |
| **Rust** | 1.70–1.84 | 1.84 |

### Web sunucuları

`nginx` · `apache` · `caddy` · `frankenphp` · **`swoole`** · **`roadrunner`**

Son ikisi Laravel Octane'in iki sürücüsüdür; ikisi de HTTP sunucusunun
kendisidir ve Traefik 80 yerine 8000'e bakar.

### Servisler

| Kategori | Servisler |
|----------|-----------|
| **Veritabanı** | MySQL · MariaDB · PostgreSQL · MongoDB · Cassandra · ClickHouse · MS SQL Server |
| **Önbellek** | Redis · Memcached · Valkey · Dragonfly |
| **Kuyruk / mesaj** | RabbitMQ · Kafka · Soketi · Beanstalkd |
| **Arama** | Elasticsearch · Kibana · Meilisearch · Typesense · Solr |
| **Depolama** | MinIO |
| **İzleme** | Grafana · Prometheus · Graylog |
| **Geliştirici** | MailHog · Mailpit · Blackfire |
| **Yönetim arayüzü** | phpMyAdmin · Adminer · pgAdmin · Kafbat · mongo-express · phpCacheAdmin |

### PHP eklentileri

80'den fazla eklenti tanınır (`apcu`, `imagick`, `intl`, `redis`, `swoole`,
`mongodb`, `xdebug`, `sqlsrv`…). Varsayılan set, seçtiğiniz PHP sürümüyle
**kurulabilir olduğu doğrulanmış** bir settir.

---

## Mimari

### Genel görünüm

```text
┌─────────────────────────────────────────────────────────────────────┐
│  StackVo Desktop  (host'ta normal bir kullanıcı süreci)             │
│                                                                     │
│  ┌───────────────┐   contracts/ipc.json    ┌────────────────────┐   │
│  │  Ön yüz       │  348 komut / 75 olay    │  Arka uç (Rust)    │   │
│  │  Vue 3        │◄───────────────────────►│  130 modül         │   │
│  │  Vuetify 3    │      Tauri IPC          │  ~76k satır        │   │
│  │  Pinia        │                         │                    │   │
│  │  ~38k satır   │                         │                    │   │
│  └───────────────┘                         └─────────┬──────────┘   │
└──────────────────────────────────────────────────────┼──────────────┘
                                                       │ bollard / compose
                          ┌────────────────────────────▼───────────────┐
                          │  Docker · Podman · Colima · OrbStack        │
                          │  ┌────────┐ ┌──────────┐ ┌───────────────┐  │
                          │  │Traefik │ │ shop     │ │ mysql · redis │  │
                          │  │(proxy) │ │ (proje)  │ │ (servisler)   │  │
                          │  └────────┘ └──────────┘ └───────────────┘  │
                          └────────────────────────────────────────────┘
```

### Bir isteğin yolculuğu

Kullanıcı *Proje oluştur*'a bastığında:

```text
Vue bileşeni
  └─ composable (src/composables/useX.js)        durum, markup yok
       └─ api.projectCreate(spec)                src/lib/ipc.js
            └─ invoke('project_create', {...})   Tauri IPC
                 └─ #[tauri::command] project_create      src-tauri/src/commands.rs
                      ├─ state.root()                     çalışma alanı, yoksa hata
                      ├─ manifest::parse / validate       şema; serbest JSON değil
                      ├─ scaffold::write(...)             projenin dosyaları
                      ├─ generator::render(...)           compose + Dockerfile + proxy
                      └─ runner::run_operation(...)       docker compose, akış hâlinde
                           └─ olaylar: project:creating → project:created
```

### Arka uç katmanları

Bağımlılık okları **hep aşağı** bakar:

```text
  entry        2.0k   lib.rs, main.rs, menü, tepsi
      ▼
  commands.rs 14.9k   IPC yüzeyi: 348 komut — doğrulama ve orkestrasyon
      ▼
  domain      89.7k   107 modül: generator, manifest, certs, hosts, mail,
      ▼               xdebug, profile, worktree, policy, audit… (Tauri tipi yok)
  platform     6.6k   engine (Docker), runner, elevate, pty, watcher, git
      ▼
  primitives   2.3k   error, events, progress, inflight, logging, contracts
```

`commands.rs` `AppHandle`'dan bahseden **tek** dosyadır. Altındaki her şey
testten, `diagnose` örneğinden ve MCP yüzeyinden — uygulama çalışmadan —
çağrılabilir.

### Ön yüz düzeni

```text
src/
  views/          9 sayfa, rota başına bir tane
  components/     ortak bileşenler + project/ (45 panel) ve settings/ (23 panel)
  composables/    18 dosya: durum ve sınır çağrıları, markup yok
  stores/         Pinia: app, appearance, inventory, metrics, operations
  lib/            ipc.js (üretilen istemci), format, events
  i18n/           en.js, tr.js
```

Kural: **sayfa panelleri birleştirir, panel markup'a sahiptir, composable duruma
sahiptir ve yalnızca composable `api` ile konuşur.**

### Neden konteyner değil de masaüstü uygulaması?

| | Konteyner içindeki panel | StackVo Desktop |
|--|--------------------------|-----------------|
| Çalışma biçimi | konteyner, root, `chmod 666 docker.sock` | host süreci, sizin kullanıcınız |
| Docker kapalıyken | erişilemez | açılır, raporlar, başlatmayı teklif eder |
| Host metrikleri | konteyner içinden `/proc` (yanlış) | host üzerinde `sysinfo` |
| Yığını durdurmak | imkânsız (kendini öldürür) | `compose_down` |
| hosts dosyası | elle `sudo tee -a /etc/hosts` | fark gösterilir, tek yetkili yazma |
| Windows | yalnızca WSL2 | yerel — kabuk yok, bash yok |
| Kurulum boyutu | ~600 MB imaj | ~27 MB, CLI dâhil |

### Teknoloji yığını

| Katman | Teknoloji |
|--------|-----------|
| Kabuk | Tauri 2 (WebView, Rust) |
| Arka uç | Rust — `bollard` (Docker API), `serde`, `tokio` |
| Ön yüz | Vue 3 (`<script setup>`), Vuetify 3, Pinia, Vue Router, vue-i18n |
| Terminal | xterm.js + gerçek PTY |
| Proxy / TLS | Traefik + mkcert |
| Test | Vitest, Playwright, axe-core, `cargo test`, farksal fixture testleri |
| Paket doğrulama | minisign + sha256 |

Ayrıntılı harita: **[ARCHITECTURE.md](ARCHITECTURE.md)**

---

## Yapılandırma

### Dosyalar nerede?

| | macOS | Windows | Linux |
|--|-------|---------|-------|
| Uygulama günlüğü | `~/Library/Logs/StackVo/` | `%LOCALAPPDATA%\StackVo\logs\` | `~/.local/state/stackvo/logs/` |
| Tercihler | `~/Library/Application Support/StackVo/` | `%APPDATA%\StackVo\` | `~/.config/stackvo/` |
| Yığın durumu | `~/.stackvo/` | `~/.stackvo/` | `~/.stackvo/` |

`~/.stackvo` sizindir: taşıyabilir, silebilirsiniz. Uygulamanın **başlamak için**
oradaki hiçbir şeye ihtiyacı yoktur.

### Ortam değişkenleri

| Değişken | Ne yapar |
|----------|----------|
| `STACKVO_ROOT` | Çalışma alanının yerini değiştirir |
| `STACKVO_LOG` | Günlük seviyesi — ör. `stackvo_desktop=debug` |
| `STACKVO_POLICY_FILE` | Politika dosyasını root olmadan denemek için |
| `DOCKER_HOST` | Alışıldık anlamıyla; şema ayıklaması yapılır |

### Parolaları `.env` dışına almak

**Ayarlar → Kimlik bilgileri nerede tutulsun** bir parolayı işletim sisteminin
kasasına (Keychain / Credential Manager / Secret Service) taşır ve yerine bir
referans bırakır:

```sh
SERVICE_MYSQL_ROOT_PASSWORD=keychain:SERVICE_MYSQL_ROOT_PASSWORD@a1b2c3d4
```

Bu, parolayı yedeklenen/senkronlanan/destek yazışmasına yapıştırılan dosyanın
dışına çıkarır. **Diskten tamamen kaldırmaz**: Compose'un okuduğu üretilmiş
compose dosyasında gerçek değer hâlâ vardır. Bu sınır belgede açıkça yazılıdır.

### Çok makineli kurulum — yönetici politikası

| | Yol |
|--|-----|
| macOS | `/Library/Managed Preferences/com.stackvo.desktop.json` |
| Windows | `%ProgramData%\StackVo\policy.json` |
| Linux | `/etc/stackvo/policy.json` |

```json
{
  "schemaVersion": 1,
  "settings": { "DEFAULT_TLD_SUFFIX": "corp.test", "SERVER_TYPE": "nginx" },
  "locked": ["DEFAULT_TLD_SUFFIX"],
  "registryPrefix": "registry.corp.example/proxy"
}
```

- `settings` hem varsayılanı hem `.env`'i ezer.
- `locked` o anahtarların Ayarlar'dan değiştirilmesini reddeder.
- `registryPrefix` üretilen tüm imaj referanslarının başına eklenir (zaten bir
  kayıt defteri adı taşıyanlar hariç).
- **Bu bir güvenlik sınırı değildir** ve belge bunu açıkça söyler; işbirliği
  yapan bir uygulamaya kurumun niyetini bildirir, o kadar.

`Ayarlar → Uyumluluk` politikanın *söylediğini* değil, bu makinede **fiilen
geçerli olup olmadığını** ölçer.

---

## Güvenlik ve gizlilik

- **Telemetri yok.** Bu cümle bir vaat değil, bir testle korunuyor
  (`src-tauri/tests/privacy_claims.rs`).
- **Günlükler maskelenir.** Parola ve token değerleri yazılırken maskelenir;
  tanılama paketi ikinci kez maskeler, `.env` ve proje kaynaklarını içermez.
- **Paket zinciri doğrulanır.** Katalog dizini minisign ile imzalıdır; her
  manifest ve her dosya sha256 ile doğrulanır. İmza varsa **kontrol edilir**;
  başarısız imza reddir, sessiz geçiş yoktur.
- **Sızmış kimlik bilgisi taraması.** `.env` ve git'in izlediği tüm dosyalarda
  anahtarın *adı* değil **değeri** aranır (`AKIA…`, `ghp_…`, PEM başlığı).
  Bulgu parmak izi ve maskeli önizleme taşır, değeri asla.
- **Hangi konteyner dışarı çıkabiliyor?** Tahmin edilmez, Docker'a sorulur:
  `internal` ağın ağ geçidi yoktur, dolayısıyla tüm ağları internal olan bir
  konteyner **kanıtlanabilir biçimde** dışarı çıkamaz.
- **Denetim izi.** Geri alınamayan işlemler ve asistanın yaptığı her yazma
  çağrısı — reddedilenler dâhil — kaydedilir.

Ayrıntı: [SECURITY.md](SECURITY.md) · [PRIVACY.md](PRIVACY.md)

---

## Benzer araçlarla karşılaştırma

Aşağıdaki tablo, aynı kategorideki araçların **yaklaşım farklarını** özetler;
"daha iyi/kötü" değil, "hangisi neyi seçmiş" tablosudur.

| | **StackVo** | Herd | ServBay | Laragon | DDEV | Laradock | Devilbox | FlyEnv |
|---|---|---|---|---|---|---|---|---|
| Yaklaşım | Docker + masaüstü | Yerel ikili | Yerel ikili | Yerel ikili | Docker + CLI | Docker + compose | Docker + compose | Yerel ikili |
| Arayüz | Masaüstü + CLI + TUI + MCP | Masaüstü | Masaüstü | Masaüstü | CLI | Yok | Web intranet | Masaüstü |
| Platform | mac · Win · Linux | mac · Win | mac · Win | Win | mac · Win · Linux | hepsi | hepsi | mac · Win · Linux |
| Proje izolasyonu | Konteyner | Site | Site | Site | Konteyner | Paylaşılan yığın | Paylaşılan yığın | Site |
| Otomatik HTTPS | Var (mkcert) | Var | Var | Var | Var | elle | Var | Var |
| Dal başına ortam + **ayrı DB** | Var | Yok | Yok | Yok | kısmî | Yok | Yok | Yok |
| İstek düzeyinde "neden yavaş" | Var (profil+sorgu+dump) | kısmî (Pro) | Yok | Yok | Yok | Yok | Yok | Yok |
| İsteği yeniden gönderme | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| Uygulama içi mail kutusu | Var | Var (Pro) | Var (Pro) | Var | web arayüzü | web arayüzü | web arayüzü | Var |
| Adlandırılmış DB anlık görüntüsü | Var (+ zamanlanmış) | Yok | zamanlanmış | zamanlanmış | Var | Yok | Yok | Yok |
| Monorepo tek proje | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| MCP / AI entegrasyonu | Var — 38 araç, yetki sınırlı | Yok | Var | Yok | Yok | Yok | Yok | Var |
| Üretim imajı kurma | Var | Yok | Yok | Yok | Yok | Var | Yok | Yok |
| Devcontainer dışa aktarma | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| İçe aktarma kaynağı | **7** | 1 | birkaç | Yok | birkaç | Yok | Yok | birkaç |
| Kaynak maliyetini ölçme | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| Yönetici politikası (MDM) | Var | Yok | takım planı | Yok | Yok | Yok | Yok | Yok |
| Taşınabilir kurulum | Yok (mimari gereği) | Yok | Yok | Var | Yok | Yok | Yok | Var |
| Codespaces / Gitpod içinde | Yok | Yok | Yok | Yok | Var | Var | Var | Yok |
| Fiyat | Ücretsiz, MIT | Ücretsiz + Pro $99/yıl | Ücretsiz + Pro | Ücretsiz | Ücretsiz, Apache-2 | Ücretsiz, MIT | Ücretsiz, MIT | Ücretsiz |

<sub>Tablo Ağustos 2026 itibarıyla, projelerin kendi belgelerine dayanır. Bir
satır hatalıysa lütfen issue açın — düzeltiriz.</sub>

### Dürüst olmak gerekirse: Docker'ın bedeli

| | StackVo | Host'a PHP kuran bir araç |
|---|---|---|
| İlk kurulum | uygulama (~27 MB) **+ Docker ve imajlar (GB)** | tek yükleyici, ~100 MB |
| İlk `up` | imaj kurulumu — dakikalar | saniyeler |
| **PHP sürümü değiştirme** | manifest'i değiştir, imajı yeniden kur | anında |
| Boştaki bellek | Docker VM + Traefik + açık servisler | yalnızca dil çalışma zamanı |

Karşılığında aldığınız şey, hiçbirinin veremediği şeydir: her projenin ortamı
bir konteynerdir; makinenizde çalışan şey `brew` geçmişinizin değil, bir
Dockerfile'ın söylediği şeydir.

**Bilinçli iki sınır:**

- **Taşınabilir kurulum yok ve olamaz.** İmajlar ve volume'lar Docker'ın kendi
  deposunda yaşar; `STACKVO_ROOT` cevabın yarısıdır — çalışma alanınız sizinle
  gelir, motor gelmez.
- **Codespaces/Gitpod içinde çalışmaz.** Bu bir masaüstü uygulamasıdır. Bunun
  yerine **devcontainer dışa aktarır** — burada kurulan proje bir bulut
  ortamında açılabilir.

---

## Kaynaktan derleme ve geliştirme

### Gereksinimler

- Node.js **22+** (`.nvmrc` mevcut)
- Rust (kararlı; sürüm `src-tauri/rust-toolchain.toml` ile sabit)
- Tauri 2 sistem bağımlılıkları — [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/)
- Docker (uygulamayı çalıştırmak için)

### Derleme ve çalıştırma

```bash
git clone https://github.com/stackvo/stackvo.git
cd stackvo
npm install
npm run tauri:dev          # uygulamayı geliştirme modunda çalıştır

npm run tauri:build        # yükleyicileri üret
```

### Test ve kontroller

```bash
npm test                   # vitest + Rust birim/entegrasyon/farksal testler
npm run test:js            # yalnızca ön yüz
npm run test:e2e           # Playwright (erişilebilirlik dâhil)
npm run lint               # eslint + prettier
npm run audit              # cargo-deny + npm audit
npm run contracts:check    # IPC sözleşmesi tutarlılığı
npm run diagnose           # başsız uçtan uca kontrol
npm run bundle:budget      # paket boyutu bütçesi
```

CI bunların tamamını **Linux, macOS ve Windows** üzerinde, `cargo clippy -D
warnings` ve `cargo fmt --check` ile birlikte çalıştırır.

### Projeye özgü birkaç kural

- **Sözleşme önce gelir.** `contracts/ipc.json`'da olmayan bir komut ne
  `lib.rs`'e kaydedilebilir ne de CLI'dan sürülebilir; testler bunu zorlar.
- **Üretilen dosyalar elle düzenlenmez.** `generated/` her zaman yeniden
  üretilebilir olmalıdır; `generator_verify` bunu gerçek makinede kanıtlar.
- **Panel ve stilleri birlikte taşınır.** `<style scoped>` yalnızca kendi
  bileşenine ulaşır; `tests/pane-styles.spec.js` bunu denetler.

Ayrıntı: [CONTRIBUTING.md](CONTRIBUTING.md)

---

## Durum ve yol haritası

| Faz | Kapsam | Durum |
|-----|--------|-------|
| 0 | Yapılandırma sözleşmesini dondur, eklenti matrisini çıkar, IPC yüzeyini türet | Tamam |
| 1 | Tauri + Vue iskeleti; salt okunur görünümler, gerçek metrikler | Tamam |
| 2 | `bollard` ile konteyner kontrolü; akış hâlinde derleme/günlük | Tamam |
| 3 | Tepsi, bildirimler, dosya izleyici, hosts yardımcısı, PTY, otomatik başlatma | Tamam |
| 4 | Üreticinin Rust'a taşınması, yerel Windows, imzalı otomatik güncelleme | Sürüyor |
| 5 | Yayın altyapısı (imzalama anahtarı ve güncelleme uç noktası) | Sürüyor |

**Hiçbir commit'in kapatamayacağı işler** ayrıca yazılıdır: güncelleme uç
noktasının yayımlanması, imza anahtar çiftinin üretilmesi, Apple ve Windows
sertifikalarının satın alınması. Bunlar iş değil, karar ve satın alma
kalemleridir; `npm run updates:check` eksik uç noktayı, yayın akışı ise
imzasız her hedefi uyarı olarak raporlar.

---

## Sık sorulan sorular

<details>
<summary><b>Docker şart mı?</b></summary>

Evet. Docker Desktop, Docker Engine ya da API uyumlu bir çalışma zamanı
(Podman, Colima, OrbStack) gerekir. Motorun adı yalnızca bir etikettir; hiçbir
davranış "hangisi" olduğuna göre dallanmaz.
</details>

<details>
<summary><b>Neden kod imzalı değil?</b></summary>

Uygulama yalnızca GitHub Releases üzerinden dağıtılıyor. Apple Developer
üyeliği ve Authenticode sertifikası, kimliğe bağlı ve tekrarlayan maliyetlerdir;
zincirdeki son dış bağımlılık olarak bilerek atlandı. Karşılığında: her sürümde
`SHA256SUMS` yayımlanır ve güncelleyici minisign imzasını doğrular.
</details>

<details>
<summary><b>Mevcut StackVo (Bash/web arayüzü) kurulumum ne olacak?</b></summary>

Olduğu gibi çalışır. Her iki araç da aynı `stackvo.json` ve `.env` dosyalarını
okur; birinde oluşturulan proje diğerinde çalışır. Bu uyumluluk gelenekle değil,
depoya işlenmiş bir sözleşme ve bir doğrulayıcı ile korunur.
</details>

<details>
<summary><b>Aynı servisin iki sürümünü aynı anda çalıştırabilir miyim?</b></summary>

Evet. Servisler "örnek" (instance) olarak kurulur; MySQL 8.0 ve 8.4 yan yana
çalışabilir, projeler istediğine bağlanır.
</details>

<details>
<summary><b>Windows'ta durum ne?</b></summary>

Saf mantık (sürücü harfi → bind mount dönüşümü, named pipe algılama,
`DOCKER_HOST` şema ayıklaması) `cfg` kapısı olmadan yazıldı ve **her platformda**
test ediliyor; `windows-latest` CI matrisinde. Derleyicinin cevaplayamadığı
kısım — UAC ile hosts yazımı, gerçek Docker Desktop'a named pipe, tarayıcıda
alan adı çözümü — hâlâ bir Windows makinesinde doğrulanmayı bekliyor. Bu satır
aksini yazana kadar durum budur.
</details>

<details>
<summary><b>Verilerim nereye gidiyor?</b></summary>

Hiçbir yere. Telemetri yok, uzak sunucuya rapor yok. Tek istisna, **sizin
bastığınız** düğmeler: paket kataloğunu yenilemek, güncelleme kontrolü ve
"bağımlılıklarımda güvenlik açığı var mı" sorgusu. Sonuncusu, paket adlarını
makine dışına gönderdiği için ayrı bir düğmedir ve bunu söyleyen cümle düğmenin
üstündedir. Tamamı: [PRIVACY.md](PRIVACY.md)
</details>

<details>
<summary><b>Sunucuda / CI'da kullanabilir miyim?</b></summary>

Tasarım hedefi değil. CLI ve yerel HTTP yüzeyi başsız kullanımı teknik olarak
mümkün kılar; ancak uçtan uca test edilmediği için "desteklenir" denmiyor.
</details>

---

## Katkı, destek ve lisans

| Belge | İçerik |
|-------|--------|
| [CONTRIBUTING.md](CONTRIBUTING.md) | Nasıl derlenir, kontroller ne ister |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Kodun haritası |
| [SUPPORT.md](SUPPORT.md) | Soru nereye gider, hata raporuna ne eklenir |
| [SECURITY.md](SECURITY.md) | Açığı **özel olarak** bildirin, issue açmayın |
| [ACCESSIBILITY.md](ACCESSIBILITY.md) | EN 301 549 biçiminde uygunluk beyanı |
| [PRIVACY.md](PRIVACY.md) | Ne saklanır, hangi adrese çıkılır |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | İnsanlara nazik olun, işi hak ettiği kadar tartışın |
| [CHANGELOG.md](CHANGELOG.md) | Her değişikliğin gerekçesiyle birlikte kaydı |
| [README.md](README.md) | The same document in English |

**Lisans:** [MIT](LICENSE) © 2026 Fahrettin Aksoy

<div align="center">

**[Başa dön](#stackvo)** &nbsp;·&nbsp; [Read this in English](README.md)

</div>
