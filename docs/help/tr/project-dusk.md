# Tarayıcı testleri (Dusk)

Laravel Dusk, uygulamanıza karşı gerçek bir tarayıcı sürüyor. Bunun burada çalışması için iki şeyin doğru olması gerekiyor, ve insanların beklediği yalnızca biri.

## Kolay yarısı: konteynerin içinde bir tarayıcı

StackVo bunu ifade etmeyi zaten biliyordu. Bir Sail projesini içe alırken onun `selenium` servisini **adıyla** tanıyor, ve bir projenin `stackvo.json`'ı kendi konteynerlerini tanımlayabiliyor. Bu kart da birini tanımlıyor:

* imaj, Apple Silicon'da **`selenium/standalone-chromium`**, başka yerde `selenium/standalone-chrome`. Bu bir tercih değil: Google arm64 Chrome yayımlamıyor, yani `chrome` imajının arm64 manifesti yok — ve emülasyon altında koşan bir tarayıcı, sebebini kimsenin bulamayacağı bir zaman aşımıdır;
* **etiketli**, asla `latest` değil — etiketsiz bir imaj, onu geçen ay çeken kişinin altından kayar. Etiket sizin `stackvo.json`'ınızda duruyor, yani onu yükseltmek bir StackVo sürümünü beklemek değil sizin düzenlemeniz;
* **host portu yok**, çünkü tanımlı bir konteynerin hiç olmaz. Bu projenin ağının dışından ona ulaşması gereken bir şey yok, ve bir deponun iki klonu aksi hâlde 4444 için kavga ederdi.

Yanına `.env.dusk.local` yazılıyor. Dusk o dosyayı bir koşu boyunca `.env` yerine yüklüyor; bu da onu, yalnızca konteyner ayaktayken bir anlamı olan bir sürücü adresi için doğru yer yapıyor:

```
APP_URL=https://<alan adınız>
DUSK_DRIVER_URL=http://stackvo-<proje>-chromium:4444/wd/hub
```

**Asla üzerine yazılmıyor.** Bir test koşusu boyunca `.env`'inizin yerini alan bir dosyanın içeriğini sizin seçmiş olmanız gerekir; o yüzden dosya zaten varsa bu kart size ne *yazacak olduğunu* gösterip sizinkine dokunmuyor.

Projeniz `local` dışında bir ortam kullanıyorsa dosyayı ona göre yeniden adlandırın — Dusk `.env.dusk.<ortam>` arıyor ve ad tutmuyorsa hiçbir şey yüklemiyor.

## Zor yarısı: sertifika

Tarayıcının `https://<alan adınız>` açması gerekiyor. Tarayıcı bir konteynerin içinde ve o konteyner, StackVo'nun **bu makineye** kurduğu sertifika otoritesini hiç duymamış.

Yani test bir sertifika uyarısında düşüyor — ve bir test çerçevesinin sürdüğü tarayıcının içindeki bir sertifika uyarısı, sertifika uyarısı gibi görünmüyor. Sayfanız yüklenmemiş gibi görünüyor, ve siz kendi kodunuza bakmaya gidiyorsunuz.

Güven düğmesi CA'yı **iki** yere koyuyor, ve iki olmalarının sebebi ayrı ayrı başarısız olmaları:

| Adım | Onu kim okuyor |
| --- | --- |
| Sistem paketi (`update-ca-certificates`) | `curl`, JVM, OpenSSL kullanan her şey |
| Chromium'un NSS veritabanı (`certutil`) | **Chromium'un kendisi** — testinizin geçip geçmeyeceğine karar veren bu |

İkincisi her imajda olmayan bir araca ihtiyaç duyuyor, o yüzden "güven başarısız" içine katlanmak yerine kendi başına bildiriliyor. `certutil: not found` diyen bir adım, üzerine hareket edebileceğiniz bir cümledir.

İkisi de o konteynerin içinde root olarak koşuyor, çünkü imaj `seluser` olarak çalışıyor ve iki konum da onun yazabileceği yerler değil. Bu, kendi projenizin tanımladığı bir konteynere, kendi makinenizde bir `docker exec`.

**Konteyner yeniden oluşturulduğunda tekrar koşulması gerekiyor.** Konteynerin yazılabilir katmanına yazıyor; bu, yaklaşımın işleyişi, bir kusuru değil — ve cümlenin bir dipnotta değil düğmenin yanında olmasının sebebi de bu.

## Veritabanı

Dusk **gerçek bir veritabanına** vuruyor. Testin sonunda geri alınan bir işlem değil; birim testlerini seve seve yerelde koşan insanların Dusk'ı koşmamasının sebebi de bu.

StackVo'nun buna kendi cevabı bu sayfada zaten var: bir **worktree**, bir dala kendi veritabanını, kendi ana bilgisayar adını ve kendi ortamını veriyor. Bu kart bunu söylüyor ve orada duruyor — test paketinizi sorulmadan başka bir veritabanına taşıyan bir kart, sizin adınıza bir karar almış olurdu.

## Bunun yapmadığı şey

Testlerinizi koşmuyor. `stackvo artisan dusk`'ın koşabileceği bir ortam kuruyor.

Ve `DuskTestCase`'inize dokunmuyor. İçindeki sürücü, pencere boyutu ve Chrome bayrakları sizin kodunuz. Chrome konteynerin içinde paylaşılan bellek tükettiğinde — klasik `session not created` çökmesi — çözüm o dosyanın Chrome seçeneklerindeki `--disable-dev-shm-usage`. StackVo bunu sizin için ayarlayamaz: tanımlı bir sidecar'ın `shm_size` seçeneği yok, ve bunu düzeltiyormuş gibi yapan bir düğme hiçbir şeyi düzeltmeyen bir düğme olurdu.
