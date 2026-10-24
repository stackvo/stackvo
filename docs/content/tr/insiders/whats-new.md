# Yenilikler

Her sürüm, uygulamayı kullanan kişinin gördüğü hâliyle.
[Değişiklik günlüğü](changelog.md) her değişikliği gerekçesiyle taşır; bu
sayfa yapabildiklerinizi değiştirenleri.

## 0.3.3 — 24 Ekim 2026

Bakım sürümü: aşağıdaki düzeltmeler ve on yayımlanmış güvenlik duyurusunu
geride bırakan TLS, arşiv ve eşzamansız çalışma kütüphaneleri.

**macOS**

- **Dock simgesi yerini dolduruyor.** macOS 26 onu gri bir karonun üstünde
  küçük çiziyordu; artık komşuları gibi Apple'ın simge biçiminde.
- **Uygulama Finder, Dock ya da Spotlight'tan açıldığında Docker Compose
  bulunuyor.** Bu açılışlar `/usr/local/bin` ve `/opt/homebrew/bin`'i
  görmez; her `.dmg` kurulumu gereksinim kapısında "Docker Desktop'ı
  güncelleyin" diyerek duruyordu, güncellenecek bir şey de yoktu. Uygulama
  bu dizinleri artık kendisi ekliyor; aynı düzeltme `git` ve `mkcert`'e de
  ulaşıyor.

**Servisler**

- **Başlat, yarım kalmış bir etkinleştirmeyi tamamlıyor.** İmaj çekilirken
  kesilen bir etkinleştirme servisi konteynersiz "açık" bırakıyor, Başlat da
  yalnızca "konteyner bulunamadı" diyebiliyordu. Başlat artık eksik olanı
  oluşturuyor; başarısız bir etkinleştirme kendini geri alıp Etkinleştir'i
  yeniden sunuyor.

**Güncellemeler**

- **Yeni bir sürüm açılışta bir kez duyurulur**; pencerenin altındaki satır
  Güncellemeler sayfasına götürür. Şimdiye dek yalnızca Ayarlar'ı açmak
  soruyordu; panelde kalan bir kurulum hiç haber almıyordu.

**Erişilebilirlik**

- **Komut paleti düğmesi tuşlarını söylüyor.** Erişilebilir adı artık
  üstünde yazan `⌘K` ya da `Ctrl+K`'yi içeriyor; sesli denetim okuduğunuz
  sözcükleri anlıyor.

## 0.3.2 — 12 Ekim 2026

**Projeler**

- **Goravel, Masonite, AdonisJS ve Loco** yeni proje panelinde; her çalışma
  zamanı için bir uygulama iskeleti, her birinin kendi belgelerinin dediği
  biçimde kurulur.
- **Her şablon çatının en yeni sürümünü kurar.** Gin ve Echo artık sürüm
  sabitlemiyor; en yenisi `composer.lock`'ta olduğu gibi `go.mod` ve
  `go.sum`'a iniyor. Kurulu `composer.json`'ın istediğinin altında seçilen
  bir PHP sürümü reddedilmek yerine o tabana yükseltiliyor.
- **Go 1.24'ten 1.27'ye**, varsayılan 1.27; **Node 24 ve 25**, varsayılan 24.

**Düzeltilenler**

- **Tümünü başlat, tümünü durdur ve tümünü yeniden başlat hiçbir şey
  yapmıyordu.** Artık çalışıyor; toplu eylemde başarısız olan konteynerler
  sessizce atlanmak yerine raporlanıyor.
- **Geçici bir PECL ya da NodeSource kesintisi tüm derlemeyi
  düşürmüyor.** Her kurulum yeniden deneniyor; hata "docker exited with
  code 1" yerine başarısız adımı adıyla söylüyor.
- **Mail sayfası yakalayıcıyı yeniden etkinleştirebiliyor**; Mailpit ya da
  MailHog, hangisi kuruluysa.

## 0.3.1 — 6 Ekim 2026

**Servisler**

- **Katalogda 28 servis**, uygulamayla gönderilmek yerine paket deposundan
  yayımlanıyor: Qdrant, Neo4j, Memgraph, Nessie, OpenSearch ve OpenSearch
  Dashboards, OpenTelemetry Collector, Loki, Tempo, Karapace, Apicurio
  Registry, Redpanda Connect, Flink, Spark, Temporal ve arayüzü, Zitadel,
  Unleash, Vault, OpenBao, Ollama, MLflow, Label Studio, Argilla, GreenMail,
  Icecast, Postal ve Backstage.

**Düzeltilenler**

- **"Çek ve kullan" doğru imzalanmış bir kataloğu reddedip duruyordu**:
  "imzalı, ama bu makinenin güvendiği anahtarların hiçbiriyle değil".
  Reddedilen bir dizin artık hatırlanmıyor; takılı kalmış bir makine bir
  sonraki yenilemede kurtuluyor.

## 0.3.0 — 22 Eylül 2026

**Projeler**

- **Proje, supervisor'ının çalıştırdığı süreçleri bildiriyor.** Konteynerin
  içine eklenen bir kuyruk işçisi bir sonraki yeniden derlemede yok oluyordu.
  `stackvo.json`'daki `processes` bloğu — bir id, bir argv, isteğe bağlı
  `replicas` — konteynerin supervisor yapılandırmasına yazılır; yeniden
  derleme işçiyi geri getirir, klonlayan da alır. Supervisor paneli aynı
  yapılandırmayı çalışan konteynere itebilir; yeni süreç yeniden derleme
  olmadan hemen başlar.
