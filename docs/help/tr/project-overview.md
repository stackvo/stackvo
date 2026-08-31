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

Bir kilit dosyası olmadan bir **sürümün** yanlış olduğunu söyleyemez. Beyan `redis` diyor ve sürüm sabitlemiyorsa, kurulu herhangi bir Redis onu karşılar ve bulunan sürüm yargılanmak yerine satırın yanına yazılır. Bunu değiştiren şey **stackvo.lock yaz** düğmesidir.

### Laravel yarısı: projenizin istediği PHP

`composer.json`, projenin **platformdan** ne istediğini söyler — `"php": "^8.3"`, ve bir dizi `ext-*` gereksinimi. `stackvo.json` ise imajın ona ne verdiğini söyler. Benimseme anından sonra ikisi hiç karşılaştırılmıyordu, ve bunun ürettiği hata sık ve pahalı:

`composer.json` `^8.3` diyor. `stackvo.json` `8.2` diyor. İmaj sorunsuz derleniyor. Sonra `composer install` **konteynerin içinde** bir platform gereksinimi hatasıyla düşüyor — ve o hata PHP'yi adıyla söylüyor, ama değiştirilmesi gereken dosyayı söylemiyor. Siz bir composer hatasına bakıyorsunuz ve çözüm bir manifest satırı.

O yüzden iki satır daha kontrol ediliyor, ve ikisi de yeni bir şey ölçmüyor:

| Satır | Neyi neye tutuyor |
| --- | --- |
| `composer.json`'ın istediği PHP | kısıtın ilk `major.minor`'ünü, manifestinizdeki `php.version`'a |
| Gerektirdiği her `ext-*` | `php.extensions`'a — eksik uzantı başına bir satır, çünkü onarım uzantı başına ve adın kendisi onarımın tamamı |

**`require-dev` okunmuyor.** Bir geliştirme gereksinimi test paketi için bir araçtır, ve bir projenin hazırlığını onun üzerinden düşürmek, çalışan bir kurulumu bozuk ilan etmek olurdu.

**Okunamayan bir kısıt bir başarısızlık değil, `Bilinmiyor`.** `*` ve çıplak bir `^8` içinde `major.minor` yok, ve StackVo tahmin etmek yerine bunu söylüyor — bu kartın geri kalanına hükmeden kuralın aynısı.

Ve manifestinde `php` bloğu olmayan bir proje, `composer.json`'ı ne derse desin bu satırların hiçbirini almıyor.

Aynı cevap `stackvo verify <proje>` ile de alınır.

## stackvo.lock

`stackvo.json` hangi servisleri söyler; `stackvo.lock` hangi **sürümleri** — ve depoda onun yanında durur. Her ekosistemin vardığı ayrımın aynısı: manifest niyettir, kilit olgudur, ve birincisini tekrar üretilebilir yapan şey ikincisidir.

```json
{
  "lockVersion": 1,
  "at": "2026-08-30T09:14:02Z",
  "services": [
    { "service": "redis", "version": "7.2", "source": "official", "sha256": "9f2c…" }
  ]
}
```

**Bunu bir sürüm listesi değil de kilit yapan şey `sha256`.** Servis kurulurken katalogun beyan ettiği paket manifestosunun digest'i. Aynı sürüm numarası iki kez yayımlanabilir; digest sayesinde başka birinin katalogundan gelen "redis 7.2" ile resmî katalogdan gelen "redis 7.2" farklı iki cevaptır — ki bir sürüm listesinin göremediği ikame tam olarak budur.

Dosya var olduğunda, yukarıdaki kontrol daha önce veremediği üç cevabı kazanır:

| Gördüğünüz | Anlamı |
| --- | --- |
| Farklı sürüm | Kilit 7.2 diyor, bu makine 7.0 çalıştırıyor. İki numara da yazılır, çünkü tek başına biri üzerine iş yapılabilecek bir şey değildir. |
| Farklı paket | Sürüm uyuyor, digest uymuyor. Kilidin adlandırdığı katalogdan yeniden kurun, ya da artık referans bu makineyse yeniden kilitleyin. |
| Artık beyan edilmiyor | Kilit, `stackvo.json`'un düşürdüğü bir servisi adlandırıyor. Kurulacak bir şey yok — yeniden kilitleyin. |

### Yalnız siz bastığınızda yazılır

Bu dosyayı kendi başına hiçbir şey tazelemez, ve bu kasıtlıdır. Uygulamanın sessizce güncellediği bir kilit, makinenin sürüklendiği yeri kaydederdi — yani makineyle her zaman aynı fikirde olur, onunla asla anlaşmazlığa düşemezdi. Başarısız olamayan bir kontrol, hiç kontrol olmamasından kötüdür.

### Neyi kilitlemez

- **Çalışma zamanı ve web sunucusu.** `stackvo.json` zaten `php.version` ve `server` taşıyor. Bir olgunun ikinci kopyası, iki kopyanın ayrışmasının yoludur.
- **StackVo'nun kendi çektiği imgeler** — tünel koşucuları, karşılama sayfası. Onlar makineye aittir, herhangi bir projeye değil: tek bir `cloudflared` on projeye hizmet eder. Kendi sabitlemeleri var: bir yöneticinin politika dosyasında, `imagePins` altında.
- **Kurulu olmayan ya da kurulu olup kapalı olan hiçbir şey.** Uydurma bir girdi yazmak yerine hangisini ve niçin atladığını söyler — beş servisinizin üçünü sessizce kapsayan bir kilit, beşini kapsadığına inandığınız kilittir.

`stackvo lock <proje>` ile de çalışır; CI betiğine girecek biçim de budur.
