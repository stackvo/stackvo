# Projeler

Bu çalışma alanındaki tüm projeler ve konteynerlerinin durumu. Bir projeye ait ayrıntılı işlemler için satırın sonundaki **Detay** butonuna basın.

## Üstteki çubuk

| Kontrol | Ne yapar |
| --- | --- |
| Çalışıyor rozeti | Kaç projenin çalıştığını, kaç projenin olduğunu gösterir. |
| Artı | Yeni proje panelini açar. |
| Yenile | Listeyi motordan yeniden okur. |
| Üç nokta | Sahiplenilmemiş kod panelini açar. |

## Tablo

| Sütun | Ne gösterir |
| --- | --- |
| Favori | Yıldız. Favoriler listenin üstünde toplanır. |
| Alan adı | Projenin adresi. Alan adı çözülmüyorsa satır bunu söyler ve hosts kaydını ekleyecek bir işlem sunar. |
| Çalışma ortamı | PHP, Node ya da başka bir çalışma zamanı ve sürümü. |
| Repo | Git deposu bilgisi. Uzak sunucusu olmayan bir depo da belirtilir. |
| Sunucu | Projeyi sunan web sunucusu. |
| Yapılandırma | Manifest geçerli mi, üretilmiş dosyalar güncel mi. |
| Durum | Çalışıyor, durmuş ya da derlenmemiş. |

Bir proje başka bir projenin dalıysa satırda **{proje} dalı** yazar. Böylece iki ilgisiz görünen satır yerine, tek bir uygulamanın iki dalı olduğu anlaşılır.

## Satır işlemleri

| İşlem | Ne yapar |
| --- | --- |
| Durdur / Başlat | Konteyneri indirir ya da ayağa kaldırır. |
| Yeniden başlat | Aynı konteyneri durdurup başlatır. |
| Yeniden derle | Dockerfile'ı yeniden üretir, imajı derler, konteyneri yeniden yaratır. |
| Terminal | Konteyner içinde bir kabuk açar. |
| Tarayıcıda aç | Projenin adresini açar. |
| Detay | Proje detay sayfasına gider. |
| Sil | Konteyneri ve projenin kaydını kaldırır. Diskteki klasöre dokunmaz. |
| Üç nokta menüsü | O an yapılabilecek olan işlemi sunar: derle, başlat, durdur, değişiklikleri uygula ya da hosts kaydını ekle. |

Menüde yalnızca o an anlamlı olan işlem görünür. Derlenmemiş bir projede "Başlat" değil "Derle" vardır.

## Arama ve süzgeçler

| Kontrol | Ne yapar |
| --- | --- |
| Ara | Proje adına ve alan adına göre süzer. |
| Durum süzgeci | Hepsi, çalışan, durmuş ya da derlenmemiş. |
| Yalnızca favoriler | Yıldızladıklarınızı gösterir. |
| Süzgeçleri temizle | Tüm süzgeçleri kaldırır. |

## Yeni proje

Artı butonu bir panel açar. İki yol vardır:

- **Boş proje** — formdaki değerlerle sıfırdan bir proje.
- **Bir çatı şablonu** — Laravel, WordPress, Symfony, Next.js gibi. Çatının kendi kurulum aracı geçici bir konteynerde çalışır, sonra sonuç sahiplenilir. Çalışma ortamı, sunucu ve belge kökü kurulum aracının gerçekte yazdığından okunur.
- **Git deposundan çek** — var olan bir depoyu klonlar ve sahiplenir.

Alan adı boş bırakılırsa proje adından üretilir.

## Sahiplenilmemiş kod

Üç nokta menüsündeki bu panel, bu makinede olup StackVo'nun çalıştırmadığı kodu bulur.

| Kaynak | Ne yapar |
| --- | --- |
| Proje klasöründeki klasörler | `stackvo.json` dosyası olmayan klasörleri listeler ve **Sahiplen** ile projeye dönüştürür. Ne olduğu, klasördeki dosyalardan algılanır. |
| XAMPP ve Laragon siteleri | O araçların kurulum klasörünü okur ve sitelerini listeler. |
| Compose dosyası olan projeler | Var olan bir `docker-compose.yml` dosyasından proje çıkarır. Yazılacak `stackvo.json` ve her değerin nereden okunduğu önceden gösterilir. |

## Bilinmesi gerekenler

- İçe aktarma, diğer aracın klasörüne asla yazmaz. Site bu çalışma alanına kopyalanır. **Kopyalamak yerine taşı** seçilirse kopya tamamlandıktan sonra aslı silinir ve diğer araç o siteyi artık sunmaz.
- Sahiplenme sırasında tanınan bir şey yoksa varsayılanlar kullanılır; panel bunu satır satır söyler.
- Compose'dan içe aktarırken StackVo karşılığı olmayan servisler ayrıca listelenir. Onları kendiniz ele almanız gerekir.
- Bir projeyi silmek kodunuzu silmez. Yalnızca konteyner ve kayıt kaldırılır.
