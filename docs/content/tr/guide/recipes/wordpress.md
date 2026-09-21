# WordPress

`https://blog.loc` adresinde bir WordPress sitesi; MySQL'i, her terminalden
WP-CLI'ı ve üzerinde çalıştığınız eklenti ya da temanın ait olduğu klasörde
olmasıyla.

## 1. Oluşturun

**Projeler → +**, sonra çerçeve şablonları altından **WordPress**. Adı `blog`
olsun. Şablon WordPress'i indirir ve sonucu sahiplenir: PHP, nginx ve belge
kökü (`public/` değil, proje kökü) gelenden okunur.

**Oluştur** imajı derler ve projeyi açar. `https://blog.loc` kurucuyu
gösterir; kurucu bir veritabanı ister.

## 2. Veritabanı verin

Çalışan örnek yoksa **Katalog → Yayında olanlar** MySQL kurar; **Proje → Bu
projenin ihtiyaç duyduğu servisler** projeye bildirir. Kart sunucu adını,
portu ve bir tıkla parolayı gösterir. Bunları WordPress kurucusuna girin ya
da `wp-config.php` dosyasını kendiniz yazın:

```bash
stackvo wp config create --dbname=blog --dbuser=... --dbpass=... --dbhost=...
stackvo wp core install --url=https://blog.loc --title=Blog --admin_user=admin --admin_email=siz@example.com
```

`stackvo wp`, projenin klasöründen, konteynerin içindeki WP-CLI'dır.

## 3. Eklenti ya da tema üzerinde çalışın

Proje klasörü WordPress köküdür; `wp-content/plugins/sizinki/` ve
`wp-content/themes/sizinki/` her zamanki yerindedir. Editörünüzde düzenleyin,
konteyner dosyayı anında görür. Eklenti kendi deposuysa `wp-content/plugins/`
içine klonlayın; **Proje → Bu deponun geri kalanı** kartı *sitenin* depo
olduğu durum içindir.

## 4. Mail

WordPress maili `wp_mail()` ile gönderir. **Mail → Mailpit etkinleştir**
yakalayıcıyı başlatır; Mail sayfasının söylediği sunucu ve porta yöneltilmiş
bir SMTP eklentisi her iletiyi **Mail** sayfasındaki gelen kutusuna koyar.
Makineden hiçbir şey çıkmaz.

## 5. Yüklemeler, sınırlar, PHP

Büyük yüklemeler ve uzun içe aktarmalar önce PHP'nin sınırlarına çarpar.
**Ayarlar → Web sunucuları → İstek sınırları** her PHP projesi için yükleme
boyutunu ve çalışma süresini belirler; **Proje → PHP ayarları** bu proje için
geçersiz kılar. PHP sürümü ve eklentiler projenindir, **Proje → Proje
ayarları** içinde; PHP 7.4'teki eski bir site ile 8.4'teki yenisi yan yana
çalışır.

## Her gün

```bash
stackvo wp plugin list
stackvo wp search-replace 'https://blog.example.com' 'https://blog.loc'
stackvo wp db export
stackvo shell
```

Bir üretim sitesinin veritabanı dökümü aynı yoldan geri yüklenir: önce
**Proje → Snapshot’lar** adlandırılmış bir kopya alır, içe aktarma adıyla
geri alınabilir.
