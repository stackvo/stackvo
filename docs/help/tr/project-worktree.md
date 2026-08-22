# Worktree'ler

Bir git dalına kendi ortamını verir: kendi klasörü, kendi adresi, kendi veritabanı. İki dal aynı anda çalışır ve checkout'unuza git'in fark edeceği hiçbir şey yazılmaz.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Yeni worktree | Formu açar. |
| Dal | Hangi dalın ortamı olacağı. Zaten başka bir yerde checkout edilmiş bir dal seçilemez. |
| Dalı oluştur | Dal yoksa yenisini açar. |
| Ad | Yeni projenin adı. Boş bırakılırsa daldan türetilir. |
| Veritabanı | Yok, yeni ve boş, ya da bu çalışma alanınınkinin kopyası. |
| Hangi motorda | Kopyanın hangi veritabanı örneğinde duracağı. |
| Oluştur | Klasörü, projeyi ve seçilen veritabanını hazırlar. |
| Kaldır | Worktree'yi siler. Dalı ve veritabanını silmek ayrı anahtarlardır, ikisi de varsayılan olarak kapalıdır. |

## Form ne gösterir

Ad, adres ve veritabanı adı formda önceden gösterilir. Bu isimler arka uçtan gelir, yani ekranda gördüğünüz, oluşturulacak olanın aynısıdır.

Bir şey yapılamıyorsa buton pasif kalır ve sebebi ilgili alanın yanında yazar.

## Bu proje bir worktree ise

Kart bunun yerine hangi projenin dalı olduğunu, dalını, adresini ve veritabanını gösterir. Worktree'nin worktree'si oluşturulamaz.

## Bilinmesi gerekenler

- Veritabanı kopyalamak, kaynağın boyutuna göre zaman alır.
- Kaldırmak klasörü siler. Orada commit'lenmemiş bir çalışmanız varsa önce commit'leyin.
