# OAuth geri dönüş adresi

Bir kimlik sağlayıcısının konsoluna yapıştırılacak yönlendirme adresi.

Yönlendirme tarayıcıya gönderilir; sağlayıcı bu adresi kendisi çağırmaz. Yani akışın kendisi için yerel adres çalışır. Değişen tek şey, sağlayıcının kaydederken bu metni kabul edip etmediğidir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Geri dönüş yolu | Uygulamanızdaki rota, örneğin `/auth/callback`. Normalleştirilip geri yansıtılır. |
| Kopyala | Yerel ya da genel adresi panoya alır. |

## İki adres

| Adres | Ne zaman çalışır |
| --- | --- |
| Yerel | `https://<proje>.loc/auth/callback`. Bu makinedeki akış için her zaman. |
| Genel | Çalışan tünel üzerindeki aynı yol. Yalnızca bir tünel çalışırken vardır. |

## Hangi sağlayıcı hangisini kabul eder

Kart sağlayıcıları ikiye ayırıp her birinin kuralını yazar:

- Özel bir uygulama için herhangi bir adresi kabul edenler yerel adresi alır.
- Alan adını doğrulayan ya da genel olarak çözülebilir bir ad isteyenler tünel gerektirir.

## Bilinmesi gerekenler

- Anonim bir tünelin adresini kaydetmeyin. Adres her başlatmada değişir, kayıt ertesi gün geçersiz olur.
- Sağlayıcı genel adres istiyorsa adresini saklayan bir tünel sağlayıcısı kullanın.
