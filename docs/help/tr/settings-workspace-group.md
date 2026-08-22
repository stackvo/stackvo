# Çalışma dizini

Bu uygulamanın yönettiği checkout. Projeler, üretilen dosyalar ve `.env` burada durur.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Klasör seç | Başka bir çalışma alanına geçer. |
| Klasörü aç | Dizini Finder ya da Dosya Gezgini'nde gösterir. |

## Bilinmesi gerekenler

- Çalışma alanını değiştirmek uygulamanın gördüğü her şeyi değiştirir: projeler, servisler, ayarlar. Eski dizindeki hiçbir şey silinmez.
- Bir dizinin çalışma alanı sayılması için StackVo'nun tanıdığı bir yapıda olması gerekir. Tanınmayan bir dizin seçilirse uygulama bunu söyler.
