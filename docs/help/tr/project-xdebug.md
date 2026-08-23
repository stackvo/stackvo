# Xdebug

Bu proje için adım adım hata ayıklama.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Etkin / Devre dışı | Xdebug'i açar ve kapatır. |

## İlk açış farklıdır

İlk kez açmak uzantıyı imaja ekler ve **yeniden derleme** gerektirir. Ondan sonrası yalnızca konteyneri yeniden başlatır: uzantı imajda kalır ve kapalıyken hiçbir maliyeti olmaz.

İkinci açışın ilkinden çok daha hızlı olması normaldir.

## IDE ayarları

Kart, IDE'nize gireceğiniz değerleri listeler:

| Alan | Ne için |
| --- | --- |
| Port | Xdebug'in bağlanacağı port. |
| IDE anahtarı | Oturumu tanımlayan anahtar. |
| Sunucu adı | `PHP_IDE_CONFIG` değeri. |
| Yol eşlemesi | Konteyner yolu ile makinenizdeki yolun karşılığı. Bu olmadan kesme noktaları tutmaz. |
| Xdebug sürümü | Kurulu sürüm. |

## Bilinmesi gerekenler

- Kart "çalışan konteyner Xdebug ayarlarını taşımıyor" diyorsa projeyi yeniden başlatın.
- Komut satırından `stackvo up` bu yapılandırmayı katmanlamaz ve konteyneri onsuz oluşturur.
- Xdebug ile profilleyici aynı uzantının iki kipidir. İkisi aynı anda açık olamaz.

## IDE kurulumu

Yukarıdaki üç değer bir IDE'nin ihtiyaç duyduğu her şeydir ve yol eşlemesi (path mapping) insanların yanlış yaptığı tek şeydir — her yerel ortam aracının dokümantasyonu, bir kesme noktasına hiç varılmamasının olağan sebebi olarak onu gösterir. Bu bölüm o değerleri sizin yerinize doldurur.

| Kontrol | Ne yapar |
| --- | --- |
| Yapılandırmayı yaz | Projenin `.vscode/launch.json` dosyasına, eşlemesi doldurulmuş bir `Listen for StackVo: <proje>` girdisi ekler. |
| Güncelle | Port ya da eşleme değiştikten sonra yeniden yazar. |
| Kaldır | Yalnızca o girdiyi siler. |
| Bloğu kopyala | Yazılmayan bir dosya için yapıştırılacak yapılandırma. |

**VS Code yazılır, PhpStorm yazılmaz.** PhpStorm `.idea/php.xml` ve `.idea/workspace.xml` dosyalarını bellekte tutar ve çıkarken yeniden yazar; yani çalışan bir PhpStorm'un altından düzenlenen dosya, PhpStorm'un üzerine yazdığı dosyadır — ve elinizde bir şeyi yapılandırdığını söyleyen bir araçla buna katılmayan bir IDE kalır. Bu yüzden adı ve iki kökü de doldurulmuş sunucu girdisi yapıştırmanız için sunulur.

İçinde yorum satırı olan bir `launch.json` — ki VS Code'un kendi oluşturduğu dosya böyledir — yeniden yazılmaz, bildirilir: düzenlemeyi mümkün kılmak için yorumları silmek, sizin kendi notlarınızı silmek olurdu.

Yol eşlemesi **uzaktan yerele** yazılır ve yerel taraf bu makinenin yolu değil `${workspaceFolder}`'dır; böylece dosya, depoyu klonlayan bir arkadaşınızda da çalışır.

### Dinleyen bir şey var mı?

Bir kesme noktasına hiç varılmamasının diğer sebebi hiçbir dosyada değildir: IDE'nin hata ayıklama portunu dinliyor olması gerekir ve hiçbir IDE bunu yüksek sesle söylemez. Listenin üstündeki satır, işletim sisteminin kendi dinleyen soket tablosunu okur ve portu tutan süreci adlandırır ya da hiçbir şeyin dinlemediğini söyler.

Bu bir okumadır, bağlantı değil. Bir şeyin cevap verip vermediğini görmek için hata ayıklama portunu aramak, IDE'nizde hemen kopan bir hata ayıklama oturumu olarak görünürdü.

## Uyarının düğmesi olduğunda

Üç durum üç farklı iş ister ve panel her birini, sizi sayfanın başka bir yerinde aramaya bırakmak yerine, sorunun yazdığı yerde sunar.

| Ne diyor | Ne gerekiyor | Neden |
| --- | --- | --- |
| Henüz imajda değil | **Yeniden üret ve derle** | Eklenti imaja derlenir; bir şey olabilmesi için önce imajın derlenmesi gerekir. Dakikalar. |
| İmajda var, konteynerde yok | **Konteyneri yeniden oluştur** | Konteynerin ortam değişkenleri oluşturulurken sabitlenir; yeniden başlatmak yetmez. Saniyeler. |
| Kapalı, ama konteyner hâlâ onunla çalışıyor | **Konteyneri yeniden oluştur** | Aynı sebep, ters yönden: kapatmak, zaten ayakta olan bir konteynere uzanmaz. |

Hiçbiri kendiliğinden yapılmaz. Yeniden derleme konteyneri yeniden oluşturur ve dakikalar sürer; sessizce böyle bir işi başlatan bir anahtar, sizin istemediğiniz bir sürpriz olurdu — bu yüzden panel sorar, ki düğmesi olan bir uyarı zaten budur.

Kapatmak **yeniden derleme yapmaz** ve bu bilinçlidir: eklenti imajda kalır, kapalıyken hiçbir maliyeti yoktur, böylece daha sonra tekrar açmak bir derleme değil bir konteyner yeniden oluşturma işidir.

Panel, iş bittiğinde kendini yeniden okur. Bu komutlar iş *başladığı* anda döner — operasyon konsolu bunun içindir — yani yalnızca düğme döndüğünde yeniden okuyan bir ekran size eski konteyneri gösterirdi.
