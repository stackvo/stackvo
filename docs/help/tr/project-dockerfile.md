# Dockerfile

Bu projenin imajının derleneceği dosya. `stackvo.json`'dan üretilir; elle yazılmaz. Kart varsayılan olarak kapalıdır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Başlık | Dosyayı açar ve kapatır. |
| Kip seçimi | Dosyanın nasıl üretileceği. Aşağıya bakın. |

## İki kip

| Kip | Ne yapar |
| --- | --- |
| Üretilen | Üreticinin gerçekte yazdığı hâl. Kurulamayan eklentiler sessizce atlanır. |
| Katı | Kurulamayan bir eklenti varsa üretmeyi reddeder ve hangisi olduğunu söyler. |

## Rozet

Kartın başlığındaki rozet, diskteki üretilmiş dosyanın hâlâ güncel olup olmadığını söyler. Bayat diyorsa projeyi yeniden derleyin.

Rozet yalnızca Üretilen kipinde gösterilir. Katı kip tanımı gereği farklı bir çıktı verir, o yüzden orada karşılaştırmanın anlamı olmaz.

## Bilinmesi gerekenler

- Bu dosyayı düzenleyemezsiniz. İçeriğini değiştirmek için `stackvo.json`'u değiştirin.
- Dosya, projeyi her yeniden derlediğinizde yeniden üretilir.
