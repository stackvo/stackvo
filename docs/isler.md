# StackVo Desktop — Açık İşler

**Tarih:** 2026-09-01
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
| **A. Yayın koşusu** | 4 | Yeşil bir yapı matrisi, bir insanın tuşu, bir Windows makinesi |
| **B. Bakım borcu** | 3 | Bir depo ayarı turu, Dependabot'un ilk haftası, bir doğrulayıcı |
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

**Kritik yol, ve 2026-09-01'de değişti.** Sürüm numarası seçildi (`0.2.0`, üç dosyada ve
`CHANGELOG.md`'de) ve sürüm notu yazıldı, yani zincirin başındaki iki halka düştü. Yerine
**daha önce hiç yazılmamış bir engel** geçti: yayın koşusu hiç yeşil olmadı. Sırası şu:
**yapı matrisini onar** → **etiketle** → **Publish** → `updates:check` → **Windows'ta elle
tur**. StackVo'nun kendi iki minisign anahtarı duruyor; **içerik anahtarının
döndürülebildiği pencere ilk varlıklı sürümde kapanıyor**, yani rotasyon gerekiyorsa
yayından önce yapılmalı.

---

## Süreç: nerede kaldık

| | Ne | Durum |
| --- | --- | --- |
| PR #58 | Kırmızı olan her şeyi yeşile döndürdü | ✅ birleşti |
| Bu değişiklik | Kalan CI kırmızısı, yerel kapının delikleri, sertleştirme, `0.2.0` | ⏳ tek PR |

**Kural setinin sırası kritik.** `.github/rulesets/main.json` üç yeni denetimi
(`CodeQL (javascript)`, `clippy (sarif)`, `review`) zorunlu kılıyor, ve bir denetim ancak
workflow'u `main`'de varsa GitHub'ın gözünde var olur. **Bu PR birleşmeden içe aktarılırsa
sonraki her PR var olmayan denetimleri sonsuza kadar bekler.** Sıra: birleştir → içe aktar.

Etiketleme de birleşmeden sonra: `v0.2.0`, ve ondan önce yayın matrisinin düşen üç satırı
onarılmalı — biri (`ubuntu-24.04-arm`) artık `tools/linux/run.sh --bundle` ile yerelde
cevaplanabiliyor ve o koşu yeşil.

### Ve bugünün dersi, buraya yazılıyor çünkü tekrarlanabilir

PR #58 `cargo test` ve `npm test` yeşil diye "ağaç yeşil" denip gönderildi; ardından gelen
koşu **dört ayağın dördünde de** kırmızıydı. İki cümle de doğruydu: `ci.yml` işletim
sistemi başına on bir şey soruyor, ve o ikisi onun ikisi.

`tools/before-push.sh` on birini de soruyor — açılış satırı *"everything CI will ask,
asked here first"* — ve koşturulmadı. Koşturulunca aynı hataları doksan saniyede verdi;
öğrenilme biçimi ise bir merge, dört iş ve dört ekran görüntüsü oldu.

**Kural: `tools/before-push.sh` koşmadan hiçbir dal push edilmez.** Platformlar arası
olanlar için `--all`, ve o da artık `--windows-test` içeriyor — Windows takımı wine altında
yerelde koşabiliyordu ve bu dosya onu hiç sormamıştı. `--windows` **derleme denetimi**,
koşu değil; aradaki fark tam olarak Windows'ta düşen üç testti.

**Ve kapının kendisi de ölçülmeli.** Konteyner imajı iki gün eskiydi ve `run.sh` onu yalnız
*yoksa* kuruyordu, yani `--bundle` ve `--windows-test` bozuk bir yapı gibi okunan bayat bir
konteynerde düşüyordu. Artık Dockerfile'ın sha256'sı imaja etiket olarak yazılıyor ve her
koşuda karşılaştırılıyor.

---

## A. Yayın koşusu

1. ❌ **Yapı matrisi altı hedefin üçünde düşüyor** (yapılmadı) — **ve bu maddenin yeni
   olması bir bulgu.** Belge daha önce *"zincirin kod tarafı bitti ve artık dışsal bir
   bağımlılığı da yok"* diyordu; oysa `release.yml`'in son koşusu (`32986607714`,
   2026-08-26) `windows-latest / x86_64`, `ubuntu-24.04-arm / aarch64` ve
   `windows-11-arm / aarch64` satırlarında düştü. macOS'un iki satırı ve Linux x86_64
   geçti. Yani "Publish bir insanın tuşu" doğru değil: basılacak bir şey üretilmiyor.

   Bu, A-2'nin (eski numarayla A-5) Windows turundan **ayrı** bir iş. Tur, imzasız bir
   yapının o makinede ne yaptığını soruyor; bu madde yapının **var olmasını** sağlıyor.

