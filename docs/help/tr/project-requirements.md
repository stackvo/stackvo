# Bu projenin ihtiyaç duyduğu servisler

Projenin hangi veritabanına, önbelleğe ya da kuyruğa ihtiyacı olduğu, ve bu makinede açık olup olmadığı.

## İki liste, iki farklı şey

| Liste         | Nereden gelir           | Ne demek                                                                                                                  |
| ------------- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Beyan edilmiş | `stackvo.json`          | Biri bunu yazdı ve commit'ledi. Klonlayan herkes aynısını görür.                                                          |
| Çıkarılmış    | Projenin `.env` dosyası | Uygulamanın tahmini. `DB_CONNECTION=pgsql` gibi anahtarlardan okunur. Her satırın yanında hangi anahtardan çıktığı yazar. |

Tahmin asla kendiliğinden yazılmaz. Yazmak ayrı bir butondur, çünkü yazdığınız an bu, ekibin karar olarak okuyacağı bir dosyaya girer.

## Durumlar

| Durum                 | Anlamı                                                             |
| --------------------- | ------------------------------------------------------------------ |
| Bu makinede açık      | Servis çalışıyor.                                                  |
| Burada açık değil     | Proje istiyor ama bu makinede yok.                                 |
| Bu sürümde şablon yok | Servisin adı tanınmıyor. Dosyadan silinmez, sadece işleme alınmaz. |

## Kontroller

| Kontrol            | Ne yapar                                                                                        |
| ------------------ | ----------------------------------------------------------------------------------------------- |
| N servisi aç       | `.env` yazılır, compose dosyaları yeniden üretilir ve servisler başlatılır.                     |
| stackvo.json'a yaz | Seçtiğiniz tahminleri projenin manifestine beyan olarak ekler. Bu bir commit'lik değişikliktir. |

## Projenin taşıdığı ön ayar

Bir manifest _hangi_ servisleri söyler. Hangi **sürümleri** olduğunu söyleyemez, ve yanındaki bir avuç paylaşılabilir ayarı da taşıyamaz — çünkü onlar `.env`'de, yani kimsenin işlemediği tek dosyada.

**Ön ayar** işte o yarıdır, ve manifestin yanında, depoda, `stackvo.preset.json` olarak durur. **Ayarlar → Çalışma alanı**'ndan dışa aktarıp oraya kaydedin; klonlayan bir iş arkadaşınız bu kartta projenin bir ön ayar taşıdığını ve uygulamanın neyi değiştireceğini söyleyen bir satır görür.

- **Sizin için hiçbir şey uygulanmaz.** Farkı görür ve düğmeye basarsınız — Ayarlar'dan ön ayar içe aktarmanın çalıştığı gibi. Başkasının klonuyla gelen bir dosya, siz bir sayfayı açtınız diye yığınınızı yeniden yazmamalı.
- **Yığınınız zaten uyuştuğunda satır kaybolur**, yani uyguladıktan sonraki durumda. Dosya değişirse geri gelir.
- **Bir ön ayar asla sır taşıyamaz.** Servis başına açık/sürüm ve genel ayarların bir izin listesini tutar; içinde parola koyacak bir yer yoktur.

## Bilinmesi gerekenler

- Bir servisi açmak bu makineyi değiştirir; beyan etmek repoyu değiştirir. İkisi ayrı kararlardır.
- Yazmadan önce kontrol edin: tahmin, `.env` içindeki eski bir anahtardan da gelmiş olabilir.
