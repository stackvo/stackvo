# Bu ağda

Bu projeyi aynı ağdaki bir telefondan ya da başka bir bilgisayardan açar.

Ad, sslip.io üzerinden çözülür. Hiçbir şey kaydedilmez, hiçbir şey yayımlanmaz, ağdan dışarı trafik çıkmaz.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Diğer cihazların çözebileceği bir adla yanıt ver | Paylaşılan adı açar ve bu tercihi manifeste yazar. |
| Aç | Adresi tarayıcıda açar. |
| Kopyala | Adresi panoya alır. |

## Paylaş'tan farkı

| | Bu ağda | Paylaş |
| --- | --- | --- |
| Nereye ulaşır | Yerel ağ | İnternet |
| Yardımcı konteyner | Gerekmez | Gerekir |
| Bedeli | Ziyaret eden cihazda sertifika uyarısı | Adres genele açık |

Sertifikayı bu makinenin kendi otoritesi verir. Bir telefon o otoriteyi tanımaz, o yüzden ziyaretçi bir uyarı görür ve kabul etmesi gerekir.

## Bilinmesi gerekenler

- Bu, çalışan bir konteynere yönlendirme değil; yönlendiricideki ve sertifikadaki bir addır. Proje dururken de doğrudur.
- Adres görünmüyorsa bu makinenin adres üretebileceği bir ağ bağlantısı yoktur.
