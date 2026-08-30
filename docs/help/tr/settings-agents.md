# Yapay zekâ asistanları

StackVo MCP sunucusunu bu makinedeki asistanlara tanıtır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kur | Seçtiğiniz istemcinin yapılandırma dosyasına `stackvo` girdisini yazar. |
| Kaldır | Yalnızca o girdiyi siler. |
| Yazma izni ver | Asistana yalnız okuma değil, yığını değiştirme yetkisi de tanır. |
| Yalnız bu projeler | Kaydı, adını verdiğiniz projelere sınırlar. |
| Yazma süresi | Yazma yarısını, sunucunun her başlayışından bu süre sonra bitirir. |

## Ne değişir

Bu sunucuya sahip bir asistan, "shop.loc neden açılmıyor?" sorusunu ön kontrol raporundan, hosts dosyasından, sertifikadan ve konteyner durumundan cevaplayabilir. Tahmin etmek yerine bakar.

## Bilinmesi gerekenler

- Yazma dosya olarak yapılır: uygulama dosyayı okur, tek anahtarı ekler ve geri yazar. Diğer sunucularınız ve tanımadığı anahtarlar korunur.
- Yazmadan önce dosyanın yanına `.stackvo-backup` uzantılı bir yedek bırakılır.
- Yazma izni vermek asistana yığını durdurma ve değiştirme yetkisi verir. Vermezseniz asistan yalnızca okur.
- **Anahtarı güvenle açılabilir yapan ayar, bir proje adı vermektir.** Hiçbir proje adı verilmemişse yazma izni on iki yazma aracını birden devreder ve içlerinde `stack_down` vardır — yani bu makinedeki her konteynerin durması tek bir çağrı uzaktadır. Bir proje adlandırdığınızda on iki, bir projenin sınırlayabildiği dörde iner: `xdebug_set`, `project_start`, `project_stop`, `project_restart`. Diğer sekizi hiç sunulmaz, çünkü hiçbir proje sınırı "her şeyi durdur"u söylediğinden daha azı hâline getiremez.
- Sınır okumayı da kapsar, ve tam şu kadar: bir projeyi adlandıran hiçbir araç, sınırın dışındaki bir proje için cevap vermez — yani başka bir projenin manifesti, istek izleri, profili ve kayıt dosyaları kapalıdır, ve gördüğü proje listelerinde yalnız sınırının içindekiler bulunur. Bu bir **bilgi yalıtımı değildir**: makine geneli cevaplar çalışmayı sürdürür, çünkü onlar tek bir projeye değil makineye dairdir — doctor, hosts tablosu, posta yakalayıcı, bir veritabanı servisinin sorgu kaydı, kimliğiyle bir konteynerin kaydı. Bunları da sınırlamak, asistanı verdiğiniz projeyi teşhis edemez hâle getirirdi.
- **Yazma süresi** yazma araçlarını kendiliğinden bitirir. Okumalar çalışmayı sürdürür; o andan sonra araç listesini yeniden soran bir istemciye doğrusu söylenir, sormayan ise çağırdığında adıyla reddedilir. Yeniden vermek için sunucuyu yeniden başlatın — asistanı kapatıp açmak bunu yapar.
- Kontrollerin altında görünen bayraklar, dosyaya yazılacak olanın tam olarak kendisidir. O satır, bu asistana neye izin verildiğinin kaydıdır — ve altı ay sonra biri sorduğunda okuyacağınız şey odur.

## Yapay zekâ kuralları

Sunucuyu tanıtmak araçları erişilebilir yapar; kullanılmasını sağlamaz. Bu yığını hiç görmemiş bir asistan kaynağı okur, nginx'i tahmin eder ve üretilmiş bir dosyayı düzenlemeyi önerir — çünkü kimse ona sorunun tek bir araç çağrısıyla cevaplandığını söylememiştir.

**Yapay zekâ kuralları**, asistanın zaten okuduğu yönerge dosyasına kısa bir bölüm yazar: `CLAUDE.md`, `AGENTS.md` (Codex ve Zed), `.cursor/rules/stackvo.mdc`, `.github/instructions/stackvo.instructions.md` (VS Code ve Copilot), `.windsurf/rules/stackvo.md` veya `GEMINI.md`.

| Kontrol | Ne yapar |
| --- | --- |
| Proje kuralları nereye yazılsın | Kuralların hangi projeye yazılacağı. Çalışma alanı kökü, yığının tamamı üzerinde açılan bir asistan içindir. |
| Kuralları yaz | StackVo bloğunu o dosyaya ekler; dosya yoksa oluşturur. |
| Güncelle | Uygulamanın eski bir sürümünün yazdığı bloğu değiştirir. |
| Kaldır | Bloğu çıkarır. Dosyanın geri kalanı olduğu gibi kalır. |

**Proje içinde** seçeneği depoyla birlikte taşınır; depoyu klonlayan bir arkadaşınız da aynı yönergeyi alır. **Bu makinede** seçeneği o asistanın her oturumu için geçerlidir — StackVo ile ilgisi olmayan projeler dahil. Yalnızca bazı asistanlar genel bir dosya okuduğu için orada yalnızca onlar listelenir.

### Basmak neden güvenli

Yalnızca `<!-- stackvo:rules:begin -->` ile `<!-- stackvo:rules:end -->` arasındaki bölüm yazılır. Dosyadaki her şey olduğu gibi geri gelir, işaret içermeyen bir dosyanın üzerine yazılmaz sonuna eklenir ve önce yanına `.stackvo-backup` kopyası bırakılır. Cursor ile VS Code'un ihtiyaç duyduğu ön bilgi (front matter) yalnızca dosya oluşturulurken yazılır; sonradan daraltırsanız öyle kalır.

### Kurallar ne diyor

Hangi soruyu hangi aracın cevapladığını; üretilmiş dizindeki her şeyin üzerine yazıldığını ve değiştirilmesi gerekenin girdi olduğunu; Docker'ı elle sürmenin bir sonraki üretimin sahiplenmeyi beklediği bir adı ve portu kapattığını; ve bir yazma aracının yığının tamamını durdurabileceğini — yani bir göç (migration) öncesi anlık yedek almayı ve çağırmadan önce sormayı.
