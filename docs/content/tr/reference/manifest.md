# Manifest

`stackvo.json` proje klasöründe durur ve projeyi anlatır. Elle düzenlemeniz
beklenen tek dosyadır; gerisi ondan üretilir. Sözleşme depodaki
`contracts/project.schema.json`; bu sayfa o sözleşmenin düz sözlerle hâli.

İki alan zorunlu: `name` ve `domain`. Gerisinin varsayılanı var.

## Üst düzey

| Alan | Tip | Varsayılan | Ne söyler |
| --- | --- | --- | --- |
| `name` | metin | — | Projenin kimliği. Klasör adıyla aynı olmalı. Küçük harf, rakam, nokta, tire, alt çizgi. |
| `domain` | metin | — | Projenin açıldığı adres. Gelenek: `<ad>.loc`. Hosts satırı gerekir; uygulama yazar. |
| `runtime` | `php` `node` `python` `go` `ruby` `rust` `bun` `deno` | `php` | Projeyi ne çalıştırır. Yoksa PHP. |
| `server` | `nginx` `apache` `caddy` `frankenphp` `swoole` `roadrunner` | `nginx` | PHP için web sunucusu. Diğer çalışma zamanlarında yok sayılır; Traefik doğrudan uygulamanın portuna gider. |
| `document_root` | metin | `public` | Web sunucusunun yayımladığı klasör, projeye göre. Başta bölü yok. |
| `aliases` | metin listesi | `[]` | Aynı projenin cevap verdiği ek adlar. `*.shop.loc` jokerdir: sertifikaya ve yönlendirmeye girer, hosts dosyasına girmez. |
| `lan_share` | boolean | `false` | Ağdaki diğer cihazların çözebileceği bir adda da cevap ver, sslip.io üzerinden. |
| `services` | kimlik listesi | `[]` | Projenin ihtiyaç duyduğu servisler — `mysql`, `redis`, `mailpit` — katalog kimliğiyle. Ortamın klonlayan iş arkadaşına giden yarısı. |
| `commands` | nesne | — | Projenin düğme olarak sunduğu komutlar. Aşağıda. |
| `hooks` | nesne | `{}` | Kurulumdan sonra, başlatmadan sonra, durdurmadan önce çalışacak komutlar. |
| `schedule` | liste | `[]` | Zamanlayıcıdaki adlı işler, her birinin kendi logu. |
| `processes` | nesne | `{}` | Container'ın kendi supervisord'u altında uzun ömürlü süreçler — kuyruk işçisi, zamanlayıcı. Buradan üretildikleri için yeniden derlemede kaybolmazlar. |
| `components` | nesne | — | Bu deponun diğer dizinleri, her biri kendi çalışma zamanı ve adresiyle. |
| `sidecars` | nesne | — | Projenin ihtiyaç duyduğu ama katalogda olmayan konteynerler. Projeyle gelir, projeyle gider. |
| `providers` | nesne | — | Proje verisinin gerçekte yaşadığı adlı yerler ve oradan nasıl çekilip geri gönderileceği. |

## Çalışma zamanı bloğu

Çalışma zamanının adını taşıyan tek blok. PHP için `php`, Node için `node`,
diğerleri için `python`, `go`, `ruby`, `rust`, `bun` ya da `deno`. İçindeki
her alan isteğe bağlı; olmayan alan ekosistemin varsayılanını alır.

=== "php"

    ```json
    "php": {
      "version": "8.4",
      "extensions": ["redis", "intl", "gd"],
      "xdebug": false
    }
    ```

    | Alan | Ne söyler |
    | --- | --- |
    | `version` | `5.6`'dan `8.5`'e. Varsayılan `8.4`. |
    | `extensions` | İmaja derlenir. Seçilen sürümde kurulamayan, kaydetmeden önce işaretlenir. |
    | `xdebug` | Eklentinin imajda olup olmadığı. Xdebug kartından değişir. |

=== "node"

    ```json
    "node": {
      "version": "22",
      "package_manager": "pnpm",
      "install": "pnpm install",
      "build": "pnpm build",
      "start": "pnpm start",
      "port": 3000
    }
    ```

    | Alan | Ne söyler |
    | --- | --- |
    | `version` | `16`'dan `23`'e. Varsayılan `22`. |
    | `package_manager` | Corepack'i açar; `package.json` içindeki `packageManager` sürüm sabitler. |
    | `install`, `build`, `start` | Üç komut. `build` boş olabilir. |
    | `port` | Uygulamanın konteynerde dinlediği port. `0.0.0.0` adresine bağlanmalı. |

=== "python, go, ruby, rust, bun, deno"

    ```json
    "python": { "version": "3.12", "install": "pip install -r requirements.txt", "start": "python app.py", "port": 8000 }
    ```

    Aynı beş alan — `version`, `install`, `build`, `start`, `port` — yoksa
    ekosistemin varsayılanlarıyla.

## Komutlar

```json
"commands": {
  "reindex": {
    "exec": ["php", "artisan", "app:reindex"],
    "about": "Arama dizinini yeniden kur",
    "interactive": false
  }
}
```

| Alan | Ne söyler |
| --- | --- |
| kimlik (anahtar) | Küçük harf, rakam ve tire. Düğmenin adı. |
| `exec` | Program ve argümanları, liste olarak. Kabuk yok, hiçbir şey yorumlanmaz. |
| `about` | Düğmenin altındaki tek satır. |
| `interactive` | Terminal gerekip gerekmediği. |

