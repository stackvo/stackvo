# Geliştirme sunucusu

Node projeleri içindir. İmaja gömülü üretim derlemesi yerine, kaynağınız canlı bağlanmış hâlde projenin geliştirme sunucusunu çalıştırır.

Bu kapalıyken konteyner, derlendiği anda alınmış bir kod kopyası taşır. Dosya düzenlemek hiçbir şeyi değiştirmez.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Açık / Kapalı | Kaynak bağlamayı ve geliştirme sunucusunu açar ya da kapatır. |
| Geliştirme komutu | Çalıştırılacak komut. Üretim komutunun yerine geçer; kart hangisinin yerine geçtiğini yazar. |

## Projenizin de bir şeye ihtiyacı var

Kartın alt kısmı, deponuzda yapmanız gereken ayarı gösterir. Yazılmaz, yalnızca gösterilir; çünkü o dosya sizindir.

İki tipik sorun:

- Vite, yapılandırmasının tanımadığı bir alan adına 403 döner. Alan adını izin listesine ekleyin.
- Sıcak yenileme istemcisine tarayıcının gerçekte hangi portta olduğu söylenmelidir. Proxy'nin arkasında bu 443'tür, geliştirme sunucusunun kendi portu değil.

Kart yapılandırmanızı okur ve bu iki noktanın karşılanıp karşılanmadığını söyler.

## Bilinmesi gerekenler

- `package.json` içinde Vite, Nuxt ya da Next bulunamazsa verilecek öneri yoktur. Kaynak bağlaması yine de çalışır.
- Geliştirme kipi açık ama konteyner kaynak bağlaması olmadan oluşturulmuşsa kart bunu söyler; projeyi yeniden ayağa kaldırın.
