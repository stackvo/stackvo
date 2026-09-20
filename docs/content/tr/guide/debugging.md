# Hata ayıklama

Dört araç, hepsi projenin **Hata ayıklama** sekmesinde; artı her projenin
sinyallerini tek yerde toplayan iki sayfa.

<figure markdown>
![Bir projenin Hata ayıklama sekmesi](../screenshots/web/project-detail-debugging.webp){ loading=lazy }
<figcaption>Hata ayıklama: Xdebug, profilleyici, sorgu kaydı ve dump'lar tek isteğin etrafında.</figcaption>
</figure>

## Xdebug

**Proje → Xdebug → Etkin.** İlk seferde eklenti imaja girer ve yeniden
kurulum ister; sonraki her seferde yalnızca konteyner yeniden başlar ve
eklenti kapalıyken hiçbir maliyeti yoktur.

Kart IDE'nizin istediklerini listeler — port, IDE anahtarı, sunucu adı — ve
yol eşlemesini: konteynerdeki `/var/www/html`, dışarıdaki proje klasörünüz.
Bir şeyin dinleyip dinlemediğini de söyler; "kesme noktam neden vurmuyor" tek
ekranda cevaplanır.

## Dump'lar, istekler ve işler

**Proje → Hata ayıklama sinyalleri → dump() ve dd() yakala**. Üç şey cevabın
içine düşmek yerine buraya gelir:

- **Dump'lar** — değer, dosya ve satır. Satıra tıklayın, editörünüzde açılır.
- **İstekler** — çalıştırma başına bir satır, durum ve süreyle; fatal ile
  ölenler ve `artisan` komutları dâhil.
- **İşler** — uygulamanın başlattığı worker'dan deneme başına bir satır,
  bitti mi fırlattı mı.

Yakalama kapalıyken hiçbir şey birikmez. Önce açın, sonra incelediğiniz
sayfayı yenileyin. Yakalama sayfalar arasında açık kalır; kuyruk worker'ından
gelen bir dump siz başka yerdeyken de yakalanır — kenar çubuğundaki
**Dump'lar** sayfası bunların bütün projelerden biriktiği yerdir.

## Loglar

**Proje → Loglar** konteynerin çıktısını ya da projenin yazdığı herhangi bir
log dosyasını okur; kaynağı üstten seçin. Arama, düzenli ifade, seviye
filtresi, takip ve duraklat. Yığın izindeki konteyner yolu tıklanabilir.

Kenar çubuğundaki **Loglar** sayfası her projenin çıktısını tek canlı akışta
gösterir — başka yerde çalışırken açık bırakılacak sayfa.

## Bu istek neden yavaştı?

**Proje → Örnekleyici profilleyici (php-spx) → Buradan kaydet**, sayfayı
tarayıcıda yükleyin, durdurun. Sonra **Proje → Bu istek neden yavaştı**
sayfasını açıp bir isteğe tıklayın: üç kaynak tek eksende hizalanır:

- örnekleyici profilleyicinin (php-spx) gördüğü,
- veritabanına gerçekten ne sorulduğu — aynı sorgunun 40 kez sorulması dâhil,
- uygulamanızın kendi `dump()` çağrıları.

Bulgular önce gelir, iki renkte: **kehribar** değiştirilecek bir şey, **mavi**
kanıtın kapsayamadığı bir şey. Sonra çubuk: veritabanı sürücüsündeki zaman ve
geri kalanı.

**Yeniden oynat** kaydedilen isteği profilleyici açıkken yeniden gönderir ve
iki sayıyı yan yana koyar — "değişikliğim işe yaradı mı" tek tıkla.

## Profilleyici ve izler

**Proje → Profilleyici** Xdebug'ın kendi profilleyicisidir: adım adım hata
ayıklama, profilleme ya da iz, tek seferde biri. Bir istek istemeden hiçbir
şey kaydedilmez — adrese `?XDEBUG_TRIGGER=1` ekleyin, kayıt alev grafiği
olarak açılır.
