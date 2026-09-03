# StackVo Desktop — Açık İşler

**Tarih:** 2026-09-03
**Kapsam:** Yalnızca **geliştirmesi kalan** maddeler.

Bu belge her turda budanıyor: yapılan ve yapılmayacağına karar verilen her madde, alt
maddeleriyle birlikte siliniyor. Yapılanların gerekçesi ve ölçümü `CHANGELOG.md`'de,
mimarisi `ARCHITECTURE.md`'de duruyor — burada tekrarlanmıyor. Bir maddenin buradan
düşmesi, işinin bittiği anlamına gelir.

**Maddelerin işaretlenişi.** Burada yalnız **yapılmamış** iş duruyor: her madde ❌ ile
başlıyor ve başlığının sonunda `(yapılmadı)` taşıyor. Yarısı yapılmış maddelerin yapılmış
yarısı da silindi, yani bir başlık artık **yalnızca kalanı** adlandırıyor.

---

## Durum

| Blok | Madde | Ne bekliyor |
| --- | --- | --- |
| **A. Yayın koşusu** | 1 | Bir Windows makinesi |
| **B. Bakım borcu** | 2 | Sekiz elle geçiş, bir doğrulayıcı |
| **C. Arayüz borcu** | 3 | Aracın kendi sınırı, ilk yayın, bir ürün kararı |
| **D. Stratejik** | 1 | **Kullanıcıya sorulacak bir karar** |

**Dağıtım kararı: yalnız GitHub Releases, kod imzalama yok.** Uygulama hiçbir mağazada
yayınlanmayacak — App Store, Microsoft Store, Snap, Flathub — ve Homebrew cask / winget
gibi ikinci bir kanal da yok.

> **Kararın maliyeti, ve bu satır bilerek burada duruyor.** İmzasız ve notarize edilmemiş
> bir `.dmg`'yi macOS Gatekeeper *"bozuk, çöpe taşı"* diyerek açtırmaz; Windows'ta
> SmartScreen imzasız kuruluma engel çıkarır. Bedel yazılı ve duruyor — iki README ve
> `docs/RELEASE-NOTES-0.2.0.md` imzasız bir yapının iki platformdaki adımını veriyor.
> **Karar geri alınabilir:** imzalamaya dönülürse iş, o sırları depoya koymaktan ibaret.

