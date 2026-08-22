# Kimlik bilgileri nerede tutuluyor

Veritabanı şifreleri, token'lar ve sunucu kimlikleri `.env` yerine bu makinenin anahtar deposunda durabilir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Taşı | Değeri Keychain, Credential Manager ya da Secret Service içine kaydeder ve `.env`'de bir referans bırakır. |
| Geri al | Değeri anahtar deposundan `.env` dosyasına geri yazar. |

## Ne kazandırır, ne kazandırmaz

Taşımak, değeri yedeklenen, senkronlanan ve destek konularına yapıştırılan dosyadan çıkarır.

Değer hâlâ `generated/docker-compose.dynamic.yml` içine yazılır; Compose onu oradan okur. Yani bu işlem şifreyi `.env`'den çıkarır, diskten çıkarmaz.

## Bilinmesi gerekenler

- `stackvo.sh` komut satırı aracı anahtar deposunu okuyamaz. Bu çalışma alanında onu da kullanıyorsanız kimlik bilgilerini `.env`'de bırakın.
- Bu makinede uygulamanın ulaşabildiği bir anahtar deposu yoksa hiçbir şey taşınamaz; kart bunu söyler.
- Anahtar deposunu işaret eden bir kimlik bilgisi çözülemiyorsa dosya üretimi engellenir. Anahtar zincirinizi açın ya da değeri geri alın.
