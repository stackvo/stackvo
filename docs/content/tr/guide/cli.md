# Komut satırı

Pencerenin yaptığı her şeyin bir `stackvo` komutu var. Uygulamayı kullanmak
için gerekmez — CLI betikler, CI adımları ve `cd` ile girdiğiniz projede
çalıştırmak istediğiniz tek komut içindir.

<figure markdown>
![Terminal sekmesi](../screenshots/web/project-detail-terminal.webp){ loading=lazy }
<figcaption>Terminal sekmesi: projenin konteynerinin içinde bir kabuk.</figcaption>
</figure>

## PATH'e koymak

`stackvo` ve `stackvo-mcp` uygulamanın içinde gelir. **Ayarlar → Araçlar →
Ekle** onları uygulamanın kendi dizinine bağlar ve kabuğunuzun başlangıç
dosyasına tek satır yazar. **Kaldır** satırı geri alır.

## Günlük komutlar

```bash
stackvo status                        # projeler ve servisler
stackvo up shop / down shop           # başlat / durdur
stackvo restart shop
stackvo logs shop --follow            # canlı loglar
stackvo open shop                     # tarayıcıda aç
stackvo doctor                        # neyin bozuk olduğu ve çözümü
stackvo tui                           # tam ekran terminal arayüzü
```

## Projenin konteynerinde

Bir projeye `cd` ile girin ve yazın:

```bash
stackvo php -v            # projenin PHP'si, PHP'si olmayan bir makinede
stackvo artisan migrate --force
stackvo composer install
stackvo npm run build
stackvo wp plugin list    # ayrıca console, rails, bundle, yarn, pnpm
stackvo python -V         # ve ruby, go, cargo, bun, deno
stackvo shell             # konteynerde etkileşimli kabuk
stackvo exec <program>    # gerisi
```

Komut adından sonraki her şey olduğu gibi aktarılır ve çıkış kodu geçirilir —
`stackvo artisan test` komutunu bir CI betiğinde anlamlı kılan budur.

## Betiklerin güvenebileceği şeyler

Her komutta `--json`, cevap stdout'ta, anlatım stderr'de, her biri tek anlama
gelen çıkış kodları. Tablo [başvuruda](../reference/cli.md); güvence, terminalde
gördüğünüz tablonun aynı değerden üretilmesidir, ikisi birbirinden ayrılamaz.

## TUI

`stackvo tui` terminaldeki penceredir: projeler, servisler, loglar, başlat ve
durdur — SSH ile ulaştığınız bir makine için, ya da tercihen.
