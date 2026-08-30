# Hata ayıklama sinyalleri

Kodun ne yaptığını yanıttan alıp burada gösterir. Üç şey gelir: `dump()` ve `dd()` çağrılarına verilen değerler, projenin karşıladığı her **istek** için bir satır, ve işçinin bitirdiği her **kuyruk işi** için bir satır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| dump() ve dd() yakala | Yakalamayı açar. Anında etkilidir, konteynere dokunulmaz. |
| Sinyal | Yalnız dump'ları, yalnız istekleri ya da yalnız işleri gösterir. |
| Kaynağa göre süz | Web, CLI ya da kuyruk — kodun nerede çalıştığı. |
| Ara | Görünen satırları süzer; durum kodu da aranabilir. |
| Kopyala | Görünenleri panoya alır. |
| Duraklat | Yeni satırların listeye eklenmesini durdurur. |
| Temizle | Listeyi ve kaydedilmiş olayları siler. |
| Satıra tıklamak | Dump'ın tamamını açar. |

## Üç sinyal

- **Dump** satırı değeri, dosyayı ve satır numarasını taşır. Satıra tıklamak dosyayı editörünüzde açar.
- **İstek** satırı her çalıştırma için birdir; HTTP durumunu ve ne kadar sürdüğünü söyler. PHP'nin kendi kapanış kancasıyla yazılır, yani ölümcül hatayla biten bir isteğin de satırı olur — bir `artisan` komutunun da.
- **İş** satırı her deneme için birdir: iş sınıfı, bitip bitmediği ve ne kadar sürdüğü. `--tries=3` ile her seferinde hata veren bir iş üç satır üretir, çünkü kuyruğun yaptığı budur.

## Bilinmesi gerekenler

- Yakalama kapalıyken hiçbir şey birikmez. Önce açın, sonra incelediğiniz sayfayı yeniden yükleyin.
- Yakalama sayfalar arasında açık kalır. Kuyruk işçisinden ya da bir konsol komutundan gelen dump da yakalanır.
- `dd()` isteği sonlandırır; `dump()` sonlandırmaz. İkisi de burada görünür, ve `dd()` sonrası istek satırı onun koyduğu 500'ü gösterir.
- İş satırları bu uygulamanın başlattığı işçiden gelir. Kendi terminalinizde çalıştırdığınız bir `queue:work` bu değildir ve satır üretmez.
- Yakalamayı açmak **bundan sonrasını** gösterir. İşçinin daha önceki çıktısı geriye dönük okunmaz.
- Konteyner köprüyü taşımıyorsa kart bunu söyler ve konteyneri yeniden oluşturmayı önerir.
