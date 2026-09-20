# Yenilikler

Her sürüm, uygulamayı kullanan kişinin gördüğü hâliyle.
[Değişiklik günlüğü](changelog.md) her değişikliği gerekçesiyle taşır; bu
sayfa yapabildiklerinizi değiştirenleri.

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
