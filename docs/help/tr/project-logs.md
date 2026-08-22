# Loglar

Konteynerin kendi çıktısı ve projenin yazdığı log dosyaları.

## Araç çubuğu

| Kontrol | Ne yapar |
| --- | --- |
| Kaynak seçici | Hangi akışın okunacağı: konteyner çıktısı ya da projenin log dosyalarından biri. |
| Ara | Görünen satırları süzer. Eşleşen kısım satırın içinde işaretlenir. |
| Düzenli ifade | Aramayı düzenli ifade olarak yorumlar. |
| Seviye süzgeci | Yalnızca seçtiğiniz seviyeleri gösterir. |
| Kopyala | Görünen satırları panoya alır. |
| Takip et | Yeni satır geldikçe en alta kayar. Kapatırsanız okuduğunuz yer sabit kalır. |
| Duraklat | Akışı durdurur. Sürdürünce beklemiş satırlar gelir. |

## Bilinmesi gerekenler

- Konteyner çıktısı yalnızca stdout ve stderr'i taşır. Uygulamanızın kendi log dosyasına yazdıkları orada görünmez; kaynak seçiciden dosyayı seçin.
- Log dosyaları ancak proje derlendikten sonra vardır.
- Bir yığın izindeki konteyner yolu tıklanabilir: dosyayı editörünüzde açar.
- "Buradan itibaren canlı" çizgisi, akış açıldığı anı işaretler. Üstündeki satırlar dosyada zaten yazılıydı.
