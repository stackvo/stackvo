# Paylaş

Bu projeye yönlendiren geçici bir genel adres. `.loc` alan adına ulaşamayan webhook göndericileri ve dış servisler içindir.

Tünel istemcisi bir yardımcı konteyner olarak çalışır ve dışarı bağlanır. Bu makinede hiçbir port açılmaz.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Sağlayıcı | Tüneli hangi servisin taşıyacağı. Her satır, anahtarın saklı olup olmadığını söyler. |
| Genel adres al | Yardımcı konteyneri başlatır ve adresi gösterir. |
| Durdur | Yardımcı konteyneri indirir; adres anında çalışmaz olur. |
| Kopyala | Adresi panoya alır. |
| Anahtar | Sağlayıcının hesap anahtarını işletim sisteminin kasasında saklar. Bir daha gösterilmez. |

## Sağlayıcı seçmek

| Tür | Adres | Hesap |
| --- | --- | --- |
| Anonim hızlı tünel | Her başlatmada değişir | Gerekmez |
| Adresi saklayan sağlayıcı | Sabit kalır | Gerekir |

Değişen adres "webhook geldi mi" için yeterlidir. Bir panoya bir kez kaydedeceğiniz adres için sabit adres gerekir.

## Bilinmesi gerekenler

- Tünel konteynere yönlendirir. Proje durmuşsa adres çalışıyor görünür ama 502 döner.
- Adres çalıştığı sürece geneldir. Eline geçiren herkes yerel projenize ulaşır.
- İlk başlatma sağlayıcının imajını indirir, o yüzden sonrakilerden uzun sürer.
