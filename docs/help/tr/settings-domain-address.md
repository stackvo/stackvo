# Adresler

Projelerin ve servislerin yanıt verdiği adreslerin nasıl kurulduğu. Her ana bilgisayar adı bu soneğin altında toplanır; tek bir sertifikanın hepsini kapsamasını sağlayan da budur.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Ad alanı | Tüm adresleri tek bir üst alan altında toplar. İsteğe bağlıdır; boş bırakırsanız yalnızca uzantı kullanılır. |
| Uzantı | Adreslerin uzantısı: `.loc`, `.test` gibi. |

Kart, seçiminizin adresleri nasıl etkileyeceğini önceden gösterir.

## Uzantı seçimi

| Uzantı | Durum |
| --- | --- |
| `.test`, `.localhost` | Yerel kullanım için ayrılmıştır. Güvenlidir. |
| `.loc` | Kayıtlı bir TLD değildir, yaygın olarak kullanılır. |
| `.dev` | Gerçek bir TLD'dir ve tarayıcıların HSTS listesindedir. Altındaki hiçbir adres düz HTTP ile açılmaz, uyarıyı geçme imkânı yoktur. Kullanacaksanız önce HTTPS'i açın. |

## Bilinmesi gerekenler

- Soneki değiştirmek yeni bir sertifika ister. Kaydettikten sonra Sertifikalar kartına bakın.
- Var olan projeler kendi `stackvo.json` dosyalarındaki alan adını korur. Yeni sonek yalnızca bundan sonra kurulacakları etkiler.
- Kaydetmek yetmez: yönlendirme etiketlerinin yeni soneki alması için yeniden üretmek gerekir. O ana kadar yığın eski etiketlerle yanıt verir.
