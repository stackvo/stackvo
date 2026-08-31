# Önizleme ve denetim

Her iki tema aynı anda, ve her birinin gerçekte ürettiği kontrast.

## Ne gösterir

| Bölüm | Nedir |
| --- | --- |
| İki kart | Açık ve koyu tema, mevcut ayarlarınızdan üretilip yan yana çizilir. Hiçbiri uygulamanın kullandığı tema değildir — oldukları yerde çizilirler. |
| Tablo | Her çiftin ölçülen kontrast oranı, iki temada da, WCAG notuyla birlikte. |
| Ton şeridi | Ana rengin ve ondan türetilen rengin Material ton merdiveni. Her blok bir ton; üzerine gelince kodu ve renk kodu görünür. |

## Notlar

- **AAA** — 7:1 ve üzeri. WCAG'nin gövde metni için gelişmiş seviyesi.
- **AA** — 4.5:1 ve üzeri. Uygulamanın her yerde karşılamak üzere kurulduğu seviye.
- **Zayıf** — 4.5:1 altı. Uygulamayla gelen hiçbir bileşim bu değeri vermez; görüyorsanız bir renk seçimi hatalıdır ve satır hangisi olduğunu söyler.

## Bilinmesi gerekenler

- **İkincil metin** satırı iki tema rengi arasındaki kontrast değildir. Alt yazılar, ipuçları ve alan etiketleri yarı saydam çizilir; satır bu yüzden yüzeye gerçekten bindirilen rengi ölçer — bu değer daha düşüktür ve uygulamanın geçip geçmediğine karar veren sayıdır.
- **Buton metni** satırları uygulamanın saklamadığı bir rengi ölçer. Vuetify dolu bir butonun yazı rengini dolgunun kendisinden seçer; o iki satır sizin yerinize verilmiş bir kararı denetler.
- Kontrast düzeyini değiştirmek ikincil metin satırını ve dört durum satırını hareket ettirir. Gövde metnini ve buton metnini etkilemez; onlar her düzeyde kendi başlarına geçer.
- Ton şeridi Material'ın kendi renk motoruyla çizilir; adımları sayıya göre değil göze göre eşit aralıklıdır. Yalnızca iki aksanı kapsar: buradaki nötr paletler elle seçilmiştir ve yanlarına motorla türetilmiş bir şerit koymak, uygulamanın hiç çizmediği renkleri göstermek olurdu.
- Önizleme bir resimdir: klavye gezinmesi ve ekran okuyucular tarafından atlanır, çünkü altındaki tablo aynı şeyleri sözle söyler.