- **Denetlenen süreç, tamamıyla.** Supervisor satırındaki bilgi düğmesi onu
  açar: oraya kimin koyduğu, yapılandırma bloğu, bellek, iş parçacığı ve
  CPU, takip eden bir log ve durdur, başlat, yeniden başlat ile bir sinyal.
  Panel başlığı daemon'ın kendi denetimlerini taşır; web sunucusunu da
  düşüren ikisinden önce onay ister.
- **Site gibi adlandırılmış bir klasör o site olarak benimsenir.**
  `api.showtv`, `api.showtv.stackvo.loc` değil `api.showtv.loc` olur; joker
  sertifikanın asla ulaşamayacağı dördüncü bir etiket yok.

**Güncellemeler**

- **Beta kanalı.** Ön sürüm kararlı bir kuruluma hiçbir yoldan ulaşmaz;
  betaları **Ayarlar → Hakkında → Beta sürümleri de al** altından açın, beta
  kurulumu kararlı sürümleri de almaya devam eder.
- **Her sürümde tek `SHA256SUMS.txt`**, kurulum paketlerinin yanında; sürüm
  notlarının 0.2.0'dan beri söz verdiği gibi.

**Yardım**

- **Yardım belgeleri her zaman çekilir, asla paketlenmez**; düzeltilen bir
  cümle yeni kurulum paketi olmadan size ulaşır. Çevrimdışıyken panel
  "henüz yazılmadı" değil çevrimdışı olduğunuzu söyler.

**Belgeler**

- **Bir belge sitesi**, [stackvo.github.io/stackvo](https://stackvo.github.io/stackvo/),
  İngilizce ve Türkçe: başlangıç, görev başına bir kılavuz, sade dille
  referans ve bu sayfa.

**Güvenlik**

- **rustls 0.23.45**, CVE-2025-61730 için, duyurunun yayımlandığı gün.
- **Katalog imza anahtarı döndürüldü**; yayımlanan dizin yeni anahtarla
  yeniden imzalandı.

## 0.2.0 — 2 Eylül 2026

İlk yayımlanan sürüm: on kurulum paketi, imzasız, `SHA256SUMS` ve
güncelleyicinin denetlediği bir minisign imzasıyla doğrulanır.

**Projeler**

- **Monorepo tek projedir.** `api/` Go, `web/` Next.js ve `worker/` Python
  olan bir depo tek kayıt, tek başlatma, tek sertifika ve tek alan adı
  kümesidir.
- **`stackvo.lock`**, projenin gerçekten derlendiği servis sürümlerini ve
  paket özetlerini kaydeder; klonlayan aynı yığını alır.
- **Doğru ortamla `git bisect`.** Her adım commit'i, projenin o zaman
  bildirdiği PHP ve servis sürümleriyle çalıştırır; bugün makinede
  olanlarla değil.
- **Çalışma alanının kökündeki bir dosya** içindeki her projeye komut ekler;
  bir paket kendi komutlarını getirebilir, Redis kurmak `redis-cli` verir.
- **RoadRunner**, Swoole'un yanına Laravel Octane sürücüsü olarak geldi.

**Hata ayıklama**

- **Bu istek neden yavaştı**, profili, sorgu kaydını ve zaman çizelgesini
  tek isteğin etrafında tek ekrana koyar.
- **Başarısız olan isteği yeniden oynatma**, gövde, başlıklar ve oturum
  dâhil; yeniden oynatma bir veritabanı snapshot'ına bağlanabilir, böylece
  bir POST'a iki kez basmak güvenlidir.
- **Konteynerin içinde editör:** imajda çalışan VS Code; dil sunucusu,
  `composer` ve `artisan` orada, ana makinede PHP yok.

**Paylaşım**

- **Paylaşılan tünel parola isteyebilir** ve sağlayıcı izin verdiğinde
  adresini başlatmalar arasında koruyabilir.

**Makine**

- **Podman tanınıyor**, önce rootless.
- **Bu makineden ne çıkabilir**, hangi konteynerlerin internete
  ulaşabildiğini ve hangi çekmelerin kuruluşun aynasını atladığını listeler.
- **Docker'ın maliyeti ölçülüyor**, tartışılmıyor; *benim makinemde
  çalışıyor* iki makinenin gerçekte çalıştırdığı karşılaştırılarak
  cevaplanıyor.
- **Dört yığın eylemi de önce soruyor**, çünkü tümünü başlat ve tümünü durdur
  makinedeki her konteynere etki eder.
- **Uygulama çöktüğünü söylüyor**; bir sonraki açılışta bir kez, raporu açan
  bir düğmeyle.
- **Bir kerelik tanıtım**, kurulum ekranlarından sonra; dört kapı ve hiç
  karşılama yerine.

**Düzeltilenler**

- Türkçe pencereye basılan kırk yedi İngilizce cümle.
- Apache için yapılandırılmış bir DDEV projesi nginx olarak içe aktarılıyordu.
- Manifest editörü yanlış alan adlarıyla kaydediyor ve projenin belge kökünü
  değiştirebiliyordu.

**Windows**

- Test takımı Windows'ta çalışıyor; bulduğu iki ürün hatası düzeltildi.