2. ❌ **Publish** (yapılmadı) — #1'den sonra: etiketle, koşuyu rehearsal'da uçtan uca
   doğrula, sonra bas. **Publish bir insanın tuşu.**

   Bir ayrıntı düzeltildi ve buraya yazılıyor, çünkü eskisi yanlış teşhise götürüyordu:
   `v0.1.0` sürümü **taslak değil, yayımlanmış** durumda (2026-08-23) — ama **sıfır
   varlığı var**. Yani `latest.json`'ın bugün 404 vermesinin sebebi `releaseDraft: true`
   değil, hiçbir koşunun varlık üretmemiş olması. `tools/check-updater-endpoint.mjs` bu
   iki sebebi zaten ayrı ayrı yazıyor ve sırayla bakmayı söylüyor; doğru olan o.

3. ❌ **Güncelleyici ucu** (yapılmadı) — `npm run updates:check` ile doğrula. #2'den sonra
   anlamlı. Bugün 404 veriyor ve verdiği açıklama doğru.

4. ❌ **Windows'ta elle tur** (yapılmadı) — `preflight` → proje oluştur → `up` → tarayıcıda
   aç. Kategorinin 13/17'si Windows'ta ve bir CI koşusu bu soruyu cevaplamıyor. **Bir
   Windows makinesi gerekiyor.**

   **İki ölçüm bu tura ekli**, çünkü başka yerden doğrulanamıyorlar — imzasız bir yapının
   **kendini güncelleyebildiği** (güncelleyici minisign anahtarı ayrı ve duruyor, ama macOS
   güncellemeden sonra yeniden karantinaya alabilir), ve SmartScreen'in kurulumu tamamen mi
   engellediği yoksa bir tıkla mı geçildiği. README ve sürüm notu ikincisini "bir tık" diye
   yazıyor; turun doğrulayacağı şey o.

---

## B. Bakım borcu

1. ❌ **Sekiz major bağımlılık geçişi** (yapılmadı) — vuetify 3→4, pinia 2→4, vite 7→8,
   vitest 3→4, eslint 9→10, jsdom 26→30, vue-i18n 9→11, vue-router 4→5. (On paket, ama
   birlikte hareket eden çiftler bir kez sayıldı.) Belge bunu "yedi" diye taşıyordu;
   bugün ölçülen sayı bu, ve `dependabot.yml`'in başlığı da düzeltildi.

   **Engeli kalktı.** `dependabot.yml` majorları hem "kendi başlarına açılır" diye
   anlatıp hem `ignore` ile susturuyordu; artık gruplar `update-types: [minor, patch]`
   alıyor ve `ignore` yok, yani majorlar tek tek PR olarak gelecek. npm tarafının
   `open-pull-requests-limit`'i 5'ten 10'a çıkarıldı ki sekiz major, gruplanmış yama
   PR'larının arkasında kalmasın. Bu maddeyi kapatacak olan şey artık zaman.

2. ❌ **Dal koruması içe aktarılmadı** (yapılmadı) — **ve bu bir depo ayarı turu, bir
   geliştirme değil.** `.github/rulesets/main.json` yazıldı ve içinde ne olduğunun
   gerekçesi de duruyor, ama **GitHub `.github/rulesets/` dizinini okumuyor** — dosya
   burada durduğu için hiçbir şey korunmuyor. Bir kez içe aktarılmalı:

   ```text
   Settings → Rules → Rulesets → New ruleset → Import a ruleset
   ```

   ya da `gh api repos/stackvo/stackvo/rulesets --method POST --input .github/rulesets/main.json`

   Ondan sonrası bir PR meselesi. Dosyada bilerek **kapalı** bırakılan üç şey var ve
   üçünün de gerekçesi ağaçtan ölçüldü: zorunlu onay 1 değil 0 (tek katkıcı var, 1 her
   PR'ı kilitlerdi), lineer geçmiş yok (72 commit'in 35'i merge), imzalı commit yok
   (yerel commit'ler bugün imzasız — sıra: anahtarı kur, `Verified` gördüğünü doğrula,
   sonra kuralı ekle).

3. ❌ **`validate-contracts.mjs` tip alanlarını hâlâ denetlemiyor** (yapılmadı) — sözleşmenin
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
   Engeli kaldıran adım burada daha fazla kod değil, **ilk yayın** — A-2 olduğunda
   `beta.json` aynı koşunun bir çıktısı daha oluyor.

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
