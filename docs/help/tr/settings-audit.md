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

## Bir asistanın yaptıkları

Bir asistanın MCP sunucusu üzerinden yaptığı her yazma çağrısı burada kayıtlıdır — **reddedilenler dahil**, ki çoğu zaman daha ilginç satır odur: yığının tamamını durdurmayı deneyip "yapamazsın" cevabı almış bir asistan, bir sonraki sefer neye izin vereceğinize karar verirken tam olarak görmek istediğiniz şeydir.

Yukarıdaki eşiğin genişletildiği tek yer burasıdır, ve bir gerekçesi var. Bu pencereden bir kapsayıcı başlatmak kaydedilmiyor, çünkü düğmeye basan kişi olup bittiğini gördü. Aynı işlemi bir asistan istediğinde, onu kimse görmedi.

## Birini geri almak

Asistanın işlemi, **onu ne geri alır** bilgisini taşır; bu plan çağrı çalışmadan **önce** hesaplanıp satıra yazılır. Bunun önemi şu: `stack_down`'ın durdurduğu şey yalnızca durdurmadan önce vardır — düğmeye bastığınızda hesaplanan bir plan, çoktan değişmiş bir makineye göre hesaplanmış olurdu.

| İşlem | Geri al ne yapar |
| --- | --- |
| Yığının tamamını durdurdu | Öncesinde çalışanları başlatır — önce servisler, sonra projeler |
| Bir projeyi ya da servisi başlattı/durdurdu | Diğerini yapar |
| Xdebug'ı açtı/kapattı | Eski hâline döndürür |

İşlemlerin çoğunun geri alması yoktur, ve satır bunun sebebini tutamayacağı bir söz veren düğme yerine kendi cümlesiyle söyler: bir **yeniden başlatma** zaten geri alınacak durumun içinden geçti; **generate** saklanmayan bir çıktının üzerine yazdı, onarım girdiyi değiştirmektir; **sertifika yenilemesi** de saklanmayan bir sertifikanın yerine geçti; **anlık görüntü almak** bir dosya ekledi ve hiçbir şeyi değiştirmedi.

Geri alma bir dizidir, işlem (transaction) değil. Altı çağrının dördüncüsü başarısız olursa ilk üçü yapılmış kalır ve kayıt nerede durduğunu söyler. Kayıt her iki yarıyı da tutar — işlemin olduğunu ve birinin onu geri aldığını — çünkü dosyaya yalnızca ekleme yapılır: geri alma satırı, geri aldığı satırı düzenlemez, adını anar.

## Bilerek olmayanlar

Bu pencereden bir kapsayıcıyı başlatmak ya da durdurmak burada değil, hiçbir okuma da değil. Aynı düğmenin geri aldığı bir işlem için kimsenin döndürülmeyen bir kayda ihtiyacı yoktur, ve her şeyi kaydeden bir kayıt kimsenin okumadığı bir kayıttır.

## Bilmekte fayda var

- Her satır ne zaman, hangi işlem, neye yapıldı ve nasıl bittiğini söyler — başarılı oldu, denenmeden reddedildi, ya da denendi ve olmadı. İptal edilen bir parola istemi de kaydedilir: birinin makineye sorulduğunu bilmesi gerekebilir.
- En yeni girdi en üstte.
- Liste sınırlıdır. Gösterilenden fazla girdi varsa kart kaç tane olduğunu söyler.
- Değerler hiçbir zaman kaydedilmez — yalnız hangi anahtar ya da hangi servis. Parola taşıyan bir kayıt, kimseye veremeyeceğiniz bir kayıttır.
- Dosya JSON Lines biçimindedir ve uygulama günlüğünün yanında durur; herhangi bir metin düzenleyiciyle okuyabilir ya da bir ayrıştırıcı olmadan `grep`leyebilirsiniz.
- Bir satır zarar görmüşse — süreç yazma sırasında öldürülmüşse — atlanır ve kart kaç tane olduğunu söyler. Kaydın geri kalanı geçerlidir.
