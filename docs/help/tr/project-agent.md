# Bir asistana bu proje hakkında ne söyleniyor

Depoda çalışan bir asistanın neyin içinde çalıştığını bilmesi için uygulamanın depoya yazdığı iki dosya.

## Yapay zekâ kuralları

Asistanın zaten okuduğu yönerge dosyasına yazılan kısa bir bölüm — `CLAUDE.md`, `AGENTS.md` (Codex ve Zed), Cursor için `.cursor/rules/stackvo.mdc`, VS Code için `.github/instructions/stackvo.instructions.md`, `.windsurf/rules/stackvo.md` veya `GEMINI.md`.

| Kontrol | Ne yapar |
| --- | --- |
| Kuralları yaz | StackVo bloğunu o dosyaya ekler; dosya yoksa oluşturur. |
| Güncelle | Uygulamanın eski bir sürümünün yazdığı bloğu değiştirir. |
| Kaldır | Bloğu çıkarır. Dosyanın geri kalanı olduğu gibi kalır. |

Bunlar Ayarlar → Yapay zekâ kuralları'nın yazdığı kuralların aynısı; burada bir açılır listeden seçilen ada değil, doğrudan bu projeye nişan alıyor. Bu makinedeki **her** proje için geçerli kurallar ve MCP sunucusunun kendisini tanıtmak Ayarlar altında.

Kurallar şunu söyler: hangi soruyu hangi araç cevaplar, üretilmiş dizindeki her şeyin üzerine yazılır ve değiştirilmesi gereken girdidir, Docker'ı elle sürmek bir sonraki üretimin sahiplenmeyi beklediği bir adı ve portu kapatır, ve bir yazma aracı yığının tamamını durdurabilir.

### Basmak neden güvenli

Yalnızca `<!-- stackvo:rules:begin -->` ile `<!-- stackvo:rules:end -->` arasındaki bölüm yazılır. Dosyadaki her şey olduğu gibi geri gelir, işaret içermeyen bir dosyanın üzerine yazılmaz sonuna eklenir ve önce yanına `.stackvo-backup` kopyası bırakılır.

## Bağlam dosyası

`.stackvo/context.json` her üretimde her proje için yazılır ve açılacak bir şey yoktur: alan adı, çalışma zamanı, container içindeki yol ve çalışan her servisin **ağ içinden** ulaşılabilen adresi.

Yalnız adlar ve adresler. Parolalar projenin kendi `.env` dosyasındadır ve buraya bilerek tekrarlanmaz — dosya bir kaynak ağacına iniyor ve kaynak ağacı yanlışlıkla commit edilen bir şeydir.

PHP projelerinde dizin bağlandığı için dosya canlıdır. Kaynaktan derlenen bir çalışma zamanında bağlama yoktur; dosya container'a bir sonraki derlemede ulaşır.
