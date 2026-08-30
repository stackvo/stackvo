# Yapılandırma

Bu projenin `stackvo.json` dosyasında yazan hâli. Alanlar salt okunurdur; değiştirmek için karttaki **Yapılandır** butonunu kullanın.

## Alanlar

| Alan | Anlamı |
| --- | --- |
| Alan adı | Projenin tarayıcıda açıldığı ad. |
| Takma adlar | Aynı projeye giden ek adlar. `*.` ile başlayan bir takma ad joker'dır: sertifikaya ve yönlendiriciye girer ama hosts dosyasına giremez, o yüzden tek başına çözülmez. |
| PHP / Node sürümü | Konteynerin çalıştırdığı sürüm. |
| Konteyner yolu | Kodunuzun konteyner içindeki yeri. Her zaman `/var/www/html`. |
| Erişim URL · HTTP / HTTPS | Projenin yanıt verdiği adresler. |
| SSL durumu | Sertifikanın verilip verilmediği. |
| Sunucu | nginx, Apache ya da Swoole. |
| Host yolu | Projenin bu makinedeki klasörü. |
| Tür | Projenin şablonu. |
| Doküman kökü | Web sunucusunun yayımladığı alt klasör. Laravel'de `public`. |

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Yapılandır | Proje ayarları panelini açar. Buradaki alanların çoğu oradan değişir. |
| Kopyala | Değeri panoya alır. |
| Adrese tıklamak | Adresi tarayıcıda açar. |

## PHP eklentileri

Konteynerde derli olan eklentilerin listesi. Eklenti eklemek imajı değiştirir, yani projeyi yeniden derlemek gerekir.

## Sorun bölümü

`stackvo.json` sözleşmeye uymuyorsa buraya yazılır. Hata kodu, dosyadaki yol ve açıklama gösterilir. Uyarılar projeyi çalıştırmayı engellemez; hatalar engeller.

## Bilinmesi gerekenler

- Alan adı çözülmüyorsa bu kart bir uyarı ve hosts kaydını ekleyecek bir buton gösterir.
- Buradaki değerleri değiştirmek çoğu zaman yeniden derleme ister. Yeniden başlatmak yetmez.

## Bu makine uyuyor mu?

Depo, projenin neye ihtiyacı olduğunu beyan eder — servisleri, alan adı, manifesti — ve **Kurulumumu kontrol et**, bu makinede onların olup olmadığını satır satır cevaplar. Bu, işe başlamanın diğer yarısıdır: bu kategorideki her araç *kurmanıza* yardım eder, ve hiçbiri klonlamadan bir saat sonra gerçekten sorduğunuz soruyu cevaplamaz — *"kurdum; peki neden hâlâ çalışmıyor?"*

Yeni hiçbir şey ölçülmez. Beş olgunun dördü proje listesinin zaten hesapladıklarıdır — manifest doğrulamadan geçiyor mu, imaj burada hiç derlendi mi, üretilmiş ağaç `stackvo.json`'dan eski mi, alan adı hosts dosyasında mı — beşincisi de servis tablosudur.

Beyan edilmiş bir servis üç şekilde başarısız olabilir ve bunlar üç ayrı cümledir:

| Gördüğünüz | Anlamı |
| --- | --- |
| Eksik | Servis katalogda var ama burada kurulu değil. Market'ten kurun. |
| Farklı | Kurulu ama **kapalı** — ve sahip olduğunuz sürümler sağda yazar, çünkü "kur" yanlış talimat olurdu. |
| Bilinmiyor | Bu yapı o adı hiç duymadı. Ya yazım hatası ya da yayımlanmış katalog bu uygulamadan yeni. |

**Bilinmiyor, projeyi düşürmez.** Uygulamanın yapmaktan kaçındığı bir kontrol, bir şeyin bozuk olduğunun kanıtı değildir; sormadığı bir soru için "hazır değil" diyen bir doğrulayıcı, insanların görmezden gelmeyi öğrendiği doğrulayıcıdır.

Geçenler dahil her satır gösterilir. Yalnızca bir şey bozukken beliren bir sonuç, "kontrol etti ve iyiyim" ile "kontrol etmedi"yi ayırt edilemez kılardı.

Bir **sürümün** yanlış olduğunu henüz söyleyemez. Beyan `redis` diyor ve sürüm sabitlemiyorsa, kurulu herhangi bir Redis onu karşılar ve bulunan sürüm yargılanmak yerine satırın yanına yazılır — hangisinin olması gerektiğini söylemek bir kilit dosyası ister.

Aynı cevap `stackvo verify <proje>` ile de alınır.
