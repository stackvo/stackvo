# Yapılandırma

Bu projenin `stackvo.json` dosyasında yazan hâli. Alanlar salt okunurdur; değiştirmek için karttaki **Yapılandır** butonunu kullanın.

## Alanlar

| Alan | Anlamı |
| --- | --- |
| Alan adı | Projenin tarayıcıda açıldığı ad. |
| Takma adlar | Aynı projeye giden ek adlar. `*.` ile başlayan bir takma ad joker'dır: sertifikaya ve yönlendiriciye girer ama hosts dosyasına giremez, o yüzden tek başına çözülmez. |
| PHP / Node sürümü | Konteynerin çalıştırdığı sürüm. |
| Konteyner yolu | Kodunuzun konteyner içindeki yeri. Her zaman `/var/www/html`. |
| Erişim URL · HTTP / HTTPS | Projenin yanıt verdiği adresler. |
| SSL durumu | Sertifikanın verilip verilmediği. |
| Sunucu | nginx, Apache ya da Swoole. |
| Host yolu | Projenin bu makinedeki klasörü. |
| Tür | Projenin şablonu. |
| Doküman kökü | Web sunucusunun yayımladığı alt klasör. Laravel'de `public`. |

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Yapılandır | Proje ayarları panelini açar. Buradaki alanların çoğu oradan değişir. |
| Kopyala | Değeri panoya alır. |
| Adrese tıklamak | Adresi tarayıcıda açar. |

## PHP eklentileri

Konteynerde derli olan eklentilerin listesi. Eklenti eklemek imajı değiştirir, yani projeyi yeniden derlemek gerekir.

## Sorun bölümü

`stackvo.json` sözleşmeye uymuyorsa buraya yazılır. Hata kodu, dosyadaki yol ve açıklama gösterilir. Uyarılar projeyi çalıştırmayı engellemez; hatalar engeller.

## Bilinmesi gerekenler

- Alan adı çözülmüyorsa bu kart bir uyarı ve hosts kaydını ekleyecek bir buton gösterir.
- Buradaki değerleri değiştirmek çoğu zaman yeniden derleme ister. Yeniden başlatmak yetmez.
