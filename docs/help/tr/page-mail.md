# Mail

Projelerinizin gönderdiği postalar, makineden çıkmadan yakalanmış hâlde.

## Yakalayıcı kapalıysa

Sayfanın tamamı bir açma teklifidir. **Etkinleştir** butonu `.env` dosyasını yazar, yapılandırmayı yeniden üretir ve konteyneri başlatır. İlk çalıştırma imajı indirir, bir dakika sürebilir.

Uygulama, siz istemeden `.env` dosyasına dokunmaz. Sayfayı açmak hiçbir şeyi değiştirmez.

## Bilinmesi gerekenler

- Yakalayıcı çalışırken uygulamanızın gönderdiği hiçbir posta makineden çıkmaz. Hepsi burada durur.
- Bir mesajı gerçek bir adrese iletmek isterseniz Aktarım sunucusu ayarlanmalıdır; ayarlanmadan gönderme reddedilir.
