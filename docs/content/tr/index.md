---
template: home.html
title: StackVo
hide:
  - navigation
  - toc
  - footer
hero:
  eyebrow: Yerel geliştirme, bir pencerede
  title: Masaüstü uygulaması olan bir yerel geliştirme ortamı
  text: >-
    Proje başına bir Docker konteyneri; kendi çalışma zamanı sürümü, kendi
    veritabanı ve kendi alan adıyla — https://shop.loc, güvenilir sertifika
    dâhil. PHP, Node, Python, Go, Ruby ya da Rust. StackVo konteynerin içinde
    değil makinenizde çalışır, bu yüzden Docker kapalıyken bile açılır.
  meta: macOS · Windows · Linux · ücretsiz, MIT lisanslı
  tabs_label: Ekran görüntüleri
  shots:
    - { file: web/dashboard.webp, label: Panel, alt: "Panel: CPU, bellek, disk, ağ ve çalışan projeler" }
    - { file: web/projects.webp, label: Projeler, alt: "Projeler sayfası: her proje ve konteynerinin durumu" }
    - { file: web/project-detail.webp, label: Proje sayfası, alt: "Bir projenin sayfası: her biri tek konuya ait 45 panel" }
    - { file: web/mail.webp, label: Mail, alt: "Posta kutusu: uygulamanın gönderdiği her şey, pencerede okunur" }
    - { file: web/logs.webp, label: Loglar, alt: "Bütün projelerin logları tek yerde, canlı" }
  secondary:
    label: Başlayın
    href: getting-started/
stack:
  title: Eksiksiz bir yerel yığın
  text: >-
    Çalışma zamanları, web sunucuları, veritabanları ve araçlar; örnek olarak
    kurulur, proje başına seçilir. Laravel, Symfony, WordPress ve Next.js hazır
    ayar alır; gerisi çalışma zamanı olan bir klasördür.
  link: Tam liste
  href: reference/supported-stack/
  more: >-
    Ayrıca arama, izleme, kuyruk ve yönetim arayüzleri — ve 80'den fazla PHP
    eklentisi.
  groups:
    - title: Çalışma zamanları
      icon: material/code-tags
      list:
        - { name: PHP 5.6–8.5, icon: simple/php }
        - { name: Node.js, icon: simple/nodedotjs }
        - { name: Python, icon: simple/python }
        - { name: Go, icon: simple/go }
        - { name: Ruby, icon: simple/ruby }
        - { name: Rust, icon: simple/rust }
    - title: Web sunucuları
      icon: material/server
      list:
        - { name: nginx, icon: simple/nginx }
        - { name: Apache, icon: simple/apache }
        - { name: Caddy, icon: simple/caddy }
        - { name: FrankenPHP }
        - { name: Swoole }
        - { name: RoadRunner }
    - title: Veritabanları
      icon: material/database
      list:
        - { name: MySQL }
        - { name: MariaDB, icon: simple/mariadb }
        - { name: PostgreSQL, icon: simple/postgresql }
        - { name: MongoDB, icon: simple/mongodb }
        - { name: ClickHouse, icon: simple/clickhouse }
        - { name: Cassandra, icon: simple/apachecassandra }
        - { name: MS SQL Server }
    - title: Önbellek ve mesajlaşma
      icon: material/memory
      list:
        - { name: Redis, icon: simple/redis }
        - { name: Valkey }
        - { name: Memcached }
        - { name: Dragonfly }
        - { name: RabbitMQ, icon: simple/rabbitmq }
        - { name: Kafka, icon: simple/apachekafka }
    - title: Araçlar
      icon: material/tools
      list:
        - { name: Mailpit, icon: material/email-outline }
        - { name: phpMyAdmin, icon: simple/phpmyadmin }
        - { name: Adminer, icon: simple/adminer }
        - { name: pgAdmin, icon: material/database-search }
        - { name: Grafana, icon: simple/grafana }
        - { name: MinIO, icon: simple/minio }
