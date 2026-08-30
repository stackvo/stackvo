# İstek zaman çizelgesi

Bir sayfa yüklenirken kodun elinde ne olduğunu sandığı, veritabanına ne sorduğu ve ne gönderdiği — tek bir eksende.

## Bir satır neye göre yerleşir

| Satır | Neye göre yerleşir |
| --- | --- |
| Dump'lar ve istekler | İsteğe göre gruplanır. İkisi de hangi istekte olduğunu bilir. |
| Sorgular, postalar ve işler | Yalnızca zamana göre yerleşir, grupların dışında. |

Sorgular gruplanmaz çünkü hiçbir veritabanı günlüğü hangi HTTP isteğinin bir ifadeyi ürettiğini kaydetmez. Etrafına bakıp tahmin etmek, iki istek ilk çakıştığında sessizce yanlış olurdu. Bir iş aynı durumun başka sebeplisidir: onu kuyruğa koyan istek, iş başlamadan önce bitmiştir.

Satırların ikisi **an** değil **aralık**tır — bir istek ve bir iş bir süreyi kaplar — ve her biri bittiği yere çizilir; süresinin ve sonucunun bilinebilir hâle geldiği an orasıdır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Veritabanı | Hangi veritabanının sorgularının ekseneleneceği. |
| Yenile | Çizelgeyi yeniden okur. |

## Bilinmesi gerekenler

- Sorgu günlüğü kayıtta değilse burada yalnız dump'lar görünür. Üstteki karttan kaydı açın, sayfayı yeniden yükleyin, sonra burayı tazeleyin.
- Dump yakalama da açık olmalı; yoksa gruplayacak bir şey olmaz. İstekleri ve işleri eksene getiren de aynı anahtardır.
