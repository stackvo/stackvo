# Disk G/Ç

Anlık disk okuma ve yazma hızı, altında son ölçümlerin geçmişiyle.

## Bilinmesi gerekenler

- Bu, makinenin tamamının disk trafiğidir; yalnızca konteynerlerin değil.
- macOS ve Windows'ta bind mount edilen dizinler diske çok daha fazla iş çıkarır. Sürekli yüksek yazma görüyorsanız projenin Performans katmanı kartına bakın: `vendor` ve `storage/framework` gibi dizinleri bir Docker birimine taşımak ölçülebilir fark yaratır.
