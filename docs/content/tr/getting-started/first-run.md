# İlk çalıştırma

İlk açılış adım adım: tek soru, güvenilecek bir sertifika ve kendi adresinde açılan ilk proje. On dakika; çoğu ilk imaj indirmesi.

<figure markdown>
![Yeni proje paneli](../screenshots/web/project-new.webp){ loading=lazy }
<figcaption>Yeni proje paneli: ad, alan adı, çalışma zamanı ve servisler tek formda.</figcaption>
</figure>

## 1. Çalışma alanı seçin

İlk açılış tam olarak tek soru sorar: **çalışma alanı nerede olsun?** Boş bir
klasör gösterin; uygulama gerekli her şeyi kendisi yazar. Varsayılan
`~/.stackvo`; `STACKVO_ROOT` ortam değişkeni bunu değiştirir.

```text
~/.stackvo/
├── .env                    yığının ayarları (servisler, portlar, TLD)
├── generated/              üretilen compose ve Dockerfile dosyaları — silinebilir
├── certs/                  sertifika
└── projects/
    └── shop/
        ├── stackvo.json    manifest — elle düzenlenen tek dosya
        └── Dockerfile      üretilir
```

Veritabanı yok. **Durum klasörün kendisidir.** `generated/` istediğiniz an
silinebilir, gerektiğinde yeniden üretilir. Mevcut bir StackVo çalışma alanı
olduğu gibi kullanılır, içindeki hiçbir şey değiştirilmez.

## 2. Sertifikaya güvenin

StackVo her projeyi ve servisi kapsayan tek bir joker sertifika üretir.
Tarayıcınızın kabul etmesi için, imzalayan otoriteye bu makinede bir kez
güvenilmesi gerekir.

**Ayarlar → Sertifikalar → CA'ya güven (terminalde)**. macOS'ta bu terminalinizi açar
ve komutu çalıştırır, çünkü güven ayarları orada yalnızca etkileşimli
değiştirilebilir. Sonra tarayıcıyı tamamen kapatıp yeniden açın — açık bir
tarayıcı eski güven listesini kullanmaya devam eder.

!!! note "Firefox"
    Firefox kendi güven deposunu taşır. Yalnızca makinede `certutil` varsa
    doldurulur; yoksa Safari ve Chrome yeşil, Firefox hâlâ uyarır. Kart her
    depoyu ve ne yapılacağını söyler — `certutil` sağlayan `nss` paketini
    kurun ve güven adımını yeniden çalıştırın.

## 3. Proje oluşturun

**Projeler → +** yeni proje panelini açar. Üç yol var:

| Başlangıç | Ne olur | Ne zaman |
| --- | --- | --- |
| **Çerçeve şablonu** | Çerçevenin kendi yükleyicisi geçici bir konteynerde çalışır, sonuç benimsenir. Laravel, WordPress, Symfony, Next.js ve daha fazlası. | Sıfırdan başlıyorsunuz. |
| **Boş proje** | Formdaki değerlerden bir proje. Yükleyici çalışmaz. | Kodu siz getireceksiniz ya da bir iskeletiniz var. |
| **Git'ten klonla** | Depoyu klonlar ve geleni benimser. | Kod zaten var. |

Bir ad verin — `shop` — alan adı türetilir: `shop.loc`. Şablonda çalışma
zamanı, web sunucusu ve belge kökü yükleyicinin **gerçekten yazdığından**
okunur, o alanlar kaybolur. Boş projede siz seçersiniz: nginx üstünde PHP 8.4,
ya da Node 22, ya da katalogdan başka bir çalışma zamanı.

!!! tip "İlk şablon birkaç dakika sürer"
    Yükleyicinin imajı bir kez indirilir. Sonra aynı türden yeni proje hızlıdır.

## 4. Oluştur'a basın

Düğmenin arkasında:

```text
stackvo.json  ──►  üretici  ──►  Dockerfile + compose parçası + Traefik yönlendirmesi
                                   │
                                   ├─ konteyner ayağa kalkar
                                   ├─ hosts dosyanıza bir satır — önce fark gösterilir, sonra tek parola sorulur
                                   └─ sertifika yeni alan adını kapsar  →  https://shop.loc
```

Kurulum Docker'ın kendi çıktısını akıtır; terminalde göreceğiniz satırların
aynısını okursunuz. Hiçbir şey tahmin edilmez.

## 5. Açın

Proje satırındaki alan adına tıklayın. Tarayıcı `https://shop.loc` adresini
sertifika uyarısı olmadan açar.

## Az önce ne oldu

| Şey | Nerede | Neden önemli |
| --- | --- | --- |
| `stackvo.json` | `projects/shop/` | Projeyi anlatan tek dosya. Commit'leyin, takım arkadaşınız aynı ortamı alır. |
| Dockerfile, compose parçası | `generated/` | Her seferinde manifest'ten üretilir. Elle düzenlemeyin; manifest'i değiştirin. |
| Bir konteyner | Docker | Projenin PHP'si ya da Node'u, web sunucusu, eklentileri. |
| Bir hosts satırı | Hosts dosyanız | `shop.loc` → bu makine. |
| Sertifika | `certs/` | Yeni alan adını kapsayacak şekilde yeniden üretildi. |

## Çalıştığını kontrol edin

- :material-check-circle-outline: Proje satırı **Çalışıyor** diyor.
- :material-check-circle-outline: Alan adı HTTPS ile uyarısız açılıyor.
- :material-check-circle-outline: **Proje → Genel Bakış** seçtiğiniz çalışma zamanı sürümünü gösteriyor.

Sonraki adım: [Günlük kullanım](../guide/everyday-use.md) — yedi sayfa ve
nerede ne var.
