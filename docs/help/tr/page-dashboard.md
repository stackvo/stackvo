# Panel

Yığının ve makinenin anlık durumu. Buradaki hiçbir şey bir ayar değildir; hepsi ölçümdür.

## Genel bakış

| Kart | Ne gösterir |
| --- | --- |
| Projeler | Kaç projenin çalıştığı, kaç projenin durduğu. |
| Servisler | Kaç servis örneğinin aktif olduğu. |
| İmajlar | Bu makinedeki Docker imajlarının sayısı ve boyutu. |
| Sağlık | Çözülmeyen alan adı, eksik sertifika gibi dikkat isteyen durumlar. |

## Ölçümler

| Kart | Ne gösterir |
| --- | --- |
| İşlemci yükü | Makinenin işlemci kullanımı; sistem, kullanıcı, nice ve boşta olarak ayrılmış. |
| İşlemci geçmişi | Son ölçümlerin grafiği. |
| Bellek | Kullanılan ve boş bellek. |
| Disk G/Ç | Anlık okuma ve yazma hızı, geçmişiyle. |
| Ağ trafiği | Anlık indirme ve yükleme, geçmişiyle. |

## Bilinmesi gerekenler

- Ölçümler bu makinenin tamamına aittir, tek bir projeye değil. Bir projenin kendi kullanımı için o projenin Gösterge sekmesine bakın.
- İşlemci dağılımı ilk ölçümde görünmez. Sayaçlar birikimlidir, yani ilk okuma uygulamanın kendi açılışını anlatırdı; ikinci ölçüm gelene kadar beklenir.
- Sağlık kartındaki bir uyarı genellikle tek tıkla çözülür: eksik hosts kaydı için sunulan buton onu ekler.
