# Telescope, Horizon ve Pulse

Üçü de bir web panosuyla geliyor. Üçü de `local` ortamında kimlik doğrulaması olmadan açılıyor. StackVo bu projeyi zaten kendi alan adı altında, tarayıcınızın güvendiği bir sertifikayla servis ediyor.

Yani `https://shop.loc/horizon` baştan beri çalışıyordu. Hiçbir yerde bunu söyleyen bir şey yoktu; o yüzden kimse tıklamadı.

Bir bağlantı bu kartın ucuz yarısı olurdu, ve işe yarayan yarısı değil.

## Üçü de sessizce boş kalıyor, ve her konteyner yeşil duruyor

| Pano | İçinde hiçbir şey olmadan öylece durmasının sebebi |
| --- | --- |
| **Horizon** | Kuyruk bağlantısının `redis` olması gerekiyor. Ve ölçüm grafikleri, `horizon:snapshot` **beş dakikada bir** koşana kadar düz kalıyor — anlık görüntüyü yazan başka bir şey yok |
| **Telescope** | `telescope:install` ve `migrate` koşmuş olmalı. Ve günlük bir `telescope:prune` olmadan `telescope_entries` proje açık kaldığı sürece büyüyor — belirtisi yavaş bir pano değil, dolu bir disk |
| **Pulse** | Deposu MySQL, MariaDB ya da PostgreSQL istiyor ve **SQLite'ı reddediyor**. Redis üzerinden topluyorsa *kuyruğunkinden ayrı* bir Redis bağlantısı istiyor. Ve `pulse:check` uzun süren bir süreç |

Sonuncusu, Pulse'un neden zamanlamada değil **İşçiler** kartında göründüğünün sebebi. `horizon:snapshot` ve `telescope:prune` başlar, bir şey yapar ve çıkar — zamanlama tam da bunun içindir. `pulse:check` ise geri dönmeyen bir döngüdür, ve ona bir zamanlama satırı, her tetiklendiğinde ikinci bir kopya başlatırdı. `pulse:work` ona yalnızca `.env`'iniz `PULSE_INGEST=redis` dediğinde katılır; depo üzerinden toplamada aktaracağı bir şey yoktur.

## Bu kartın gerçekte bildiği şey

**StackVo `.env` ve `composer.lock` okuyor. `config/*.php` okumuyor.**

`config:cache` koşmuş bir projenin, bu uygulamanın göremediği derlenmiş bir yapılandırması vardır ve o, iki dosyanın da dediğinden başka bir şey diyor olabilir. O yüzden bu karttaki hiçbir şey bir hüküm değil:

* her satır **okuduğu anahtarı adıyla söylüyor** ve bulduğu değeri aktarıyor;
* önbelleğe alınmış yapılandırmayla ilgili cümle, panelin tepesinde değil, **o satırın yanında** duruyor.

İkincisi bilinçli. Ekranın tepesindeki bir uyarı ile en alttaki bir satır, insanın kendi kafasında birleştirmek zorunda kaldığı iki şeydir; ve bir şeyi ölçmeden "bozuk" diyen bir kontrol, insanların görmezden gelmeyi öğrendiği kontroldür.

**İki şey hiç iddia edilmiyor.** Telescope'un göçlerinin koşup koşmadığı, bunun sormadığı bir veritabanı hakkında bir sorudur — bir önkoşul olarak yazılıyor, bir durum olarak bildirilmiyor. Ve Horizon'un desteklemediği **Redis Cluster** kullanıp kullanmadığınız, `config/database.php`'nin o değerle ne yaptığını tahmin etmeden `.env`'den okunamaz. O yüzden ölçülmüş gibi görünen bir satır yerine burada bir cümle.

## İki zamanlanmış komut

Eksik olan varsa bu kart onu **bu projenin zamanlamasına** eklemeyi öneriyor — Zamanlayıcı kartının gösterdiği tablonun aynısı, kendi günlüğü ve son çalışmasıyla, `stackvo.json`'da saklanarak depoyla birlikte gezen.

Kendi fiiliyle değil, zamanlamanın tek yazıcısı üzerinden giriyor; böylece manifest ile üretilmiş zamanlama birbirinden ayrılamıyor. Zaten orada olan bir iş, etiketiyle değil **çalıştırdığı artisan komutuyla** eşleniyor — yani onu yeniden adlandırmak bu öneriyi ikinci kez getirmiyor.

## Adresler

Her bağlantı o panonun kendi varsayılan yolunu kullanıyor. Panosunu taşımış bir proje onu `config/*.php` içinde taşımıştır — bu uygulamanın okumadığı dosyada — ve bağlantının yanındaki satır bunu söylüyor.

## Scout: servis açık, dizin boş

Meilisearch ve Typesense katalog servisleri, yani birini açmak Market'te bir tık. Söylenmeyen şey **sonraki adım**.

Boş bir Meilisearch her aramaya *hiçbir şey* döndürür. Uygulama bozuk görünür, ve bu olurken **her konteyner yeşildir**. Bu, bu kartın bütün deseninin en saf hâli: parça burada, ve hiç söylenmemiş olan şey onun önkoşulu.

Bu maddeye bir düğme değil bir cümle düşüyor, ve sebebi söylenmeye değer. Komutlar şunlar:

```
php artisan scout:import "App\Models\Post"
php artisan scout:sync-index-settings
```

Birincisi **StackVo'nun bilemeyeceği bir model sınıf adı** alıyor. Tahmin ettiği bir şeyi doldurup çalıştıran bir düğme, komut kataloğunun tam olarak reddettiği şeydir — `migrate:fresh`'i dışarıda tutan kuralın aynısı.

İhtiyacınız olan mekanizma zaten var: bir proje kendi komutlarını `stackvo.json`'ın `commands` bloğunda tanımlıyor ve onlar yerleşiklerin yanında görünüyor. `scout:import` satırınızı oraya koyun.

Bu not yalnızca **iki** koşul da doğruyken görünüyor — `composer.lock`'ta `laravel/scout` *ve* `meilisearch` ya da `typesense` olan bir `SCOUT_DRIVER`. Biri tek başına yanlış cümle olurdu: `database` sürücüsündeki bir Scout'un dolduracak dizini yoktur, ve başka bir şey için çalıştırdığınız bir Meilisearch bu kartı ilgilendirmez.
