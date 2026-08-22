# Nerede geçerli

Her sunucu bir dosya üzerinden yapılandırılmaz. Bu kart, yukarıdaki ayarların hangi sunucuya yazılabildiğini söyler.

## Durum

| Sunucu | Durum |
| --- | --- |
| nginx | İstek sınırları ve ek yönergeler yazılır. |
| Caddy | İstek sınırları ve ek yönergeler yazılır. |
| FrankenPHP | Yalnızca ek yönerge yazılabilir; Caddyfile'ı istek sınırlarını taşımaz. |
| Apache | Kendi Dockerfile'ı içinde yapılandırılır; yönerge eklenecek dosyası yoktur. |
| Swoole | Satır içi bir betikle yapılandırılır; yönerge eklenecek dosyası yoktur. |

## Bilinmesi gerekenler

- Desteklenmeyen bir sunucuda ayarları değiştirmek hiçbir şey yapmaz. Kart bunu önceden söyler ki sessizce etkisiz kalmasın.
- Sunucu seçimi Proje varsayılanları bölümündedir; her proje kendi sunucusunu da seçebilir.
