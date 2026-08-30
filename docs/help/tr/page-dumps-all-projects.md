# Tüm projeler

Yakalaması açık her projenin dump çıktısını tek listede gösteren görüntüleyici.

## Araç çubuğu

| Kontrol | Ne yapar |
| --- | --- |
| Proje seçici | Hangi projelerin gösterileceği. Boş bırakmak hepsi demektir. |
| Sinyal | Dump, istek ya da iş. |
| Ara | Görünen satırları süzer; durum kodu da aranabilir. |
| Kopyala | Görünenleri panoya alır. |
| Duraklat | Yeni dump'ların eklenmesini durdurur. |
| Temizle | Listeyi ve kaydedilmiş olayları siler. |
| Yardım | Yakalamanın nasıl çalıştığını anlatır. |

## Satırlar

Her satır hangi projeden geldiğini gösterir. Bir dump hangi dosyanın hangi satırından geldiğini söyler ve satıra tıklamak tamamını açar; bir istek durumunu ve süresini, bir iş de sınıfını ve nasıl bittiğini söyler.

## Bilinmesi gerekenler

- Yakalama proje başına açılır ve konteynere dokunulmadan çalışır.
- `dd()` isteği sonlandırır, `dump()` sonlandırmaz. İkisi de burada görünür.
- İş satırları bu uygulamanın başlattığı işçilerden gelir; kendi terminalinizdeki bir `queue:work`'ten değil.
- Bir projenin konteyneri köprüyü taşımıyorsa o proje için uyarı çıkar ve konteyneri yeniden oluşturmak gerekir.
