# Makine geneli komutlar

Bu çalışma alanındaki her projede çalıştırabileceğiniz komutlar.

Bir proje kendi komutlarını `stackvo.json`'da tanımlar. Bu, onun bir üst katmanı: çalışma alanınızın kökünde tek bir dosya, hepsine birden komut ekler ve bunun için kimsenin deposunu düzenlemeniz gerekmez.

## Dosya

Projelerinizin yanında bir `commands.json` oluşturun — yol panelin üstünde yazıyor. Bir projenin `commands` bloğuyla aynı şekildedir:

```json
{
  "commands": {
    "tail": {
      "exec": ["tail", "-f", "storage/logs/laravel.log"],
      "about": "Uygulama günlüğünü izle"
    },
    "shell": { "exec": ["bash"], "interactive": true }
  }
}
```

Her komut, her projenin hızlı komut menüsünde geldiği dosyayla işaretlenmiş olarak görünür.

## Bilinmesi gerekenler

- **`exec` bir liste, bir satır değil.** Kelimeler konteynere tek tek geçirilir; kabuk yoktur, yani boru, yönlendirme ve `&&` düz metindir. Tek satırda iki komut, iki komuttur.
- **Projenin konteynerinde çalışır, başka hiçbir yerde.** `host` biçimi yoktur. Bu makinede çalışması gereken bir adım _kancadır_ (hook); onu proje tanımlar ve çalışmadan önce bir özete karşı onaylanır.
- **`interactive: true`**, uygulama içi konsol yerine Tercihler'de seçtiğiniz terminali açar. Soru soran her şey için kullanın — bir REPL, bir kabuk.
- **Aynı id'yi kullanıyorsa projenin kendi komutu kazanır.** Onun dosyası işlenmiş ve paylaşılmıştır; bu dosya sizindir, ve panel her satırın hangi dosyadan geldiğini söyler.
- **Hâlihazırda gömülü olan bir id reddedilir** ve panel hangisi olduğunu söyler. `migrate` her yerde `php artisan migrate` demektir, ve etiketi çalıştırdığı şeyi anlatmayan bir düğme, hiç olmayandan kötüdür.
- **Arayüz dosyanın kendisidir.** Burada bilerek form yok: aynı JSON'u yazmanın ikinci bir yolu, bir editör kullandığınız ilk anda onunla çelişirdi.
