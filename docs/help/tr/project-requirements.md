# Bu projenin ihtiyaç duyduğu servisler

Projenin hangi veritabanına, önbelleğe ya da kuyruğa ihtiyacı olduğu, ve bu makinede açık olup olmadığı.

## İki liste, iki farklı şey

| Liste | Nereden gelir | Ne demek |
| --- | --- | --- |
| Beyan edilmiş | `stackvo.json` | Biri bunu yazdı ve commit'ledi. Klonlayan herkes aynısını görür. |
| Çıkarılmış | Projenin `.env` dosyası | Uygulamanın tahmini. `DB_CONNECTION=pgsql` gibi anahtarlardan okunur. Her satırın yanında hangi anahtardan çıktığı yazar. |

Tahmin asla kendiliğinden yazılmaz. Yazmak ayrı bir butondur, çünkü yazdığınız an bu, ekibin karar olarak okuyacağı bir dosyaya girer.

## Durumlar

| Durum | Anlamı |
| --- | --- |
| Bu makinede açık | Servis çalışıyor. |
| Burada açık değil | Proje istiyor ama bu makinede yok. |
| Bu sürümde şablon yok | Servisin adı tanınmıyor. Dosyadan silinmez, sadece işleme alınmaz. |

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| N servisi aç | `.env` yazılır, compose dosyaları yeniden üretilir ve servisler başlatılır. |
| stackvo.json'a yaz | Seçtiğiniz tahminleri projenin manifestine beyan olarak ekler. Bu bir commit'lik değişikliktir. |

## Bilinmesi gerekenler

- Bir servisi açmak bu makineyi değiştirir; beyan etmek repoyu değiştirir. İkisi ayrı kararlardır.
- Yazmadan önce kontrol edin: tahmin, `.env` içindeki eski bir anahtardan da gelmiş olabilir.
