# Octane yeniden yükleme

Octane uygulamanızı **bir kez** açıyor ve bellekte tutuyor. Bütün amacı bu — ve ilk hafta herkesi yakalayan şey de bu.

`routes/web.php`'ye bir rota ekliyorsunuz. Sayfayı yeniliyorsunuz. 404 alıyorsunuz. Rota dosyada var, ve çalışan sunucuda yok, çünkü sunucu başladığından beri o dosyayı okumadı. Siz de kendi kodunuzda bir yazım hatası aramaya gidiyorsunuz.

## `octane:start --watch` niye değil

O, Laravel'in kendi cevabı ve çalışıyor. Bedeli **imajınıza Node ve chokidar kurmak**: bir konteynerin içinde çalışan, StackVo'nun host'tan zaten izlediği bir bind mount'u yoklayan ikinci bir dosya izleyicisi — ki macOS ve Windows'ta bu, yoklamanın pahalı türü.

StackVo proje dizininizi zaten izliyor ve projenizin konteynerinde zaten komut çalıştırıyor. O yüzden buradaki cevap tek bir eylem:

```
php artisan octane:reload
```

İmajınıza **hiçbir şey eklemiyor**. Bu da onu belgelenmiş yoldan yalnızca farklı değil, kesinlikle daha iyi yapıyor.

`octane:reload` bir yeniden başlatma değil. Sunucu işçilerini değiştiriyor ve soketini açık tutuyor; yani zaten yolda olan istekler dışında dışarıdan hiçbir şey fark etmiyor.

## Anahtar kapalı, ve siz aksini söyleyene kadar kapalı kalıyor

Bir istek işlenirken gelen bir yeniden yükleme **o isteği öldürür**. Bu, kimisinin istediği kimisinin istemediği bir takas, ve bir varsayılan bunu onların yerine karara bağlayamaz. O yüzden otomatik yeniden yükleme proje başına kapalı, ve açmak, sonucu yanına yazılmış bir karar.

Bu bir **tercih, manifest ayarı değil**. `stackvo.json`'ınız depoyla birlikte geziyor ve projenin *ne olduğunu* anlatıyor; "kaydettiğimde işçilerimi yeniden yükle" ise bir geliştiricinin bir makinede istediği şey, ve bir iş arkadaşınız bunu bir `git pull`'dan miras almamalı.

## Neyin kayıt sayıldığı

Yalnızca Octane'in kendi izlediği yollar: `app`, `bootstrap`, `config`, `database`, `public`, `resources`, `routes`, `composer.lock` ve `.env`.

`node_modules`, `vendor`, `public/build`, `public/hot` ya da `.git` içindeki her şey hangi derinlikte olursa olsun yok sayılıyor; editör takas dosyaları da öyle. Bu olmasa Vite çalıştırmak bir yeniden yükleme döngüsü olurdu — bir ön yüz derlemesi `public/build` içine birkaç yüz dosya yazıyor ve hiçbiri uygulamanızı değiştirmiyor.

**İki saniye geciktirmeli**, ki bu manifest izleyicisinin geciktirmesinden çok daha uzun. İkisi farklı sorulara cevap veriyor: o, *"editörüm bu dosyayı üç kez mi yazdı"* diye soruyor — bir kayıt hakkında; bu ise *"geliştirici bir şeyleri değiştirmeyi bıraktı mı"* diye soruyor — bütün bir işlem hakkında. Bir `composer install` binlerce dosyaya dokunur, ve dosya başına bir yeniden yükleme, hiç açılmayı bitiremeyen bir sunucu demektir.

## Nerede geçerli değil

Projeniz nginx, Apache ya da Caddy ile servis ediliyorsa PHP-FPM üzerinden çalışıyordur ve o, dosyayı her istekte okur. Bellekte değiştirilecek bir şey yoktur, o yüzden bu kart hiçbir şey yapmayacak bir düğme sunmak yerine bunu söylüyor.
