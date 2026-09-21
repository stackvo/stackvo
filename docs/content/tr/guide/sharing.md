# Paylaşım ve ekip

Makinenizden dışarı dört yol, dört farklı ihtiyaç için.

<figure markdown>
![Bir projenin Yayın sekmesi](../screenshots/web/project-detail-release.webp){ loading=lazy }
<figcaption>Paylaşım: herkese açık tünel, LAN adresi, üretim imajı ve devcontainer.</figcaption>
</figure>

## Herkese açık URL

**Proje → Paylaş → Genel URL al**. Tünel istemcisi yan konteyner
olarak çalışır ve dışarı bağlanır; bu makinede port açılmaz. Webhook
gönderenler ve `.loc` alan adına ulaşamayan dış servisler için.

Dokuz sağlayıcı: Cloudflare (anonim ve adlı), ngrok, Tailscale, zrok, Pinggy,
localtunnel, localhost.run, LocalXpose.

| Tür | Adres | Hesap |
| --- | --- | --- |
| Anonim hızlı tünel | Her başlatmada değişir | Gerekmez |
| Adres tutan sağlayıcı | Aynı kalır | Gerekir; token işletim sisteminin anahtar deposuna gider |

**Parola iste** bağlantının önüne temel kimlik doğrulaması koyar. **Durdur**
yan konteyneri indirir, adres anında çalışmayı bırakır.

## Aynı Wi-Fi'daki telefon

**Proje → Bu ağda** — başka cihazların çözebileceği bir ad, ağdan hiçbir şey
çıkmaz, telefonda kabul edilecek tek sertifika uyarısı. Bkz. [Alan adları ve
HTTPS](domains-https.md).

## İş arkadaşına aynı yığın

`stackvo.json` *hangi* servisleri söyler. Hangi **sürümleri** söyleyemez,
çünkü onlar `.env` içinde yaşar — kimsenin commit'lemediği tek dosya. O yarı
bir **preset**'tir: `stackvo.preset.json`, manifest'in yanında, depoda.

```json
{
  "services": { "mysql": { "enabled": true, "version": "8.4" },
                "redis": { "enabled": true, "version": "7.2" } },
  "settings": { "DEFAULT_TLD_SUFFIX": "loc" }
}
```

**Ayarlar → Çalışma alanı**'ndan dışa aktarın. İş arkadaşınız klonlar, projeyi
açar ve **Servisler** kartı bir preset olduğunu söyleyip **önce planı
gösterir** — başkasının klonuyla gelen bir dosya, bir sayfa açıldı diye bir
yığını yeniden yazmamalı. Preset asla gizli bir değer taşıyamaz; şemada
koyacak yer yok.

## StackVo'su olmayan biri

**Proje → Devcontainer → Dosyaları yaz** projeye bir `.devcontainer/` koyar;
iş arkadaşınız depoyu VS Code ya da GitHub Codespaces'te açıp aynı konteyneri
alır: aynı PHP sürümü, eklentiler ve web sunucusu, aynı sürümlerde aynı
servisler. Parolalar gitignore'lu bir `.devcontainer/.env` dosyasından okunan
adlar olarak çıkar. Dosyalar commit'lenmek içindir ve her seferinde
manifest'ten yeniden yazılır.

## Yayına aldığınız imaj

**Proje → Üretim imajı**, projenin zaten çalıştırdığı imajdan yayınlanabilir
bir imaj kurar — aynı PHP, aynı eklentiler, aynı web sunucusu; kodunuz içinde,
Xdebug dışarıda. **Denetle** doğrular, **Gönder** bir kayıt defterine yollar,
**Dağıtım tarifi** onu çalıştıran bir compose dosyası verir. Yalnızca
doğrulanmış imaj gönderilir, yalnızca kayıt defteri adı taşıyan bir etikete.
