# Projeler

Proje, bir çalışma zamanı olan bir klasördür; kendi konteynerinde çalışır,
kendi adresinde cevap verir. Onunla ilgili her şey **Projeler** sayfasında ve
projenin kendi sayfasındadır.

<figure markdown>
![Bir projenin ayrıntı sayfası](../screenshots/web/project-detail.webp){ loading=lazy }
<figcaption>Bir proje: konteyneri, servisleri, alan adı ve araçları tek sayfada.</figcaption>
</figure>

## Satır

| Eylem | Ne yapar |
| --- | --- |
| **Başlat / Durdur** | Konteyneri ayağa kaldırır ya da indirir. İki durumda da dosyalarınıza dokunulmaz. |
| **Yeniden başlat** | Aynı konteyneri durdurup başlatır. |
| **Yeniden kur** | Dockerfile'ı yeniden üretir, imajı kurar, konteyneri yeniden yaratır. |
| **Terminal** | Konteynerin içinde bir kabuk. |
| **Tarayıcıda aç** | Projenin adresi. |
| **Sil** | Konteyneri ve kaydı kaldırır. **Diskteki klasörünüz kalır.** |

Üç nokta menüsü yalnızca şu an geçerli eylemi gösterir: hiç kurulmamış bir
proje Başlat değil **Kur** der.

## Yeniden başlat mı, yeniden kur mu?

| Değiştirdiğiniz | Basın |
| --- | --- |
| Kod | Hiçbir şey. Klasörünüzden bağlanır; sayfayı yenileyin. |
| Ortam değişkeni, bir ayar | **Yeniden başlat** |
| PHP sürümü, bir eklenti, web sunucusu | **Yeniden kur** — imajın kendisi değişti |

Uygulama hangisinin gerektiğini söyler. Yeniden kurulum gerektiren bir
değişiklikten sonra satırdaki **Yapılandırma** sütunu, siz basana kadar
üretilen dosyaların eski olduğunu söyler.

## PHP sürümünü değiştirmek

**Proje → Yapılandırma → Yapılandır**, sürümü seçin, kaydedin. Uygulama imajın
yeniden kurulması gerektiğini söyler ve tek düğmeyle kurar.

Aynı değişiklik dosyadan da gelebilir. Deponuzdaki `stackvo.json`'ı düzenleyin,
uygulama fark eder:

```jsonc
"php": { "version": "8.1" }   // 8.4 idi
```

Eklentiler aynı şekilde: `"extensions": ["redis", "intl", "gd"]`. Seçilen PHP
sürümünde kurulamayan bir eklenti, kaydetmeden önce panelde işaretlenir.

## Manifest, `stackvo.json`

Projeyi anlatan tek dosya ve elle düzenlemeniz beklenen tek dosya.
Commit'leyin, takım arkadaşınız aynı ortamı alır.

```json
{
  "name": "shop",
  "framework": "laravel",
  "php": { "version": "8.4", "extensions": ["redis", "intl", "gd"] },
  "server": "nginx",
  "domain": "shop.loc",
  "services": ["mysql", "redis", "mailpit"]
}
```

Gerisi — Dockerfile, compose parçası, yönlendirme — her seferinde bundan
üretilir. Onları asla düzenlemeyin; manifest'i değiştirin. **Proje → Manifest**
dosyayı metin olarak gösterir ve kaydederken doğrular: sözleşmeyi bozan bir
dosya reddedilir ve anahtar adıyla söylenir.

## Konteyner için ortam değişkenleri

**Proje → Proje ayarları** konteynere verilen değişkenleri tutar.
`.stackvo/site.json` içinde saklanır, depoyla birlikte gider. Uygulamanızın
`.env` dosyasına *yazılmaz*; o dosya çerçevenindir.

## Kendi komutlarınız

Projenizin sık ihtiyaç duyduğu bir komut — reindex, seed, ne olursa —
manifest'te yaşar ve **Proje → Komutlar**'da düğme olarak görünür:

```json
"commands": {
  "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Arama dizinini yeniden kur" }
}
```

Komutlar projenin konteynerinde çalışır, başka yerde değil. *Her* projede
çalıştırdıklarınız için **Ayarlar → Makine geneli komutlar** aynı biçimi çalışma
alanının kökündeki tek `commands.json` içinde alır; aynı kimliği bildiren
proje kazanır.

## Tek proje olarak monorepo

**Proje → Bu deponun geri kalanı**, diğer dizinleri kendi çalışma zamanlarıyla
bileşen olarak ekler — `api/` Go, `web/` Next.js, `worker/` Python. Tek kayıt,
tek başlatma, tek sertifika; her bileşen kendi Dockerfile'ını ve
yönlendirmesini alır, hiçbiri host'ta port açamaz.
