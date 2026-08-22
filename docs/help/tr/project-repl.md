# Çalışma tezgâhı

Bir parça kod yazın, uygulama ayağa kalkmış hâlde bu projenin içinde çalıştırın, dönen sonucu okuyun.

Tek satırlık işler için yukarıdaki terminal daha iyidir. Burası, sürekli düzenlediğiniz yirmi satır içindir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Şununla çalıştır | Kodu hangi çalıştırıcının yürüteceği. |
| Kod | Kodun kendisi. `⌘/Ctrl + Enter` de çalıştırır. |
| Çalıştır | Kodu gönderir; çıktıyı, çıkış kodunu ve stderr'i gösterir. |
| Geçmiş | Çalıştırdığınız parçalar. Birine tıklamak onu çalıştırıcısıyla editöre geri koyar. |
| Unut | Geçmişi temizler. |

## İki tür çalıştırıcı

| Tür | Ne verir |
| --- | --- |
| Uygulama ayağa kalkmış | Modelleriniz, yapılandırmanız ve konteyneriniz. |
| Çıplak | Dilin kendisi. Çatı yok, veritabanı bağlantısı yok. |

Her satır hangisi olduğunu söyler. Yanlışını seçmek, on dakika boyunca modellerinizin hiç yüklenmediğini fark etmemeye yol açar.

## Bilinmesi gerekenler

- Görmek istediğinizi yazdırın: `dump()`, `echo`, `print`. Son ifadenin değeri kendiliğinden yazdırılmaz.
- Başarıyı çıkış kodu belirler, stderr'in boş olması değil. Pek çok dil başarılı koşuda da stderr'e yazar.
- Koşunun bir süre sınırı vardır. Sınırda durdurulduysa kart bunu söyler.
- Projenin çalışıyor olması gerekir; boş bir kod çalıştırılmaz.
