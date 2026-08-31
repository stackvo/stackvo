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

## Üçüncü katman: bir paketin getirdiği komutlar

O menüde bir kaynak daha var, ve o sizin yazdığınız bir dosya değil.

**Bir paket kendi komutlarını getirebilir.** Redis paketini kurmak size `redis-cli`'yi de verebilir — `ddev-redis` kurmanın `ddev redis-cli`'yi vermesi gibi, tek farkla: her baytı okunmadan önce doğrulandı. Bir imza kayıt defterine kefil oluyor, kayıt defteri manifestin özetini söylüyor, manifest yanındaki her dosyanın özetini söylüyor — ve bunların hepsi yalnız kurulumda değil **her okumada** yeniden kontrol ediliyor.

O satırlar yukarıdaki ikisinden, menünün size söylediği bir biçimde farklı:

| | Nerede çalışır | Nereden gelir |
| --- | --- | --- |
| Gömülü ve proje satırları | **projenizin konteynerinde** | bu uygulama ya da projenin `stackvo.json`'ı |
| Makine geneli satırlar | **projenizin konteynerinde** | sizin `commands.json`'ınız |
| **Paket satırları** | **o servis örneğinin konteynerinde** | kurulu paket |

Bir paket satırı, çalıştığı örnekle etiketleniyor — `redis-7-2 içinde` — çünkü *"bu, projenize dokunmuyor"*, birine bir düğmeye basmadan önce söylenecek türden bir şeydir.

Sınırlama buradaki her yerdeki biçimin aynısı:

- **Yalnızca açık örnekler.** Çalışması beklenmeyen bir konteynere karşı bir komut, hatası `No such container` olan bir düğmedir.
- **Konteyner adı türetilir, asla beyan edilmez.** Bir paket, bir host portunu adlandıramadığı gibi başkasının konteynerini de adlandıramaz.
- **Id örneği taşır** — `redis-7-2:redis-cli` — yani bir servisin kurulu iki sürümü tek bir çakışma değil iki satırdır, ve bir proje aynı dizgeyi beyan ederek bir paketin komutunu gölgeleyemez.
- **Açıklama paket yazarınındır**, hangi dilde yazdıysa o dilde, ve pencere kendi dilini onun adına iddia etmek yerine bunu işaretler.
