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

## CA nerede güveniliyor

"Güveniliyor" tek bir kelimeydi, ama birden fazla depo var.

| Depo | Kim kullanıyor |
| --- | --- |
| Bu makinenin güven deposu | Safari, Chrome, Edge, `curl` ve işletim sistemine soran her şey |
| Firefox'un kendi deposu | Yalnız Firefox — sistem deposunu kullanmaz, profil başına kendi deposunu taşır |

O ikinci satır, uğruna bir öğleden sonra kaybedilen satırdır. mkcert, Firefox'un deposuna **yalnızca makinede `certutil` varsa** kurar; yoksa bir uyarı basıp devam eder. Kurulum işe yaramış gibi görünür, sistem deposu yeşildir, ve Firefox her sayfayı reddeder.

Kart her depoyu adıyla söylüyor, ve cevabın hayır olduğu yerde ne yapılacağını da: `nss` paketini kurun (`certutil` onunla gelir) ve güven adımını yeniden çalıştırın. Ne güvenilen ne güvenilmeyen olarak gösterilen bir depo, o tarayıcının burada kurulu olmadığı anlamına gelir — kimsenin düzeltmesi gereken bir şey değildir.

## Neden Let's Encrypt'ten gerçek sertifika değil?

Çünkü bu adlar için verilemezler, ve sebepleri keşfedilmek yerine söylenmeye değer:

- Bir kamu sertifika otoritesi, **kamuya açık DNS'te bir adı kontrol ettiğinizi** doğrular. `shop.loc` kamuya açık DNS'te değildir ve hiç olmayacaktır — otoritenin kontrol edeceği bir şey yoktur.
- HTTP-01 doğrulaması, bu makinenin 80 numaralı portunun internetten erişilebilir olmasını ister. Bir yönlendiricinin arkasındaki dizüstü değildir.
- DNS-01 doğrulaması bunu aşar, ama gerçek bir alan adı **ve** onu tutan DNS sağlayıcısının API jetonunu ister — bu gerçek bir kurulumdur, ve yerel bir geliştirme ortamının varsayabileceği bir şey değildir.

Kamu sertifikasının burada gerçekten kazandıracağı şey, **başka cihazların** — aynı ağdaki bir telefon, bir meslektaşın dizüstü — sizin CA'nızı kurmadan güvenmesidir. İhtiyaç buysa, projeyi bir tünelle paylaşın: sağlayıcı TLS'i kendi kamu sertifikasıyla sonlandırır, ve ona her cihaz zaten güvenir.

## Bilinmesi gerekenler

- Sertifika üretmek için `mkcert` gerekir. Kurulu değilse kart bunu söyler ve yenileme yapılamaz.
- macOS güven ayarlarını yalnızca etkileşimli olarak değiştirtir; pencereli bir uygulama bunu kendi başına yapamaz. Buton bu yüzden terminalinizi açar.
- Güven verdikten sonra tarayıcıyı tamamen kapatıp açın. Açık bir tarayıcı eski güven listesini kullanmayı sürdürür.
