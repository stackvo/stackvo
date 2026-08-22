# Manifest

Projenin `stackvo.json` dosyası, ham metin olarak. Kart varsayılan olarak kapalıdır; başlığa tıklayarak açın.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Başlık | Düzenleyiciyi açar ve kapatır. Kapatmak yazdıklarınızı silmez. |
| Kaydet | Metni dosyaya yazar. Anahtar sırası sözleşmeye göre düzeltilir. |
| Compose ile ayağa kaldır | Kaydedilen manifestten compose dosyalarını üretir ve yığını başlatır. |

## Bilinmesi gerekenler

- Yukarıdaki Yapılandırma kartında gördüğünüz her alan bu dosyada yazar. Oradan değiştirmek daha güvenlidir; burası dosyanın kendisini görmek isteyenler içindir.
- Kaydetmek geçerliliği denetler. Sözleşmeye uymayan bir dosya reddedilir ve hangi anahtarın sorunlu olduğu söylenir.
- Buradan yapılan bir değişiklik, sayfanın geri kalanını yeniden yükler.
