# Ön ayarlar

Bir görünümü adlandırıp saklayın, tek tıkla geri dönün — ve başka bir yere taşıyın.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Ön ayar adı | Kaydedilecek ismi yazarsınız. |
| Kaydet | Şu anki tüm görünüm ayarlarını o isimle saklar. |
| Ön ayara tıklamak | O görünümü uygular. |
| Sil | Ön ayarı kaldırır. |
| Ayar olarak kopyala | Şu anki görünümü JSON olarak panoya koyar; bu uygulamanın başka bir kopyası için. |
| Vuetify teması olarak kopyala | Her iki temayı taşıyan bir `createVuetify` çağrısı olarak panoya koyar; bu uygulama olmayan bir proje için. |
| Bir görünümün JSON’u | Başka yerden kopyalanmış ayarları yapıştırıp İçe aktar’a basarsınız. |

## Bilinmesi gerekenler

- Bir ön ayar tema, renk, palet, yazı tipi, ölçek, yoğunluk ve köşe yuvarlaklığını birlikte tutar.
- Ön ayarlar bu makinede yaşar ve çalışma alanıyla taşınmaz. Bir görünümün başka bir makineye gitmesinin yolu, onu ayar olarak kopyalamaktır.
- İçe aktarım uygulanmadan önce denetlenir. Bu sürümün tanımadığı bir alan ya da kontrollerin sunmadığı bir değer atlanır ve mesajda adıyla söylenir; görünümün geri kalanı yine de gelir.
- Hiç görünüm olmayan bir yapıştırma kısmen uygulanmaz, tümden reddedilir — yanlış bir yapıştırma, üzerine eklemeye çalıştığınız görünümü sıfırlayamaz.
- Vuetify tema parçası saklanmaz, üretilir. Açık ve koyu paletleri uygulamanın çizdiği hâliyle taşır; ana renginizden türetilen ikincil renk de dahil.
