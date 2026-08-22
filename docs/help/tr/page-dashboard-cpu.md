# İşlemci yükü

Bu makinenin işlemci kullanımı. Halkanın ortasındaki yüzde, kullanılan toplam paydır.

## Dağılım

Halkanın yanındaki liste, kullanımın nereye gittiğini gösterir:

| Pay | Anlamı |
| --- | --- |
| Sistem | Çekirdek işleri. |
| Kullanıcı | Normal öncelikli uygulamalar. |
| Nice | Düşük öncelikli işler. |
| Boşta | Kullanılmayan pay. |

## Bilinmesi gerekenler

- Bu, makinenin tamamının kullanımıdır; yalnızca konteynerlerin değil. Tek bir projenin kullanımı için o projenin Gösterge sekmesine bakın.
- Dağılım her zaman görünmez. Sayaçlar birikimli olduğu için ilk ölçüm uygulamanın kendi açılışını anlatırdı; bu durumda halka yalnızca kullanılan ve boşta paylarını gösterir.
