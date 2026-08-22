# Konteynerler

Yığının tamamını ayağa kaldırır ya da indirir. Tek bir projeyi değil, çalışma alanındaki her şeyi etkiler.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Ayağa kaldır | Yapılandırmayı yeniden üretir ve konteynerleri kurar. |
| İndir | Yığındaki konteynerleri durdurur ve kaldırır. |

## Bilinmesi gerekenler

- Ayağa kaldırmak compose seviyesinde çalışır: dosyalar yeniden üretilir ve konteynerler yeniden kurulur. Tek bir proje için bunu proje sayfasından yapın.
- İndirmek verilerinizi silmez. Veritabanı birimleri yerinde kalır.
- İşlem çıktısı alttaki konsolda görünür.
