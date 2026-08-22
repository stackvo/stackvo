# Stripe webhook'ları

Canlı Stripe olaylarını bu projeye iletir. CLI dışarı doğru bağlanır, yani buradan hiçbir şeyin internetten erişilebilir olması gerekmez.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Gizli ya da kısıtlı API anahtarı | İşletim sisteminin kasasında saklanır. Alan saklananı göstermez. |
| Kaydet | Anahtarı yazar. |
| Unut | Anahtarı siler. |
| Başlat | `stripe listen` çalıştırır ve olayları konteynere iletir. |
| Durdur | Oturumu bitirir. |
| Kopyala | İmzalama sırrını panoya alır. |

## Tünelden farkı

Tünelin adresi her başlatmada değişir, yani webhook kaydını ve imzalama sırrını her seferinde yenilemek gerekir. Bu yöntemde adres yoktur; imzalama sırrı oturum boyunca sabit kalır.

## Bilinmesi gerekenler

- Mümkün olan yerde kısıtlı anahtar kullanın. Bu, gerçek olayları üzerinde çalıştığınız bir uygulamaya iletir.
- İmzalama sırrı her oturumda yeniden basılır. Durdurup başlattıysanız yeni sırrı uygulamanıza koyun.
