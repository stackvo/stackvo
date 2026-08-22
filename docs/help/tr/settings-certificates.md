# HTTPS sertifikası

Tek bir joker sertifika; panoyu, her servisi ve her projeyi kapsar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Sertifikayı yenile | Sertifikayı, güncel alan adı listesiyle yeniden üretir. |
| CA'ya güven (terminalde) | Terminalinizi açar ve otoriteye güven vermek için gereken komutu çalıştırır. |

## Kartın gösterdikleri

| Bilgi | Anlamı |
| --- | --- |
| Güncel / Yeniden üretilmeli | Sertifikanın kapsamı, şu anki alan adlarıyla uyuşuyor mu. |
| CA güveniliyor / güvenilmiyor | Bu makine, sertifikayı veren otoriteye güveniyor mu. |
| Bitiş tarihi | Sertifikanın geçerlilik süresi. |
| Kapsanan | Sertifikanın kapsadığı alan adları. |
| Kapsam dışı | Kapsanmayan adlar. Bunlar tarayıcı uyarısı verir. |

## Bilinmesi gerekenler

- Sertifika üretmek için `mkcert` gerekir. Kurulu değilse kart bunu söyler ve yenileme yapılamaz.
- macOS güven ayarlarını yalnızca etkileşimli olarak değiştirtir; pencereli bir uygulama bunu kendi başına yapamaz. Buton bu yüzden terminalinizi açar.
- Güven verdikten sonra tarayıcıyı tamamen kapatıp açın. Açık bir tarayıcı eski güven listesini kullanmayı sürdürür.
