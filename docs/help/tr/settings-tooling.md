# Araçlar

`stackvo`'yu kabuğunuzun bulabileceği yere koyar ve bu uygulamanın host'ta çalıştırdığı araçları bildirir.

## Komutlar

`stackvo` yığını terminalden çalıştırır — `stackvo up`, `stackvo artisan migrate`, `stackvo logs`. `stackvo-mcp` ise **Yapay zekâ asistanları** sayfasının tanıttığı sunucudur. Kurulu bir StackVo **ikisini de taşır**, kendi ikilisinin yanında; yani sayfa onları hiçbir şey derlenmeden bulur. Bir checkout'tan derlemek için:

```
npm run sidecars
```

İkisi de uygulamanın kendi dizinine bağlanır — macOS'ta `~/Library/Application Support/StackVo/bin`, Windows'ta `%APPDATA%\StackVo\bin`, Linux'ta `~/.local/share/stackvo/bin`. `~/.stackvo` değil: orası *yığının* durumu, başka bir yeri gösterebilirsiniz ve onu silmek baştan başlamanın desteklenen yolu. Yığınınızı sıfırladığınızda kaybolan bir `PATH` girdisi, hiçbir yeri göstermeyen bir girdidir.

macOS ve Linux'ta girdiler sembolik bağ, yani yeniden derleme hiçbir şeye basmadan devreye girer. Windows'ta kopya — orada sembolik bağ bu uygulamanın istemediği bir yetki gerektiriyor — dolayısıyla güncellemeden sonra yeniden basmak gerekir.

## PATH'iniz

| Kontrol | Ne yapar |
| --- | --- |
| Ekle | İki komutu da bağlar ve o kabuğun başlangıç dosyasına tek satır yazar. |
| Güncelle | Eski bir dizini gösteren satırı değiştirir. |
| Kaldır | Satırı geri alır. Bağlantılar kalır. |
| Satırı kopyala | Satırın kendisi — bu uygulamanın düzenlememesi gereken bir yere yapıştırmak için. |

Kabuk başına tek dosya: zsh için `.zshrc`, bash için macOS'ta `.bash_profile` ve diğer yerlerde `.bashrc`, fish için `config.fish`, Windows'ta da PowerShell profili. macOS Terminal login kabuğu açar, Linux terminalleri açmaz; bash'in platforma göre değişmesinin sebebi bu.

### Basmakta ne güvenli

Yalnızca `# stackvo:path:begin` ile `# stackvo:path:end` arasındaki bölge yazılır. O dosyadaki başka her şey bayt bayt geri gelir, işareti olmayan bir dosya yeniden yazılmaz sonuna eklenir, ve önce yanına `.stackvo-backup` kopyası bırakılır. Kaldırma da yalnız o bölgeyi alır.

Dizin `PATH`'e **başa** eklenir. İçindeki tek adlar `stackvo`, `stackvo-mcp` ve bu uygulamaya yönetmesini söylediğiniz araçlar; sona eklemek, yönetilen bir `mkcert`'in yarım kaldırılmış bir kopyaya yenilmesi demek olurdu — ki bu düğmeye tam da o durumdan çıkmak için basılır.

### Bir sonraki kabuğa uygulanır

Başlangıç dosyası kabuk açılırken okunur. Halihazırda açık olan terminaliniz değişikliği görmez; yeni bir tane açın ya da dosyayı `source` edin. Bu doğru olduğu sürece sayfa bunu söyler.

## Host araçları

Dört program, ve bunlar uygulamanın her konteynerin **dışında** çalıştırdığı dört program:

| Araç | Olmazsa |
| --- | --- |
| Docker | Hiçbir şey çalışmaz. Her proje bir konteyner. |
| Docker Compose | Üretilen yığın compose dosyaları; onları bu çalıştırır. |
| Git | Worktree yok, proje sayfalarında dal adı yok, yeni projeye klonlama yok. |
| mkcert | Yığın yine çalışır, her tarayıcı `.loc` için uyarır. |

**sizinki** rozeti, bulunan kopyanın sizin olduğu anlamına gelir — Homebrew'un, dağıtımın, Docker Desktop'ın — ve bu uygulama ona dokunmaz. **yönetilen** ise bu uygulamanın kurduğu anlamına gelir.

Yalnız mkcert'te Kur düğmesi var, ve bu bir eksiklik değil. Docker, arkasında bir kurulum ve bir sanal makine olan bir uygulama; `PATH`'e bırakılmış çıplak bir istemci yokluğundan kötü olurdu, çünkü o zaman `docker` eksik olmak yerine `docker ps` hata verirdi. Git her platformla gelir, macOS'ta istemek de bu uygulamanın yarışmaması gereken bir sistem kurulumunu açar.

### İndirilen ne ile karşılaştırılır

SHA-256, StackVo'nun bu yapısına gömülüdür — platform başına bir tane. Dosyayla birlikte **çekilmez**: tarif ettiği şeyin yanında sunulan bir sağlama toplamı bir kontrol değildir, çünkü birini değiştirebilen ötekini de değiştirebilir. Baytlar eşleşmeden hiçbir şey yazılmaz, eşleşmezlik de yeniden denenmez, reddedilir.

Bilerek Güncelle düğmesi yok. Sessizce upstream'i takip eden bir kurulum, sabitlenmiş bir sağlama toplamının sabit olmaktan çıkma yoludur; yeni bir mkcert, birileri ona baktıktan sonra bu uygulamanın bir sürümüyle gelir.

## Burada olmayanlar

`composer`, `node`, `npm`, `bun`, `wp`. Bu türden diğer araçlar bunları host'unuza indirir ve `PATH`'e sarmalayıcı koyar. Burada onlar projenin konteynerinde, o projenin bildirdiği sürümle çalışır — `stackvo composer install`, ya da projenin kendi sayfasındaki düğmeler. Host'taki bir kopya, "hangi composer çalışıyor" sorusuna ikinci bir cevap olurdu ve yanlış olan o olurdu: projenin PHP'sinden haberi yok.
