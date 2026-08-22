# Dump'lar

`dump()` ve `dd()` çıktılarını yanıttan alıp burada gösterir. Biçimlendirmeyi, projenizin konteyneri içinde çalışan Symfony'nin kendi dump sunucusu yapar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| dump() ve dd() yakala | Yakalamayı açar. Anında etkilidir, konteynere dokunulmaz. |
| Ara | Görünen dump'ları süzer. |
| Kopyala | Görünenleri panoya alır. |
| Duraklat | Yeni dump'ların listeye eklenmesini durdurur. |
| Temizle | Listeyi ve kaydedilmiş olayları siler. |
| Satıra tıklamak | Dump'ın tamamını açar. |

## Bilinmesi gerekenler

- Yakalama kapalıyken hiçbir şey birikmez. Önce açın, sonra incelediğiniz sayfayı yeniden yükleyin.
- Yakalama sayfalar arasında açık kalır. Kuyruk işçisinden ya da bir konsol komutundan gelen dump da yakalanır.
- `dd()` isteği sonlandırır; `dump()` sonlandırmaz. İkisi de burada görünür.
- Konteyner köprüyü taşımıyorsa kart bunu söyler ve konteyneri yeniden oluşturmayı önerir.
