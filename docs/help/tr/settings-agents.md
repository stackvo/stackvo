# Yapay zekâ asistanları

StackVo MCP sunucusunu bu makinedeki asistanlara tanıtır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kur | Seçtiğiniz istemcinin yapılandırma dosyasına `stackvo` girdisini yazar. |
| Kaldır | Yalnızca o girdiyi siler. |
| Yazma izni ver | Asistana yalnız okuma değil, yığını değiştirme yetkisi de tanır. |

## Ne değişir

Bu sunucuya sahip bir asistan, "shop.loc neden açılmıyor?" sorusunu ön kontrol raporundan, hosts dosyasından, sertifikadan ve konteyner durumundan cevaplayabilir. Tahmin etmek yerine bakar.

## Bilinmesi gerekenler

- Yazma dosya olarak yapılır: uygulama dosyayı okur, tek anahtarı ekler ve geri yazar. Diğer sunucularınız ve tanımadığı anahtarlar korunur.
- Yazmadan önce dosyanın yanına `.stackvo-backup` uzantılı bir yedek bırakılır.
- Yazma izni vermek asistana yığını durdurma ve değiştirme yetkisi verir. Vermezseniz asistan yalnızca okur.

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
