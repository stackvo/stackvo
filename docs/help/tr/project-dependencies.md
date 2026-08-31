# Bu proje neye bağımlı

StackVo, her **servis paketinin** her dosyasını çalıştırmadan önce bir digest'e karşı doğrular, hareketli etiketi reddeder ve geldiği indeksin imzasını kontrol eder.

Bu sırada yanındaki proje `composer.lock` ve `package-lock.json`'dan dört yüz kütüphane çekiyor ve şimdiye kadar burada hiçbir şey onlara bakmamıştı. Bu ters bir durum: servis paketleri, bu projenin yayımladığı ve arkasında durabildiği bir katalog. Bağımlılıklar ise başkasının kodu — hem çok daha fazlası, hem de sizin izinlerinizle çalışıyor.

## Kilit dosyalarını oku — makinenizden hiçbir şey çıkmaz

Kilit dosyasının zaten söylediği üç şey, ve her biri yüksek sesle söylenmeye değer:

| Bulgu | Niçin bir bulgu |
| --- | --- |
| **Düz HTTP üzerinden çekiliyor** | Ağ yolunda kim varsa neyin geleceğine o karar verir. StackVo tam da bu yüzden **kendi** katalogu için `http://` reddediyor; bir projenin bunu dört yüz kütüphane için yapması aynı deliğin büyüğüdür. Paket paket adlandırılır. |
| **Bütünlük özeti yok** | O baytları doğrulayan hiçbir şey yok. Sayı olarak bildirilir — eski bir araçla yazılmış bir kilitte bu her paket olabilir, ve dört yüz aynı satır kimsenin okumadığı bir ekrandır. Kilidi güncel bir paket yöneticisiyle yeniden üretmek genellikle özetleri ekler. |
| **Başka bir indeksten** | Kusur değil. Özel bir ayna olağan bir şeydir. Ama bu bir tedarik zinciridir, ve kimsenin yazmadığı bir tedarik zinciri kimsenin izlemediği zincirdir. |

**Doğrudan ve dolaylı ayrı tutuluyor**, ve kartın tamamı bu ayrımın üzerine kurulu: doğrudan bağımlılık **sizin** seçtiğiniz bir sürüm, dolaylı olan ise başkasının sizin adınıza seçtiği bir sürüm. `composer.lock` hangisinin hangisi olduğunu kaydetmez — bu olgu yalnızca `composer.json`'da vardır, ve oradan okunur.

**Geliştirme bağımlılıkları dahildir** ve ayrı işaretlenmez. Bu makinede kuruludurlar ve aynı konteynerde çalışırlar; *"o sadece bir dev bağımlılığı"*, gerçek olaylara yol açmış bir cümledir.

## Danışmaları kontrol et — bu, makinenizden çıkar

İkinci düğme, **bu paketlerin adlarını ve sürümlerini** kamuya açık zafiyet veritabanı `api.osv.dev`'e gönderir.

Yanında başka hiçbir şey gitmez: kimlik yok, proje adı yok, yol yok, dosya içeriği yok. Yine de bu gerçek bir ifşadır — liste, hangi kütüphaneleri hangi sürümlerde kullandığınızı söyler. Rapora katılmak yerine, üstünde o cümleyle ayrı bir düğme olmasının sebebi budur; `PRIVACY.md`'nin aynı sözlerle bunu yazmasının sebebi de.

Geri gelen şey danışma **kimlikleridir** — `GHSA-…`, `CVE-…`. Aradığınız şey bir kimliktir. Burada türetilecek bir "önem derecesi" sözcüğü, bu uygulamanın verecek durumda olmadığı bir hüküm olurdu.

Başarısız bir sorgu bir hatadır, asla boş bir sonuç değil. *"Hiçbir şey bulunamadı"* ile *"soramadım"*, böyle bir ekranda aynı görünmemelidir.

## Bilinmesi gerekenler

- **Kilit dosyası olmaması, temiz proje demek değildir.** İki dosya da yoksa kart bunu söyler; bir şey yanlış değil diye rapor vermez.
- Yalnız `composer.lock` ve `package-lock.json` okunur: bu uygulamanın kendi çalışma zamanlarının en çok başvurduğu ikisi, ve JSON olan ikisi. `yarn.lock`, `pnpm-lock.yaml`, `go.sum` ve `Cargo.lock` her biri başka bir biçimdir; ölçülmemiş bir biçime karşı ezberden yazılmış bir ayrıştırıcı, bir raporun sessizce projenin yarısını kaçırmaya başlamasının yoludur.
- Lockfile v2 hem düz `packages` haritasını hem eski iç içe ağacı taşır. Varsa düz olan okunur, böylece hiçbir şey iki kez sayılmaz.
