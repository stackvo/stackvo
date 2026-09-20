# Dal başına ortam

Bir git dalı kendi ortamını alır: kendi klasörü, kendi adresi, **kendi
veritabanı**. İki dal aynı anda çalışır ve çalışma kopyanıza git'in fark
edeceği hiçbir şey yazılmaz. Bulut "önizleme ortamlarının" sattığı şey, yerelde
ve ücretsiz.

<figure markdown>
![Worktree kartı](../screenshots/web/project-detail-worktrees.webp){ loading=lazy }
<figcaption>Worktree'ler: dal başına bir ortam, her biri kendi alan adı ve veritabanıyla.</figcaption>
</figure>

## Oluşturmak

**Proje → Worktree'ler → Yeni worktree.**

| Alan | Ne yapar |
| --- | --- |
| **Dal** | Hangi dal ortamı alacak. **Dal oluştur** yeni bir dal açar. |
| **Ad** | Yeni projenin adı. Boş bırakılırsa daldan türetilir. |
| **Veritabanı** | Yok, yeni ve boş, ya da **bu çalışma alanınınkinin kopyası**. |
| **Süre** | Kendi dalınız için boş. Bir süre girilirse son kullanma tarihli bir sandbox olur. |

Ad, adres — `feature-checkout.shop.loc` — ve veritabanı adı bir şeye basmadan
önce gösterilir. Arka uçtan gelirler; ekrandaki, oluşturulacak olandır.

## Veritabanı gerçekten ayrı

Dalın veritabanında verilen oturum yalnızca o veritabanına ulaşır. Dal,
türediği veritabanını okuyamaz.

## Kaldırmak

**Kaldır** worktree'yi siler. Dalı silmek ve veritabanını silmek ayrı
anahtarlardır, ikisi de varsayılan olarak kapalı.

## Asistan için sandbox

**Süre** altında bir süre seçin, worktree bir sandbox olur: tek bir iş için,
var olduğunu hatırlamayacak biri tarafından kurulmuş bir ortam. Süresi dolunca
kendiliğinden biter. Bir dalı [yapay zekâ asistanına](ai-assistants.md)
vermeyi makinenizi vermekten farklı kılan budur.

Her worktree aynı zamanda bir proje satırıdır: başlat, durdur, loglar, hepsi.
