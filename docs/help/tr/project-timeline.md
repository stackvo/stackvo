# İstek zaman çizelgesi

Bir sayfa yüklenirken kodun elinde ne olduğunu sandığı, veritabanına ne sorduğu ve ne gönderdiği — tek bir eksende.

## İki tür satır

| Satır | Neye göre yerleşir |
| --- | --- |
| Dump'lar | İsteğe göre gruplanır. Bir dump hangi istekte olduğunu bilir. |
| Sorgular ve postalar | Yalnızca zamana göre yerleşir, grupların dışında. |

Sorgular gruplanmaz çünkü hiçbir veritabanı günlüğü hangi HTTP isteğinin bir ifadeyi ürettiğini kaydetmez. Etrafına bakıp tahmin etmek, iki istek ilk çakıştığında sessizce yanlış olurdu.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Veritabanı | Hangi veritabanının sorgularının ekseneleneceği. |
| Yenile | Çizelgeyi yeniden okur. |

## Bilinmesi gerekenler

- Sorgu günlüğü kayıtta değilse burada yalnız dump'lar görünür. Üstteki karttan kaydı açın, sayfayı yeniden yükleyin, sonra burayı tazeleyin.
- Dump yakalama da açık olmalı; yoksa gruplayacak bir şey olmaz.
