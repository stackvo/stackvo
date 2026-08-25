# Bu istek neden yavaştı

Kaydedilmiş tek bir istek ve etrafındaki üç ölçüm aleti: profil, sorgu günlüğü ve zaman çizelgesi. Bu sekmede üçünün de kendi kartı var ve her biri sorunun üçte birini cevaplıyor. Bu kart sorunun kendisini cevaplıyor.

## Bir kayıtla başlayın

Kart bir php-spx kaydı üzerinde açılır, çünkü burada bir isteği adlandıran, ne zaman başladığını ve ne kadar sürdüğünü söyleyen tek şey bir kayıttır. Henüz hiçbir şey kaydedilmediyse aşağıdaki **php-spx** kartından bir tane yapın: uzantıyı açın, sonra incelediğiniz sayfayı ondan isteyin.

Başka bir isteğe bakmak için seçiciden başka bir kayıt seçin. Çalışma alanınızda günlüğü okunabilen birden fazla veritabanı varsa onu da seçin.

## Kanıt ne diyor

Bulgular en üstte, çünkü cevap onlar. İki renk:

- **Kehribar** — değiştirilecek bir şey. Satır başına bir kez koşan bir sorgu şekli, zamanını veritabanını beklemekle geçiren bir istek, koşunun beşte birini tutan tek bir fonksiyon.
- **Mavi** — kanıtın kapsayamadığı bir şey. Sorgu günlüğü kapalıydı, iz bütün okunamayacak kadar uzundu, başka bir kayıt bununla çakışıyor.

## Zaman nereye gitti

Çubuk, koşuyu bir veritabanı sürücüsünün içinde geçen zaman ile geri kalan her şey olarak ikiye ayırır.

Veritabanı yarısı, sürücünün **kendi gövdesinde** geçen zamandır — `PDO`, `mysqli`, `pg_*`, `SQLite3`, Mongo sürücüsü. Bir çatının sorgu katmanı (Laravel'in `Connection::run`'ı, Doctrine'in `executeQuery`'si) PHP sayılır, çünkü bekleme onun altında olur ve ikisini birden saymak aynı zamanı iki kez saymak olur.

Aşağıdaki ifade listesi sorgular gösterirken ayrım veritabanında hiçbir şey olmadığını söylüyorsa, kayıt bu soruyu cevaplayamıyor demektir: php-spx PHP'nin kendi fonksiyonlarını profillemeyecek şekilde ayarlıdır, yani bekleme sürücüyü çağıran sizin fonksiyonunuza yazılmıştır. Kart bunu söyler ve anahtar php-spx kartındadır.

## Üç liste

Varsayılan olarak kapalı, çünkü bulgu değil kanıt onlar.

| Liste | Ne tutuyor |
| --- | --- |
| Fonksiyonlar | Kendi gövdesindeki zamana göre en ağır fonksiyonlar; koşudaki payları ve kaç kez çağrıldıkları. |
| İfadeler | Önce tekrarlanan şekiller, sonra günlüğün bu isteğin içinde tuttuğu her ifade — isteğin başından ne kadar sonra düştüğüyle damgalı. |
| Tek eksende | Dump'lar, ifadeler ve postalar birlikte, olma sıralarıyla. |

## Bağlantı nasıl kuruluyor, ve neyi yapamaz

Profil dışındaki her şey isteğe **zamanla** bağlanır. Bir kayıt duvar saatinden bir dilim iddia eder — ne zaman başladığı, artı ne kadar sürdüğü — ve bu kart o dilimin neleri tuttuğunu gösterir.

Bu gerçek bir sınır ve gizlenmiyor, söyleniyor:

- Bir veritabanı günlüğü ifadeyi ve bağlantıyı kaydeder, **hangi HTTP isteğinin ona sebep olduğunu değil**. Burada hiçbir şey tahmin etmiyor. Kaydedilen istek koşarken siteniz başka ne yapıyorduysa o da bu listelerde.
- Başka bir kayıt aynı dilimin bir kısmını iddia ediyorsa kart bunu söyler. Zamanla bağlanan her şey o zaman ikisi arasında paylaşılıyordur.
- Dump'lar istisnadır: hata ayıklama köprüsü her birinin hangi istekte olduğunu yazar, yani onlar gerçekten kendi bağlantılarını taşır.

Bir ifadeyi bir isteğe kesin olarak bağlamak, uygulamanızın bunu söylemesini gerektirir — bir başlık, SQL'e eklenmiş bir yorum, kodunuzun içinde bir toplayıcı. Bu özelliğin var olma sebebi tam da buna ihtiyaç duymamaktır.

## Dilimin kendisi nereden geliyor

Kart, pencerenin iki şeyden hangisi olduğunu söyler — çünkü ikisi eşit güvenilirlikte değil.

- **İzlendi.** İsteği StackVo'nun kendisi gönderdi — bu kartın php-spx komşusundaki düğmeden ya da `stackvo spx-record` ile — yani saati iki yanında da tuttu. Bu, php-spx'in kendi zaman damgası ne anlama gelirse gelsin koşuyu içine alır.
- **Hesaplandı.** Kayıt başka bir yerde, çoğunlukla php-spx'in tarayıcıdaki kendi kontrol panelinde alındı. Pencere o zaman damgası artı koşunun süresidir, iki ucunda yuvarlama için birer saniye payla — php-spx başlangıç zamanını tam saniye olarak yazar, yani gerçek başlangıç o saniyenin içinde bir yerdedir.

Hesaplanan pencere, o damganın koşunun **başı** olduğunu varsayar. Dosyanın yazıldığı an olduğu ortaya çıkarsa, hesaplanan bir pencere bir süre geç oturur. Düğmeden kaydetmek soruyu tamamen ortadan kaldırır.

## Bilinmeye değer

- Listeler 25 satırla sınırlı. Başlıklardaki sayılar gerçek sayılardır, böylece gösterilmeyeni görebilirsiniz.
- Sorgu günlüğü her ifadede yazma başarımına mal olur. İşiniz bitince **Sorgu günlüğü** kartından kapatın.
