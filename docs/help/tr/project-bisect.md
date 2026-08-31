# Bunu yapan commit'i bul

`git bisect` her adımda aralığı ikiye böler: davranışın olduğu bir revizyonla olmadığı bir revizyonu söylersiniz ve yaklaşık log₂(n) cevap sonra onu getiren commit elinizdedir.

## Git'in bilemeyeceği yarı

`git bisect` **kodu** taşır, başka hiçbir şeyi değil.

Üç ay önce bu proje PHP 8.3 beyan ediyordu ve `redis`'i 7.0'da kilitlemişti. Bugün bu makinedeki konteyner 8.4 ve 7.2. Yani o aralıktaki her adım **eski kodu yeni bir ortamda** çalıştırıyor — ve aramanın sonunda suçladığı commit masum olabilir, çünkü davranış diff'le değil çalışma zamanıyla değişmiştir.

Bu kategorideki başka hiçbir araç bu konuda bir şey yapmıyor, çünkü hiçbiri bir commit'in hangi ortamı istediğini bilmiyor. Burası biliyor: `stackvo.json` hep depoyla birlikte geldi, ve `stackvo.lock`'tan beri servis sürümleri de öyle. İkisi de **test edilen revizyonda** okunuyor, çalışma kopyanıza dokunmadan, ve ayrışan ne varsa commit'in altında listeleniyor.

"Bu makine, bu commit'in beklediğiyle uyuşuyor" boş bir alan değil, bir sonuçtur: ortam sizin bisect'inizin içinde değil demektir — yani aramanın suçladığı şey koddur.

## Sizin yerinize hiçbir şey değiştirilmiyor

Ortamı da beraberinde getiren bir düğme yok, ve bu kasıtlı. Eski bir servis sürümüne uymak, veri biriminizi taşıyan bir konteyneri değiştirmek demek — ve on adımlık bir bisect bunu yirmi kez yapardı. Bir diff hakkındaki soruyu cevaplamak için bir veritabanını yok etmek, bu uygulamanın sizin adınıza yapacağı bir takas değil.

Liste bir cümledir; üzerine iş yapmak sizin kararınız. Verileceği yer Market sayfası, ve orası önce sorar.

## Üç düğme, iki değil

| Düğme | Ne zaman |
| --- | --- |
| **Burada bozuk** | Aradığınız davranış burada var. |
| **Burada çalışıyor** | Yok. |
| **Bunu test edemiyorum** | Bu commit derlenmiyor ya da özellik henüz yok. |

Üçüncüsü git'in kendi `skip`'i ve göründüğünden önemli. O olmadan, derlenmeyen bir commit ilerlemek için *burada çalışıyor* diye işaretlenir — ki bu, aramayı aşağı akışta hiçbir şeyin fark edemeyeceği bir biçimde zehirler.

## Neler reddediliyor, ve niçin

- **Commit'lenmemiş değişiklikler.** Bir bisect, çalışma kopyanızı başkalarının commit'lerinde gezdirir. Önce commit'leyin ya da stash'leyin. Bu, hiçbir şey kıpırdamadan önce, adıyla reddediliyor — bir git hatasının size terminal için yazılmış bir cümle olarak ulaşmasındansa.
- **Revizyon olmayan bir şey.** Yazdığınız şey `git`'e argüman olarak ulaşır, ve `-` ile başlayan bir değeri git bir **seçenek** olarak okur — üstelik git'in program adı alan birkaç seçeneği vardır. Yalnız git'in kendi revizyon alfabesi kabul ediliyor: `main`, `v1.2.3`, `HEAD~5`, `origin/main`, `abc1234`.

## Bilinmesi gerekenler

- **Durdur ve çalışma kopyamı geri koy**, `git bisect reset` çalıştırır ve sizi başladığınız dala döndürür. Cevap bulunduktan sonra da açık kalır, çünkü o ekran da diğer her adım gibi ayrık bir HEAD'dir.
- Bir bisect başlatmak ve bir adımı işaretlemek denetim kaydına yazılır. "Dosyalarım eskisi gibi değil", bir geliştiricinin kendi makinesi hakkında sorabileceği en gürültülü sorudur ve cevabının bir yerde olması gerekir.
- Adım tahmini git'indir; burada yeniden hesaplanmaz, geri okunur — geçmişin şekline, atlamalara ve birleştirme ebeveynlerine bağlıdır.
- `stackvo.json` ya da `stackvo.lock` var olmadan önceki bir commit hiçbir fark listelemez. Bisect o aralıkta yine çalışır; yalnızca ortam yarısı yoktur — ki diğer her aracın her zaman bulunduğu yer orasıdır.
