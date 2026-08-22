# Yerel API

Bu çalışma alanı hakkındaki soruları HTTP üzerinden cevaplayan, salt okunur bir yüzey.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Başlat | Dinleyiciyi açar ve bir token üretir. |
| Durdur | Dinleyiciyi kapatır. |
| Deneyin | Örnek bir istek gösterir. |

## Ne yapar, ne yapmaz

MCP sunucusunun kullandığı araç tablosunun salt okunur yarısını servis eder. Yalnız `127.0.0.1` üzerinde dinler, başka hiçbir yerde.

Hiçbir şey yazmaz, komut çalıştırmaz, parola göstermez.

## Token

Token yalnızca bir kez gösterilir ve diske hiç yazılmaz. Kaybederseniz durdurup yeniden başlatın; yenisi üretilir.

## Bilinmesi gerekenler

- Siz başlatana kadar kapalıdır. Kimsenin haberi olmayan bir dinleyici, kimsenin kapatmadığı dinleyicidir.
- Token'a sahip olan bu makinedeki her şey bu API'yi kullanabilir.
