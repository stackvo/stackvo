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
