# Yayında olanlar

Kaynağın yayımladığı paketler ve bu makinede hangi sürümlerin kurulu olduğu.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kur | Sürümü bu makineye indirir. |
| Kaldır | Kurulu sürümü siler. |
| Örnek ekle | O sürümden çalışan bir servis örneği oluşturur. |
| Desteği bitmiş sürümleri göster | Gizlenen eski sürümleri listeye katar. |

## Destek durumu

| Etiket | Anlamı |
| --- | --- |
| Destekli | Üretici yama vermeyi sürdürüyor. |
| Kullanımdan kalkıyor | Destek bitiş tarihi yaklaşıyor. |
| Desteği bitti | Üretici artık yama vermiyor. |

Desteği bitmiş bir sürüm çalışmaya devam eder. Yama almaması, bozuk olmasıyla aynı şey değildir.

## Bilinmesi gerekenler

- Desteği bitmiş sürümler katalogdan değil, listelerden tutulur. `.env`'inde o sürüm yazan bir çalışma alanının göç edebilmesi gerekir; sürüm düşüren bir indeks, birinin çalışan servisinin kaynağını kaybettiği indekstir.
- Bir sürümü kaldırmadan önce onu kullanan bir örnek olup olmadığına bakın; kart bunu söyler.
- Kurmak yalnızca dosyaları indirir. Hiçbir şey çalışmaya başlamaz.
