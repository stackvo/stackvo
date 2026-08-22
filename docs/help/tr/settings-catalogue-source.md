# Kaynak

Servis paketlerinin nereden çekildiği ve o adresin çalışıp çalışmadığı.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Katalog adresi | Bir `https://` adresi ya da bir klasör. GitHub depo adresi, dosyaların gerçekte sunulduğu yere çevrilir. |
| Test et | Adresi çekmeden önce dener ve kaç paket bulduğunu söyler. |
| Klasör seç | Yerel bir katalog klasörü seçer. |
| Çek ve kullan | Katalogu indirir ve yürürlüğe koyar. |

## Bilinmesi gerekenler

- StackVo hiçbir servisi kendi içinde taşımaz. Bir katalog çekilene kadar hiçbir servis kullanılabilir değildir.
- Bir yönetici kaynağı sabitlemiş olabilir. O durumda buradaki adres yok sayılır ve kart hangi adrese sabitlendiğini yazar.
- Bu makine imzalı katalog istiyorsa ve yayımlanmış bir imzalama anahtarı yoksa çekme reddedilir. İmzasıza düşülmez.
