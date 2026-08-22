# Tüm projeler

Yakalaması açık her projenin dump çıktısını tek listede gösteren görüntüleyici.

## Araç çubuğu

| Kontrol | Ne yapar |
| --- | --- |
| Proje seçici | Hangi projelerin gösterileceği. Boş bırakmak hepsi demektir. |
| Ara | Görünen dump'ları süzer. |
| Kopyala | Görünenleri panoya alır. |
| Duraklat | Yeni dump'ların eklenmesini durdurur. |
| Temizle | Listeyi ve kaydedilmiş olayları siler. |
| Yardım | Yakalamanın nasıl çalıştığını anlatır. |

## Satırlar

Her satır hangi projeden ve hangi dosyanın hangi satırından geldiğini gösterir. Satıra tıklamak dump'ın tamamını açar.

## Bilinmesi gerekenler

- Yakalama proje başına açılır ve konteynere dokunulmadan çalışır.
- `dd()` isteği sonlandırır, `dump()` sonlandırmaz. İkisi de burada görünür.
- Bir projenin konteyneri köprüyü taşımıyorsa o proje için uyarı çıkar ve konteyneri yeniden oluşturmak gerekir.
