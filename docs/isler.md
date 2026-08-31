# StackVo Desktop — Açık İşler

**Tarih:** 2026-08-30
**Kapsam:** Yalnızca **geliştirmesi kalan** maddeler.

Bu belge her turda budanıyor: yapılan ve yapılmayacağına karar verilen her madde, alt
maddeleriyle birlikte siliniyor. Yapılanların gerekçesi ve ölçümü `CHANGELOG.md`'de,
mimarisi `ARCHITECTURE.md`'de duruyor — burada tekrarlanmıyor. Bir maddenin buradan
düşmesi, işinin bittiği anlamına gelir.

**Maddelerin işaretlenişi.** Her madde numarasından hemen sonra bir **işaretle** başlar ve
başlığının sonunda **parantez içinde hükmünü** taşır — göz, satırın başında durumu görüyor,
ve okuyan cümlenin sonunda onu okunabilir hâliyle buluyor.

| İşaret | Hüküm | Anlamı |
| --- | --- | --- |
| ⚠️ | `(yarım)` | Bir yarısı yapıldı; kalanın gerekçesi maddede yazılı |
| ❌ | `(yapılmadı)` | Hiç başlanmadı |

---

## Durum

| Blok | Madde | Ne bekliyor |
| --- | --- | --- |
| **A. Yayın koşusu** | 5 | Bir sürüm numarası kararı ve bir Windows makinesi |
| **B. Bakım borcu** | 1 | Zaman |
| **C. Arayüz borcu** | 3 | Üçü de yarım; kalanların gerekçesi maddelerinde |
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

Bedel silinmedi ve artık yazılı: README'nin *"Opening a build that is not code-signed"*
bölümü iki platformun kendi adımını veriyor, ve `readme_claims.rs` onu `release.yml`'in
kendi uyarılarına karşı tutuyor — çift, yanlış tarafa şöyle gidiyor: biri imzalamayı
ekler, iş akışı uyarmayı bırakır, README ise insanlara hiç görmeyecekleri bir kutuyu
geçmelerini anlatmaya devam eder. **Karar geri alınabilir:** imzalamaya dönülürse iş, o
sırları depoya koymaktan ibaret; kodda ya da koşuda değişecek bir şey yok.

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

---

## C. Arayüz ve hardcode borcu

1. ⚠️ **README'yi son kullanıcıya çevir** (yarım) — dört bölümün dördü yazıldı
   (*Installing it*, *What Docker costs you*, *Coming from something else*, *What it does
   that gets missed*); iki madde açık kaldı:
   - ❌ **Ekran görüntüsü ve rozet yok.** Bir ekran görüntüsü çalışan bir yapı ve bir
     insan gerektiriyor; buradan üretilemez. **Açık, ve sahibi kullanıcı.**
   - ❌ **Türkçe README yok.** Yazılabilir ama ikinci bir README ikinci bir bayatlama
     yüzeyi: `readme_claims.rs` yalnız İngilizcesini denetliyor, ve denetlenmeyen bir
     çeviri altı ay sonra farklı bir ürünü anlatır. Yazılacaksa gate'in iki dosyayı da
     sayması gerekiyor. **Açık.**

   **Yeni gate:** `the_readme_counts_the_surfaces_it_advertises` — yedi `release_*`, yedi
   `worktree_*` ve `imports::ALL`'un yedi kaynağını ağaçtan sayıp README'yle
   karşılaştırıyor. `imports.rs`'in başlığı "**Two** of them" derken `ALL` yediyi
   taşıyordu; aynı sınıfın README tarafı artık kapalı.
2. ⚠️ **Kanal ve sürüm başına yükseltme notu** (yarım) — Sürüm notu ölçüldüğünde zaten
   gösteriliyordu (`Settings.vue`'nun güncelleme kartı); eksik olan yalnızca metnin
   işaretsizliğiydi ve `lang=""` aldı. Kalan: ikinci kanal.

   **İkinci kanal bir bağımlılık, eksik değil** — ve `channel.rs`'e yazıldı. `tauri.conf.json`
   tek uç tanımlıyor; güncelleyici eklentisi uç listesini biri cevap verene kadar geziyor,
   yani ikinci bir girdi kanal *seçmez*, **yedek** olur: manifesti bir an cevap vermeyen bir
   kararlı kurulum beta'yı alırdı. Uçta kanal yer tutucusu yok, `check()`'te uç geçersiz
   kılma yok.

   Bugün bir *ayar* eklemek, `channel.rs`'in kendi notunun uyardığı hatayı kurmak olurdu:
   *"kimsenin yayımlamadığı bir kanal, güncellemeleri sessizce durduran bir ayardır."*
   Biri "beta"yı işaretler, uç `latest.json` vermeye devam eder, `offer` doğru şekilde
   `otherChannel` der, ve o kişi **hiçbir şey almaz** — hatasız, çünkü hiçbiri yanlış değil.

   Engeli kaldıran adım burada daha fazla kod değil, **ilk yayın**: bir sürüm numarası
   seçilip bir publish olduğunda `beta.json` aynı koşunun bir çıktısı daha oluyor ve uç
   sorusu karşısında gerçek bir cevabı olan gerçek bir soru hâline geliyor.
3. ⚠️ **Çökme raporu** (yarım) — Bildirme yapıldı (`crash::reports` / `unseen` /
   `mark_seen`, iki IPC komutu ve kabukta kapatılabilir bir satır; rapor zaten teşhis
   paketine giriyordu). Kalan: göndermek — ve o bir kod değil bir **ürün kararı**.

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

1. ⚠️ **Paylaşılabilir teşhis bağlantısı ve onboarding doğrulaması** (yarım) — U-4
   yapıldı (`stackvo verify <proje>` ve proje sayfasındaki düğme; gerekçesi ve ölçümü
   `CHANGELOG.md`'de). Kalan U-3, ve bilerek bekliyor.

   **U-3 (paylaşılabilir teşhis bağlantısı) bilerek beklidi.** Üç parça da ağaçta — paket,
   dokuz sağlayıcılı tünel, parola muhafızı, sayfa üretimi — ama birleştiren fiil, bu
   makinenin durumunu **internete yayımlamak**. Maskelenmiş olsa bile bu dışa dönük bir
   eylem: bir tünel açar, bir adres üretir ve onu birine verir. Bu uygulamanın duruşu böyle
   bir şeyi *sormadan* yapmamak, ve bir meslektaşın paketini **dosya olarak**
   karşılaştırmak zaten yapıldı — yani bugün cevaplanabilen soru, zip göndermeden de
   cevaplanıyor. Bağlantı, tünel yığınının üzerine kurulacak ayrı bir onay akışı; sırası
   gelmedi, ve sebebi burada yazılı.
