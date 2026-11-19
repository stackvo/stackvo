# Proje ayarları

Bu uygulamanın projenin konteynerine uyguladığı ayarlar. `.stackvo/site.json` içinde tutulur, yani bir takım arkadaşınız klonladığında onunla birlikte gelir.

## Ortam değişkenleri

Konteynere verilir. Uygulamanızın `.env` dosyasına yazılmaz — o dosya framework'ündür.

| Kontrol | Ne yapar |
| --- | --- |
| Ad / Değer | Bir değişken satırı. |
| Değişken ekle | Yeni satır açar. |
| Kaldır | Satırı siler. |
| Kaydet | `.stackvo/site.json` dosyasını yazar. |

Değişiklikler konteyner yeniden oluşturulunca geçerli olur.

## Dizin listesi göster

Index dosyası olmayan klasörlerde gezilebilir bir liste sunar. İndirme klasörü ya da derleme çıktısı için işe yarar.

Bu bir web sunucusu yönergesidir. Apache ve Swoole'un bunun için yapılandırma dosyası yoktur; öyle bir projede anahtar yerine sebebi görürsünüz.

## SSH ajanımı ilet

`composer install` ve `git pull` komutlarının konteyner içinden özel depolara ulaşmasını sağlar. İmaja hiçbir anahtar kopyalanmaz.

Bedeli şudur: o konteynerde çalışan her şey, konteyner ayakta olduğu sürece anahtarlarınızla imzalayabilir. Bu makinede çalışan bir SSH ajanı yoksa iletilecek bir şey de yoktur ve anahtar kapalı kalır.

## Host dizinleri

Projenin yanındaki bir dizini konteynerin içinde görünür kılar. Kullanım: projenin yanına klonlanmış ortak bir kütüphane, uygulamanın `../cbglib` gibi göreli bir yolla yüklediği. Konteyner yalnızca `/var/www/html` dizinini görür; mount olmadan o yol içeride yoktur.

| Kontrol | Ne yapar |
| --- | --- |
| Kaynak | Bu makinedeki dizin — projeye göreli (`../cbglib`) ya da mutlak. |
| Konteynerdeki yol | Nerede görüneceği, örneğin `/var/www/cbglib`. |
| Salt okunur | Varsayılan olarak açık. Konteyner dizini okuyabilir ama değiştiremez. |
| Kaydet ve onayla | `.stackvo/site.json` dosyasını yazar ve her satırı bu makinede onaylar. |

Neler bağlanabilir:

- Kaynak, sembolik bağlar çözüldükten sonra **projeler klasörünüzün içinde** var olan bir dizin olmalı. Projeler klasörünün kendisi ya da bu proje olamaz.
- Hedef mutlak bir yol olmalı. `/var/www/html` ya da altı olamaz; `/etc`, `/usr`, `/var/log` gibi bir sistem dizini de olamaz.

Neden onay var: `.stackvo/site.json` depoyla birlikte gelir, yani içindeki bir mount'u push yetkisi olan herkes yazmış olabilir. Bir mount ancak bu panelden, bu makinede kaydedildikten sonra uygulanır. Onay, StackVo klasöründeki `site-mounts.json` dosyasında tutulur; oraya hiçbir depo yazamaz. Klonla gelen bir mount uyarıyla listelenir ve uygulanmaz. Kaynağı, hedefi ya da modu sonradan elle değiştirmek de yeniden onay gerektirir.

Değişiklikler konteyner yeniden oluşturulunca geçerli olur. Bash CLI (`stackvo up`) bu mount'ları uygulamaz.
