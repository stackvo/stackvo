# StackVo Desktop — Açık İşler

**Tarih:** 2026-08-31
**Kapsam:** Yalnızca **geliştirmesi kalan** maddeler.

Bu belge her turda budanıyor: yapılan ve yapılmayacağına karar verilen her madde, alt
maddeleriyle birlikte siliniyor. Yapılanların gerekçesi ve ölçümü `CHANGELOG.md`'de,
mimarisi `ARCHITECTURE.md`'de duruyor — burada tekrarlanmıyor. Bir maddenin buradan
düşmesi, işinin bittiği anlamına gelir.

**Maddelerin işaretlenişi.** Burada yalnız **yapılmamış** iş duruyor: her madde ❌ ile
başlıyor ve başlığının sonunda `(yapılmadı)` taşıyor. Yarısı yapılmış maddelerin yapılmış
yarısı da silindi, yani bir başlık artık **yalnızca kalanı** adlandırıyor — yapılanın
gerekçesi ve ölçümü `CHANGELOG.md`'de.

---

## Durum

| Blok | Madde | Ne bekliyor |
| --- | --- | --- |
| **A. Yayın koşusu** | 5 | Bir sürüm numarası kararı ve bir Windows makinesi |
| **B. Bakım borcu** | 4 | Zaman, bir depo ayarı turu, bir sözleşme düzeltmesi, bir Dependabot kararı |
| **C. Arayüz borcu** | 4 | README'ye bir görüntü ve rozetler, bir çeviri kararı, ilk yayın, bir ürün kararı |
| **D. Stratejik** | 1 | **Kullanıcıya sorulacak bir karar** |

**Dağıtım kararı: yalnız GitHub Releases, kod imzalama yok.** Uygulama hiçbir mağazada
yayınlanmayacak — App Store, Microsoft Store, Snap, Flathub — ve Homebrew cask / winget
gibi ikinci bir kanal da yok. Bunun sonucu olarak **Apple Developer Program ve
Authenticode maddeleri kaldırıldı**, ve zincirin son dışsal bağımlılığı da onlarla
birlikte düştü.

> **Kararın maliyeti, ve bu satır bilerek burada duruyor.** Apple Developer Program ve
> Authenticode **mağazayla ilgili değildir** — GitHub Releases'ten indirilen bir dosya
> için de gerekirler. İmzasız ve notarize edilmemiş bir `.dmg`'yi macOS Gatekeeper
> *"bozuk, çöpe taşı"* diyerek açtırmaz; Windows'ta SmartScreen imzasız kuruluma engel
> çıkarır. Yani "mağazaya girmiyoruz" ile "imzaya ihtiyacımız yok" aynı şey değildir, ve
> ikincisi ayrı bir karardır — verildi, ve bedeli aşağıda bir madde.

Bedel yazılı ve duruyor — README imzasız bir yapının iki platformdaki adımını veriyor.
**Karar geri alınabilir:** imzalamaya dönülürse iş, o sırları depoya koymaktan ibaret;
kodda ya da koşuda değişecek bir şey yok.

**Kritik yol.** Zincirin kod tarafı bitti ve artık **dışsal bir bağımlılığı da yok**.
Sırası şu: **sürüm numarası kararı** → **Publish** → `updates:check` → **Windows'ta elle
tur**. StackVo'nun **kendi** iki minisign anahtarı — güncelleyici
ve paket indeksi — bu kararın dışında ve duruyor; **içerik anahtarının döndürülebildiği
pencere ilk varlıklı sürümde kapanıyor**, yani rotasyon gerekiyorsa yayından önce
yapılmalı, sonra değil.

---

## A. Yayın koşusu

1. ❌ **Sürüm numarası** (yapılmadı) — sürümü yükselt (ör. `0.2.0`), `CHANGELOG.md`'de
   sürüm başlığını aç, 4300 satırlık `Unreleased`'i oraya taşı, etiketle. **Numarayı
   seçmek kullanıcının kararı**; seçildiği an üç dosyadaki yükseltme ve CHANGELOG
   bölümlemesi buradan yapılabilir.
2. ❌ **Sürüm notu** (yapılmadı) — kullanıcıya dönük **kısa** bir tane. Mevcut CHANGELOG bir
   mühendislik günlüğü ve sürüm notu olarak kullanılamaz. Bu buradan yazılabilir, #1'e bağlı.
