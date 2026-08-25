# Konteynerin içinde düzenleyici

VS Code'u, bu projenin çalıştığı konteynerin üstünde açar. Dil sunucusu, uzantılar, terminal, `composer` ve `artisan` — hepsi imajın içinde çalışır; bu makinede PHP olmak zorunda değildir.

Bu, Xdebug kartının öteki yarısıdır. O kart makinenizdeki bir düzenleyiciyi konteynerdeki hata ayıklayıcıya bağlar; bu kart düzenleyiciyi konteynerin içine koyar.

## Nasıl açılır

VS Code'un "şu konteynere bağlan" diye bir komut satırı yok; çalışan bir konteyneri bir **adres** üzerinden açıyor. StackVo bu adresi zaten bildiği üç şeyden kuruyor: konteynerin adı, kaynağınızın bağlandığı dizin ve kaynağın gerçekten bağlı olduğu olgusu.

Adres, düğmeye basılabilsin ya da basılamasın kartın üstünde durur. VS Code bu makinede kurulu ama `code` komutu yoksa düğme yine çalışır — uygulamanın kendi URL işleyicisi kullanılır. VS Code hiç yoksa, adresi VS Code'un bulunduğu makinede Klasör Aç penceresine yapıştırın.

Hiçbir şey kaydedilmez. Adres kart her okunduğunda yeniden türetilir; yani yeniden oluşturulan bir konteyner ya da adı değişen bir proje geride eski bir adres bırakamaz.

## Ne zaman reddeder

| Kartın dediği | Neden |
| --- | --- |
| Konteyner çalışmıyor | Bağlanacak bir şey yok. Projeyi başlatın. |
| Konteyner kaynağın kopyasını taşıyor | Düzenleyici kusursuz çalışır ve her şeyi kaybeder. Aşağıya bakın. |

İkincisi, bu kartın uyarmak yerine reddetme sebebidir. PHP projesi deponuzu konteynere bind mount eder; oradaki bir düzenleyici gerçekten sizin dosyalarınızı düzenler. Node ya da Go projesinin imajı ise `COPY . .` ile derlenir; konteyner, imaj derlendiği anda alınmış bir **kopya** taşır. O kopyanın üstünde açılan düzenleyici dosyaları gösterir, sorunsuz kaydeder ve yazılan her satır bir sonraki yeniden derlemede çöpe gider — ekranda bunu söyleyen hiçbir şey olmadan.

Node projesi bu durumdan çıkarılabilir: Çalışma zamanı sekmesinde dev sunucusunu açın ve projeyi yeniden ayağa kaldırın; kaynak bağlanır. Öteki çalışma zamanlarının böyle bir karşılığı yok, onlar için cevap burada biter.

Kart, manifest'i değil **konteynerin kendi** mount tablosunu okur. Dev sunucusunu açmak, konteyner yeniden oluşturulana kadar hiçbir şey yapmayan bir dosya yazar; ikisi ayrıldığında doğru olan konteynerdir.

## Bilmekte fayda var

- **Alpine imajları.** Node projesi Alpine üstünde koşar ve VS Code onun için bir sunucu derlemesi yayımlar. Bu bir sorun değil, bir kayıt. JetBrains böyle bir derleme yayımlamıyor; PhpStorm'un ayrı bir soru olmasının sebebi bu.
- **İndirme.** VS Code konteynerin içine yaklaşık yüz megabaytlık bir sunucu açar. StackVo bunu adlandırılmış bir volume'da tutar, böylece yeniden derleme onu çöpe atmaz. Bu volume'dan önce oluşturulmuş bir konteyner bunu kartta söyler ve kendini yeniden oluşturmayı önerir.
- **git.** Araç zincirinde git yoksa düzenleyici, geçmişini okuyamadığı bir çalışma kopyası açar. Düzenlemek yine çalışır.
