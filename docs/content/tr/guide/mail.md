# Mail

Projelerinizin gönderdiği her şey makineden çıkmadan yakalanır ve pencerede
okunur. Tarayıcı sekmesi yok, gerçek posta kutusu yok.

<figure markdown>
![Mail sayfası](../screenshots/web/mail.webp){ loading=lazy }
<figcaption>Mail: projelerin gönderdiği her ileti, makineden çıkmadan yakalanır.</figcaption>
</figure>

## Açmak

**Mail → Mailpit etkinleştir** `.env` dosyasını yazar, yapılandırmayı yeniden üretir
ve yakalayıcıyı başlatır. İlk çalıştırma imajını indirir ve bir dakika
sürebilir. Sayfayı açmak hiçbir şeyi değiştirmez; yalnızca düğme değiştirir.

Yakalayıcı çalışırken uygulamanızın gönderdiği hiçbir şey makineden çıkmaz.

## Okumak

Solda bir mesaj seçin, sağda okuyun. Arama `from:ali@example.com` ve
`subject:"fatura"` anlar.

| Sekme | Gösterdiği |
| --- | --- |
| **Önizleme** | HTML gövde, korumalı çerçevede. |
| **Metin**, **Kaynak**, **Başlıklar** | Düz gövde, ham mesaj, her başlık. |
| **Ekler** | Dosyalar; kaydedebilirsiniz. |
| **Uyumluluk** | Kullanılan HTML ve CSS'in posta istemcilerince desteklenip desteklenmediği — yeşil, turuncu, kırmızı. |
| **Bağlantılar** | Mesajdaki her bağlantı. **Bağlantıları denetle** her birini çeker. |

## İletmek

Yakalanan bir mesajı gerçek bir adrese iletmek için yapılandırılmış bir relay
gerekir. Yoksa gönderme sessizce düşürülmez, reddedilir.