3. ❌ **Publish** (yapılmadı) — yayın koşusunu rehearsal'da uçtan uca doğrula, sonra bas;
   `releaseDraft: true` olduğu için basılmadıkça `latest.json` 404 verir. **Publish bir
   insanın tuşu.**
4. ❌ **Güncelleyici ucu** (yapılmadı) — `npm run updates:check` ile doğrula. #3'ten sonra
   anlamlı.
5. ❌ **Windows'ta elle tur** (yapılmadı) — `preflight` → proje oluştur → `up` → tarayıcıda aç.
   Kategorinin 13/17'si Windows'ta ve bir CI koşusu bu soruyu cevaplamıyor. **Bir Windows
   makinesi gerekiyor.**

   **İki ölçüm bu tura ekli**, çünkü başka yerden doğrulanamıyorlar — imzasız bir yapının
   **kendini güncelleyebildiği** (güncelleyici minisign anahtarı ayrı ve duruyor, ama macOS
   güncellemeden sonra yeniden karantinaya alabilir), ve SmartScreen'in kurulumu tamamen mi
   engellediği yoksa bir tıkla mı geçildiği. README ikincisini "bir tık" diye yazıyor;
   turun doğrulayacağı şey o.

---

## B. Bakım borcu

1. ❌ **Yedi major bağımlılık geçişi** (yapılmadı) — Dependabot açıldıktan **sonra**, yayın
   öncesi değil.
