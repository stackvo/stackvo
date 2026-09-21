# Node.js

`https://app.loc` adresinde bir Next.js, Nuxt, SvelteKit, Astro, Vite ya da
NestJS projesi; dev sunucusu kaynağınıza karşı çalışır, üretim derlemesi tek
anahtarla hazırdır.

## 1. Oluşturun

**Projeler → +**, sonra şablonlar altından çerçeve (**Next.js**, **Nuxt**,
**Vue (Vite)**, **React (Vite)**, **SvelteKit**, **Astro**, **Angular**,
**NestJS**) ya da başka her şey için **Node** çalışma zamanıyla **Boş
proje**. Adı `app` olsun.

Boş projede form üç komut ve bir paket yöneticisi sorar:

| Alan | Tipik değer |
| --- | --- |
| Paket yöneticisi | `npm`, `pnpm`, `yarn` ya da `bun`. Corepack'i açar; `package.json` içindeki `packageManager` sürümü sabitler. |
| Kurulum komutu | `npm ci` |
| Derleme komutu | `npm run build`; isteğe bağlı |
| Başlatma komutu | `npm run start` |

Şablon bunları çerçevenin kurucusunun yazdıklarından doldurur.

## 2. Dev sunucusu

İmaj, kodunuzun derleme anında alınmış bir kopyasını taşır; bir dosyayı
düzenlemek yeniden derleyene kadar hiçbir şeyi değiştirmez. **Proje → Dev sunucusu** açıkken kaynağınızı konteynere canlı bağlar ve üretim komutu yerine
dev komutunu (`npm run dev`) sıcak yeniden yüklemeyle çalıştırır.

Kart çerçevenizin yapılandırmasını da okur ve iki şeyin karşılanıp
karşılanmadığını söyler:

- **Sunucu adı.** Vite tanımadığı bir sunucu adı için 403 döner; `app.loc`
  izin listesinde olmalı.
- **Sıcak yeniden yükleme portu.** Proxy'nin arkasında tarayıcı dev
  sunucusunun kendi portunda değil 443'tedir ve istemciye söylenmesi gerekir.

Kart eklenecek satırları gösterir. Yazmak yerine gösterir, çünkü dosya
sizindir.

## 3. Veritabanı, API

Node projesi de servisleri PHP projesi gibi ister: **Katalog → Yayında
olanlar** PostgreSQL, MongoDB ya da Redis kurar, **Proje → Bu projenin
ihtiyaç duyduğu servisler** bildirir ve kart `.env` için bağlantı değerlerini
gösterir. `api/` ve `web/` olan bir monorepo tek projedir; bkz.
[Projeler](../projects.md).

## 4. Üretim

**Proje → Dev sunucusu** kapalıyken bir yeniden derleme, derleme komutunu ve
başlatma komutunu çalıştırır; bir sunucunun çalıştıracağı şey budur. **Proje
→ Üretim imajı** aynı manifestten göndereceğiniz imajı derler.

## Her gün

```bash
stackvo npm install
stackvo pnpm test
stackvo node -v
stackvo shell
```

Projenin klasöründen, konteynerin içinde; ana makinede Node gerekmez.
