# Docker motoru

Konteynerleri çalıştıran motorun durumu.

## Kartın gösterdikleri

| Alan | Anlamı |
| --- | --- |
| Durum | Çalışıyor ya da çalışmıyor. |
| Platform | Docker Desktop, Colima, OrbStack ya da Docker Engine. |
| Soket | Uygulamanın motora bağlandığı soket ya da adlandırılmış boru. |
| Bağlam | Kullanılan Docker bağlamı. |
| Sürüm | Motorun sürümü. |
| API sürümü | Konuşulan API sürümü. |

## Bilinmesi gerekenler

- Motor çalışmıyorsa uygulama hiçbir şey yapamaz. Kart, motoru başlatacak bir buton sunar.
- Birden fazla Docker kurulumu varsa bağlam hangisine bağlanıldığını söyler. Beklediğiniz konteynerleri görmüyorsanız önce buraya bakın.