features:
  title: Diğerlerinin yapmadığı
  text: >-
    Bu kategorideki her araç size PHP, bir veritabanı ve bir alan adı verir.
    Bunlar yalnızca burada olanlar.
  list:
    - icon: material/source-branch
      title: Git dalı başına tam ortam
      text: Her worktree kendi alan adını ve kendi veritabanını alır. Önizleme ortamları, yerelde ve ücretsiz.
    - icon: material/speedometer
      title: Bu istek neden yavaştı?
      text: Profilleyici, sorgu günlüğü ve dump() çağrılarınız tek eksende; kaydedilen isteği tek tıkla yeniden oynatın.
    - icon: material/package-variant-closed
      title: Proje başına bir konteyner
      text: PHP 5.6–8.5, Node, Python, Go, Ruby, Rust — çalışan şey brew geçmişinizin değil, bir Dockerfile'ın söylediğidir.
    - icon: material/robot-outline
      title: Yapay zekâ asistanları için MCP, tasmalı
      text: 38 araç. Yazma işlemleri açık bir bayrak, proje kapsamı ve süre sınırının arkasında.
    - icon: material/rocket-launch-outline
      title: Üretim imajını kurar
      text: Aynı manifest, yayına aldığınız imajı üretir ve bulut için devcontainer dışa aktarır.
    - icon: material/chart-timeline-variant
      title: Docker'ın size maliyetini ölçer
      text: '"shop bugün 4,2 GB·saat tuttu ve 38 dakika CPU kullandı." Bunu söyleyen tek araç.'
download:
  title: İndir
  text: >-
    Sürüm başına altı yükleyici, platform başına iki. Kod imzası bilinçli
    olarak yok; her sürüm SHA256 sağlama toplamları yayımlar ve güncelleyici
    minisign imzasını doğrular.
  hero_label: '{os} için indir'
  hero_fallback: İndir
  version_fallback: Son sürüm
  yours: Sisteminiz
  others: Diğer biçimler
  integrity: Her dosyanın yanında SHA256 sağlama toplamı, minisign imzalı güncellemeler
  all: Tüm indirmeler GitHub'da
  help: İşletim sistemi "imzasız" derse
  list:
    - key: mac
      icon: material/apple
      title: macOS
      note: macOS 10.15 ve üzeri
      formats:
        - label: Apple Silicon (.dmg)
          file: 'StackVo_{v}_aarch64.dmg'
        - label: Intel (.dmg)
          file: 'StackVo_{v}_x64.dmg'
    - key: win
      icon: material/microsoft-windows
      title: Windows
      note: Windows 10 ve üzeri
      formats:
        - label: x64 yükleyici (.exe)
          file: 'StackVo_{v}_x64-setup.exe'
        - label: ARM64 yükleyici (.exe)
          file: 'StackVo_{v}_arm64-setup.exe'
    - key: linux
      icon: material/linux
      title: Linux
      note: x86_64 ve aarch64
      formats:
        - label: .deb, x86_64
          file: 'StackVo_{v}_amd64.deb'
        - label: .rpm, x86_64
          file: 'StackVo-{v}-1.x86_64.rpm'
        - label: AppImage, x86_64
          file: 'StackVo_{v}_amd64.AppImage'
        - label: .deb, aarch64
          file: 'StackVo_{v}_arm64.deb'
        - label: .rpm, aarch64
          file: 'StackVo-{v}-1.aarch64.rpm'
        - label: AppImage, aarch64
          file: 'StackVo_{v}_aarch64.AppImage'
steps:
  title: Çalışan bir projeye üç adım
  text: Terminal yok, yapılandırma dosyası yok, Docker bilgisi gerekmez. Pencere sorar, siz cevaplarsınız.
  alt: Yeni proje çekmecesi
  caption: Yeni proje çekmecesi — ad, alan adı, çalışma zamanı ve servisler tek panelde.
  cta: İndirip deneyin
  list:
    - title: Çalışma alanı için bir klasör seçin
      text: İlk açılış tam olarak tek soru sorar. Boş bir klasör yeter; gerisini uygulama yazar.
    - title: Projeye ad verin, yığını seçin
      text: Laravel, WordPress, Symfony, düz PHP, Node… sonra çalışma zamanı sürümü, web sunucusu ve servisler.
    - title: Oluştur'a basın
      text: Kurulum Docker'ın kendi çıktısını akıtır. Bittiğinde alan adı tarayıcıda sertifika uyarısı olmadan açılır.
