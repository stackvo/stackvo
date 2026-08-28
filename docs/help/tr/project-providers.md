# Veri çekme ve gönderme

Bu projenin verisinin gerçekten durduğu adlandırılmış yerler — bir staging sitesi, canlı — ve veritabanının bir kopyasını oradan çekmenin ya da buradakini geri göndermenin yolu.

Tarif `stackvo.json` içinde yazılı, yani depoyla birlikte geziyor ve klonlayan takım arkadaşınıza da geliyor.

```json
"providers": {
  "staging": {
    "about": "staging sitesi",
    "image": "ghcr.io/example/dbtools:1",
    "pull": ["fetch-dump", "--out", "dump.sql"],
    "env": { "REMOTE_HOST": "staging.example.com" },
    "secrets": ["SSH_KEY"]
  }
}
```

## Başlangıç noktaları

Hiç tarif yazmamış bir projeye, bu sürümün getirdiği üç tarif kartın üzerinde sunuluyor:

| Tarif | Neye ulaşıyor | İki yön de var mı? |
| --- | --- | --- |
| `mysql-remote` | Bu makinenin doğrudan erişebildiği bir MySQL veya MariaDB sunucusu | Evet |
| `postgres-remote` | Bu makinenin doğrudan erişebildiği bir PostgreSQL sunucusu | Evet |
| `upsun` | Bir Upsun (Platform.sh) ortamı, kendi CLI'ı üzerinden | Yalnız çekme — CLI'ın `db:sql`'i dosya değil sorgu alıyor |

**Eklemek hiçbir şeyi onaylamaz.** Tarif sizin `stackvo.json` dosyanıza yazılıyor ve sonra elle yazdığınız bir tarifle aynı onaydan geçiyor. Her birinde bir yer tutucu sunucu, veritabanı veya proje kimliği var; yani onayladığınız sürüm tanımı gereği eklenen sürüm değil — ve kart hangi kelimeleri değiştirmeniz gerektiğini sayıyor.

İnsanların sorduğu iki tarif burada yok, ve nedenleri aramaya çıkmadan önce bilmeye değer:

- **SSH artı `mysqldump`** — bir tarifin konteyneri sizin makinenizden hiçbir yol almıyor, yani ajan soketi de anahtar dosyası da yok; `ssh` ise ne bir ortam değişkeniyle ne de bir komut kelimesiyle kimlik doğruluyor. Engel eksik kabuk değil: `mysqldump --result-file` boruya ihtiyaç duymuyor.
- **Pantheon** — çalıştırılacak bir Terminus imajı yok, ve `terminus backup:get` indirilmesi gereken bir URL döndürüyor; bu ikinci bir komut, dolayısıyla bir kabuk demek.

## Kurallar, ve her birinin sebebi

**Komut bir kelime listesidir, komut satırı değil.** Kabuk yok; boru da yok, yönlendirme de, `$DEĞİŞKEN` de. `["pg_dump", "-Fc"]` yazın, `"pg_dump -Fc | gzip"` değil.

**Bu makinede değil, bir konteynerde koşar.** Tarif bir depodan geliyor, ve depo klonladığınız şeydir.

**Parolalar ve anahtarlar adlandırılır, yazılmaz.** `secrets` altında listeleyin, değerleri karttan doldurun. İşletim sisteminizin anahtarlığına gider ve konteynere yalnız tek bir koşu boyunca ortam değişkeni olarak ulaşır.

**Çekme `/stackvo/dump.sql` yazar; gönderme onu okur.** Bu yol sabittir. StackVo oraya kendi geçici klasörünü bağlar ve sonrasında siler.

## Denetimler

| Denetim | Ne yapar |
| --- | --- |
| Veritabanı | Çekilenin hangi örneğinize ineceği, ya da gönderilenin hangisinden okunacağı. |
| Çekmeyi onayla / Göndermeyi onayla | Tam olarak o komutu onaylar. Her yön için ayrı ayrı. |
| Şimdi çek / Şimdi gönder | Koşturur. |
| Önce yerine geçeceği şeyin kopyasını al | Çekilen üstüne yazmadan önce yerel veritabanının anlık kopyasını alır. Varsayılan olarak açık. |
| Onayı geri çek | Bir dahakine yeniden sorar. |

## Bilinmesi gerekenler

- **Çekmeyi onaylamak, göndermeyi onaylamak değildir.** Ayrı ayrı onaylanır ve biri ötekini ucuzlatmaz.
- **Tarifi düzenlemek yeniden sordurur.** Onay imajı, komutun her kelimesini, ortamı ve gizli değerlerin adlarını kapsar. `about`'u yeniden yazmak kapsamaz, çünkü o hiçbir şeye karar vermiyor.
- **Çekme geri alınabilir.** Dumps kartının kullandığı restore ile bitiyor, ve o restore yerine geçeceği şeyin kopyasını alıyor.
- **Gönderme geri alınamaz.** Bu makine olmayan bir yere yazıyor. Denetim kaydına yazılıyor; çekme yazılmıyor, çünkü çekme yalnız bu makineyi değiştiriyor.
- Hiçbir şey zamanlamayla göndermiyor, ve gönderecek bir yol da yok.
- Bir yönetici her iki yönü de bütün bir filo için kapatabilir. Sizin adınıza onaylayamaz.
