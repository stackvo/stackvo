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

### Oturumları yakalamak, ki bir POST da yeniden gönderilebilsin

Yukarıdaki ret, yalnızca oturumu kaydeden bir şey olmadığı sürece doğrudur. **Oturumları yakala** bunu değiştirir — ve neye çevirdiği, basmadan önce okunmaya değer.

**Bu projenin istek çerezlerini ve form girdisini diske yazar.** Bir oturum çerezi kimlik bilgisinin **kendisidir**; onun işe yarar bir maskelenmiş sürümü yoktur, çünkü saklanmasının tüm sebebi değerin kendisidir. Dolayısıyla bu bir ayar değil, bir izindir — ve öyle inşa edildi:

| Kural | Niçin |
| --- | --- |
| **Siz basana kadar kapalı** | Köprü, dump'larınızı gösteren bayraktan **ayrı** ikinci bir bayrak olmadan hiçbir şey yazmaz. "Dump'larımı göster" ile "oturum jetonumu kaydet" iki farklı izindir. |
| **Dakika, asla süresiz** | Beş ile altmış arası. Pencere bir uzunluk olarak değil, **bittiği an** olarak saklanır — uygulama açıldığında saatini yeniden başlatan bir pencere, hiç kapanmayan bir penceredir. |
| **Kendi kendine biter** | Uygulama o saatin tamamında kapalı kalmış olsa bile. Süre dolumu, yalnızca siz bakarken çalışan bir zamanlayıcıyla değil, **her soruluşta** denetlenir. |
| **Durdurmak siler** | Yalnızca "yeni yakalama yok" değil: zaten alınmış olanlar da kaldırılır, ve düğme kaçını sildiğini söyler. Hasadını geride bırakarak biten bir izin, yalnızca bittiğine *inandığınız* izindir. |

**Önce hata ayıklama köprüsünün açık olması gerekir, ve düğme bunu söyler — hiçbir şey kaydetmeyecek bir pencereyi açtırmak yerine.** İki bayrak bilerek ayrı izinlerdir, ama birbirinden bağımsız değildir: yakalama bayrağını okuyan tek şey köprünün prepend dosyasıdır. Köprü kapalıyken pencere açmak, izni verir, denetim kaydına satırı yazar ve hiçbir şey yakalamaz — geriye sizin POST'unuzun yeniden gönderilemeyeceği sonucu kalır. Önce **Dump'lar** bölümünden açın, sonra pencereyi başlatın.

**Hiçbir ekran yakalananı göstermez.** Uyarının altındaki liste istek satırını, çerez **sayısını** ve gövde **boyutunu** verir — yeniden gönderilecek bir şey olduğunu size söylemeye yeter, ve oturum jetonunuzun var olduğu ikinci bir yer değildir. Denetim kaydında da yoktur: orası pencerenin açıldığını ve ne kadarlığına açıldığını kaydeder, yani birinin sonradan tarihlendirebilmesi gereken kısmı.

Yakalanmış bir oturum bir kayda, istek satırı **ve** saat birlikte kullanılarak, iki saniyelik bir pencerede bağlanır. Tek başına ikisi de yanlış olurdu: yalnız satır, bir ziyaretçinin sepetini başka bir ziyaretçinin aynı sayfaya ait kaydına bağlardı. Eşleşmeyen bir kayıt eski reddi kelimesi kelimesine korur, çünkü o kayıt için hâlâ doğrudur.

### Tekrarı bir anlık görüntüden başlatmak

Bir oturum yakalandığında bir POST yeniden gönderilebiliyor — ve bu, o şeyin **yeniden yapılması** demek. İkinci bir sipariş. İkinci bir satır. İkinci bir tahsilat.

Bu, onu reddetmek için bir sebep değil: bir POST'ta tekrara bilerek basıyorsunuz. Bu, iki kez basmayı güvenli kılan tek şeyi sunmak için bir sebep — ve StackVo'da o zaten var: **adlandırılmış bir veritabanı anlık görüntüsü**.

**Tekrarı bir anlık görüntüden başlat** altından birini seçin, ikinci koşudan önce geri yüklenir. Dört kural, ve her biri bir kolaylık değil bir ret:

| Kural | Niçin |
| --- | --- |
| Tekrardan **önce** geri yüklenir, sonra asla | Sonra yüklemek, tekrarın yaptığı şeyi silerdi — ki bir POST'u tekrarlamanızın sebebi tam olarak ona bakmaktı. |
| Şu an oradakinin **güvenlik kopyası** önce alınır | Her geri yüklemenin aldığı ağın aynısı. Bir profil ekranındaki bir düğme, geri alınamaz olan eylem olmamalı. |
| **StackVo asla seçmez** | Aslın hangi durumda koştuğunu bilemez, ve birini seçmek, sormadığınız bir soruyu sahip olmadığı veriyle cevaplamak olurdu. |
| Bir başarısızlık **tekrarı durdurur** | İkinci koşunun o anlık görüntüden başlamasını istediniz. İsteği yine de göndermek, onu seçmediğiniz bir durumdan koşturmak ve doğru olmayan bir öncül altında bir sayı basmak olurdu. |

**Ne kazandırdığı, açıkça: tekrarlanabilirlik, kıyaslanabilirlik değil.** İlk kayıt, o an veritabanında ne varsa ona karşı koştu ve bunu kaydeden bir şey yok. Anlık görüntü, tekrarı belirtilmiş bir noktadan yeniden koşabileceğiniz anlamına gelir — iki sayının kontrollü bir deney olduğu anlamına değil. Anlık görüntünün adı sonucun yanında gösteriliyor; böylece ikinci koşunun öncülü hafızanızda değil ekranda duruyor.

Yazacak olan satırlar işaretli, ve işaret ekranın bir dizgeden tahmin etmesinden değil kaydın kendisinden geliyor: istek satırı `GET` olmayan her şey. Bir GET de yazabilir ve bunu buradaki hiçbir şey bilemez; işaretin daha fazlasını vaat etmek yerine neyi ölçtüğünü söylemesinin sebebi bu.

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