Komutlar projenin konteynerinde çalışır, başka yerde değil.

## Kancalar

```json
"hooks": {
  "post-build": [["composer", "install"]],
  "post-start": [["php", "artisan", "migrate", "--force"]],
  "pre-stop": []
}
```

Her biri konteynerde çalışan komut listesidir. `pre-start` bilerek yok:
başlatmadan önce çalışacak konteyner yok.

## Zamanlama

```json
"schedule": [
  { "label": "Gece raporu", "cron": "0 2 * * *", "exec": ["php", "artisan", "report:nightly"], "enabled": true }
]
```

Adlı işler, her birinin kendi son çalışması ve kendi logu — tek ya hep ya hiç
zamanlayıcı süreci yerine.

## Süreçler

```json
"processes": {
  "scheduler": { "exec": ["php", "artisan", "schedule:work"] },
  "queue": {
    "exec": ["php", "artisan", "queue:work", "rabbitmq", "--queue=photos,default", "--tries=3"],
    "replicas": 2,
    "stopWait": 30
  }
}
```

Uzun ömürlü süreçler; projenin container'ında `php-fpm` ve web sunucusunun
yanında zaten çalışan `supervisord` tarafından yürütülür. Her anahtar, üretilen
`supervisord.conf` içinde bir `[program:<id>]` bloğu olur.

Bu dosya **her yeniden derlemede sıfırdan üretilir**; bloğun var olma sebebi
budur: container içinde elle eklenen ya da `conf.d/` altına bırakılan bir
program bir sonraki derlemede kaybolur. Burada tanımlandığında yeniden derleme
onu geri getirir ve klonlayan ekip arkadaşı da alır. Supervisor paneli bunları
imajın kendi iki sürecinin yanında listeler; manifest, çalışan daemon'da henüz
olmayan bir şey tanımlıyorsa yapılandırmayı derlemeden uygulamayı önerir.

| Alan | Varsayılan | Ne söyler |
| --- | --- | --- |
| `exec` | — | Program ve argümanları, dizi olarak. `/var/www/html` içinde çalışır. Kabuk yok. |
| `enabled` | `true` | `false` girdiyi dosyada tutar, daemon'ın dışında bırakır. |
| `replicas` | `1` | Kaç kopya. Birden fazlaysa tek grupta `<id>_00`, `<id>_01`, … olur. |
| `stopWait` | `10` | SIGTERM sonrası supervisord'un öldürmeden önce beklediği saniye. İş ortasındaki kuyruk işçisi daha fazlasını ister. |

Yalnızca `nginx` ve `caddy` projeleri supervisord çalıştırır; diğer sunucularda
blok saklanır ve hiçbir şey yapmaz. `php-fpm`, `nginx` ve `caddy` ayrılmış
kimliklerdir. Loglar container'ın stdout'una, yani Loglar sekmesine gider.

`schedule` ile karıştırmayın: o bir komutu zamanlayıcıyla başlatır ve çıkmasını
bekler. Workers paneli de Laravel'in sabit işçi komutlarını yan container'larda
çalıştırır. Buradaki süreç herhangi bir komuttur ve container ayakta olduğu
sürece çalışır tutulur.

## Bileşenler

```json
"components": {
  "api":    { "runtime": "go",     "path": "api",    "port": 8080 },
  "web":    { "runtime": "nodejs", "path": "web",    "port": 3000 },
  "worker": { "runtime": "python", "path": "worker" }
}
```

Tek proje olarak monorepo. Her bileşen kendi Dockerfile'ını, compose
servisini ve yönlendirmesini alır; `domain` aksini söylemedikçe adres
`<bileşen>.<alan adı>`. `version`, `install`, `build` ve `start` çalışma
zamanı bloğundaki gibi çalışır. Hiçbir bileşen host'ta port açamaz.

## Yan konteynerler

```json
"sidecars": {
  "chromium": { "image": "selenium/standalone-chromium:latest", "about": "Dusk için tarayıcı", "env": { "SE_NODE_MAX_SESSIONS": "2" } }
}
```

Projenin yanında, kataloğun sağlamadığı bir konteyner. Projenin kendi compose
bloğuna yazılır, projeyle kalkar ve iner, paylaşılan servis değildir.

## Tam örnek

```json
{
  "name": "shop",
  "domain": "shop.loc",
  "aliases": ["admin.shop.loc"],
  "runtime": "php",
  "server": "nginx",
  "document_root": "public",
  "php": { "version": "8.4", "extensions": ["redis", "intl", "gd"] },
  "services": ["mysql", "redis", "mailpit"],
  "commands": {
    "reindex": { "exec": ["php", "artisan", "app:reindex"], "about": "Arama dizinini yeniden kur" }
  },
  "hooks": { "post-start": [["php", "artisan", "migrate", "--force"]] }
}
```

## Yanındaki iki dosya

| Dosya | Ne taşır | Commit'lenir mi? |
| --- | --- | --- |
| `.stackvo/site.json` | Konteyner için ortam değişkenleri, dizin listeleme, SSH agent iletme. | Evet, depoyla gider. |
| `stackvo.preset.json` | Servis **sürümleri** ve `.env`'de yaşayan paylaşılabilir ayarlar. Asla gizli değer taşıyamaz. | Evet — bkz. [Paylaşım ve ekip](../guide/sharing.md). |
