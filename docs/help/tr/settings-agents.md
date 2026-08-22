# Yapay zekâ asistanları

StackVo MCP sunucusunu bu makinedeki asistanlara tanıtır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kur | Seçtiğiniz istemcinin yapılandırma dosyasına `stackvo` girdisini yazar. |
| Kaldır | Yalnızca o girdiyi siler. |
| Yazma izni ver | Asistana yalnız okuma değil, yığını değiştirme yetkisi de tanır. |

## Ne değişir

Bu sunucuya sahip bir asistan, "shop.loc neden açılmıyor?" sorusunu ön kontrol raporundan, hosts dosyasından, sertifikadan ve konteyner durumundan cevaplayabilir. Tahmin etmek yerine bakar.

## Bilinmesi gerekenler

- Yazma dosya olarak yapılır: uygulama dosyayı okur, tek anahtarı ekler ve geri yazar. Diğer sunucularınız ve tanımadığı anahtarlar korunur.
- Yazmadan önce dosyanın yanına `.stackvo-backup` uzantılı bir yedek bırakılır.
- Yazma izni vermek asistana yığını durdurma ve değiştirme yetkisi verir. Vermezseniz asistan yalnızca okur.
