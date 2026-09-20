# Komut satırı

`stackvo <komut> [argümanlar] [bayraklar]`. Buradaki her komut pencerede de
var; CLI aynı çekirdeğin terminalden hâli. Komutlar `stackvo --help` çıktısıyla
aynı gruplarda.

## Okumalar

Hiçbir şey değişmez. Her an güvenle çalıştırılır.

| Komut | Ne cevaplar |
| --- | --- |
| `status` | Bir şeyin çalışıp çalışmayacağı: çalışma alanı, motor, her başlangıç gereksinimi, kaç proje açık. |
| `doctor` | Tam tanı: gereksinimler, tutan programıyla port çakışmaları, eksik hosts satırları, eskimiş üretilmiş dosyalar, disk. |
| `verify <proje>` | Bu makinenin deponun bildirdiği ihtiyaçlarla eşleşip eşleşmediği — ve hangi satırın eşleşmediği. |
| `projects` | Her yönetilen proje, alan adı ve açık olup olmadığıyla. |
| `project <proje>` | Bir proje bütünüyle: manifest, konteyner, Xdebug, PHP. |
| `services` | Paylaşılan servisler ve sağlıkları. |
| `logs <kimlik>` | Bir konteynerin çıktısı, proje ya da servis için. |
| `certs` | Sertifika: neyi kapsıyor, neyi kapsamıyor, ne zaman doluyor. |
| `db` | Veritabanı servisleri, veritabanları, çalışıp çalışmadıkları. |
| `mail` | Mail yakalayıcının gelen kutusu. |
| `mcp` | Hangi asistanlarda `stackvo-mcp` kayıtlı. |
| `rules` | Hangi yapay zekâ kural dosyaları StackVo bloğunu taşıyor. |
| `tools` | `stackvo` nereden kurulu, hangi kabuk dosyaları PATH satırını taşıyor. |
| `ide <proje>` | IDE'nin adım adım hata ayıklama için istedikleri ve bir şeyin dinleyip dinlemediği. |
| `spx <proje>` | Örnekleyici profilleyici: kurulu, bağlı, açık. |
| `spx-top <proje>` | Bir kaydın zamanı nerede geçirdiği. |

## Yığını değiştirenler

| Komut | Ne yapar |
| --- | --- |
| `up` | Yığını kaldırır. Eksik imajları kurar, ilk çalıştırma dakikalar sürer. |
| `down` | Bütün yığını indirir: her profil, projeler dâhil. |
| `start <proje>` | Bir projenin konteynerini başlatır, sonra post-start kancalarını çalıştırır. |
| `stop <proje>` | Pre-stop kancalarını çalıştırır, sonra konteyneri durdurur. |
| `restart <proje>` | Durdurup başlatır, iki uçta da kancalarla. |
| `generate` | Compose dosyalarını, Dockerfile'ları ve ayarları manifest'lerden yeniden üretir. |
| `lock <proje>` | `stackvo.lock` yazar: bu makinenin çalıştırdığı servis sürümleri ve paket özetleri. |
| `xdebug <proje> on\|off` | Adım adım hata ayıklamayı açar ya da kapatır. İlk `on` yeniden kurulum ister. |
| `certs-renew` | Projelerin alan adları için sertifikayı yeniden üretir. |
| `mcp-install <asistan>` / `mcp-remove` | `stackvo-mcp`'yi bir asistana kaydeder ya da girdiyi çıkarır. `stackvo mcp` kimlikleri listeler. |
| `rules-install <dosya>` / `rules-remove` | Yapay zekâ kural bloğunu bir dosyaya yazar ya da çıkarır. |
| `path-install` / `path-remove` | Komutları PATH'e bağlar ya da satırı geri alır. |
| `tool-install <araç>` / `tool-remove` | Bir host aracını — bugün `mkcert` — uygulamaya gömülü özetle doğrulayarak indirir. |
| `ide-install <proje> <ide>` | Hata ayıklama yapılandırmasını o projede bir IDE'nin dosyasına yazar. |
| `spx-record <proje> <yol>` | Tarayıcı olmadan bir isteğin profilini çıkarır. |
| `spx-build <proje>` | php-spx'i projenin PHP sürümü için geçici konteynerde derler. |
| `market-bundle <dizin>` | Kataloğu ve her paketi tek dizine yazar, ağı olmayan makine için. |

## Ekranlar

| Komut | Ne yapar |
| --- | --- |
| `tui` | Pencere, terminalde. |

## Projenin konteynerinde

Proje klasörünün içinden çalıştırılır. Komut adından sonraki her şey olduğu
gibi aktarılır, çıkış kodu geri gelir.

| Komut | Çalıştırdığı |
| --- | --- |
| `php`, `composer`, `artisan`, `console` | PHP ve araçları |
| `npm`, `yarn`, `pnpm`, `node`, `bun`, `deno` | JavaScript çalışma zamanları ve paket yöneticileri |
| `python`, `ruby`, `bundle`, `rails`, `go`, `cargo` | Diğer çalışma zamanları |
| `wp` | WP-CLI |
| `shell` | Etkileşimli kabuk |
| `exec <program> …` | Gerisi. Denetim kaydına yazılır. |

`stackvo artisan --help` artisan'a gider; uygulamanın kendi yardımı için
`--help` önce yazılır.

## Kabuk tamamlama

`stackvo completions <kabuk>` bash, zsh, fish ya da PowerShell için parçayı
basar. `path-install` bunu kurar.

## Betiklerin güvenebileceği sözleşme

| Kural | Ayrıntı |
| --- | --- |
| `--json` | Her komutta. Gördüğünüz tablo o değerden üretilir, ikisi ayrışamaz. |
| stdout / stderr | Cevap stdout'ta, anlatım stderr'de. Başarısızlık stdout'u boş bırakır. |
| Bilinmeyen bayraklar | Hata, asla yutulmaz. |

| Çıkış kodu | Anlamı |
| --- | --- |
| `0` | Tamam |
| `1` | Başarısız |
| `2` | Hatalı komut satırı |
| `3` | Bu makinede kurulu çalışma alanı yok |
| `4` | Docker'a ulaşılamıyor |
| `127` | Projede olmayan bir çalışma zamanı |
