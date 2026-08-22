# Katalog

Servislerin nereden geldiği ve bu makinede hangi sürümlerin olduğu.

Servisler sayfası neyin çalıştığını gösterir; burası neyin çalışabileceğini.

## Sayfanın iki bölümü

| Bölüm | Ne için |
| --- | --- |
| Yayında olanlar | Kaynağın yayımladığı paketler ve sürümleri. Buradan **kurarsınız**: dosyalar diske iner. |
| Servis örnekleri | Bu çalışma alanının çalıştırdığı sürümler. Buradan **örnek eklersiniz**: bu çalışma alanı o sürümü çalıştırmaya başlar. |

İkisi ayrı işlerdir. Bu ayrım olmasaydı "MySQL 9.4'ü 8.0 ile yan yana denemek istiyorum" ile "veritabanımı değiştir" aynı butona basmak olurdu.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Katalogda ara | Paketleri süzer. |
| Kaynak | Katalogun nereden çekildiği. Adres Ayarlar → Katalog bölümünde tutulur. |

## Bilinmesi gerekenler

- StackVo içinde hiçbir servis taşımaz. Bir kaynak gösterilene kadar hiçbir şey kullanılabilir değildir.
- Bir kaynak imzasızsa kart bunu söyler.
- Bu çalışma alanı servisleri hâlâ `.env` içinde tutuyorsa sayfa bir göç önerir. Göç veriyi taşımaz: birimler sahiplenilir, portlar korunur ve eski konteyner adı ağ takma adı olarak yaşamayı sürdürür.
