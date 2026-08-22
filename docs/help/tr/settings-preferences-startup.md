# Başlangıç ve kapatma

Uygulama açılırken ve kapanırken ne olacağı.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Makineyle başlat | StackVo oturum açıldığında kendiliğinden çalışır. |
| Pencere kapatılınca | Uygulamadan çıkmak yerine tepsiye küçültür ya da tamamen kapatır. |
| Çıkarken konteynerleri durdur | Uygulama kapanırken yığını indirir. |

## Bilinmesi gerekenler

- Uygulamayı kapatmak konteynerleri kendiliğinden durdurmaz. Docker onları çalıştırmayı sürdürür; ayar açıksa durdurulurlar.
- Tepsiye küçültmek uygulamayı çalışır durumda tutar, yani zamanlanmış yedekler ve boşta askıya alma çalışmaya devam eder.
