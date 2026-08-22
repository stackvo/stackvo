# Ayarlar

Uygulamanın ve çalışma alanının tüm ayarları. Sağdaki liste konuya göre böler.

## Bölümler

| Grup | İçindekiler |
| --- | --- |
| Uygulama | Görünüm, yerelleştirme, tercihler, yapay zekâ asistanları, yerel API. |
| Çalışma alanı | Dizin ve kontrol, alan adı ve ağ, sertifikalar, kimlik bilgileri. |
| Stack | Web sunucuları, katalog, proje varsayılanları. |
| Yardım | Doktor, uygulama günlüğü, hakkında. |

## Ayar nerede saklanır

| Nerede | Ne | Ne zaman değişir |
| --- | --- | --- |
| `preferences.json` | Uygulamanın kendi tercihleri: editör, tema, dil. | Anında. |
| Çalışma alanının `.env` dosyası | Stack'i ilgilendiren her şey: alan adı soneki, sürümler, sunucu ayarları. | Çoğu yeniden üretme ister. |
| Proje `stackvo.json` | Tek bir projeye ait olan her şey. | Proje sayfasından. |

## Bilinmesi gerekenler

- `.env`'e yazan bir ayarı değiştirdikten sonra çoğu zaman yeniden üretmek gerekir. Kart bunu söyler.
- Bir yönetici bazı ayarları kilitlemiş olabilir. Kilitli bir ayar pasif görünür ve hangi dosyanın kilitlediği yazar.