compare:
  title: Benzer araçlarla karşılaştırma
  text: >-
    Hangisi daha iyi değil, hangisi neyi seçmiş. Projelerin kendi
    belgelerinden derlendi.
  words:
    'yes': Var
    'no': Yok
    partial: kısmî
  columns: [StackVo, Herd, ServBay, Laragon, DDEV, FlyEnv]
  rows:
    - [Yaklaşım, Docker + masaüstü, Yerel ikili, Yerel ikili, Yerel ikili, Docker + CLI, Yerel ikili]
    - [Platform, mac · Win · Linux, mac · Win, mac · Win, Win, mac · Win · Linux, mac · Win · Linux]
    - [Proje izolasyonu, Konteyner, Site, Site, Site, Konteyner, Site]
    - [Dal başına ortam + ayrı DB, Var, Yok, Yok, Yok, kısmî, Yok]
    - ['İstek düzeyinde "neden yavaş"', Var, kısmî (Pro), Yok, Yok, Yok, Yok]
    - [İsteği yeniden oynatma, Var, Yok, Yok, Yok, Yok, Yok]
    - [Uygulama içi mail kutusu, Var, Pro, Pro, Var, web arayüzü, Var]
    - [Adlandırılmış DB snapshot'ı, Var + zamanlanmış, Yok, zamanlanmış, zamanlanmış, Var, Yok]
    - [Yapay zekâ asistanları için MCP, 38 araç yetki sınırlı, Yok, Var, Yok, Yok, Var]
    - [Üretim imajı kurma, Var, Yok, Yok, Yok, Yok, Yok]
    - [İçe aktarma kaynağı, '7', '1', birkaç, Yok, birkaç, birkaç]
    - [Fiyat, 'Ücretsiz, MIT', Ücretsiz + Pro $99/yıl, Ücretsiz + Pro, Ücretsiz, 'Ücretsiz, Apache-2', Ücretsiz + Pro $10]
  more: Tam tablo ve karşılığında Docker'ın bedeli
  href: reference/comparison/
pricing:
  eyebrow: Fiyat
  price: $0
  per: hep
  title: Her şey, herkes için
  text: >-
    Pro katmanı yok ve olmayacak. Hiçbir şey bir hesabın, lisans anahtarının
    ya da uyarı ekranının arkasında değil.
  list:
    - Her özellik, her platformda
    - Hesap yok, lisans anahtarı yok, telemetri yok
    - MIT lisanslı — ne olursa olsun kod erişilebilir kalır
    - Güncellemeler minisign imzasıyla doğrulanır
  cta: İndir
  aside_title: Bunun açık anlamı
  facts:
    - icon: material/account-outline
      title: Tek kişi bakıyor
      text: Sorunlar okunur ve çoğu cevaplanır. "Çoğu" dürüst kelime; cevap süresi taahhüdü yok.
    - icon: material/cash-off
      title: Fonlanmıyor
      text: Arkasında bugün bir şirket, vakıf ya da sponsorluk yok. Bu kategorideki birkaç aracın var; bunun yok.
    - icon: material/file-document-outline
      title: Destek sözleşmesi yok
      text: MIT lisansı, ne olursa olsun kodun erişilebilir kalması demektir. Birinin orada olup düzelteceği anlamına gelmez.
  aside_link: Bunu değiştiren şey sponsorluk
faq:
  title: Sık sorulan sorular
  text: Kısa cevaplar. Uzunları belgelerde ve README'de.
  more: Başka bir sorunuz mu var?
  more_label: Tartışmalarda sorun
  more_href: https://github.com/stackvo/stackvo/discussions
  list:
    - q: Gerçekten ücretsiz mi?
      a: >-
        Evet. Ücretsiz, MIT lisanslı, her platformda her özellik. Ücretli katman, hesap, telemetri yok.
    - q: Yalnızca Laravel için mi?
      a: Hayır. Laravel, Symfony ve WordPress hazır ayar alır, ama proje bir çalışma zamanı olan herhangi bir klasördür — PHP 5.6'dan 8.5'e, Node, Python, Go, Ruby ya da Rust — ve bunları karıştıran bir monorepo tek projedir.
    - q: Docker şart mı?
      a: >-
        Evet: Docker Desktop, Docker Engine ya da Podman, Colima, OrbStack. Çalışmıyorsa uygulama açılır, söyler ve başlatmayı önerir.
    - q: Neden kod imzalı değil?
      a: >-
        Sertifikalar kimliğe bağlı, yinelenen maliyetlerdir. Bunun yerine her sürüm SHA256 özetleri yayımlar ve güncelleyici minisign imzasını doğrular.
    - q: Aynı servisin iki sürümünü aynı anda çalıştırabilir miyim?
      a: >-
        Evet. Servisler örnektir: MySQL 8.0 ve 8.4 yan yana, her proje istediğine bağlanır.
    - q: Yerel bir araca göre Docker'ın bedeli ne?
      a: Docker ve imajlarını içeren bir ilk kurulum, projenin ilk açılışında bir imaj kurulumu ve Docker VM'in boştaki belleği. Karşılığında her projenin ortamı bir konteynerdir — çalışan şey bir Dockerfile'ın söylediğidir.
    - q: Windows'ta durum ne?
      a: Mantık her platformda test edilir ve Windows CI matrisindedir; UAC üzerinden hosts dosyası yazımı, gerçek bir Docker Desktop'a karşı adlandırılmış boru ve tarayıcıda alan adı çözümü gerçek makinede henüz doğrulanmadı.
    - q: Verilerim nereye gider?
      a: Hiçbir yere. Telemetri yok, raporlama yok. Ağa çıkan çağrılar yalnızca sizin bastıklarınız — kataloğu yenilemek, güncelleme denetlemek, açık veritabanına danışmak — ve sizin için yapılan bir tanesi, yardım paneli açıldığında belgeyi depodan çekmek.
more:
  title: Ve sonrası
  text: Pencerenin yaptığı diğer şeyler, her biri kendi sayfasıyla.
  list:
    - title: Mail, dump ve loglar pencerede
      href: guide/everyday-use/
      text: Gönderilen e-postalar yakalanır, dump() çıktıları toplanır, bütün projelerin logları tek yerde canlıdır.
    - title: Adlandırılmış veritabanı snapshot'ları
      href: help/page-project-detail/
      text: Migration öncesi bir tane alın, adıyla geri yükleyin. Zamanlanmış snapshot da var.
    - title: Herkese açık tünel, sekiz sağlayıcı
      href: help/project-tunnel/
      text: Cloudflare, ngrok, Tailscale, zrok, Pinggy, localtunnel, localhost.run, LocalXpose.
    - title: Yedi araçtan içe aktarma
      href: help/settings-workspace-import/
      text: XAMPP, Laragon, MAMP, Valet, Sail, Herd, DDEV — projelerinizi taşır.
    - title: Doktor
      href: help/settings-diagnostics/
      text: Neyin bozuk olduğunu ve nasıl düzeleceğini satır satır söyler; çoğu bulgunun yanında bir düğme vardır.
    - title: Terminal ve TUI de var
      href: guide/cli/
      text: Pencerenin yaptığı her şeyin bir stackvo komutu var; CLI betikler ve CI içindir.
sponsor:
  eyebrow: Sponsorluk
  title: Sponsor olun
  secondary: Ya da depoya yıldız verin
  text: >-
    StackVo ücretsizdir, MIT lisanslıdır ve tek kişi tarafından bakılır;
    arkasında bir şirket ya da ücretli bir katman yoktur. Size zaman
    kazandırıyorsa, bakımının sürmesini sağlayan şey sponsorluktur — "çoğu
    sorun cevaplanır" cümlesini "sorunlar cevaplanır" yapan da.
  label: GitHub'da sponsor olun
  href: https://github.com/sponsors/stackvo
---

StackVo, makinenizdeki yerel geliştirme ortamını yöneten bir masaüstü
uygulamasıdır. [Başlarken](getting-started/index.md) ile devam edin.
