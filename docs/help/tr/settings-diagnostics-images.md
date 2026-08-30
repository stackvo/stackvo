# Bu uygulamanın çektiği imgeler

StackVo'nun çalıştırdığı ama derlemediği konteynerler: tünel sağlayıcıları, karşılama sayfası, tünel muhafızı ve performans yardımcısı.

## Bu liste neden var

StackVo, sürümü hareketli bir etiket olan bir **paketi** kurmayı reddediyor — `latest`, `stable`, `edge`, `main`, `master` — çünkü sabit bir manifestin altında değişen bir imgenin geri dönebileceğiniz bir sürümü yoktur.

Bu listedeki imgelerin altısı `latest` üzerinde. Kural, bu uygulama dışında herkese uygulanıyordu. Bunlardan birinin yayıncısı bozuk bir derleme gönderirse, siz hiçbir şeyi değiştirmeden, bir sonraki konteyner başlangıcında makinenize gelir.

Bu liste, onu düzeltmenin ilk yarısı: artık hangilerinin hareketli olduğunu görebiliyorsunuz.

## Birini sabitlemek

Bir yöneticinin politika dosyası herhangi birini depo adına göre sabitleyebilir:

```json
{
  "schemaVersion": 1,
  "imagePins": {
    "cloudflare/cloudflared": "cloudflare/cloudflared:2024.8.2",
    "nginx": "nginx@sha256:…"
  }
}
```

- **En güçlü biçim digest**; sabit bir etiket olağan olanı. İkisi de kabul ediliyor, çünkü etiketi reddetmek çoğu kişinin gerçekten üretebileceği cevabı reddetmek olurdu.
- **Sabitleme aynı depoyu adlandırmalı.** `"nginx": "alpine:3"` bir sabitleme değil, bir yazım hatasıdır; uygulanmak yerine reddedilir ve bildirilir — sessizce başka bir şey çalıştıran bir sabitleme, düzeltmeye çalıştığı hareketli etiketten kötü olurdu.
- **Sabitleme kayıt defteri önekinden önce uygulanır**, yani Docker Hub'ı aynalayan bir makinede de çalışır.

## Bilinmesi gerekenler

- Etiketler uygulamanın kendi içinde bilerek sabitlenmedi. Bir sabitleme seçmek, var olan bir sürümü adlandırmak demektir; bu, yayın anında bir kayıt defterine karşı doğrulanacak bir şeydir, kaynak dosyanın iddia edebileceği bir şey değil.
- **Hareketli etiket**, gerçekten çalışacak referansın hâlâ öyle bir etikette olduğu anlamına gelir. Sabitlediğiniz bir satır işaretlenmeyi bırakır.
- Nereden geldikleri işin öbür yarısı ve aynı dosyada: `registryPrefix` onları kurumunuzun aynasına yönlendirir.
