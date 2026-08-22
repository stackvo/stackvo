# Xdebug

Bu proje için adım adım hata ayıklama.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Etkin / Devre dışı | Xdebug'i açar ve kapatır. |

## İlk açış farklıdır

İlk kez açmak uzantıyı imaja ekler ve **yeniden derleme** gerektirir. Ondan sonrası yalnızca konteyneri yeniden başlatır: uzantı imajda kalır ve kapalıyken hiçbir maliyeti olmaz.

İkinci açışın ilkinden çok daha hızlı olması normaldir.

## IDE ayarları

Kart, IDE'nize gireceğiniz değerleri listeler:

| Alan | Ne için |
| --- | --- |
| Port | Xdebug'in bağlanacağı port. |
| IDE anahtarı | Oturumu tanımlayan anahtar. |
| Sunucu adı | `PHP_IDE_CONFIG` değeri. |
| Yol eşlemesi | Konteyner yolu ile makinenizdeki yolun karşılığı. Bu olmadan kesme noktaları tutmaz. |
| Xdebug sürümü | Kurulu sürüm. |

## Bilinmesi gerekenler

- Kart "çalışan konteyner Xdebug ayarlarını taşımıyor" diyorsa projeyi yeniden başlatın.
- Komut satırından `stackvo up` bu yapılandırmayı katmanlamaz ve konteyneri onsuz oluşturur.
- Xdebug ile profilleyici aynı uzantının iki kipidir. İkisi aynı anda açık olamaz.