2. ❌ **Deponun GitHub tarafındaki sertleştirme eksikleri** (yapılmadı) — Katkı yüzeyi tam:
   `CODEOWNERS`, üç ekosistemli `dependabot.yml`, iki konu şablonu ve kapalı boş konu, PR
   şablonu, `SECURITY.md`'de 72 saatlik bir söz. Eksik olan **koşunun ve deponun kendi
   sertleştirmesi**, ve altı parçası var:

   - ❌ **Kod taraması yok.** `cargo deny` ve `npm audit` bağımlılık tarafını okuyor; kod
     deseni tarafını okuyan hiçbir şey yok, yani deponun Security sekmesinde code scanning
     boş. CodeQL JS/TS'i doğrudan alıyor; Rust tarafı için clippy'nin SARIF çıktısı aynı
     sekmeye yükleniyor — ikisi de var olan koşunun **üstüne** bir iş, içine değil.
   - ❌ **PR'da `dependency-review` yok.** Dependabot bir zafiyeti *sonradan* bir PR olarak
     bildiriyor; `dependency-review-action` onu birleşmeden önce durduruyor. Ayrım
     "haftaya haberimiz olur" ile "`main`'e giremez" arasında, ve ikincisi bir dosya.
   - ❌ **İş akışlarında `permissions:` yazılı değil.** `ci.yml` ve `nightly.yml` hiç yazmıyor;
     `release.yml` yalnız `build` işinde yazıyor, `preflight` varsayılanı devralıyor. Yani en
     az yetki dosyada değil, bir depo ayarının onay kutusunda duruyor — ve ayar değişirse
     kimse fark etmez. Üstüne bir `permissions: contents: read` satırı bunu kağıda geçirir.
   - ❌ **Eylemler etikete sabitli, SHA'ya değil.** Ve bu tam olarak `dependabot.yml`'in
     kendi notunun *"hiçbir lockfile'da olmayan tedarik zinciri deliği"* dediği şey:
     `actions/checkout@v4` hareketli bir etiket, `tauri-apps/tauri-action@v0` ondan da geniş,
     `dtolnay/rust-toolchain@stable` ise bir **dal ucu** — Dependabot sonuncusunu
     yükseltemez bile, çünkü yükseltilecek bir sürüm yok. SHA'ya sabitleme, o dosyaya
     yazılmış gerekçenin gereğini yapmak.
   - ❌ **`concurrency` grubu yok.** Üç işletim sistemli matris, ayrı bir sürücü işi, kapsam
     ve tedarik zinciri işleri: art arda iki push, iki tam koşu. Bir `concurrency` grubu ve
     `cancel-in-progress` bunun bedeli iki satır. Aynı yerde `checkout`'un
     `persist-credentials: false`'ı da yok.
   - ❌ **Dal koruması hiçbir yerde yazılı değil.** `CODEOWNERS` sahibi söylüyor ama zorunlu
     incelemeyi kimse söylemiyor: hangi denetimler zorunlu (`ci.yml`'in iş adları), lineer
     geçmiş açık mı, imzalı commit isteniyor mu. Bunlar depo ayarında yaşıyor ve bir depo
     ayarı incelenemiyor — ruleset olarak dışa aktarılıp `.github/rulesets/` altına
     konursa, korumanın değişmesi de bir PR olur.

   **Bilerek dışarıda bırakılanlar:** `FUNDING.yml` (bağış kanalı yok), `CITATION.cff` (bu bir
   kütüphane değil), `GOVERNANCE.md` (tek bakımcı — `CODEOWNERS` zaten cevap), stale bot
   (kapatılacak kadar konu yok). OpenSSF Scorecard ayrı bir iş değil: yukarıdaki altı madde
   yapılınca skoru zaten yükselten şeyler onlar, rozet sonrasının kararı. **CI rozeti bu
   maddede değil**, C-1'de duruyor.
3. ❌ **`contracts/ipc.json` iki yerde struct'ın gerisinde** (yapılmadı) — ekran
   görüntüsü turunda çıktı: `CertStatus` tipinde `rejected` ve `error` alanları yok, oysa
   `certs.rs` ikisini de döndürüyor **ve** `CertificatesPane` `certs.rejected.length`'i
   muhafazasız okuyor; aynı tipte `notAfter` sözleşmede `string?`, Rust'ta `Option<i64>`.
   Uygulama doğru çalışıyor, yanlış olan sözleşme — ve sözleşmeye bakarak yazılan her
   sahne (test sahnesi dahil) bu yüzden eksik yazılıyor. `tools/validate-contracts.mjs`
   bu sınıfı yakalamıyor: komut listesini denetliyor, tip alanlarını değil.
4. ❌ **Dependabot'un yeri ve izi — bir karar** (yapılmadı) — soru iki parça, ve cevapları
   ayrı: *"CI yml'de halledemez miyiz"* ile *"Contributors'ta görünmesin"* aynı şeyi
   sormuyor.

   **Bir kısmı gerçekten yml'de halledilir.** Haftalık zamanlanmış bir iş `npm outdated` ve
   `cargo update --dry-run` koşturup çıktısını tek bir konu olarak açabilir; *"ne kadar
   geride kaldık"* sorusu böyle de cevaplanır, ve B-1'in beklediği şey aslında budur.
   Dependabot'un bunun üstüne koyduğu üç şey var ve üçü de elle yazılan bir yml'de yok:
   - **Zafiyet güncellemeleri çizelge beklemez** — Advisory Database'e bağlı olduğu için
     yayımlandığı gün PR açar; zamanlanmış bir iş bunu en iyi ihtimalle haftaya görür.
     (Uyarıların kendisi ayrı bir depo ayarı, `dependabot.yml`'e bağlı değil, ve **hiç
     commit üretmez** — yani Contributors sorusunun dışında.)
   - **Sürüm notu ve uyumluluk skoru PR gövdesinde** — yükseltmeyi okunabilir yapan şey o;
     `npm outdated` çıktısı iki sürüm numarası verir, gerekçe vermez.
   - **`github-actions` ekosistemi** — B-2'nin dördüncü maddesi eylemleri SHA'ya
     sabitlemeyi istiyor, ve SHA'ya sabitlenmiş bir eylemi elle güncellemek her seferinde
     etiketten SHA'ya bakmak demek. O maddeyi sürdürülebilir kılan şey Dependabot.

   **Contributors'ta görünmemesi ayrı ve çözülebilir.** O liste commit'in **yazarına**
   bakıyor: `dependabot[bot]` listeye ancak ağaçta yazarı o olan bir commit varsa girer.
   Yani ayar Dependabot'u kapatmak değil, **dalını birleştirmemek** — PR'ı bir okuma
   listesi gibi kullanmak, yükseltmeyi kendi commit'inle yapmak, PR'ı kapatmak. GitHub'ın
   botları bu listeden kendiliğinden eleyip elemediği zaman içinde değişti, yani ona
   güvenilmez; garanti eden tek şey bota commit yazdırmamak. Bedeli dürüstçe: otomatik
   birleştirmeden vazgeçmek, ve her bump'ın bir insanın on saniyesi olması.

   **Ve bugün ikisi arasında duran bir tutarsızlık var:** `dependabot.yml` hem *"majorlar
   gruplardan çıkarıldı, kendi başlarına açılır"* diye yazıyor, hem de her iki ekosistemde
   `ignore: '*' + semver-major` ile majorları tamamen susturuyor. Yani **B-1'in beklediği
   yedi major geçişi Dependabot'tan hiç gelmeyecek** — ya o `ignore` kalkacak, ya B-1
   baştan elle yapılacak bir iş olarak yazılacak. Karar hangisi olursa olsun, ikisi şu an
   birbirini yalanlıyor.

---

## C. Arayüz ve hardcode borcu

1. ❌ **`readme_claims` beş yerde kırık, ve iki ekran hâlâ çekilemiyor** (yapılmadı) —
   görüntü tarafı bitti: otuz yedi dosya `docs/screenshots/` altında, iki README'de dört
   sütunlu ızgarayla ve hepsi [`docs/screenshots/README.md`](screenshots/README.md)'de;
   rozetler de kondu. Açık kalan iki şey:
   - ❌ **README yeniden yazılınca `readme_claims.rs`'in beş iddiası düştü** — MCP araç
     sayısı, üretici varsayılanı, yayın komutu sayısı, proje kapsamı, uygulama tutamağı
     sayısı. Cümleler `docs/README-legacy.md`'ye taşındı, gate ise hâlâ `README.md`'ye
     bakıyor. Ya cümleler yeni README'ye dönecek, ya gate yeni dosyayı ölçecek — **bir
     karar**, ve o karar verilene kadar `cargo test` kırmızı.
   - ❌ **İki ekran bu araçtan çıkamaz** — dal başına worktree ortamı (çalışan bir git
     ağacı istiyor) ve `stackvo tui` (bir pencere değil, terminal programı).

2. ❌ **Türkçe README yok** (yapılmadı) — kapıdaki tek dosya İngilizce, oysa uygulamanın
   kendi yardımı `docs/help/tr` ve `docs/help/en` ile zaten iki dilli. Türkçesi olmayan
   şey, depoya ilk bakan kişinin okuduğu dosya.

   **Bedeli çeviri değil, çevirinin denetlenmemesi.** `readme_claims.rs` yalnız
   İngilizcesini ağaçtaki gerçeğe karşı tutuyor; denetlenmeyen bir çeviri altı ay sonra
   farklı bir ürünü anlatır, ve kimse fark etmez çünkü onu kıracak bir test yoktur.
   Yazılacaksa iş iki parça: `README.tr.md`, **ve** gate'in iki dosyayı da sayması.
3. ❌ **İkinci güncelleme kanalı** (yapılmadı) — bir bağımlılık, eksik değil; gerekçesi
   `channel.rs`'e yazılı. `tauri.conf.json` tek uç tanımlıyor; güncelleyici eklentisi uç
   listesini biri cevap verene kadar geziyor, yani ikinci bir girdi kanal *seçmez*,
   **yedek** olur: manifesti bir an cevap vermeyen bir kararlı kurulum beta'yı alırdı.
   Uçta kanal yer tutucusu yok, `check()`'te uç geçersiz kılma yok.

   Bugün bir *ayar* eklemek, `channel.rs`'in kendi notunun uyardığı hatayı kurmak olurdu:
   *"kimsenin yayımlamadığı bir kanal, güncellemeleri sessizce durduran bir ayardır."*
   Biri "beta"yı işaretler, uç `latest.json` vermeye devam eder, `offer` doğru şekilde
   `otherChannel` der, ve o kişi **hiçbir şey almaz** — hatasız, çünkü hiçbiri yanlış değil.

   Engeli kaldıran adım burada daha fazla kod değil, **ilk yayın**: bir sürüm numarası
   seçilip bir publish olduğunda `beta.json` aynı koşunun bir çıktısı daha oluyor ve uç
   sorusu karşısında gerçek bir cevabı olan gerçek bir soru hâline geliyor.
4. ❌ **Çökme raporunu göndermek** (yapılmadı) — çökmeyi bildirmek yerinde; kalan onu bir
   yere göndermek, ve o bir kod değil bir **ürün kararı**.

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

   Parçaların hepsi ağaçta — paket,
   dokuz sağlayıcılı tünel, parola muhafızı, sayfa üretimi — ama birleştiren fiil, bu
   makinenin durumunu **internete yayımlamak**. Maskelenmiş olsa bile bu dışa dönük bir
   eylem: bir tünel açar, bir adres üretir ve onu birine verir. Bu uygulamanın duruşu böyle
   bir şeyi *sormadan* yapmamak, ve bir meslektaşın paketini **dosya olarak**
   karşılaştırmak zaten yapıldı — yani bugün cevaplanabilen soru, zip göndermeden de
   cevaplanıyor. Bağlantı, tünel yığınının üzerine kurulacak ayrı bir onay akışı; sırası
   gelmedi, ve sebebi burada yazılı.
