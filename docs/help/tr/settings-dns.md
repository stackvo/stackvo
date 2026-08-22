# Yerel DNS

Hosts dosyasını düzenlemeden bu çalışma alanının adlarına cevap veren bir yanıtlayıcı.

Tek bir soneke cevap verir, diğer her şeyi reddeder. Asla iletmez; üst sunucusu ve önbelleği yoktur. Bu makinenin çözümleyicisi değildir, yalnızca StackVo'nun ürettiği adların çözümleyicisidir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Yanıtlayıcıyı aç | `127.0.0.1` üzerinde belirtilen portta dinlemeye başlar. |
| Sistem çözümleyicisine bağla | Bu makinenin, soneke ait sorguları yanıtlayıcıya sormasını sağlar. Parolanızı sorar. |
| Test et | Yanıtlayıcıya ve sistemin kendisine ayrı ayrı sorar; dört sonucu ayrı gösterir. |

## Hosts dosyasından farkı

Joker adları bu çalıştırır. Hosts dosyası joker yazamaz, o yüzden `*.shop.loc` gibi bir adres yalnızca burada çözülür.

## Kartın uyarıları

| Uyarı | Anlamı |
| --- | --- |
| Yalnız UDP | TCP portu başka bir şeyin elinde. Sorguların çoğu çalışır, TCP üzerinden yeniden deneme çalışmaz. |
| Bozuk | Makine bu porta soruyor ama orada cevap veren yok. Ya yanıtlayıcıyı açın ya bağlamayı kapatın. |
| Bayat | Artık kullanılmayan bir sonekten kalmış yapılandırma. Yeniden uygulamak temizler. |

## Bilinmesi gerekenler

- Test butonu iki ayrı soru sorar: yanıtlayıcı kendi sorusuna cevap veriyor mu, ve bu makine adı gerçekten çözüyor mu. İlki geçip ikincisi kalabilir; hangisinin başarısız olduğu ne yapmanız gerektiğini söyler.
