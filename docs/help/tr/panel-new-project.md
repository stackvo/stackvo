# Yeni proje

Bir projeyi üç yoldan biriyle oluşturur: boş bir iskeletle, bir çatının kendi kurulum aracıyla, ya da var olan bir git deposundan.

## Üç yol

| Başlangıç | Ne yapar | Ne zaman |
| --- | --- | --- |
| Boş proje | Formdaki değerlerle sıfırdan bir proje kurar. Hiçbir kurulum aracı çalışmaz. | Kodu kendiniz koyacaksınız, ya da elinizde bir iskelet var. |
| Çatı şablonu | Çatının kendi kurucusu geçici bir konteynerde çalışır, sonra sonuç sahiplenilir. | Laravel, WordPress, Next.js gibi bir çatıyla sıfırdan başlıyorsunuz. |
| Git deposundan çek | Depoyu klonlar ve gelen dosyaları sahiplenir. | Kod zaten var. |

## Boş proje

Formdaki her alanı siz doldurursunuz.

| Alan | Ne için |
| --- | --- |
| Proje adı | Küçük harf; harf veya rakamla başlar, tire, alt çizgi ve nokta kullanılabilir. Örnek: `api.myapp`. |
| Alan adı | Projenin açılacağı adres. Boş bırakılırsa proje adından üretilir. |
| Ek alan adları | Projenin cevap vereceği diğer adlar. `stackvo.json`'a yazılır, yani klonlayan bir iş arkadaşınız da alır. |
| Çalışma ortamı | PHP, Node ya da katalogdaki diğer çalışma zamanları. |
| Sürüm | Seçilen çalışma ortamının sürümü. |

### PHP seçilirse

| Alan | Ne için |
| --- | --- |
| Web sunucusu | Projeyi sunacak sunucu. |
| Doküman kökü | Web sunucusunun yayımlayacağı alt klasör. Laravel'de `public`, WordPress'te proje kökü. |
| PHP eklentileri | Konteynere derlenecek eklentiler. Seçilen PHP sürümüyle kurulamayan bir eklenti işaretlenir. |

### Node ya da başka bir çalışma ortamı seçilirse

| Alan | Ne için |
| --- | --- |
| Paket yöneticisi | İmajda Corepack'i etkinleştirir; `package.json` içindeki `packageManager` alanının bir sürümü sabitlemesini bu sağlar. Sabitlemeden bırakmak imajı eskisiyle birebir aynı kurar. |
| Kurulum komutu | Bağımlılıkları kuran komut. |
| Derleme komutu | İsteğe bağlı. Boş bırakılabilir. |
| Başlatma komutu | Uygulamayı çalıştıran komut. |
| Port | Uygulamanın konteyner içinde dinlediği port. |

Uygulamanız `0.0.0.0` adresine bağlanmalıdır. Yalnızca `127.0.0.1` dinleyen bir sunucuya Traefik erişemez ve adres 502 döner.

## Çatı şablonu

Şablonlar çalışma ortamına göre gruplanmıştır: PHP, JavaScript, CMS ve e-ticaret, Python, Go, Ruby ve Rust. Grup başlığı, seçimin ima ettiği çalışma ortamıdır — Nuxt seçmek Node seçmektir.

Süreç şöyle işler:

1. Çatının kendi kurulum aracı geçici bir konteynerde çalışır. Yani `composer create-project` ya da `npx create-next-app` gerçekten çalışır; StackVo bir dosya kopyalamaz.
2. Kurucu bittiğinde sonuç sahiplenilir.
3. Çalışma ortamı, web sunucusu ve doküman kökü, kurucunun **gerçekte yazdığı** dosyalardan tespit edilir. Laravel `public/` üzerinden servis eder, WordPress proje kökünden; bu fark tahmin edilmez, okunur.

Bu yüzden şablon seçildiğinde formdaki çalışma ortamı alanları gizlenir: söyleyecekleri bir şey yoktur, cevabı kurucu verir.

İlk çalıştırma kurucu imajını indirir. Birkaç dakika sürebilir.

Tespit edilen değerler sonradan proje ayarlarından değiştirilebilir.

## Git deposundan çek

| Alan | Ne için |
| --- | --- |
| Depo adresi | SSH ya da HTTPS klon adresi. Herhangi bir sunucu olabilir; kendi GitLab kurulumunuz da dâhil. |

Klonlama **bilgisayarınızdaki git** ile yapılır. Anahtarınız, `ssh` yapılandırmanız ve sunucu izinleriniz kendi kurulumunuzdan okunur; StackVo bunların hiçbirini yönetmez. Terminalinizde çalışan bir adres burada da çalışır.

Klonlandıktan sonra:

- Depoda `stackvo.json` varsa ayarları olduğu gibi kullanılır. Takımın verdiği cevap sizindir; formdaki alanlar yok sayılır.
- Yoksa proje, gelen dosyalardan tespit edilerek yapılandırılır.

## Oluşturduktan sonra ne olur

1. Proje klasörü hazırlanır.
2. `stackvo.json` yazılır.
3. Yapılandırma üretilir: Dockerfile, compose dosyaları ve yönlendirme etiketleri.
4. İmaj derlenir ve konteyner başlatılır.
5. Alan adı hosts dosyasına yazılır ve sertifika kapsamına alınır.

## Bilinmesi gerekenler

- Alan adı, çalışma alanının soneğinin dışında kalırsa joker sertifika onu kapsamaz. Panel bunu söyler; projeyi oluşturduktan sonra sertifikaları yeniden üretin.
- Sonek `.dev` gibi tarayıcıların HSTS listesindeki bir uzantıysa adres yalnızca HTTPS ile açılır ve uyarı geçilemez. Önce Ayarlar'dan HTTPS'i açın.
- Joker bir ek alan adı sertifikaya ve yönlendiriciye ulaşır ama hiçbir hosts dosyası joker ifade edemez. O adlar siz elle eklemedikçe ya da Yerel DNS açık olmadıkça çözülmez.
- Üretici bulunmayan çalışma ortamları listede gizlenir; panel hangilerinin gizlendiğini yazar.
- Proje adı sonradan değiştirilemez. Alan adı değiştirilebilir.
