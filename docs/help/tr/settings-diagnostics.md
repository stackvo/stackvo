# Uygulama günlüğü

StackVo'nun kendi tanılama kaydı. Projelerin sunucu logları değildir; onlar proje sayfasının Loglar sekmesindedir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Klasörü aç | Log klasörünü Finder ya da Dosya Gezgini'nde gösterir. |
| Tanılama paketi kaydet | Log, başlangıç kontrolleri, doktor raporu ve varsa çökme raporlarını tek bir arşive yazar. |
| Başka bir makineyle karşılaştır | Birinin gönderdiği paketi açar ve bu makinede neyin farklı olduğunu listeler. |

## "Bende çalışıyor"

Bu tür araçların en eski şikâyeti budur, ve alışılmış cevap — *konteynerler bunu çözer* — doğru değildir: aynı compose dosyası iki farklı Docker sürümünde iki farklı şeydir.

Karşı taraftan paketini isteyin ve burada açın. Geri gelen şey yalnızca iki makinenin **anlaşmadığı** noktalar; gerisi listelenmiyor, sayılıyor:

| Olgu | Bu makine | Onlarınki |
| --- | --- | --- |
| `engine.version` | 27.1.1 | 25.0.3 |
| `service.redis-7-2` | 7.2 açık | 7.2 kapalı |
| `project.shop` | php 8.4, nginx | php 8.3, nginx |

Yalnız bir tarafın söylediği bir olgu, diğer yarısı **belirtilmemiş** olarak gösterilir — "sende bu servis var, onda yok" çoğu zaman sayfadaki en işe yarar satırdır.

Karşılaştırma, paketin içindeki tek bir dosyayı okur: `environment.json` — sürümler, motor, servisler ve her projenin beyanı. İçinde **yol yok** (ev dizini her makinede farklıdır ve birebir aynı iki kurulumu beş yerde farklı gösterirdi) ve **kimlik bilgisi ya da `.env` değeri yok** — paketin zaten gönderilebilir olmasının sebebi de bu. Karşılaştırma, bu makinenin **şu anki** hâline karşı yapılır, daha önce alınmış bir kopyasına karşı değil; soru her zaman şimdi neyin farklı olduğudur.

"Hiçbir fark yok" boş bir ekran değil, bir sonuçtur: ters giden şeyin buranın göremediği bir yerde olduğu anlamına gelir — ve bunu, bir öğleden sonrayı sürümlere harcamadan önce bilmek işe yarar.

Biri size yalnızca `environment.json` yolladıysa onu tek başına da açabilirsiniz. Bu özellikten önceki bir StackVo sürümüyle yapılmış bir paket, hiçbir şeyi karşılaştırmak yerine durumu adıyla söyler.

## Bilinmesi gerekenler

- Bir sorun bildirirken tanılama paketini ekleyin. Yalnızca log çoğu zaman yetmez.
- Parola ve token değerleri log yazılırken maskelenir.
- Paketin içi düz metindir. Göndermeden önce bir bakın.
- Bu sistemde yazılabilir bir log konumu bulunamadıysa kart bunu söyler.
