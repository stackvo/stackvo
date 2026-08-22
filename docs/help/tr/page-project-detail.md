# Proje detayı

Tek bir proje: neyden kurulduğu, neyi çalıştırdığı ve şu an ne yaptığı. Sağdaki sekmeler konuya göre böler, üstteki çubuk projenin tamamına etki eden işlemleri taşır.

## Üstteki çubuk

| Kontrol | Ne yapar |
| --- | --- |
| Durum rozeti | Motorun bildirdiği hâl: çalışıyor, durdu ya da henüz derlenmedi. Docker'dan okunur, hatırlanmaz. |
| Tarayıcıda aç | `https://<alan-adı>` adresini varsayılan tarayıcınızda açar. |
| Terminalde aç | Kendi terminal uygulamanızı, konteyner içinde bir kabukla açar. |
| Hızlı komutlar | Projenin çatısının sunduğu komutlar. Sizin terminalinizde çalışırlar. |
| Editörde aç | Proje klasörünü Ayarlar → Tercihler'de seçtiğiniz editörde açar. |
| Klasörü aç | Proje klasörünü Finder ya da Dosya Gezgini'nde gösterir. |
| Başlat / Durdur | Konteyneri ayağa kaldırır ya da indirir. Hiçbir şey yeniden derlenmez. |
| Yeniden derle | Dockerfile'ı `stackvo.json`'dan yeniden üretir, imajı derler, konteyneri yeniden yaratır. |
| Yeniden başlat | Aynı konteyneri durdurup başlatır. |
| Sil | Konteyneri ve projenin kaydını kaldırır. Diskteki klasöre dokunmaz. |
| Yenile | Sayfadaki her şeyi motordan yeniden okur. |

## Yeniden derlemek ile yeniden başlatmak

Bunlar farklı işlerdir ve karıştırılması en sık yapılan hatadır.

| İşlem | Ne yapar | Ne zaman |
| --- | --- | --- |
| Yeniden başlat | Aynı konteyner durur ve başlar. | İçerideki bir süreç takıldığında. |
| Yeniden derle | Dockerfile üretilir, imaj derlenir, konteyner yeniden yaratılır. | PHP sürümü, eklenti ya da imajı ilgilendiren bir ayar değiştiğinde. |

Bir ayarı değiştirdiniz ve hiçbir şey olmadıysa, muhtemelen yeniden başlattınız ama yeniden derlemeniz gerekiyordu.

## Sekmeler

| Sekme | İçeriği |
| --- | --- |
| Gösterge | Canlı işlemci, bellek, disk ve ağ kullanımı; dağılımı ve son günlerin geçmişi. |
| Yapılandırma | `stackvo.json`, ihtiyaç duyulan servisler, ham manifest, makineye özel değerler, worktree'ler ve Dockerfile. |
| Konteyner | Çalışan süreç: Docker'ın bildirdikleri, dışarıdan erişim yolları, işçiler ve bir kabuk. |
| Loglar | Konteyner çıktısı ve projenin log dosyaları. |
| Hata ayıklama | Xdebug, profilleyici, dump'lar, sorgu günlüğü ve zaman çizelgesi. Yalnız PHP. |
| Çalışma zamanı ayarları | PHP için `php.ini`, Node için geliştirme sunucusu. |
| Üretim imajı | Bu makineden çıkacak imaj. |

## Bilinmesi gerekenler

- Bir sekme yalnızca geçerliyse gösterilir. Node projesinde `php.ini` ve Xdebug sekmeleri yoktur.
- Hiç derlenmemiş bir projede gösterilecek konteyner bilgisi yoktur.
