# Zamanlanmış işler

Zamanlayıcıya bağlı, adı olan işler; her birinin kendi sıklığı, kendi son çalışması ve kendi logu var. Yukarıdaki İşçiler paneli Laravel'in kendi zamanlayıcısını tek bir süreç olarak çalıştırır; burası ise tek tek işlerin tablosu ve farklı bir soruya cevap verir — "zamanlayıcı ayakta mı?" değil, "*o iş* çalıştı mı?".

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Başlat / Durdur | Zamanlayıcı yan konteynerini ayağa kaldırır ya da indirir. O indiyken hiçbir iş tetiklenmez. |
| Yeni iş | Formu açar. Kaydedene kadar hiçbir şey yazılmaz. |
| Şimdi çalıştır | Bir işi hemen çalıştırır — zamanlanmış bir tetiklemeyle aynı yoldan, yani logu ve son çalışması da aynı şekilde yazılır. |
| Duraklat / Sürdür | İşi komutunu kaybetmeden zamanlamadan çıkarır. |
| Log | O işin kendi logunun sonu. |

## İş türleri

| Tür | Ne çalıştırır |
| --- | --- |
| Laravel zamanlayıcı | Her tetiklemede bir kez `php artisan schedule:run`. Zamanlamanız `routes/console.php` içindeyse bunu seçin. |
| Artisan komutu | `php artisan` ve yazdığınız komut. |
| Özel komut | Yazdığınız komut, olduğu gibi. |

Yazdığınız her kelime ayrı bir argüman olur. Kabuk yoktur; `&&`, boru, joker ve `$DEĞİŞKEN` çalışmaz — bu bilinçlidir, proje hook'larının aynı şekilde çalışmasıyla aynı nedenle. Bunlara ihtiyacı olan bir iş aslında bir betiktir: betiği adlandırın, örneğin `sh scripts/gecelik.sh`.

## Sıklık

Hazır bir seçenek seçin ya da **Gelişmiş**'i seçip cron ifadesini kendiniz yazın. Beş alan — dakika, saat, ayın günü, ay, haftanın günü — ve sözdiziminin taşınabilir alt kümesi: `*`, bir sayı, `a-b`, `*/n`, `a-b/n` ve bunların virgülle ayrılmış listeleri. `MON` gibi adlar ve `@daily` gibi makrolar kabul edilmez; sayıları yazın.

## Satırlar ne söyler

| İşaret | Anlamı |
| --- | --- |
| Yeşil saat | İş zamanlamada. |
| Gri duraklat | İş duraklatılmış. Komutunu ve logunu korur. |
| Son çalışma | En son ne zaman çalıştığı ve işe yarayıp yaramadığı. Kırmızı, başarısız demektir. |
| Yeniden başlatma sayısı | Motorun zamanlayıcıyı kaç kez yeniden başlattığı. Sıfırsa gösterilmez. |

## Bilinmesi gerekenler

- Önce projenin çalışıyor olması gerekir: iş, projenin kendi imajından türetilen bir yan konteynerde çalışır, yani siteyle aynı PHP'yi, aynı eklentileri ve aynı `.env`'i görür.
- Zamanlayıcıyı Docker `unless-stopped` ile denetler; bu uygulama kapalıyken de işleriniz tetiklenir.
- Zamanlama `stackvo.json` içinde durur, yani depoyla birlikte taşınır. Klonlayan aynı işleri alır.
- İşin adı logunun da adıdır — "Önbellek temizliği" `onbellek-temizligi.log` yazar. Bir işi yeniden adlandırmak eski logu yeniden adlandırmaz, yenisini başlatır.
- Bir işin kaydettiği başarısızlık bir sayı değil evet/hayırdır: başarısız olan her komut aynı kodu bildirir, bu yüzden sebep durumda değil logdadır.
