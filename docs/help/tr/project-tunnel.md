# Paylaş

Bu projeye yönlendiren geçici bir genel adres. `.loc` alan adına ulaşamayan webhook göndericileri ve dış servisler içindir.

Tünel istemcisi bir yardımcı konteyner olarak çalışır ve dışarı bağlanır. Bu makinede hiçbir port açılmaz.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Sağlayıcı | Tüneli hangi servisin taşıyacağı. Her satır, anahtarın saklı olup olmadığını söyler. |
| Genel adres al | Yardımcı konteyneri başlatır ve adresi gösterir. |
| Durdur | Yardımcı konteyneri indirir; adres anında çalışmaz olur. |
| Kopyala | Adresi panoya alır. |
| Anahtar | Sağlayıcının hesap anahtarını işletim sisteminin kasasında saklar. Bir daha gösterilmez. |
| Parola sorulsun | Bağlantının önüne temel kimlik doğrulaması koyar. Parolayı StackVo üretir ve tekrar gösterebilir — bir jetonun aksine bu parola, bağlantıyı açacak kişiye okunmak zorundadır. |
| Adres | Bu sağlayıcının başlatmalar arasında saklaması istenen ad — saklayabildiği durumlarda. |

## Sağlayıcı seçmek

| Tür | Adres | Hesap |
| --- | --- | --- |
| Anonim hızlı tünel | Her başlatmada değişir | Gerekmez |
| Adresi saklayan sağlayıcı | Sabit kalır | Gerekir |

Değişen adres "webhook geldi mi" için yeterlidir. Bir panoya bir kez kaydedeceğiniz adres için sabit adres gerekir.

## Bağlantının önündeki parola

Açtığınızda bu proje için bir kimlik bilgisi saklanır ve **bir sonraki başlatmadan** itibaren tünel ile proje arasına küçük bir nginx konteyneri girer — böylece tüneli hangi sağlayıcı taşırsa taşısın aynı şekilde çalışır. Zaten çalışan bir tünel parolasız açılmıştır; panel bunu söyler.

Parola işletim sisteminin kasasında durur, çalışma alanında değil, ve bu uygulamanın size tekrar gösterdiği tek sırdır: başkasının cihazındaki bir tarayıcıya yazılması gerekir.

Açıkken `Authorization` başlığı bekçinindir ve uygulamaya ulaşmaz.

## Aynı adresi korumak

Bazı sağlayıcılardan adresi başlatmalar arasında saklaması istenebilir; üçü saklayamaz ve alan sunmak yerine bunu söyler. Ad bir **istektir**: sağlayıcı adı az önceki tünelden hâlâ tutuyorsa sessizce başka bir ad atar, panel de geri gelenin istenen olmadığını söyler.

Adlı Cloudflare tüneli istisnadır ve bu alan doldurulmadan başlamaz — Cloudflare tüneli kendi panelinden yönlendirir, istemci adresi hiç yazmaz, bu yüzden gösterilen adres buraya yazdığınızdır.

## Bilinmesi gerekenler

- Tünel konteynere yönlendirir. Proje durmuşsa adres çalışıyor görünür ama 502 döner.
- Parola yoksa adres çalıştığı sürece geneldir. Eline geçiren herkes yerel projenize ulaşır.
- İlk başlatma sağlayıcının imajını indirir, o yüzden sonrakilerden uzun sürer.
