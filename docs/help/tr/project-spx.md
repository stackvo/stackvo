# Örnekleyici profilleyici (php-spx)

Açık bırakabileceğiniz profilleyici.

Xdebug'ın profilleyicisi her çağrıyı birebir kaydeder ve isteğin birkaç katına mal olur; bu, "şu fonksiyon tam olarak ne yapıyor" için doğru, "bu sayfa neden yavaş" için kullanışsızdır — onun altında bir sitede gezinemezsiniz. php-spx bunun yerine örnekleme yapar; sayfa sayfa gibi kalır ve laboratuvar sürümünü değil, yaptığınız işi profillersiniz.

İkisi de burada. Bu, yanındaki Xdebug profilleyicisinin yerine geçmez; farklı sorulara cevap verirler.

## Sırayla sağlanması gereken üç durum

| Durum | Ne demek |
| --- | --- |
| Derlendi | Eklenti bu projenin PHP sürümü için derlendi. |
| Bağlandı | Anahtar açık, yani compose overlay'i bu projeyi adlandırıyor. |
| Konteynerde | Bağlamalar konteyner oluşturulurken sabitlenir; anahtar, çalışan bir projeye ancak yeniden oluşturulduğunda ulaşır. |

### Neden derlenmesi gerekiyor

Bir eklenti, kendisini yükleyecek ikilinin PHP sürümü, ABI'si ve iş parçacığı güvenliğiyle birebir eşleşmek zorundadır ve php-spx kaynaktan derlenir — PECL'de değildir. Bu yüzden **bu projenin kendi imajından** tek kullanımlık bir konteynerde derlenir; derleyici, başlıklar ve hedef, php-fpm'in derlendiğiyle aynıdır.

Birkaç dakika sürer, ağ ister ve PHP sürümü başına bir kez yapılır — 8.4'teki her proje tek bir derlemeyi paylaşır. Çalışan konteyner bunun için hiç kullanılmaz: bu, canlı php-fpm'inizin içine bir derleyici kurmak olurdu, bir sonraki yeniden oluşturmaya kadar yaşar ve kimsenin istemediği bir yan etkidir.

Eklenti `stackvo.json`'a, Dockerfile'a ve imaja hiç girmez. Hata ayıklama köprüsü gibi bağlanır; hiç istemeyen bir proje hiçbir bedel ödemez.

## Kayıt

Üç yol var ve yalnız biri tarayıcı istiyor.

### Tek bir istek, bu panelden

Bir yol yazın ve kaydet deyin. Uygulama o tek isteği projenin kendi adresine, profilleyicinin tetikleyicisiyle birlikte gönderir; kayıt, maliyetiyle beraber aşağıdaki listede belirir. Tarayıcı yok, çerez yok, sonradan kapatılacak bir şey yok — ki bu aynı zamanda bir asistanın ya da terminalin bir sayfayı profilleyebilmesinin yolu (`stackvo spx-record <proje> /odeme`).

Adres projenindir, manifestosundan gelir. Yalnız yol sizindir ve başka bir konağı adlandıran bir yol reddedilir.

### Aynı isteği yeniden

Bir GET'in her kaydında **yeniden gönder** düğmesi var. Tam olarak o isteği profilleyici açıkken yeniden gönderir ve iki sayıyı farkıyla birlikte gösterir — performans işinin en sık döngüsü budur, ve normalde dört adım sürer: kodu değiştir, siteyi aç, sayfayı bul, geri dön ve yirmi kaydın arasından yenisini ara.

Ekranda bilerek bir **hüküm** yok. Bir koşuya karşı bir koşu, bir kıyaslama değildir: soğuk bir opcache, soğuk bir sorgu önbelleği ve makinenin o saniyede yaptığı her şey farkın içindedir. İki sayı, uygulamanın size ne anlama geldiklerini söylemesi için değil, sizin okumanız için gösteriliyor.

**Yalnız bir GET yeniden gönderilebilir**, ve düğme gizlenmek yerine reddin sebebi söyleniyor. Bir kayıt isteğin *satırını* tutar — `GET /checkout` — başka bir şeyini değil: başlıklarını değil, gövdesini değil, altında koştuğu oturumu değil; çünkü bunları kaydeden bir şey yok. Bunlar olmadan yeniden gönderilen bir POST, farklı bir istektir; CSRF'i olan herhangi bir çatıda sayfa yerine 419 cevabı verir. Cevap gibi görünüp cevap olmayan bir sonuç, bir retten daha kötü olurdu.

### Tek bir komut

Bir göç, bir kuyruk işçisi, bir test koşusu. Yavaş olan çoğu zaman bir sayfa değildir ve bunların hiçbiri tarayıcıdan profillenemez. Projenin kendi komutlarından birini seçin; profilleyicinin altında, işlem konsolunda çalışır ve aynı listeye düşer.

### Kontrol paneli, bir oturum için

SPX'in kendi paneli eklenti tarafından bu sitenin kendi adresinden sunulur — yayınlanacak bir port ve çalıştırılacak ikinci bir sunucu yoktur. Paneli açın, kaydı orada başlatın; sonra kullandığınız her sayfa kaydedilir. Profillemek istediğiniz şey bir *tıklama* olduğunda kullanılacak olan budur: bir form, bir ödeme akışı, oturumu açık bir kullanıcıyla geçen bir oturum.

Panelin örnekleme ve yerleşik fonksiyonlar için kendi denetimleri var; bu paneldeki Ayrıntı ayarı **buradan** başlatılan kayıtlar için geçerlidir.

Eklentinin yüklü olması tek başına neredeyse hiçbir şeye mal olmaz; siz istemedikçe hiçbir şey kaydedilmez.

## Ayrıntı

Bir örnekleme aralığı verilmedikçe php-spx **her çağrıyı** kaydeder — ki bu araç zaten o maliyetten kaçınmak için var. StackVo varsayılan olarak 100 µs'de bir örnekler; açık bırakmayı güvenli kılan da budur: 30 ms'lik bir isteğin onda birini tutan bir fonksiyonun arkasında hâlâ otuz örnek olur.

"Her çağrı" yine de bir seçenek ve hızlı bir fonksiyonu tahmin etmek yerine tam saymak için doğru olan o. PHP'nin kendi fonksiyonlarını profillemek izi kabaca ikiye katlar; cevabın projedeki bir şey değil de `preg_match` olduğu durumlarda açmaya değer.

## Zaman nereye gitti

Her kayıt, onu tutan fonksiyonlara açılır: her birinin koşunun ne kadarını kendi gövdesinde geçirdiği, çağırdığı her şeyle birlikte ne kadarını tuttuğu ve kaç kez çağrıldığı. Bu, kaydın kendisinden okunur; dolayısıyla bunun için de tarayıcı gerekmez.

Alev grafiği, çağrı ağacı ve zaman çizelgesi php-spx'in kendisine ait ve burada yeniden yazmaya değecek her şeyden iyi — her satırdaki ikinci düğme o kaydı kendi görüntüleyicisinde açar.

Çok uzun bir iz bir sınıra kadar okunur ve sınıra çarptığında bunu söyler. O zaman baktığınız şey, tamamının bir özeti değil, koşunun dürüstçe etiketlenmiş başlangıcıdır.

## İki profilleyiciyi aynı anda çalıştırmayın

Tek bir motora iki profilleyicinin bağlanmasını iki proje de desteklemez ve belirtisi hata değil, yanlış sayılardır. Xdebug da kayıt yapıyorsa panel bunu söyler; Xdebug modunu adım adım hata ayıklamaya geri alın.
