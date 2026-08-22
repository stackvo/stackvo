# Otomatik yedekler

Zamanlanmış veritabanı snapshot'ları. Çalışma alanında tutulur.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Snapshot alma sıklığı | Hiçbir zaman, saatte bir, günde bir ya da haftada bir. |
| Saklanacak snapshot sayısı | Bu sayının ötesindeki en eski zamanlanmışlar silinir. |

## Zamanlama nasıl ölçülür

Saatten değil, son snapshot'tan ölçülür. Üç gün kapalı kalmış bir dizüstü, açıldığında üç snapshot değil bir snapshot borçludur.

## Bilinmesi gerekenler

- Yalnızca çalışan veritabanları yedeklenir. Durmuş bir servis atlanır.
- Kendi adlandırdığınız snapshot'lar asla silinmez ve saklama sayısına dahil edilmez. Sayı yalnızca zamanlanmışlar içindir.
