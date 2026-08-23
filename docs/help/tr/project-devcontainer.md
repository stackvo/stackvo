# Devcontainer

Bu projeye bir `.devcontainer/` klasörü yazar; böylece StackVo'su olmayan biri depoyu VS Code ya da GitHub Codespaces ile açıp aynı ortamı elde eder.

## Denetimler

| Denetim | Ne yapar |
| --- | --- |
| Ne yazılacağını göster | Her dosyayı üretir ve gösterir. Henüz hiçbir şey yazılmaz. |
| Projeye dosya yaz | Dosyaları `<proje>/.devcontainer/` içine yazar. |

## Neyi taşır

- **Aynı konteyner.** Dockerfile, StackVo'nun bu projeyi derlediği dosyanın kendisi: aynı PHP sürümü, aynı eklentiler, aynı web sunucusu.
- **Projenin bildirdiği servisler**, bu makinenin kurduğu paketlerden — aynı imaj, aynı sürüm.
- **Sizin konteyner adlarınız.** `stackvo-mysql-8-4` aynen öyle kalır, çünkü projenin kendi `.env` dosyası o adı yazıyor. Burada değiştirmek, nedenini göremeyecek makinede uygulamayı bozardı.
- **Bu çalışma alanının verdiği portlar**, böylece o portlara ayarlı bir veritabanı istemcisi çalışmaya devam eder.

## Neyi taşımaz, ve neden

- **Alan adı ve HTTPS'i.** `shop.loc` bu makinenin güven deposuna kurulmuş bir sertifika otoritesi ve bu projenin parçası olmayan bir yönlendirici sayesinde çalışıyor. Uygulamaya bunun yerine iletilen porttan erişilir.
- **Parolalar.** Her biri değer olarak değil ad olarak çıkar — `DEV_MYSQL_8_4_ROOT_PASSWORD` — ve Compose bunu `.devcontainer/.env` dosyasından okur. Yanına o tek satırı içeren bir `.gitignore` yazılır.
- **İçinde parola geçen servis ayar dosyaları.** `my.cnf` taşınır; parola içeren biri taşınmaz, çünkü bir yer tutucu compose dosyasında doldurulabilir, ayar dosyasında doldurulamaz.

## Bilinmesi gerekenler

- Bu dosyalar commit edilmek içindir. Amaç budur.
- Her seferinde manifest'ten yeniden üretilirler. Bunları düzenlemek yerine `stackvo.json`'ı düzenleyip yeniden yazın.
- PHP projesinde bağımlılıklar sizin için kurulmaz. Manifest'te projenin Composer kullandığını söyleyen hiçbir şey yok, ve ilk açılışta düşen bir komut hiç olmayandan kötüdür. Node ve öteki çalışma zamanları kurar, çünkü komutu manifest'lerinde yazılı.
- Node, Python, Go, Ruby, Rust, Bun ve Deno projeleri yalnız araç zincirini alır ve ayakta kalmaları söylenir. Editörünüz o konteynere bağlanır; uygulama onun ana süreci değildir, onu siz başlatırsınız.
