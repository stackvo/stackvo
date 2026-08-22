# PHP ayarları

Bu projeye özel `php.ini` değerleri. `.stackvo/php.ini` dosyasına yazılır ve PHP'nin `conf.d` dizinine salt okunur bağlanır. PHP kendi `php.ini`'sinden sonra okuduğu için burada yazan geçerli olur.

## Alanlar

| Alan | Ne yapar |
| --- | --- |
| Bellek sınırı | Bir isteğin kullanabileceği en fazla bellek. `K`, `M` ya da `G` ekli bir sayı. Sınırsız için `-1`. |
| En büyük yükleme boyutu | Tek bir yüklenen dosyanın üst sınırı. |
| En büyük POST boyutu | Tüm gövdenin üst sınırı. En az yükleme boyutu kadar olmalı; ikisinden küçüğü geçerlidir. |
| En uzun çalışma süresi | Bir isteğin çalışabileceği saniye. Sınırsız için `0`. |

Alandaki değerler, çalışan konteynerdeki PHP'nin şu anki değerleridir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kaydet | Dosyayı yazar. |
| Dosyayı kaldır | `.stackvo/php.ini` dosyasını siler; ayarlar imajın varsayılanlarına döner. |

Boş bırakılan bir alan o yönergeyi dosyadan kaldırır.

## Bilinmesi gerekenler

- PHP yapılandırmayı başlangıçta okur. Kaydettikten sonra projeyi yeniden başlatın.
- Dosya diskte ama çalışan konteynerde bağlaması yoksa kart bunu söyler; projeyi yeniden ayağa kaldırın.
- Dosyayı elle düzenlemek de sürüm kontrolüne eklemek de güvenlidir. Kartın tanımadığı diğer yönergeler ayrı bir listede gösterilir ve korunur.
- Komut satırından `stackvo up` bu dosyayı katmanlamaz.