**v0.2.0 çıktı: 2026-09-03, koşu `33797054815`, altı satır yeşil.** Üçüncü denemede: prova
yeşildi, ilk gerçek koşu altı satırda birden kırmızıydı — üç sebeple, üçü de bir provanın
göremeyeceği türden çünkü prova imzalamıyor (boş Apple sertifikası, secret'ın sonuna
yapışmış bir `%`, ve `tauri.conf.json`'daki genel anahtarın 28 Ağustos'ta tören
anahtarından başka bir anahtarla değiştirilmiş olması; hepsi `CHANGELOG.md`'de). Preflight
artık gerçekten bir dosya imzalıyor ve imzanın anahtarını conf'takiyle karşılaştırıyor.
Yayın sonrası ölçüm: `npm run updates:check` canlı uçta *"0.2.0 for all 6 platforms, each
with a url and a signature"*; bir paket güncelleyicinin yoluyla yetkisiz indirildi ve
imzasındaki anahtar kimliği (`3b1e8a8cfd48db9b`) uygulamaya derlenen anahtar. Kalan: A-1,
Windows'ta elle tur.

**İçerik anahtarının rotasyon penceresi bu yayınla kapandı, ve zincir tutarlı — ölçüldü.**
`signing.rs`'teki `PINNED` (`96e7a6d25e81509b`) 0.2.0'ın içinde ne varsa odur;
`stackvo-service-packages`'taki `registry.json.minisig` tam o anahtarla imzalı
(2026-08-28, *"rotate the key that signs the index"*). Yani bugün imzalı dizin 0.2.0'da
doğrulanır. Bilinmesi gereken tek şey anahtarların iki makinede olduğu: güncelleyici
anahtarı (`9BDB…`) bu Mac'te `~/.stackvo-keys/updater.key` ve GitHub secret'ında; içerik
anahtarının özel yarısı 28 Ağustos'ta `73572fd`'yi yapan **öteki** makinede. Bu Mac'teki
`registry.key` (`1b0f9a1fff196225`) eski ve hiçbir yerde pinli değil; öteki makinedeki
`updater.key` (`86F7…`) ise artık hiçbir yapıya derlenmiyor ve secret'a **konmamalı** —
karışıklığı önlemek için ikisi de yeniden adlandırılabilir.

---

## Süreç: nerede kaldık

| | Ne | Durum |
| --- | --- | --- |
| PR #74 | Kapıya üçüncü renk, tek koşu kilidi, Dependabot susturuldu | ✅ birleşti |
| PR #76 | CI'ın bulduğu iki ayak; `--windows` artık clippy de soruyor | ✅ birleşti |
| PR #77 | CodeQL action'ının üç yarısı birlikte | ✅ birleşti |
| PR #80 | vue + vuetify, lisans bildirimi yanında | ✅ birleşti |
| Depo ayarları | Dal koruması içe aktarıldı; Secret Protection + push protection açıldı | ✅ |

`main` yedi ayağın yedisinde de yeşil, ve artık **korumalı**: on zorunlu denetim,
doğrudan push yok, zorla push yok, ve `strict` — yani bir PR ancak `main`'in güncel
hâliyle sınandıysa birleşebiliyor. Bugün bir sabah boyunca kırmızı görünen Dependabot
PR'larının hepsinin sebebi buydu: eski bir `main`'in üstünde koşmuşlardı.

Açık kalan iki Dependabot PR'ı — #78 (`tauri-action` 0.6.2→1.0.0) ve #79
(`attest-build-provenance` 2.4.0→4.2.2) — **bilerek bekliyor.** İkisi de yalnızca
`release.yml`'de kullanılıyor ve o dosyayı hiçbir denetim koşturmuyor. Bekleme sebebi
2026-09-03'te kalktı: `release.yml` `tauri-action` 1.0.0 ile altı satırda yeşil döndü ve
v0.2.0'ı yayımladı. Bu ikisinin doğrulaması artık bir prova koşusu.

### Ve bugünün dersi, buraya yazılıyor çünkü tekrarlanabilir

**Kural: `tools/before-push.sh` koşmadan hiçbir dal push edilmez.** Platformlar arası
olanlar için `--all`.

Kural iki kez çiğnendi ve iki kez aynı bedeli ödetti: bir kez `cargo test` ile `npm test`
yeşil diye "ağaç yeşil" denip gönderildiğinde — sonraki koşu dört ayağın dördünde de
kırmızıydı — bir kez de son düzenleme kapı koştuktan **sonra** yapıldığında (`types:tsc`).
İkisinde de kapı aynı hataları saniyeler içinde veriyordu; öğrenilme biçimi bir merge ve
bir üç-işletim-sistemi matrisi oldu.

**Kapının kendisi de ölçülmeli, ve ölçüldükçe delik çıktı.** `--windows` yalnız `check`
koşuyordu; `ci.yml` ise Windows'ta clippy'yi `-D warnings` ile koşuyor, ve aradaki fark
iki `cfg(windows)` hatasıydı — adım artık clippy'yi de soruyor, ve bu kendi kaldırılışına
karşı denendi. `--windows-test` on dakika derleyip "bu konteyner ARM64 Windows'u
derleyemiyor" diyordu, ki o bir testin düşmesiyle **aynı renkte** görünüyordu — artık bir
saniyede, `skipped` olarak, gerekçesiyle; kapının üçüncü rengi bunun için var. İki kapı
aynı anda koşunca birbirine hayalet kırmızı üretiyordu — artık `mkdir` kilidi var.
Konteyner imajı sessizce bayatlıyordu — artık Dockerfile'ın sha256'sı imaja etiket olarak
yazılıyor ve her koşuda karşılaştırılıyor.

**Makineyi okuyan bir test, kodu okumaz.** `apps` testi kurulu terminali olmayan bir
makinede düşüyordu, `pty` testleri ortam değişkenlerini okuyordu, `key_ceremony` Git
Bash'in yol ad alanını Windows'unkiyle karşılaştırıyordu. Üçü de aynı sınıf: yalnızca
üzerinde koşulmayan bir makinede görünen testler.

---

## A. Yayın koşusu

1. ❌ **Windows'ta elle tur** (yapılmadı) — `preflight` → proje oluştur → `up` → tarayıcıda
   aç. Kategorinin 13/17'si Windows'ta ve bir CI koşusu bu soruyu cevaplamıyor. **Bir
   Windows makinesi gerekiyor.**

   **İki ölçüm bu tura ekli**, çünkü başka yerden doğrulanamıyorlar — imzasız bir yapının
   **kendini güncelleyebildiği** (güncelleyici minisign anahtarı ayrı ve duruyor, ama macOS
   güncellemeden sonra yeniden karantinaya alabilir), ve SmartScreen'in kurulumu tamamen mi
   engellediği yoksa bir tıkla mı geçildiği. README ve sürüm notu ikincisini "bir tık" diye
   yazıyor; turun doğrulayacağı şey o.

## B. Bakım borcu

1. ❌ **Sekiz major bağımlılık geçişi** (yapılmadı) — vuetify 3→4, pinia 2→4, vite 7→8,
   vitest 3→4, eslint 9→10, jsdom 26→30, vue-i18n 9→11, vue-router 4→5. (On paket, ama
   birlikte hareket eden çiftler bir kez sayıldı: eslint `@eslint/js` ile, vitest kendi
   kapsam sağlayıcısıyla.)

   **Ve bu liste tek kayıt, bilerek.** `dependabot.yml`'in majorları susturmasını
   kaldırmak bir gece denendi ve sonucu **on dört pull request, on dört dal** oldu; her
   biri kırmızı, hiçbiri birleştirilebilir değil. Bir geçiş botun diff'i olarak
   incelenemez — vuetify 3→4 uygulama kodunu değiştirir, sürüm satırını değil — yani o PR
   ne birleşir ne de hatırlatıcıyı kaybetmeden kapatılır; sadece durur ve diğer her PR'ı
   görmeyi zorlaştırır. `ignore` npm ve cargo tarafına gerekçesiyle geri kondu;
   `github-actions`'ta **bilerek yok**, çünkü oradaki major bir workflow satırıdır ve
   PR'ın kendi koşusu ispatlar.

   Bu maddeyi kapatacak olan şey, sekizini tek tek elle yapmak.

2. ❌ **`validate-contracts.mjs` tip alanlarını hâlâ denetlemiyor** (yapılmadı) — sözleşmenin
   `CertStatus`'u düzeltildi (`rejected` ve `error` eklendi, `notAfter` `string?`'ten
   `number?`'a çekildi — UI zaten `new Date(saniye * 1000)` yapıyordu), ve
   `src/lib/ipc.d.ts` yeniden üretildi. Ama **sınıf açık kaldı**: doğrulayıcının E süiti
   komut listelerini üç tarafa karşı tutuyor, tip **alanlarını** hiç okumuyor. Yani bir
   Rust struct'ına eklenen alan sözleşmede eksik kalırsa bugün de kimse fark etmez —
   bu sefer ekran görüntüsü turunda çıktı, bir dahakine çıkmayabilir.

   İşin şekli: `#[derive(Serialize)]` taşıyan struct'ları `contracts/ipc.json`'daki
   karşılıklarına eşlemek ve alan adı + isteğe bağlılık üzerinden karşılaştırmak.
   `serde(rename_all)` ve `Option<T>` okunabilir; asıl zorluk struct ile sözleşme tipini
   eşleştirmek, çünkü bağ bugün yalnızca isim benzerliği.

---

## C. Arayüz ve hardcode borcu

1. ❌ **İki ekran bu araçtan çıkamaz** (yapılmadı) — dal başına worktree ortamı (çalışan
   bir git ağacı istiyor) ve `stackvo tui` (bir pencere değil, terminal programı). Otuz
   yedi görüntünün geri kalanı `docs/screenshots/` altında ve iki README'de.

   Bu bir eksik değil, aracın sınırı: `tools/screenshots.mjs` tarayıcıda koşuyor.
   Kapanacaksa iki ayrı yolla kapanır — worktree için gerçek bir git ağacı hazırlayan bir
   fikstür, TUI için bir terminal kaydedici. İkisi de bu koşunun dışında.

2. ❌ **İkinci güncelleme kanalı** (yapılmadı) — bir bağımlılık, eksik değil; gerekçesi
   `channel.rs`'e yazılı. `tauri.conf.json` tek uç tanımlıyor; güncelleyici eklentisi uç
   listesini biri cevap verene kadar geziyor, yani ikinci bir girdi kanal *seçmez*,
   **yedek** olur. Uçta kanal yer tutucusu yok, `check()`'te uç geçersiz kılma yok.

   Bugün bir *ayar* eklemek, `channel.rs`'in kendi notunun uyardığı hatayı kurmak olurdu:
   *"kimsenin yayımlamadığı bir kanal, güncellemeleri sessizce durduran bir ayardır."*
   Engeli kaldıran adım burada daha fazla kod değil, **ilk yayın** — ve o 2026-09-03'te
   çıktı (v0.2.0). `beta.json` artık aynı koşunun bir çıktısı daha olabilir; kalan iş o.

3. ❌ **Çökme raporunu göndermek** (yapılmadı) — çökmeyi bildirmek yerinde (`crash.rs`:
   `install`, `reports`, `unseen`, `mark_seen`); kalan onu bir yere göndermek, ve o bir
   kod değil bir **ürün kararı**.

   **Göndermek yapılmadı, çünkü gönderilecek yer yok ve olmaması bir söz.** `PRIVACY.md`
   açık: *"telemetri yok ve eklemek gibi bir plan yok… çökme raporlama servisi yok ve
   uygulamanın arkasında sunucu yok"*, ve gelecekteki her şeyin **opt-in, varsayılan kapalı
   ve gönderilmeden önce orada yazılı** olacağı. Bir uç eklemek kod değişikliği değil, ürün
   kararı — ve o kararı alana kadar bu maddenin "yapılmadı"sı bir eksik değil, bir söze
   uymak.

---

## D. Stratejik

Kalan tek madde, **kullanıcıya açıkça sorulması gereken** bir şey yapıyor: bu makinenin
durumunu internete yayımlıyor. Gerekçesi maddesinde.

1. ❌ **Paylaşılabilir teşhis bağlantısı (U-3)** (yapılmadı) — ve bilerek bekliyor.

   Parçaların hepsi ağaçta — paket (`diagnostics::write`), dokuz sağlayıcılı tünel, parola
   muhafızı, sayfa üretimi — ama birleştiren fiil, bu makinenin durumunu **internete
   yayımlamak**. Maskelenmiş olsa bile bu dışa dönük bir eylem: bir tünel açar, bir adres
   üretir ve onu birine verir. Bu uygulamanın duruşu böyle bir şeyi *sormadan* yapmamak, ve
   bir meslektaşın paketini **dosya olarak** karşılaştırmak (`diagnostics::compare`) zaten
   yapıldı — yani bugün cevaplanabilen soru, zip göndermeden de cevaplanıyor. Bağlantı,
   tünel yığınının üzerine kurulacak ayrı bir onay akışı; sırası gelmedi, ve sebebi burada
   yazılı.
