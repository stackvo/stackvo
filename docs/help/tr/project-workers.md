# İşçiler

Bu projenin kuyruk ve zamanlayıcı süreçleri. Yoksa her biri için bir terminal penceresi gerekirdi.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Başlat / Durdur | O süreci ayağa kaldırır ya da indirir. |

Hangi türlerin göründüğü projenizin dosyalarına bağlıdır. Kuyruğu yapılandırılmış bir Laravel projesi kuyruk işçisi alır, zamanlaması olan bir proje zamanlayıcı alır.

## Satırlar ne söyler

| İşaret | Anlamı |
| --- | --- |
| Yeşil | Süreç çalışıyor. |
| Gri | Çalışmıyor. |
| Yeniden başlatma sayısı | Motorun süreci kaç kez yeniden başlattığı. Sıfırsa gösterilmez. |

## Bilinmesi gerekenler

- Önce projenin çalışıyor olması gerekir.
- Bir işçi, başladığı andaki kodu tutar. Kuyruk işçisinin çalıştırdığı kodu değiştirdiyseniz onu durdurup başlatın.
- Kendiliğinden artan bir yeniden başlatma sayısı, yeşil rozet takmış bir çökme döngüsüdür. Nedenini Loglar sekmesinde arayın.
