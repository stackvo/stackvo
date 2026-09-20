# Başka bir araçtan geçiş

StackVo yedi yerel ortamdan içe aktarır ve aktardığı araca hiç dokunmaz.

| Kaynak | Ne okur | Kaynak | Ne okur |
| --- | --- | --- | --- |
| **XAMPP** | `htdocs` | **Laravel Sail** | `docker-compose.yml` |
| **Laragon** | `www` | **Laravel Herd** | site listesi |
| **MAMP** | `htdocs` | **DDEV** | `.ddev/config.yaml` |
| **Laravel Valet** | park edilmiş ve bağlanmış siteler | | |

## Nasıl çalışır

1. **Projeler → ⋮** paneli açar. Bu makinede ne kurulu olduğunu bulur ve bir
   şey yapmadan önce **bulduğunu size gösterir**.
2. Bir site seçip **Benimse**'ye basın. Ne olduğu — çalışma zamanı, web
   sunucusu, belge kökü — klasördeki dosyalardan algılanır. **Tümünü benimse**
   her siteyi tek geçişte alır; hosts dosyası için site başına değil bir kez
   parola sorar.
3. Site çalışma alanınıza **kopyalanır**. **Kopyalamak yerine taşı** ile,
   kopya tamamlanınca orijinal silinir ve diğer araç o siteyi sunmayı bırakır.

## Üç kural

- **Diğer araca hiçbir şey yazılmaz.** PATH düzenlemesi yok, kapatılan servis
  yok, değişen ayar yok. Fikrinizi değiştirirseniz geri alınacak bir şey yok.
- **Varsayılan kopyalamadır.** Siz aksini söyleyene kadar orijinal yerinde
  kalır.
- **Önce görürsünüz.** Compose dosyasından türetilen bir projede, yazılacak
  `stackvo.json` ve her değerin nereden geldiği yazılmadan önce gösterilir.
  StackVo karşılığı olmayan servisler ayrıca listelenir; onlar sizin
  elinizde.

## İçe aktardıktan sonra

İçe aktarılan her site diğerleri gibi bir projedir: kendi konteyneri, kendi
çalışma zamanı sürümü, sizin ekiniz altında kendi adresi. Proje satırından
başlatın ve alan adını açın. Eski araç hâlâ 80 ya da 443 portunda çalışıyorsa
durdurun ya da bırakın — iki sunucu bir portu tutamaz.

Sonraki adım: [Günlük kullanım](../guide/everyday-use.md).
