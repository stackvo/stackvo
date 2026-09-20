# Laravel

Hiçten `https://shop.loc` adresine bir Laravel projesi; veritabanıyla, çalışan
kuyruk işçisiyle ve gönderdiği her mailin pencerede yakalanmasıyla.

## 1. Oluşturun

**Projeler → +**, sonra çerçeve şablonları altından **Laravel**. Adı `shop`
olsun; `shop.loc` alan adı ardından gelir. Çerçevenin kendi kurucusu geçici
bir konteynerde çalışır ve sonuç sahiplenilir: PHP, nginx ve `public/` belge
kökü tahmin edilmez, yazdıklarından okunur.

!!! tip "İlk şablon birkaç dakika sürer"

    Kurucunun imajı bir kez indirilir. Sonraki Laravel projesi hızlıdır.

**Oluştur** imajı derler, farkı gösterdikten sonra hosts kaydını yazar ve
projeyi açar. `https://shop.loc` sertifika uyarısı olmadan karşılama
sayfasıyla cevap verir.

## 2. Veritabanı verin

Projenin `.env` dosyası `DB_CONNECTION=mysql` der ve **Proje → Bu projenin
ihtiyaç duyduğu servisler** kartı bunu okumuştur: MySQL *önerilen* olarak
görünür. Henüz çalışan bir MySQL örneği yoksa **Katalog → Yayında olanlar**
bir tane kurar; sürümü seçin. Sonra **stackvo.json'a yaz**, öneriyi bir
bildirime çevirir; depoyu klonlayan iş arkadaşınız aynısını alır.

Kart bağlantı değerlerini gösterir, parolayı bir tıkla. Bunları `.env`
dosyasına koyun (`DB_HOST`, `DB_PORT`, `DB_USERNAME`, `DB_PASSWORD`) ve
projenin klasöründen migrate edin:

```bash
stackvo artisan migrate
```

`stackvo artisan` konteynerin içinde, konteynerin PHP'siyle çalışır. Ana
makinenizde PHP olması gerekmez.

## 3. Kuyruk ve zamanlayıcı

**Proje → İşçiler** projenin dosyalarının istediği süreçleri listeler: kuyruk
yapılandırılmışsa bir kuyruk işçisi, `routes/console.php` bir şey
zamanlıyorsa zamanlayıcı. Her birini **Başlat**; konteynerin yanında çalışır,
onunla durur. **Proje → Zamanlanmış işler** tek tek işlerin tablosudur; her
birinin son çalışması ve logu vardır.

## 4. Mail

**Mail → Mailpit etkinleştir** yakalayıcıyı başlatır. `.env` içinde mailer'ı
ona yöneltin (Mail sayfası sunucu adını ve portu söyler); projenin
gönderdiği her ileti **Mail** sayfasındaki gelen kutusuna düşer; HTML'i,
metni ve başlıklarıyla.

## 5. Redis, Horizon, Telescope

Redis'i MySQL gibi ekleyin: **Katalog → Yayında olanlar**, servisler kartında
bildirin, `.env` içine `REDIS_HOST`. Paketi kurmak projenin komutlarına
`redis-cli`'ı da verir.

Horizon, Telescope ve Pulse kurulum istemez: paket kurulduğu anda
`https://shop.loc/horizon` çalışır, çünkü proje zaten kendi alan adında ve
güvenilen bir sertifikayla cevap verir. **Proje → Telescope, Horizon ve
Pulse** kartı hangisinin neden boş olduğunu söyler.

## 6. Octane

**Proje → Proje ayarları → Web sunucusu**: `swoole` ya da `roadrunner`.
İkisi de HTTP sunucusunun kendisidir; proxy 80 yerine 8000'e bakar. Octane'in
yüklediği kodu değiştirdiğinizde **Proje → Octane yeniden yükle** işçileri
yeniden başlatır.

## Hata ayıklama

- **Proje → Xdebug → Etkin** eklentiyi imaja koyar ve IDE'nizin ihtiyaç
  duyduğu değerleri gösterir. Gerisi [Hata ayıklama](../debugging.md)
  sayfasında.
- **Proje → Hata ayıklama sinyalleri** `dump()` ve `dd()` çıktısını sayfa
  yerine pencerede yakalar.
- **Proje → Tarayıcı testleri (Dusk)** Dusk'ı projenin kendi alan adına
  karşı, CA konteynerin içinde güvenilir hâlde çalıştırır.

## Her gün

```bash
stackvo artisan tinker
stackvo composer require laravel/horizon
stackvo npm run dev
stackvo shell
```

Hepsi projenin klasöründen, hepsi konteynerin içinde.
