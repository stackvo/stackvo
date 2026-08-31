# Bu konteynerin içindeki MCP sunucusu

Laravel Boost, bir asistana yalnızca sizin uygulamanızın cevaplayabileceği soruları veriyor: `users` tablosunda ne var, hangi rotalar tanımlı, bu framework sürümü ne diyor, ve gerçekten çalıştırabileceği bir `tinker`. Bunun için küçük bir MCP sunucusu kuruyor.

Sonra onu kaydediyor — ve yazdığı satır `php artisan boost:mcp`.

## O satır burada niye çalışmaz

Makinenizde bir `php` olduğunu varsayıyor. StackVo oraya bir php koymuyor, ve bu bir eksiklik değil bir karar: **PHP sürümü bir projenin özelliğidir, bir shim'in tahmin ettiği bir dizinin değil.** Projenizin PHP'si, uzantıları ve `php.ini`'si imajın içinde — depoyu açan her makinede aynı oldukları yerde.

Yani Laravel'in kendi kurucusu burada başlayamayacak bir yapılandırma üretiyor. Hiçbir şey uyarmıyor. Asistan başlamayan bir sunucu bildiriyor, ve sebebi sizin hiç görmediğiniz bir günlükte duruyor.

## Bu kartın onun yerine yazdığı şey

Bu uygulamanın zaten sahip olduğu geçiş:

```
docker exec -i stackvo-<proje> php artisan boost:mcp
```

`stackvo artisan` değil `docker exec`, iki sebeple. CLI hangi projeyi kastettiğini başlatıldığı dizinden çıkarıyor, ve bir asistan sunucularını nerede olursa orada başlatıyor — konteyneri adıyla söyleyen geçiş, yanlış projeyi seçemeyen geçiştir. Ayrıca `docker` bu uygulamanın zaten zorunlu koşulu, `stackvo` ikilisi ise `PATH`'inizde olmak zorunda değil.

`-i` var, `-t` yok: stdio üzerinden MCP bir borudur, ve bir TTY, JSON-RPC akışının ortasına satır disiplini koyar.

## İki sunucu, yan yana

Bu, StackVo'nun kendi MCP sunucusunun yerini almıyor; ikisi de ötekinin yerini alamaz:

| Sunucu | Cevapladığı soru |
| --- | --- |
| **StackVo** (Ayarlar → Asistanlar) | *"`shop.loc` niye açılmıyor?"* — preflight, hosts, sertifika SAN'ları, konteyner günlüğü |
| **Boost** (bu kart) | *"`users` tablosunda ne var?"* — şema, rota listesi, tinker, `artisan` envanteri, elinizdeki sürümün belgeleri |

Birincisi makine için bir kez kaydediliyor. Bu ise **proje başına**, proje dizininde duran dosyalara — çünkü yalnızca bu projenin konteyneri ayaktayken var olan bir sunucu, diskinizdeki her dizin için geçerli olan bir dosyaya ait değildir.

## Ne okunuyor, ve ne asla tahmin edilmiyor

| Olgu | Nereden geliyor |
| --- | --- |
| `laravel/boost`, `laravel/mcp` ya da `laravel/ai` kurulu mu | `composer.lock` — bağımlılık kartının okuduğu dosyanın aynısı, yani ikisi çelişemez |
| Bu projenin yayımladığı sunucular | kendi `routes/ai.php`'niz — içindeki `Mcp::local()` ve `Mcp::web()` satırları |
| Bugün ne kayıtlı | bu projedeki `.mcp.json`, `.cursor/mcp.json`, `.vscode/mcp.json` |

İlk argümanı bir sabit ya da bir değişken olan bir kayıt **atlanıyor**, tahmin edilmiyor. Okunacak bir dizge yok, ve burada uydurulan bir ad, asistanınızda düşen bir `artisan mcp:start something` demek olurdu.

## `Mcp::web()` hiçbir şeye ihtiyaç duymuyor

Bir `Mcp::web()` kaydı, **uygulamanızın içinde sıradan bir rotadır**. Zaten bu projenin kendi alan adı altında, tarayıcınızın güvendiği sertifikayla servis ediliyor. Başlatılacak bir süreç, uzatılacak bir sertifika, yazılacak bir hosts kaydı ve ikinci bir yönlendirici yok — o yüzden o satır size adresi gösteriyor ve düğme sunmuyor.

## Yazmanın uyduğu kurallar

Ayarlar'daki asistan kaydının uyduğu üç kuralın aynısı, aynı türden bir dosya üzerinde:

* **Oku, tek anahtarı değiştir, geri yaz.** Dosyadaki her şey — bu kodun hiç duymadığı anahtarlar dahil — hayatta kalır.
* **Ayrıştırılamayan dosya düzenlenmez.** İçinde yorum olan ya da yarısı düzenlenmiş bir `mcp.json` bildirilir ve olduğu gibi bırakılır.
* **Eski içerik saklanır**, dosyanın yanında `.stackvo-backup` olarak.

Ve bu karta ait bir kural daha: bu sunucuyu zaten çalıştıran bir kayıt **adını korur**. Onu yeniden adlandırmak, istemcinizde çalışan bir sunucu yerine iki sunucu bırakırdı.
