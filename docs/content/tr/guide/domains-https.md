# Alan adları ve HTTPS

Her proje `https://<ad>.<ek>` adresinde, tarayıcınızın güvendiği bir
sertifikayla cevap verir. Bunu üç parça sağlar; her birinin **Ayarlar** altında
bir kartı var.

<figure markdown>
![Alan adı ve ağ ayarları](../screenshots/web/settings-domain.webp){ loading=lazy }
<figcaption>Alan adı ve ağ: ek, hosts dosyası ve adresler.</figcaption>
</figure>

## Ek

**Ayarlar → Alan adı ve ağ → Adresler.** Her ana makine adı tek bir ekin altındadır;
tek sertifikanın hepsini kapsamasını sağlayan budur.

| Ek | Durum |
| --- | --- |
| `.test`, `.localhost` | Yerel kullanım için ayrılmış. Güvenli. |
| `.loc` | Kayıtlı bir TLD değil, bu iş için yaygın. Varsayılan. |
| `.dev` | Gerçek bir TLD, tarayıcıların HSTS listesinde. Altında hiçbir şey düz HTTP ile açılmaz. Önce HTTPS'i açın. |

Eki değiştirmek yeni sertifika ve yeniden üretim ister. Mevcut projeler kendi
`stackvo.json` dosyalarındaki alan adını korur.

## Hosts dosyası

`shop.loc` bu makineye, hosts dosyanızdaki tek satır sayesinde ulaşır.
**Ayarlar → Alan adı ve ağ → Hosts dosyası** her satırı ve durumunu gösterir —
çözülüyor, eksik, elle eklenmiş, artık gerekmiyor — ve **Tümünü düzelt**
eksikleri yazıp eskimişleri tek yetkili çağrıyla kaldırır. Önce fark
gösterilir ve yalnızca StackVo'nun kendi işaretleri arasındaki satırlara
dokunulur.

Joker karakterler hosts dosyasına giremez. `*.shop.loc` için **Yerel DNS**
kartını kullanın.

## Sertifika

**Ayarlar → HTTPS sertifikası.** Tek bir joker sertifika paneli, her servisi
ve her projeyi kapsar.

| Kart ne diyor | Anlamı |
| --- | --- |
| **Güncel** | Kapsam alan adlarınızla eşleşiyor. |
| **Yeniden üretilmeli** | Bir alan adı eklendi. **Yeniden üret**'e basın. |
| **CA güvenilir** | Bu makine otoriteye güveniyor. |
| **CA güvenilmiyor** | **CA'ya güven (terminalde)**'ye basın. macOS'ta terminal açılır, çünkü güven orada yalnızca etkileşimli değiştirilebilir. Sonra tarayıcıyı yeniden açın. |

Firefox kendi deposunu tutar ve yalnızca makinede `certutil` varsa doldurulur;
kart her depoyu ve ne yapılacağını söyler.

### Neden Let's Encrypt değil?

Herkese açık bir otorite, açık DNS'te bir adı kontrol ettiğinizi doğrular.
`shop.loc` açık DNS'te yok ve olmayacak; kontrol edecek bir şey yok. Herkese
açık sertifikanın sağlayacağı şey *başka cihazların* sizin CA'nız olmadan
güvenmesidir — bunun için [projeyi tünelle paylaşın](sharing.md): sağlayıcı
TLS'i kendi sertifikasıyla sonlandırır.

## Takma adlar

**Proje → Yapılandırma** aynı projeye ulaşan ek adlar alır. `stackvo.json`'a
yazılırlar, klonlayan iş arkadaşınız da alır. `*.` ile başlayan takma ad
sertifikaya ve yönlendirmeye girer ama hosts dosyasına giremez.

## Bu ağda

**Proje → Bu ağda**, projeye aynı Wi-Fi'daki bir telefonun çözebileceği bir
ad verir, sslip.io üzerinden. Ağdan hiçbir şey çıkmaz. Telefon sizin
otoritenizi tanımaz, bu yüzden kabul edeceği tek bir sertifika uyarısı alır.
