# Container içindeki supervisord

Bu projenin kendi container'ında `supervisord` çalışıyor; `php-fpm` ve web sunucusu onun altında. Bu panelde eklenecek ya da yapılandırılacak bir şey yok: container zaten biliniyor.

Bu her proje için doğru değil. StackVo'nun ürettiği imaj supervisord'u **nginx** ve **caddy** sunucuları için kullanır; apache, frankenphp ve swoole sunucularını başka türlü başlatır, PHP olmayan bir projede ise hiç supervisord yoktur. Panel bozuk görünmek yerine bunu söyler.

## Satırlar ne söyler

| İşaret | Anlamı |
| --- | --- |
| Yeşil | RUNNING. |
| Kırmızı | FATAL — pes etti, ve satır nedenini söylüyor. |
| Mavi | Başlıyor, duruyor ya da geri çekiliyor. |
| Gri | Durdurulmuş. |
| **Sürekli yeniden başlıyor** | Her seferinde geri geliyor, ve ölmeye devam ediyor. |

Sonuncusu supervisord'un bildirdiği bir şey değil. Süreç kimliğinin bakışlar arasında değişmesi izlenerek çıkarılıyor, çünkü durum sütunu bunu gösteremez: son bir dakikada kırk kez çöküp yeniden başlamış bir `php-fpm`, her sorduğunuzda RUNNING der.

## Sağlık kontrolleri

`RUNNING`, sürecin ayakta olduğu demektir. İçindeki şeyin cevap verip vermediği hakkında bir şey söylemez — işçisi tükenmiş bir `php-fpm` ile 502 döndüren bir web sunucusu, ikisi de `RUNNING` görünür.

Satırdaki kalp düğmesi o süreç için bir prob ekler: bir adrese HTTP isteği ya da bir host ve porta TCP bağlantısı. Süreç başına bir tane, çünkü bir süreç ya cevap verir ya vermez. Formda **Şimdi dene** düğmesi var, böylece kontrol kaydedilmeden önce kanıtlanabilir.

Kimlik doğrulama arkasındaki bir sağlık ucu 401 döndürür ve çalışıyordur; bu yüzden "cevap verdi" sayılan durum kodu sabit değil, bir alan.

Bilerek yapılmayan iki şey:

- **Hiçbir şeyi yeniden başlatmaz.** `php-fpm`'i sessizce yeniden başlatan yerel bir araç, insanın onu görmek için açtığı şeyi saklıyor olurdu.
- **Durmuş bir projeye prob atmaz.** Orada olmayan bir container'ın probu yalnızca başarısız olabilir.

## Bir şey bozulunca haber almak

StackVo açıkken **çalışan** her projeye arka planda yirmi saniyede bir bakılır; bir süreç **pes ettiğinde**, **sürekli yeniden başlamaya başladığında** ya da **sağlık kontrolü cevap vermeyi bıraktığında** masaüstü bildirimi düşer.

Bakma işini bu panel değil uygulamanın kendisi yapar, ve bütün mesele bu: yalnızca bu sekme açıkken çalışan bir yoklama, size ancak zaten önünüzde olanı söyleyebilir.

- **Bir şeyi bir kez söyler.** FATAL'da oturan bir süreç yirmi saniyede bir haber değildir — oraya vardığında haberdi. Düzelip yine bozulursa, o yine haberdir.
- **İlk bakışta hiçbir şey söylemez.** StackVo açıldığında zaten bozuk olan bir süreç az önce bozulmadı.
- **Hiçbir şeyi düzeltmez.** Hiçbir süreç yeniden başlatılmaz. Kimse bakmıyorken sessizce bir şeyi yeniden başlatan araç, gün yüzüne çıkarmak için yapıldığı olayı saklıyor olurdu.

Container'ında ulaşılabilir supervisord olmayan bir proje hiçbir şey tetiklemez. Çoğu proje PHP değildir ya da supervisord kullanmayan bir sunucu çalıştırır; bunu yirmi saniyede bir duyurmak, çalışma alanınızın şeklini duyurmak olurdu.

## Hiçbir şey göremediğini söylediğinde

| Ne diyor | Ne yapmalı |
| --- | --- |
| Bu projede supervisord yok | Hiçbir şey — bu projenin sunucusu supervisord kullanmıyor. |
| Projeyi yeniden derleyin | İmaj, StackVo üretilen `supervisord.conf`'a soketi eklemeden önce derlenmiş. Yeniden derlemek yeni yapılandırmayı içeri yazar. |
| Container çalışmıyor | Projeyi başlatın. |

## Bilinmesi gerekenler

- Bu container'daki yapılandırmayı **StackVo üretir** ve imaja gömer, yani burada düzenlenecek bir şey yok: çalışan bir container'ın içinde yapılan değişiklik bir sonraki derlemede kaybolur. Değiştirmek için projenin kendi ayarlarını değiştirip yeniden derleyin.
- Soket, container'ın içinde 0700 modlu bir Unix soketi. Hiçbir şey yayımlanmaz ve hiçbir port açılmaz — ona ulaşmak, zaten o container'da süreç çalıştırabiliyor olmak demektir.
- Buradan `php-fpm`'i yeniden başlatmak, container'ı yeniden başlatmadan PHP'yi yeniden başlatır: web sunucusu bağlantılarını korur ve tam bir açılışı beklemezsiniz.
- Bu süreçler loglarını container'ın stdout'una yazar, yani Loglar sekmesine. Buradaki log düğmesi supervisord'un kendi yakaladığını gösterir, ve tam bu yüzden genelde boştur.
