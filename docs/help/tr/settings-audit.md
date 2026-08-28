# Denetim kaydı

Bu uygulamanın makinenize yaptığı ve geri alınamayacak şeylerin kaydı.

Uygulama günlüğü değil. O, "neden başarısız oldu?" sorusuna cevap verir ve yedi gün sonra kendini siler. Bu, "ne yapıldı ve ne zaman?" sorusuna cevap verir ve hiç döndürülmez — üç hafta sonra hâlâ cevap verebilmesini sağlayan da budur.

## İçinde ne var

Yalnızca bu uygulamanın dışında bir şeyi değiştiren ve düğmeye ikinci kez basmanın geri almayacağı işlemler:

| Tür | Örnek |
| --- | --- |
| Yükseltilmiş yazmalar | Hosts dosyasına eklenen bir satır, yazılan bir çözümleyici dosyası |
| Sistem depoları | Yerel sertifika otoritesinin güveninin eklenmesi ya da kaldırılması |
| Yok etme | Bir projenin silinmesi, bir veritabanının var olanın üzerine geri yüklenmesi |
| Yapılandırma | `.env` yazımı — yığındaki her kapsayıcıyı yeniden yapılandırır |
| Kimlik bilgileri | Bir parolanın işletim sistemi anahtar deposuna taşınması ya da geri alınması |
| Başka uygulamaların dosyaları | MCP sunucusunun bir asistana kaydı, bir IDE hata ayıklama yapılandırması, StackVo'nun kabuğunuzun `PATH`'ine eklenmesi |

## Bilerek olmayanlar

Bir kapsayıcıyı başlatmak ya da durdurmak burada değil, hiçbir okuma da değil. Aynı düğmenin geri aldığı bir işlem için kimsenin döndürülmeyen bir kayda ihtiyacı yoktur, ve her şeyi kaydeden bir kayıt kimsenin okumadığı bir kayıttır.

## Bilmekte fayda var

- Her satır ne zaman, hangi işlem, neye yapıldı ve nasıl bittiğini söyler — başarılı oldu, denenmeden reddedildi, ya da denendi ve olmadı. İptal edilen bir parola istemi de kaydedilir: birinin makineye sorulduğunu bilmesi gerekebilir.
- En yeni girdi en üstte.
- Liste sınırlıdır. Gösterilenden fazla girdi varsa kart kaç tane olduğunu söyler.
- Değerler hiçbir zaman kaydedilmez — yalnız hangi anahtar ya da hangi servis. Parola taşıyan bir kayıt, kimseye veremeyeceğiniz bir kayıttır.
- Dosya JSON Lines biçimindedir ve uygulama günlüğünün yanında durur; herhangi bir metin düzenleyiciyle okuyabilir ya da bir ayrıştırıcı olmadan `grep`leyebilirsiniz.
- Bir satır zarar görmüşse — süreç yazma sırasında öldürülmüşse — atlanır ve kart kaç tane olduğunu söyler. Kaydın geri kalanı geçerlidir.
