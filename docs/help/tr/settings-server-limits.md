# İstek sınırları

PHP'nin önündeki web sunucusunun neyi kabul edeceği. Üretilen sunucu yapılandırmasına yazılır.

## Alanlar

| Alan | Ne yapar |
| --- | --- |
| Azami gövde boyutu | Kabul edilecek en büyük istek gövdesi. Sayı, ardından isteğe bağlı `k`, `m` ya da `g`. |
| İstemci gövde zaman aşımı | Gövdenin gelmesi için tanınan süre. |
| KeepAlive zaman aşımı | Bağlantının açık tutulacağı süre. |
| FastCGI bağlanma / gönderme / okuma zaman aşımı | PHP-FPM ile konuşurken tanınan süreler. Uzun süren isteklerde okuma zaman aşımı önemlidir. |
| TCP nodelay | Küçük paketlerin bekletilmeden gönderilmesi. |
| Gzip, seviye, türler | Yanıt sıkıştırma. Türler boşlukla ayrılmış MIME türleridir; boş bırakılırsa sunucunun kendi listesi kalır. |

Varsayılanda bırakılan bir alan için hiçbir şey yazılmaz.

## PHP'nin kendi sınırları ayrıdır

Bir yükleme, sınırların en düşüğünde reddedilir. PHP'nin `upload_max_filesize`, `post_max_size` ve `memory_limit` değerleri vardır ve onlar proje başınadır; projenin PHP ayarları kartında.

Web sunucusunun sınırını yükseltip PHP'ninkini unutmak, yüklemenin yine reddedilmesine yol açar.

## Bilinmesi gerekenler

- Değişiklikler yeniden üretmeden sonra geçerli olur.
- Bu sınırlar tüm çalışma alanı içindir, proje başına değil.
