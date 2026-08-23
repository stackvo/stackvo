# Profilleyici

Xdebug'in kendi profilleyicisi. Çıktıyı bu uygulamanın okuduğu dosyalara yazar. Ek hesap ya da ek eklenti gerekmez.

## Kipler

| Kip | Ne yapar |
| --- | --- |
| Adım adım hata ayıklama | Her istekte IDE'nize bağlanır. |
| Profilleme | Bir tetikleyici bekler, sonra o isteğin profilini dosyaya yazar. |
| İz | Her fonksiyon girişini ve çıkışını yazar. |

Biri ya da diğeri. İkisini birden açık bırakmak birini bozar, o yüzden kart tek seçim yaptırır.

## Nasıl kayıt alınır

Bir istek talep etmeden hiçbir şey kaydedilmez. İncelemek istediğiniz adrese `?XDEBUG_TRIGGER=1` ekleyin ya da aynı adı çerez olarak tanımlayın. Kart, kullanılacak tetikleyicinin tam adını gösterir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kip seçimi | Yukarıdaki üç kipten birini uygular. |
| Kayıt listesi | Yazılmış profil dosyaları. Birine tıklamak alev grafiğini açar. |
| Sil | Kayıtları temizler. |
| Konteynere uygula | Çalışan konteyner ile seçilen kip uyuşmuyorsa konteyneri yeniden oluşturur. |

## İz kaydının bedeli

İz, profilden çok daha ağırdır. Tek bir istek yüzlerce megabayta çıkabilir. Bir sayfayı kaydedin, sonra kipi geri alın.

Alev grafiği çok uzun bir izin tamamını çizemez. Böyle bir durumda kart, gördüğünüzün isteğin yalnızca başı olduğunu söyler.

## Bilinmesi gerekenler

- Önce Xdebug açık olmalı. Profilleme aynı eklentinin bir kipidir.
- Kip değişikliği konteyner yeniden oluşturulana kadar geçerli olmaz. Kart uyuşmazlığı söyler.

## Kapsam (coverage)

Dördüncü mod ve kendi başına hiçbir şey kaydetmeyen tek mod: PHPUnit'in çağırdığı API'yi açar, raporu PHPUnit yazar. Uygulandıktan sonra testlerinizi bir kapsam bayrağıyla çalıştırın — kayıt listesinde onun için hiçbir şey belirmez ve panel bunu sizi bekletmek yerine söyler.

Bu mod olmadan `--coverage-html` boş bir rapor ve çoğu kişinin hiç okumadığı bir uyarı üretir.

## Okunabilir dump ve yığın izleri

Xdebug'ın `develop`'ı beşinci bir mod değildir. `xdebug.mode` bir **listedir** ve `develop` seçtiğiniz modun yanında yer alır — yani `XDEBUG_MODE` seçiminizin yerine geçmek yerine `debug,develop` olur. `var_dump`'ı okunabilir yapar ve bir uyarıya yığın izi ekler.

İstenmedikçe kapalıdır, çünkü kendi kodunuzun bastığı çıktıyı değiştirir; hata ayıklanan kodun çıktısını değiştiren bir aracın sizin seçiminiz olması gerekir.

Anahtar ile mod düğmeleri tek bir dosya üzerindeki iki kontroldür: birini oynatmak diğerini olduğu gibi bırakır.
