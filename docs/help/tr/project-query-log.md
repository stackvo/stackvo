# Sorgu günlüğü

Veritabanına gerçekte ne soruldu. Ajan kurmadan, yeniden derlemeden ve uygulamanıza kod eklemeden çalışır.

## Bu bir oturumdur, akış değil

Kaydı açarsınız, incelediğiniz sayfayı yeniden yüklersiniz, bakarsınız, kapatırsınız. Açık bırakmak bu özelliğin daha azı değil, daha kötüsüdür: günlük her ifadeyi örneklemeden yazar ve her yazmanın bir bedeli vardır.

Durdurmak toplananı da siler, çünkü günlük ifade metnini tutar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Veritabanı | Hangi veritabanının günlüğünün okunacağı. |
| Sorguları kaydet | Kaydı açar ve kapatır. |
| Baştan başla | Toplananı siler, kayıt açık kalır. |

## İki liste

- **Tekrarlar** — aynı şeklin kaç kez sorulduğu. Bulgu budur: bir sayfanın aynı sorguyu üç yüz kez sorması burada görünür.
- **İfadeler** — kaydedilen her ifade, sırasıyla. Kanıt budur.

## Bilinmesi gerekenler

- Yalnızca günlüğü okunabilen veritabanları desteklenir: MySQL, MariaDB, Postgres ve Mongo. Çalışma alanınızda böyle bir veritabanı yoksa kart bunu söyler.
- Postgres'te ifadeler ayrıca sunucunun konteyner içindeki kendi günlük dosyasına yazılır. Durdurmak buradaki oturumu bitirir ama o dosyayı geri yazmaz.
- Günlük ifade metnini tutar. İçinde parola ya da kişisel veri geçen sorgular da kaydedilir.
