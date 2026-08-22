# Yalnız bu makine

`stackvo.local.json` dosyası. Buradaki değerler `stackvo.json`'u yalnızca bu checkout için geçersiz kılar.

Ne zaman işe yarar: test ettiğiniz bir PHP sürümü, ya da bu makinede başka bir şeyle çakışan bir alan adı.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Düzenleyici | Dosyanın içeriği. Bir parçadır, tam bir manifest değil — yalnızca değiştirmek istediğiniz anahtarları yazın. |
| Kaydet | Dosyayı yazar. |
| Kaldır | Dosyayı siler; proje `stackvo.json`'a döner. |

## Kartın söyledikleri

- **Yürürlükte** — hangi alanların şu an bu dosyadan okunduğu, alan alan listelenir.
- **Yok sayıldı** — bu makineyi değil repoyu tarif eden anahtarlar. Yalnızca `stackvo.json`'dan okunurlar.
- **git durumu** — dosyanın commit'lerin dışında tutulup tutulmadığı. git commit'liyorsa uyarı çıkar: `stackvo.local.json`'u `.gitignore`'a ekleyin, yoksa bu ayarlar tüm ekibin ayarı olur.

## Bilinmesi gerekenler

- Proje git altında değilse git durumu hiçbir şey söylemez. Bu bir uyarı değildir; sızacak bir klon yoktur.
- Bu dosya commit'lenmek için değildir. Ekibin görmesi gereken bir ayar Manifest kartına yazılır.
