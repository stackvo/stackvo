# Politika gerçekten tutuyor mu?

Bu kart yalnızca bir yöneticinin politika dosyası gönderdiği makinelerde görünür. O dosyanın maddelerini teker teker okur ve bu makinenin şu anda **ne olduğunu** raporlar — dosyanın ne söylediğini değil.

## İkisi neden farklı

Bir politika, çoktan kurulmuş bir makineye gelir. Burada bulunanların çoğu birinin kural çiğnemesi değildir; işi kalmış bir kuraldır:

| Madde | Ne zaman uygulanır | Geriye ne bırakır |
| --- | --- | --- |
| `registryPrefix` | Dosyalar **üretilirken** | Salıdan beri kimsenin yeniden üretmediği bir proje hâlâ Docker Hub'dan çeker |
| `market.allowedPackages` | Bir paket **kurulurken** | Geçen ay kurulan bir servis, onu reddedecek liste bugün gelince de kurulu kalır |
| `market.requireSignature` | **Bir sonraki** yenilemede | Önbellekteki dizin, o an yürürlükte olan kural neyse ona göre kabul edilmişti |
| `market.allowOverrides` | Bir üstünyazım **oluşturulurken** | Diskte zaten olan dosyalar, yayımlanmış paketin önünde okunmaya devam eder |

Yukarıdakilerin hepsinin onarımı aynıdır: yeniden üret, kaldır, yenile, sil. Rapor, hangisinin ve nerede olduğunu bilesiniz diye var.

## Dört durum

| Durum | Anlamı |
| --- | --- |
| **Tutuyor** | Ölçüldü, ve bu makine maddenin içinde |
| **Baypas** | Ölçüldü, ve buradaki bir şey maddenin dışında |
| **Görüş yok** | Politikanın bu konuda söylediği bir şey yok |
| **Ölçülemedi** | İki yönde de kanıt yok — sebebi her zaman yanına yazılır |

**"Görüş yok" asla geçer not değildir.** `market` bloğundaki her liste boşken *görüş yok* demektir, asla "hiçbiri" değil — dolayısıyla sessizliği yeşil bir tike çeviren bir rapor, hiç politikası olmayan bir makineyi tam uyumlu diye puanlardı ki bir uyum raporunun yapabileceği en yanıltıcı şey budur.

**"Ölçülemedi" de geçer not değildir.** Farklı görünen ama burada aynı olan iki şeyi kapsar: uygulamanın göremediği bir olgu (üretilen ağaç okunamadı, bir paket manifestosu yüklenemedi) ve uygulanacak bir şeyi olmayan bir madde — bu yapının hiç çalıştırmadığı bir depoyu adlandıran `imagePins` girdisi, hiçbir şey yapmayan bir satırdır. İkisi de uyum kanıtı değildir.

## "Hesabı verilmemiş bir şey yok" ve ne demek olmadığı

Üstteki rozet yalnızca hiçbir şey baypas edilmemişse **ve** hiçbir şey ölçülememiş değilse yeşildir.

Adı bilerek *uyumlu* değil. Politika katmanı bir güvenlik sınırı değildir — dosya da onu yönlendiren `STACKVO_POLICY_FILE` değişkeni de genellikle makineyi elinde tutan kişinin erişebileceği yerdedir. Bu kart burada ölçüleni bildirir. Birinin neyi değiştirebileceği hakkında bir şey söylemez ve kimsenin güvenerek imzalayabileceği bir sertifika değildir.

## Bilinmesi gerekenler

- Her olgu diskten gelir: yazıldığı hâliyle `.env`, üretilen ağaç, paket dizini, hatırlanan katalog kaynağı, üstünyazım dosyaları, proje manifestoları. Docker'a hiç sorulmaz — motor kapalıyken çalıştıramadığınız bir rapor, en çok ihtiyaç duyduğunuzda çalıştıramayacağınız rapordur.
- `.env`, uygulamanın çözdüğü hâliyle değil, yazıldığı hâliyle okunur. Uygulamanın kendisi kilitli bir anahtarı her zaman yöneticinin değerine çözer; ikisinin ayrışabildiği yer dosyadır ve dosyayı doğrudan okuyan her şey — terminalden `docker compose`, bir betik — dosyanın söylediğini alır.
- Ayna sorusunu aynanın kendisi cevaplar: bir dosyanın kendi baytlarına yeniden uygulamak onları değiştirecekse, o dosyaya hiç ulaşmamış demektir.
- Kancalar ve sağlayıcılar baypas edilemez — çalışırken denetlenirler — o yüzden bu satırlar kuralın fiilen ne kadarını durdurduğunu bildirir. Hiçbir şeyi durdurmayan bir ret de kırk adımı durduran bir ret de "tutuyor"dur, ama yalnızca birine bakmaya değer.
