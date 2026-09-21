# Günlük kullanım

Pencerede sıradan bir gün nasıl görünür: neler nerede, bir projenin sayfasında ne var ve zamanınızın çoğunu geçireceğiniz üç yer.

## Yedi sayfa

Sol taraftaki yedi sayfa uygulamanın tamamıdır:

| Sayfa | Ne için |
| --- | --- |
| **Panel** | Makinenin durumu: CPU, bellek, disk, ağ, çalışan projeler, Docker'ın sağlığı |
| **Projeler** | Projelerin listesi; satır üzerinden başlat / durdur / yeniden kur, alan adını aç |
| **Katalog** | Servis kurma ve sürüm seçme; aynı servisin iki örneğini yan yana çalıştırma |
| **Loglar** | Bütün projelerin günlükleri tek yerde, canlı |
| **Dump'lar** | Uygulamanızın `dump()` / `dd()` çıktıları — tarayıcıya basılmadan |
| **Mail** | Uygulamanın gönderdiği e-postalar; HTML önizleme, arama, bağlantı denetimi |
| **Ayarlar** | Alan adı, sertifika, PHP, tanılama, yedekler, yapay zekâ asistanları |

<figure markdown>
![Projeler sayfası](../screenshots/web/projects.webp){ loading=lazy }
<figcaption>Projeler sayfası</figcaption>
</figure>

## Proje sayfası

Bir projenin adına tıkladığınızda **proje sayfası** açılır: 45 panel, her biri
tek bir konuya ait — Genel Bakış, Servisler, Günlükler, Terminal, Xdebug,
Profilleyici, Bu istek neden yavaştı, Snapshot’lar, Worktree'ler, Paylaş, Üretim imajı,
Manifest…

<figure markdown>
![Bir projenin ayrıntı sayfası](../screenshots/web/project-detail.webp){ loading=lazy }
<figcaption>Bir projenin ayrıntı sayfası</figcaption>
</figure>

Her panelin köşesinde bir **?** düğmesi vardır ve o panelin ne yaptığını kendi
diliyle anlatır. O belgeler bu sitenin [Uygulama içi yardım](../help/index.md)
bölümüdür, kelimesi kelimesine.

## Bir şey ters giderse

**Ayarlar → Doktor** içindeki *Doktor*, neyin bozuk olduğunu ve nasıl
düzeleceğini satır satır söyler; çoğu bulgunun yanında düzelten bir düğme
vardır.

**Ayarlar → Uygulama günlüğü → Tanılama paketi kaydet**, günlüğü, ön
kontrolleri, doctor raporunu ve varsa çökme raporlarını tek arşive yazar.
Parolalar ve token'lar günlük yazılırken maskelenir. [Bir sorun
açarken](../community/contributing/reporting-a-bug.md) bu paketi ekleyin.

!!! tip "Terminali sevenler için"
    Yukarıdakilerin tamamının bir `stackvo` komut satırı karşılığı da var.
    Uygulamayı kullanmak için gerekli değildir — CLI, betikler ve CI için
    oradadır.
