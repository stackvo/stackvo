# Sözlük

Bu sayfaların sabit anlamla kullandığı kelimeler. Pencere kelimeyi
gösteriyorsa aynı kelimedir.

| Terim | Anlamı |
| --- | --- |
| **Çalışma alanı** | StackVo'nun her şeyi tuttuğu tek klasör; varsayılan `~/.stackvo`, `STACKVO_ROOT` taşır. Projeler, `.env`, sertifika ve üretilen dosyalar içindedir. Klasör durumun kendisidir; veritabanı yoktur. |
| **Proje** | `<çalışma alanı>/projects/` altında, kodun yanında bir `stackvo.json` olan bir klasör. Her proje tek konteyner, tek alan adı ve tek çalışma zamanı sürümüdür. |
| **Manifest** | `stackvo.json`: elle düzenlediğiniz tek dosya. Çalışma zamanını ve sürümünü, web sunucusunu, belge kökünü, projenin ihtiyaç duyduğu servisleri, alan adını ve takma adlarını, hook'larını bildirir. |
| **Yığın (stack)** | Çalışma alanının birlikte çalıştırdığı her şey: ters proxy, sertifika otoritesi, paylaşılan servisler ve her projenin konteyneri. `stackvo up` ve `stackvo down` bütününe etki eder. |
| **Servis** | Projelerin kullandığı paylaşılan bir konteyner: veritabanı, önbellek, mail yakalayıcı, yönetim arayüzü. Servisler projede değil, çalışma alanının `.env` dosyasında yaşar. |
| **Örnek (instance)** | Katalogdaki bir servisin kendi portu ve kimlik bilgileriyle çalışan bir kopyası. İki MySQL örneği iki sürüm olabilir. |
| **Katalog** | Kurulabilen servislerin listesi; uygulamaya derlenmek yerine bir kayıt defterinden imzalı paketler olarak çekilir. Kenar çubuğundaki **Katalog**. |
| **Paket** | Katalogdaki bir kayıt: bir servisin compose parçası, sabitlenmiş özetle imajı ve getirdiği komutlar. Açılmadan önce doğrulanır. |
| **Üretilen dosyalar** | `<çalışma alanı>/generated/`: manifestlerden üretilen compose dosyaları, Dockerfile'lar ve sunucu yapılandırmaları. Elle düzenlenmez, silmek güvenlidir; `stackvo generate` yeniden yazar. |
| **`.env`** | Çalışma alanının ayar dosyası: alan adı eki, hangi servislerin açık olduğu, portlar, sunucu ayarları. Ayarlar sayfası yazar; çoğu değişiklik yeniden üretme ister. |
| **Alan adı eki** | Her ana makine adının kurulduğu son: varsayılan `.loc`; `shop` adlı proje `shop.loc` adresinde cevap verir. `.env` içinde `DEFAULT_TLD_SUFFIX`. |
| **Hosts dosyası** | İşletim sisteminin `/etc/hosts` dosyası (Windows'ta karşılığı). StackVo proje adlarını, farkı size gösterdikten sonra, işaretli bir blok içine yazar; tarayıcı `shop.loc` adresini böyle bulur. |
| **CA** | StackVo'nun makine başına bir kez oluşturduğu ve bir kez güvenmenizi istediği sertifika otoritesi. Her projeyi ve servisi kapsayan tek bir joker sertifika imzalar; tarayıcı uyarısı olmamasının nedeni budur. |
| **Doktor** | Makineyi denetleyen ayar bölümü: Docker, soket, portlar, hosts dosyası, sertifika, disk. Her bulgunun yanında onarımı yazar. Terminalde `stackvo doctor`. |
| **Snapshot** | Bir projenin veritabanının adlandırılmış kopyası; migration'dan önce alınır, adıyla geri yüklenir. Dışa aktarıp sakladığınız bir dosya olan **dump**'tan farklıdır. |
| **Worktree** | Kendi ortamı olan bir git worktree'si: aynı projenin başka bir daldaki ikinci checkout'u; kendi konteyneri, alan adı ve isterseniz kendi veritabanıyla. Kılavuzda *Dal başına ortam*. |
| **Tünel** | **Paylaş** kartındaki sağlayıcılardan biri üzerinden proje için herkese açık bir URL; ağınızın dışındaki biri açabilsin diye. |
| **Hook** | Projenin sizin seçtiğiniz anda çalıştırdığı bir komut: başlatmadan sonra, durdurmadan önce. Manifestte bildirilir. |
| **Şablon** | Proje oluştururken bir çerçevenin kendi kurucusunun geçici bir konteynerde çalıştırılması; Laravel ya da Next.js projesi çerçevesinin öngördüğü gibi başlar. |
| **Kilit dosyası** | `stackvo.lock`: projenin gerçekten derlendiği servis sürümleri ve paket özetleri; klonlayan aynı yığını alır. |
| **Önayar (preset)** | Bir çalışma alanının yığın ayarlarının `stackvo.preset.json` olarak dışa aktarılmış hâli; başka makinede içe aktarılır. |
| **Çalışma zamanı (runtime)** | Projenin çalıştığı dil ve sürümü: PHP, Node, Python, Go, Ruby, Rust, Bun ya da Deno. Proje başına bir tane; monorepo birkaç parçalı tek projedir. |
| **Profil** | Birlikte başlayan konteyner grubu. Projeler ve servisler ayrı profillerdir; yığını indirmek ikisini de alır. |
| **MCP** | Model Context Protocol. `stackvo-mcp`, bir yapay zekâ asistanının bağlandığı sunucudur; araçları pencerenin ve CLI'ın kullandığı komutların aynısıdır. |
| **TUI** | `stackvo tui`: pencerenin genel görünümü terminalde; SSH ile ulaştığınız makine için. |
