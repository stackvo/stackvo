# Değişiklik isteme

Yapması gerekip yapmadığı bir şey. İyi bir istek bir durumu anlatır; özellik,
tartışmanın vardığı yerdir.

## Sormadan önce

- **Önce arayın.** Kapalı olanlar dahil
  [istekler](https://github.com/stackvo/stackvo/issues?q=is%3Aissue+label%3Aenhancement)
  ve [tartışmalar](https://github.com/stackvo/stackvo/discussions). İkizini
  açmak yerine var olan isteğe ekleyin.
- **Zaten var olmadığından emin olun.**
  [Günlük kullanım](../../guide/everyday-use.md), [SSS](../../reference/faq.md) ve en
  yakın karttaki **?** hızlı bakılır; isteklerin epeyi kimsenin bulamadığı bir
  ayar çıkar.
- **Neyin, neden reddedildiğini bilin.** Bazı bariz istekler ölçülmüş ve bir
  nedenle geri çevrilmiştir: gömülü Ollama, beşinci bir veritabanı, Mutagen.
  Nedenler kodda, açıkladıkları şeyin yanında durur. O nedene cevap veren bir
  istek, varlığından habersiz olandan çok daha ilginçtir.
- **Özellik olduğundan emin değil misiniz?** Önce
  [Discussions](https://github.com/stackvo/stackvo/discussions)'da sorun.

## İsteği yazın

Bir [özellik isteği](https://github.com/stackvo/stackvo/issues/new/choose)
açın. Formda, ne kadar yardımcı olduklarına göre sıralanmış dört alan var.

### Ne yapmaya çalışıyorsunuz

Özelliği değil durumu. "Sürekli X yapmak zorunda kalıyorum", "X için düğme
ekleyin"den fazlasını söyler: düğme en iyi cevap olmayabilir ve ne için
olduğunu bilmeden kimse buna karar veremez.

### Bugün onun yerine ne yapıyorsunuz

Geçici çözüm, ne kadar çirkin olursa olsun. Formdaki en yararlı alan budur:
boşluğun gerçekte ne kadara mal olduğunu söyler ki bu, bir özellik
tarifinin asla taşımadığı tek şeydir.

### Başka bir şey bunu yapıyor mu

Adını verin: Herd, DDEV, Lando, Laragon, ServBay, ne olursa. Kopyalamak için
değil; çalışan bir örnek, bir şeyin mümkün olup olmadığı tartışmasını bir
dakikada bitirir, yoksa bir hafta sürer.

### Uygulamanın hangi parçası

Projeler, servisler, terminaller ve loglar, profilleme, sertifikalar ve DNS,
ayarlar, CLI, MCP sunucusu. İsteği yönlendirir ve size de bir şey söyler: üç
parçaya dokunan bir değişiklik üç istektir.

## Sonra ne olur

İstekler okunur ve etiketlenir. Bazıları yapılır, bazıları ilgili bir
değişikliği bekler, bazıları bir sonraki kişinin bulması için nedeni yazılarak
reddedilir. Reddedilen bir istek kapanmış bir kapı değil, kayda geçmiş bir
cevaptır; bir cevapla tartışılabilir.

Kendiniz yapmak istiyorsanız başlamadan önce istekte söyleyin. Biçimi
üzerine on dakika, baştan yazılan bir pull request'ten kurtarır; gerisini
[Pull request açma](making-a-pull-request.md) anlatır.
