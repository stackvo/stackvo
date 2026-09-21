# Ayarlar, `.env`

Yığının ayarları `<çalışma alanı>/.env` içinde, satır başına bir `ANAHTAR=DEĞER`.
Her anahtarın tipi ve varsayılanı `contracts/env.schema.json` içinde; bu sayfa
uygulamanın bugün okuduklarını listeler. Dosyayı nadiren düzenlersiniz —
**Ayarlar** sayfası yazar — ama bir iş arkadaşının preset'i ya da bir politika
dosyası bir anahtarı adıyla andığında adları bilmek işe yarar.

## Dosya nasıl okunur

- Boş satırlar ve `#` ile başlayanlar atlanır.
- **İlk** `=` işaretinden bölünür; gerisi değerdir, kırpılır. Değerler asla
  tırnaklanmaz; tırnaklı değer tırnaklarıyla kalır ve yanlıştır.
- Boolean `true` ya da `false`, küçük harf. Listeler virgülle ayrılır.
- Dosya yalnızca bir ayar değişince yazılır. Hiç `.env` olmaması normal
  durumdur.

## Alan adı ve HTTPS

| Anahtar | Varsayılan | Ne yapar |
| --- | --- | --- |
| `DEFAULT_TLD_SUFFIX` | `stackvo.loc` | Her servis ve yönetim arayüzü adının kurulduğu ek. **Ayarlar → Alan adı ve ağ**. |
| `SSL_ENABLE` | `true` | HTTPS yönlendirmelerini ve sertifikayı üret. |
| `REDIRECT_TO_HTTPS` | `true` | Düz HTTP'yi HTTPS'e yönlendir. |
| `IDLE_SUSPEND_MINUTES` | `0` | Traefik'in bu kadar dakikadır istek yönlendirmediği projeyi durdur. `0` kapalı. |

## Servisler

Her servis bir `SERVICE_<AD>_*` ailesiyle yapılandırılır; `<AD>` katalog
kimliğinin büyük harfi, `-` yerine `_` — `mongo-express` `MONGO_EXPRESS` olur.

| Anahtar | Ne yapar |
| --- | --- |
| `SERVICE_<AD>_ENABLE` | `true` ya da `false`. Hem üretimi hem compose profilini açar. |
| `SERVICE_<AD>_VERSION` | Çalışan imaj etiketi. |
| `SERVICE_<AD>_VERSIONS` | Ayar sayfasının sunduğu etiketler. |
| `SERVICE_<AD>_HOST_PORT` | Bu makinede yayımlanan port. |
| `SERVICE_<AD>_ROOT_PASSWORD` ve benzerleri | Kimlik bilgileri. `keychain:` referansı olabilir — bkz. [Çalışma alanı ve dosyalar](workspace.md). |

## Web sunucusu sınırları

Her PHP projesinin sunucu bloğuna yazılır. **Ayarlar → Web sunucuları → İstek sınırları**.

| Anahtar | Varsayılan | Yazıldığı yer |
| --- | --- | --- |
| `SERVER_MAX_BODY_SIZE` | `1m` | `client_max_body_size`; Caddy de uyar |
| `SERVER_CLIENT_BODY_TIMEOUT` | `60` | `client_body_timeout` |
| `SERVER_KEEPALIVE_TIMEOUT` | `75` | `keepalive_timeout` |
| `SERVER_TCP_NODELAY` | `on` | `tcp_nodelay` |
| `SERVER_GZIP` | `off` | `gzip`; Caddy de uyar |
| `SERVER_GZIP_COMP_LEVEL` | `1` | `gzip_comp_level` |
| `SERVER_GZIP_TYPES` | boş | `gzip_types`; boş nginx'in kendi listesi |
| `SERVER_FASTCGI_CONNECT_TIMEOUT` | `60` | `fastcgi_connect_timeout` |
| `SERVER_FASTCGI_SEND_TIMEOUT` | `60` | `fastcgi_send_timeout` |
| `SERVER_FASTCGI_TIMEOUT` | `60` | `fastcgi_read_timeout` |

## Çalışma zamanı kataloğu

Yeni proje panelinin sundukları.

| Anahtar | Varsayılan |
| --- | --- |
| `SUPPORTED_SERVERS` | `nginx,apache,caddy,frankenphp,swoole,roadrunner` |
| `SUPPORTED_SERVERS_DEFAULT` | `nginx` |
| `SUPPORTED_LANGUAGES_PHP_VERSIONS` | `5.6` … `8.5` |
| `SUPPORTED_LANGUAGES_PHP_DEFAULT` | `8.4` |
| `SUPPORTED_LANGUAGES_PHP_EXTENSIONS` | Seçilebilir eklentiler — her zaman `php-extensions.json`'ın alt kümesi |
| `SUPPORTED_LANGUAGES_NODEJS_VERSIONS` | `16,18,20,21,22,23` |
| `SUPPORTED_LANGUAGES_NODEJS_DEFAULT` | `22` |
| `SUPPORTED_LANGUAGES_PYTHON_DEFAULT` | `3.14` |
| `SUPPORTED_LANGUAGES_GO_DEFAULT` | `1.23` |
| `SUPPORTED_LANGUAGES_RUBY_DEFAULT` | `3.3` |
| `SUPPORTED_LANGUAGES_RUST_DEFAULT` | `1.84` |

## PHP imajı

| Anahtar | Varsayılan | Ne yapar |
| --- | --- | --- |
| `PHP_DEFAULT_TOOLS` | `composer,nodejs` | Her PHP imajına gömülen araçlar. Ayrıca `git`, `wget`, `unzip`. |
| `PHP_DEFAULT_APT_PACKAGES` | bir liste | Gömülen sistem paketleri — `strace`, `vim`, `htop` dâhil. |
| `PHP_TOOL_COMPOSER_VERSION` | `latest` | |
| `PHP_TOOL_NODEJS_VERSION` | `20` | PHP imajının içindeki Node, varlık derlemeleri için. |

## Docker

| Anahtar | Varsayılan | Ne yapar |
| --- | --- | --- |
| `DOCKER_DEFAULT_NETWORK` | `stackvo-net` | Her konteynerin katıldığı ağ. |
| `STACKVO_VERSION` | — | Bilgi amaçlı: bu çalışma alanını hangi StackVo yazdı. |

!!! note "Burada olmayan anahtarlar"
    Şema, emekli Bash uygulamasından kalan ve *ölü* işaretli anahtarları da
    taşır; hiçbir şey onları okumaz. Yeni dosyalara koymayın; eskilerde
    uygulama yok sayar.
