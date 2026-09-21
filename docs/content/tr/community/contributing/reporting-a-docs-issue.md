# Doküman hatası bildirme

Üç tür doküman var ve her biri depoda kendinizin düzenleyebileceği bir dosya.

| Nerede okuyorsunuz | Nerede duruyor | Nasıl düzenlenir |
| --- | --- | --- |
| Bu site | `docs/content/en/`, `docs/content/tr/` | Her sayfanın üstündeki kalem |
| Uygulamadaki bir yardım kartı, karttaki **?** | `docs/help/en/`, `docs/help/tr/` | Aynı kalem, o kartın [Uygulama içi yardım](../../help/index.md) altındaki sayfasında |
| README | `README.md`, `README_TR.md` | GitHub üzerinde |

Yardım kartları, uygulamanın `main`'den çektiği dosyaların ta kendisidir;
düzeltilen bir cümle her kuruluma bir sonraki çalıştırmada ulaşır. Sürüm
gerekmez.

## Kendiniz düzeltin

Sayfanın üstündeki kalem o dosyayı GitHub'da açar. Düzenleyin, değişikliği
bir cümleyle anlatın; pull request'i GitHub sizin için açar. Bir yazım hatası
ya da yanlış bir kelime için sürecin tamamı budur; bir dakika sürer ve
görüldüğü anda birleştirilir.

Yapabiliyorsanız iki dili de yazın. Site her sayfanın İngilizce ve Türkçe
var olduğunu varsayar; bir dilde yazılıp diğerinde yazılmayan bir yardım
konusu testleri düşürür. Yalnızca birini yazabiliyorsanız pull request'te
söyleyin; diğer yarısı arkasından gelir.

## Bildirin

Yazmak isteyeceğinizden daha büyük bir şey için (eksik bir sayfa, ilk
satırından yanlış bir bölüm, artık uygulamayla uyuşmayan bir kılavuz) şunlarla
bir [issue](https://github.com/stackvo/stackvo/issues/new/choose) açın:

- **Nerede.** Sayfanın adresi ya da kartın adı ve bulunduğu sayfa.
- **Ne yanlış.** Yanlış, belirsiz, güncel değil ya da eksik; hangisi olduğunu
  söyleyin.
- **Ne okumayı bekliyordunuz.** Kabaca bir cümle bile yardımcı olur. Kafası
  karışan okur, neyin karıştırmayacağını söyleyecek en iyi kişidir.

Yanlıştan çok *belirsiz* olan bir şey çoğu zaman önce bir sorudur. Bunun yeri
[Discussions](https://github.com/stackvo/stackvo/discussions); cevap
genellikle burada bir paragrafa dönüşür.

## Bir sayfa ne yapmalı

Düz cümleler, kısa olanlar. Bir kontrolün ne yaptığını, neyin yazıldığını,
neyin yeniden başlatıldığını ve yeniden derlemeyi atlatıp atlatmadığını
söyleyin. Yardım kartlarının
[yazım kılavuzu](https://github.com/stackvo/stackvo/blob/main/docs/help/README.md)
tüm sitenin standardıdır.
