# Üretim imajı

Bu projenin çalıştırdığı imajdan türetilen, dağıtılabilir bir imaj. Aynı PHP sürümü, aynı eklentiler, aynı web sunucusu.

Geliştirme imajının kopyası değildir: geliştirme imajında uygulama kodu yoktur (kaynak diskinizden bağlanır) ve Xdebug vardır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| İmaj etiketi | Derlenecek imajın adı. |
| Derle | İmajı derler. |
| Denetle | Göndermeden önce imajı doğrular. |
| Gönder | Doğrulanmış imajı registry'ye gönderir. |
| Dağıtım reçetesi | İmajı çalıştıracak bir compose dosyası verir. |
| Paket yükle | Kaydedilmiş bir `.tar` dosyasını bu makinenin Docker'ına geri okur. |

## Kartın gösterdikleri

- **Dışında tutulanlar** — imaja girmeyen dosyalar.
- **Kullanılacak Dockerfile** — üretim imajının derleneceği dosya.
- **İmaj gerçekte ne içeriyor** — derlenen imajın içindekiler.

## Bilinmesi gerekenler

- StackVo yalnızca doğrulanmış bir imajı ve yalnızca registry adı taşıyan bir etiketi gönderir.
- Registry katmanları saklar. Sonradan etiketi silmek içindekini kaldırmaz.
- Paket yükle ne proje ne plan gerektirir. İnternete kapalı bir devrin alıcı ucudur.
