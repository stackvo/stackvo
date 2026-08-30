# StackVo Desktop — Durum ve Eksik Analizi

**Tarih:** 2026-08-27
**Taban:** `d363caf` (main) + bu turda yapılan temizlik
**Kapsam:** Yayına çıkış ve açık kaynak olgunluğu

---

## 0. Bu turda ne yapıldı

`docs/durum.md` **kasıtlı olarak** silindi: numaralı bir iş kuyruğu ve numaralı bir karar
kaydı, okuyanda "bu konu kapanmış" izlenimi bırakıyordu. Maddeler yeniden düşünülmek yerine
etrafından dolaşılır hale gelmişti, ve atıf arkasındaki düşünceden uzun ömürlü olmuştu.

Belgeye bağlı **~1.030 atıf** temizlendi:

| Ne | Sayı | Nasıl karara bağlandı |
| --- | --- | --- |
| Belgeyi **okuyan** testler | 4 dosya / 15 test | Ağaca dayananlar korundu, yalnız belgeyi doğrulayanlar silindi |
| `docs/durum.md` yol atfı | 81 | Kaldırıldı; cümleler kendi ifadesiyle kuruldu |
| `ADR 00NN` karar numarası | 270 | Kaldırıldı; gerekçe zaten yorumun gövdesindeydi |
| `§N` / `§3 #NN` bölüm atfı | 353 | Kaldırıldı; belgenin kendi bölümlerine ve EN 301 549'a olanlar korundu |
| `A-1`, `F-3`, `M-7`… kuyruk etiketi | 371 | Kaldırıldı |
| `C-NN`, `W-NN`, `T-N` | — | **Korundu** — `contracts/CONFLICTS.md` ve `SECURITY.md`'de yaşıyorlar |

### Testlerde ne silindi, ne kaldı

Ayrım tek bir soruya göre yapıldı: **test ağacın bir özelliğini mi ölçüyor, yoksa bir
belgedeki cümleyi mi doğruluyor?**

| Test | Karar |
| --- | --- |
| `durum_sections_agree.rs` (4 test) | **Silindi** — tamamı belgenin kendi iç tutarlılığıydı |
| `platform_matrix_claims.rs` (7 test) | **Bölündü** → `web_build_invariants.rs`. Belgedeki sayıyı doğrulayan 4 test silindi; ağacın özelliğini ölçen 3'ü kaldı (`invoke` tek dosyada, dört masaüstü komutu, beşincisinin sessizce eklenmemesi) |
| `legacy_env_claims.rs` (8 test) | Belgeyi okuyan 3 test silindi; 5'i kaldı — **sürüm kapısı dahil** (`LEGACY_SERVICES_GO_AT`, 0.4.0'da build'i kırar) |
| `architecture_claims.rs` (5 test) | ADR biçimini denetleyen 1 test silindi; bağlantı denetleyicisi kaldı |
| `policy_claims.rs`, `secrets_claims.rs` | Kaynak listesinden `durum.md` satırı çıkarıldı; testler duruyor |
| `no_dangling_docs.rs` | **Genişletildi** — artık iki silinmiş belgeyi birden koruyor; bir atıf geri yazılırsa build kırmızıya döner |

### Yan etkiler

- **`docs/accessibility.md` geri getirildi.** İş kuyruğu değil, EN 301 549 biçiminde bir uyum
  beyanı — kurumsal alıcının ve mağaza listelemesinin istediği belge. İçindeki iki `durum.md`
  atfı çıkarıldı. `docs/accessibility-transcript.md` üretilen bir artefakt olarak
  `.gitignore`'a alındı (`npm run a11y:transcript` yazıyor).
- **ARCHITECTURE.md §7a yeniden yazıldı.** "Tek belge, `docs/durum.md`" tablosu yerine
  gerekçenin nerede yaşadığını söyleyen bir tablo — ve neden ayrı bir durum belgesi
  *olmadığı*, çünkü yokluğu bir ihmal gibi görünüyor.
- **ARCHITECTURE.md'nin bağlantı denetimi kurtarıldı.** Belgedeki *tüm* yerel bağlantılar
  `durum.md`'yeymiş; silinince sıfıra düştü ve gate "parser bozuldu" diye kırıldı. Belgenin
  zaten adını andığı gerçek dosyalara (`src/`, `src-tauri/src/`, `contracts/ipc.json`,
  `commands.rs`, `CHANGELOG.md`, `SECURITY.md`, `PRIVACY.md`) bağlantı verildi.
- **`PRIVACY.md`'de ölü bir bağlantı bulundu:** `docs/adr/0010-…md` — hiç var olmamış bir
  dosyaya işaret ediyordu, hiçbir test görmüyordu. Kaldırıldı.
- **README'nin "dört hedef" iddiası düzeltildi** — matris altı hedef.
- `tools/legacy-deletion-rehearsal.mjs`'in beklenen-hata tablosundan, silinen teste ait satır
  çıkarıldı.

### Sonuç

| | Önce | Sonra |
| --- | --- | --- |
| `cargo test` | 1786 geçti / **18 kırık** | **1817 geçti / 0 kırık** |
| `vitest` | 1269 geçti / **1 suite çöküyor** | **1275 geçti / 84 dosyanın 84'ü** |
| `cargo clippy -D warnings` | temiz | temiz |
| `cargo fmt --check` | temiz | temiz |
| `npm run lint` | temiz | temiz |
| `npm run types:check` | uyumlu | uyumlu |
| `npm run contracts:check` | 0 hata / 12 uyarı | 0 hata / 12 uyarı |
| `npm run notice:check` | 643 paket | 643 paket |

Kodda kalan `durum.md` atfı: **sıfır**. Yalnız üç yerde geçiyor ve üçü de kasıtlı —
`CHANGELOG.md` (tarihsel kayıt, deponun kendi kuralıyla muaf), bu rapor (mezar taşı), ve
geri gelmesini engelleyen `no_dangling_docs.rs`.

---

## 1. Projenin güçlü yanları

- **`contracts/` bir anlaşma, belge değil.** `validate-contracts.mjs` kodu sözleşmeye karşı
  doğruluyor; `src/lib/ipc.d.ts` ondan üretiliyor ve `--check` ile tutuluyor.
- **İddia testleri.** `readme_claims.rs`, `architecture_claims.rs`, `privacy_claims.rs`,
  `hint_translations.rs`, `version_agreement.rs`, `workflow_parity.rs`… Belgedeki sayı ile
  koddaki sayı ayrışırsa build kırmızı. Bu turda kendi işlerini gördüler: belge silinince
  sessizce geçmediler, kırıldılar.
- **Üretim kodunda sıfır `TODO`/`FIXME`**, üretim yollarında yalnız **20** `unwrap`/`expect`
  (1213'ü test bloklarında).
- **Yükseltme tek noktada ve dizi tabanlı.** `elevate.rs` hiçbir şeyi string'e enterpole
  etmiyor; `mkcert -install`'ın neden reddedildiği dosya başlığında yazılı.
- **Tedarik zinciri.** Kayıt defteri anahtarı updater'dan ayrı pinlenmiş, `RETIRED` listesi
  rotasyonu taşıyor, `key_ceremony.rs` iki anahtarın aynı olduğu bir build'i reddediyor.
- **Yetenek yüzeyi dar.** `capabilities/default.json` hiçbir blanket plugin izni vermiyor;
  About penceresi ayrı ve daha dar. CSP `default-src 'self'`.
- **i18n paritesi testli.** `en.js`/`tr.js` 2179 anahtarda birebir eşit; `hints.rs` kataloğu
  locale dosyalarıyla bir Rust testiyle eşit tutuluyor.

---

## 2. P0 — Yayın öncesi bloker

### P0-1. Sürüm ve etiket tutarsız

- `v0.1.0` etiketi `dfca6f3`'ü gösteriyor; main o etiketten **37 commit** ileride.
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` — üçü de hâlâ `0.1.0`.
- `CHANGELOG.md` 4300 satır ve tamamı `## [Unreleased]`; yayınlanmış tek bir sürüm başlığı yok.

Main'den bir yayın koşusu zaten var olan bir etiketle çakışan `0.1.0` artefaktları üretir ve
updater aynı sürümü gördüğü için **hiç kimseye güncelleme önermez**.

> **Yapılacak:** sürümü yükselt (ör. `0.2.0`), `CHANGELOG.md`'de `## [0.2.0] - …` başlığını
> aç, `Unreleased` içeriğini oraya taşı, sonra etiketle.

### P0-2. Güncelleme ucu yayında değil

```
asking https://github.com/stackvo/stackvo/releases/latest/download/latest.json
404 — nothing is served at …
```

Otomatik güncelleme bugün **işlevsiz**. `releaseDraft: true` olduğu için, altı hedef de yeşil
olsa bile biri **Publish**'e basana kadar bu URL 404 verir.

### P0-3. Kod imzalama ve notarization sırları yok

`release.yml` hem Apple hem Windows imzalamayı doğru kurgulamış, ama sır yoksa uyarı basıp
devam ediyor (bilinçli bir tercih, gerekçesi yazılı). Bugün: macOS imzasız → Gatekeeper
açtırmıyor; Windows imzasız → SmartScreen uyarısı; updater paketi imzasızsa **reddediyor**.

Bunlar kod işi değil, kimlik ve para işi (Apple Developer Program + Authenticode sertifikası)
ve ikisi de günler alabiliyor. Yayın takviminde süre ayrılmalı.

### P0-4. README bir son kullanıcıya hitap etmiyor

- **Kurulum/indirme bölümü yok.** Bir masaüstü uygulamasının README'sinde "İndir" yok; olan
  tek şey `npm install && npm run tauri:dev`, yani kaynaktan derleme.
- **Sistem gereksinimleri yazılı değil** — Docker Desktop/Engine asgari sürümü, desteklenen
  OS sürümleri (`tauri.conf.json` macOS 10.15 diyor, README demiyor).
- **Tek bir ekran görüntüsü yok**, rozet yok, **Türkçe README yok** (yardım belgeleri iki dilli).

### P0-5. README'nin üreteç bölümü hâlâ kodla çelişiyor

| README | Kod |
| --- | --- |
| satır 80: *"Rust üreteci Bash'in **yanında** çalışır; onun yerini almaz."* | `commands.rs` — `GeneratorEngine::Bash` **emekliye ayrıldı**, çağrılırsa `Unsupported` |
| satır 85: *"`bash` — StackVo'nun bugün yaptığı. **Varsayılan.**"* | `#[default]` artık `Rust` üzerinde; mod üç değil **iki** davranış |
| satır 89: *"Bash her modda çalışır."* | Hiçbir modda çalışmıyor |
| satır 6: *"[StackVo](…/stackvo/stackvo) klonu gerektiriyordu"* | `origin` **tam olarak o depo** |

Kök neden: `readme_claims.rs` yalnız sayıları denetliyor; README'nin **bağlantılarını ve
iddialarını** denetleyen bir gate yok. Bkz. P1-2.

---

## 3. P1 — Güvenlik ve doğrulama

### P1-1. Yükseltilmiş kopyanın kaynağı öngörülebilir bir geçici dosya

`src-tauri/src/hosts.rs:333`, `:336` ve `src-tauri/src/dns.rs:884`:

```rust
let staged = std::env::temp_dir().join("stackvo-hosts-staged");
```

Depodaki diğer **her** geçici dosya `format!("…-{}", std::process::id())` ile
tekilleştirilmiş. Root olarak kopyalanan iki dosya ise sabit adlı. Linux'ta `temp_dir()`
paylaşımlı `/tmp`'tir: yerel bir başka kullanıcı adı önceden yaratabilir veya `write` ile
`cp` arasındaki pencerede içeriği değiştirebilir; sonuç **root yetkisiyle** `/etc/hosts`
veya `/etc/resolver/*` üzerine saldırganın seçtiği içeriğin yazılmasıdır. macOS'ta `TMPDIR`
kullanıcıya özel olduğu için maruziyet dar, ama Linux resmen destekleniyor.

> **Yapılacak:** `0700` bir dizin içinde, süreç kimliği + rastgele son ekle stage et. Depo bu
> deseni zaten her yerde kullanıyor.

### P1-2. README/SECURITY/PRIVACY bağlantı denetimi yok

`architecture_claims.rs::every_link_points_at_a_file_that_exists` yalnız `ARCHITECTURE.md`
için çalışıyor. Aynı denetim `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, `PRIVACY.md` için
yok — ve bu turda `PRIVACY.md`'de tam olarak bu yüzden yıllardır duran ölü bir bağlantı
bulundu. Denetleyiciyi dört belgeyi kapsayacak şekilde genelleştirmek tek satırlık bir liste
değişikliği ve P0-5'in tekrarını engeller.

### P1-3. `cargo-deny` yerelde kurulu değil

`npm run audit` iki yarımdan oluşuyor; `npm audit` temiz (0 zafiyet, dev dahil),
`cargo deny check` bu makinede hiç çalışmıyor. CI'da `supply-chain` işi var, ama
`CONTRIBUTING.md` "push etmeden önce çalıştır" diyor ve komut kurulum olmadan sessizce
kullanılamıyor.

---

## 4. P2 — Test ve kapsam

### P2-1. Rust çekirdeğinin kapsamı %64

| Yarım | Ölçülen | Taban |
| --- | --- | --- |
| Rust — satır | **%64,05** | %60 |
| Ön yüz — satır | %92,07 | %85 |
| Ön yüz — dal | %81,04 | %72 |
| Ön yüz — fonksiyon | %60,34 | (taban yok, bilinçli) |

Ağırlık merkezi olan 67 bin satırlık Rust çekirdeği %64'te; Docker'a, dosya sistemine ve
ayrıcalıklı yollara dokunan kod bu taraf. Taban %60 olduğu için bugün 4 puanlık bir gerileme
sessizce geçer.

### P2-2. Gerçek Docker'a karşı otomatik test yok

- Playwright e2e IPC sınırını `stage.js` ile taklit ediyor — zarif bir seam, ama Rust'ı hiç
  çalıştırmıyor.
- WebDriver sürücü testi gerçek uygulamayı açıyor: **5 test**.
- `real_checkout.rs` checkout yoksa **atlanıyor**.
- Gerçek uçtan uca doğrulama `npm run diagnose` — elle çalıştırılan bir araç.

"Docker açıkken proje ayağa kalkıyor mu" sorusunu CI'da soran hiçbir şey yok. Bir masaüstü
Docker yöneticisi için en pahalı regresyon sınıfı kör noktada.

> Linux runner'da Docker hazır geliyor. Nightly bir işte geçici workspace + tek proje
> `up`/`down` duman testi ölçülü bir yatırım.

### P2-3. `installers:check` script'i çağrılamıyor

```json
"installers:check": "node tools/check-installers.mjs"
```

Araç `--target <triple>` zorunlu istiyor; script argümansız çağrıldığı için her zaman `usage`
basıp çıkıyor. `package.json` içinde bu satırın girintisi de diğerlerinden farklı.

### P2-4. Paket bütçesi tavana 33 KB kala

```
eager (ilk boyama)    1535,5 KB   tavan 1700,0 KB
total (tüm varlıklar) 2967,1 KB   tavan 3000,0 KB
```

Toplamda **%1,1** pay kalmış. Bir Vuetify bileşeni daha eklendiğinde kapı kapanır ve bu,
bütçeyi düşünmek için en kötü andır. Ayrıca ilk boyama için 1,5 MB JS.

### P2-5. `contracts:check` 12 uyarı veriyor

- **10 tanesi aynı sınıf:** `SERVER_MAX_BODY_SIZE`, `SERVER_FASTCGI_TIMEOUT`,
  `SERVER_CLIENT_BODY_TIMEOUT`, `SERVER_KEEPALIVE_TIMEOUT`, `SERVER_TCP_NODELAY`,
  `SERVER_GZIP`, `SERVER_GZIP_COMP_LEVEL`, `SERVER_GZIP_TYPES`,
  `SERVER_FASTCGI_CONNECT_TIMEOUT`, `SERVER_FASTCGI_SEND_TIMEOUT` — ayarlanıyor ama
  `contracts/env.schema.json` tanımlamıyor. Sözleşmenin "tek gerçek kaynak" iddiası bu on
  anahtar için geçerli değil.
- ~~**1 ölü kod:** `api.appsAvailable()` tanımlı, hiçbir view veya store çağırmıyor.~~
  **Bu madde yanlıştı ve kapatıldı.** Çağrı `PreferencesPane.vue:34`'te duruyor; uyarıyı
  üreten denetleyicinin kendisi bozuktu. Ölçüm ve gerekçe H bölümünde, düzeltme
  `validate-contracts.mjs`'te. Uyarı sayısı 12'den **11'e** düştü.
- 1 beklenen (`--allow-no-manifests`).

---

## 5. P3 — Mimari ve bakım borcu

### P3-1. `commands.rs` bir tanrı-modül

15.6k satır tek dosyada (2.9k üretim + testler); **303 IPC komutunun tamamı burada.** Bir
sonraki en büyük dosya `cli.rs` 4.8k satır (4.2k üretim — yani testsiz-en-büyük dosya bu).

Deponun geri kalanıyla çelişiyor: 113 modülün 110'u konusuna göre ayrılmış (`dns.rs`,
`certs.rs`, `tunnel.rs`, `xdebug.rs`…) ama IPC katmanı tek yığın. Sonuçları: merge çakışma
yüzeyi, `rust-analyzer` gecikmesi, bir komutun sahibinin dosya adından okunamaması.

> `commands/` dizini + alt sistem başına bir dosya. `generate_handler!` listesi aynı kalır,
> davranış değişmez, `contracts:check` doğrulamayı sürdürür.

### P3-2. Boş `version` dosyası

Depo kökünde 0 baytlık, izlenen, hiçbir şeyin okumadığı bir `version` dosyası (ilk commit'ten
kalma). Silinmeli.

### P3-3. Paket üstverisi eksik

- `package.json`: `repository`, `bugs`, `homepage`, `author`, `engines` yok. `repository`
  yokluğu Dependabot dahil araçları körleştiriyor.
- `Cargo.toml`: `authors = ["StackVo"]` (LICENSE `Fahrettin Aksoy` diyor — tutarsız),
  `repository`/`homepage`/`keywords`/`categories` yok.

---

## 6. P4 — Açık kaynak olgunluğu

| Dosya | Durum | Neden gerekli |
| --- | --- | --- |
| `CODE_OF_CONDUCT.md` | ❌ yok | GitHub topluluk profili arar; katkı kabul eden projede beklenir |
| `SUPPORT.md` | ❌ yok | "Sorum var, nereye?" — issue mı, tartışma mı, e-posta mı |
| `.github/ISSUE_TEMPLATE/config.yml` | ❌ yok | Boş issue'yu kapatıp güvenlik/tartışma linki vermek için |
| `.github/ISSUE_TEMPLATE/feature_request.yml` | ❌ yok | Yalnız `bug_report.yml` var |
| `.github/dependabot.yml` | ❌ yok | npm + cargo + actions üç ekosistem, hiçbiri izlenmiyor |
| `.editorconfig` | ❌ yok | `.gitattributes` LF'i zorluyor ama editör tarafı boş |
| `.nvmrc` / `engines` | ❌ yok | CI Node 22 kullanıyor, yerelde hiçbir yerde yazmıyor |

Ek olarak: **`CHANGELOG.md` 4300 satır ve tamamı `Unreleased`.** Keep a Changelog formatını
iddia ediyor ama sürüm bölümlemesi hiç yapılmamış. Girdiler de bir "değişiklik günlüğü"nden
çok "mühendislik günlüğü" — okunması değerli ama sürüm notu olarak kullanılamaz; yayın için
kısa, kullanıcıya dönük bir sürüm notu ayrıca gerekecek.

---

## 7. P4 — Bağımlılık güncelliği

`npm audit` temiz, ama yedi paket bir major sürüm geride:

| Paket | Kullanılan | Güncel | Not |
| --- | --- | --- | --- |
| `vuetify` | 3.12.11 | 4.1.12 | UI'ın tamamı buna bağlı; major geçiş büyük iş |
| `pinia` | 2.3.1 | 4.0.3 | **iki** major geride |
| `vue-i18n` | 9.14.5 | 11.4.10 | **iki** major geride |
| `vue-router` | 4.6.4 | 5.2.0 | |
| `vite` | 7.3.6 | 8.2.2 | |
| `vitest` + `@vitest/coverage-v8` | 3.2.7 | 4.1.11 | |
| `eslint` + `@eslint/js` | 9.39.5 | 10.9.1 | |
| `jsdom` | 26.1.0 | 30.0.1 | test-only |

Hiçbiri bugün bir güvenlik sorunu değil. Ama Dependabot olmadığı için mesafe sessizce
açılıyor ve her ay geçişi pahalılaştırıyor. **Yayından hemen sonra** açılmalı; yayın öncesi
major geçiş yapılmamalı.

---

## 8. "Şunlar da olsaydı"

1. **Çökme raporlama yok.** `PRIVACY.md` bunu bilinçli bir söz olarak veriyor ve bu
   savunulabilir. Ama `crash.rs` yerel bir çökme kaydı tutuyor — kullanıcının **kendi
   isteğiyle** gönderebileceği, `diagnostics.rs` paketine iliştirilen bir "çökme raporu
   gönder" akışı sözü bozmadan görünürlük kazandırırdı.
2. **İlk açılış turu yok.** `BootstrapGate`, `RequirementsGate`, `MigrationGate`,
   `CatalogueGate` var — ama bunlar engel, tanıtım değil. 303 komutluk bir yüzeyde keşif zor.
3. **Güncelleme kanalı tek.** `channel.rs` var ama `tauri.conf.json` tek uç tanımlıyor;
   beta/stable ayrımı yayın sonrası hızlı düzeltme için değerli.
4. **CLI yerelleştirilmemiş.** GUI iki dilli, `stackvo` CLI yalnız İngilizce. Tutarlı bir
   tercih olabilir ama yazılı bir karar olarak durmuyor.
5. **Sürüm başına yükseltme notu yok.** `updates.js` manifesti zaten okuduğu için ucuz.
6. **Docker dışı bir motor yok.** Podman uyumluluğu (soket seviyesinde büyük ölçüde uyumlu)
   yol haritasında görünmüyor; Linux kullanıcıları için gerçek bir talep.

---

## 9. Önerilen sıra

**Yayından önce:**

1. README'nin üreteç bölümünü düzelt: modlar, depo kimliği. *(P0-5)*
2. README'ye **Kurulum / Sistem Gereksinimleri / Ekran Görüntüleri** ekle. *(P0-4)*
3. Bağlantı denetimini README/SECURITY/CONTRIBUTING/PRIVACY'yi kapsayacak şekilde genişlet. *(P1-2)*
4. Sürümü yükselt, CHANGELOG'u sürümle, etiketle. *(P0-1)*
5. İmzalama sırlarını temin et ve yayın koşusunu **rehearsal** modunda uçtan uca doğrula. *(P0-3)*
6. Release'i Publish et ve `npm run updates:check` ile ucu doğrula. *(P0-2)*
7. Geçici dosya stage'ini sertleştir (`hosts.rs`, `dns.rs`). *(P1-1)*

**Yayından hemen sonra:**

8. Dependabot + `CODE_OF_CONDUCT.md` + `SUPPORT.md` + issue şablonları. *(P4)*
9. `env.schema.json`'a eksik 10 `SERVER_*` anahtarını ekle; `api.appsAvailable()`'ı sil. *(P2-5)*
10. Rust kapsam tabanını ölçülen değere yaklaştır; Docker'lı nightly duman testi. *(P2-1, P2-2)*
11. `commands.rs`'i alt sistemlere böl. *(P3-1)*
12. Paket bütçesi için pay yarat veya tavanı gerekçeli yükselt. *(P2-4)*

---

## 10. Tek cümlelik hüküm

Bu, mühendislik disiplini bakımından yayına hazır olmayı hak eden bir proje; onu bugün
yayından alıkoyan şey kodun kalitesi değil, **yayınlanmamış bir sürüm, temin edilmemiş iki
imzalama kimliği ve bir son kullanıcıya hitap etmeyen bir README** — üçü de kod yazmayı
değil, sıraya koymayı gerektiren işler.

---

# Ek Rapor — Sabit kod (hardcode) ve arayüzden yönetilmesi gereken veri

**Tarih:** 2026-08-27 (ikinci tur)
**Taban:** `6e56a2f` (main) + çalışma ağacındaki belge değişiklikleri
**Sorulan iki soru:** (1) Kodda hardcode var mı? (2) Arayüzde bir **seçim** olması gereken
değişken veri doğrudan koda mı gömülü?

**Kapsam ve yöntem.** 275 kaynak dosya okundu/tarandı — **47.794 satır** JS/Vue (`src/`),
**111.843 satır** Rust (`src-tauri/src/`, 113 modül), `contracts/` (12 dosya), `tools/`
(15 betik). Tarama grep ile değil, çoğu yerde **iki listeyi karşılaştıran** küçük
betiklerle yapıldı: bir sabitin nerede tekrarlandığını grep gösterir, **birbirinden
ayrıştığını** yalnız karşılaştırma gösterir. Doğrulama için `vitest` (84/84 dosya, 1275
test geçti) ve `npm run contracts:check` (0 hata / 12 uyarı) çalıştırıldı.

**Tek cümlelik hüküm:** Bu depoda "gelişigüzel serpiştirilmiş sihirli sayı" sorunu **yok** —
zamanlayıcılar adlandırılmış, renkler tema dosyasında toplanmış, i18n disiplini neredeyse
kusursuz (şablonlarda yalnız 4 marka adı ve 2 yol dizgesi çevrilmemiş). Bulunan sorun daha
dar ve daha keskin: **aynı değerin iki ya da dört ayrı yerde sabitlenmesi**, ve
**kullanıcının arayüzden seçtiği ayarın, o değeri ikinci kez sabitleyen kod yolunda
görmezden gelinmesi.**

## Özet

| # | Bulgu | Nerede | Sınıf |
| --- | --- | --- | --- |
| A-1 | Sürüm **listelerinin** hiçbir arayüz denetimi yok | `config.rs:81-95` | Arayüz eksiği |
| A-2 | Kabul (adopt) yolu hiçbir ayarı okumuyor — PHP, sunucu, TLD | `commands.rs:7405` | **Ayar baypası** |
| A-3 | Editör/terminal/tarayıcı/DB istemcisi kataloğu genişletilemiyor | `apps.rs` | Arayüz eksiği |
| A-4 | Hızlı komutlar kataloğu derlenmiş, kullanıcı ekleyemiyor | `quickcmd.rs:120` | Bilinçli, ama yazılı değil |
| A-5 | "Varsayılan sunucu" ayarının tek tüketicisi var | `commands.rs:1073` | **Ayar baypası** |
| B-1 | Dokuz nginx yönergesi **üç** yerde sabit; bağlayan test yok + indeks hatası | `generator.rs:823,946` | **Gizli hata** |
| B-2 | `stackvo.loc` ×14, `stackvo-net` ×9, `nginx` ×7 literal | `commands.rs` | Sürüklenme riski |
| B-3 | Varsayılan PHP sürümü dört yerde; biri `8.2` diyor | `validate-contracts.mjs:489` | Sürüklenme |
| B-4 | Varsayılan Python `3.14` mü `3.13` mü | `config.rs:86` / `manifest.rs:120` | Sürüklenme |
| B-5 | 28 proje şablonu dört elle yazılmış listede, kapı yok | `NewProjectDrawer.vue:46` | Sürüklenme riski |
| B-6 | Ön yüz 5 runtime biliyor, arka uç 8 üretebiliyor | `useCatalog.js:53` | Erişilemeyen özellik |
| C-1 | Grafik renkleri `#1976D2` sabit — kullanıcının accent'i geçersiz | `useContainerStats.js:34` | **Ayar baypası** |
| C-2 | Isı ızgarasının beş yeşili tema dışı | `project-panes.css:225` | Tema dışı |
| D-1 | Uygulamanın kendi 10 imgesi sabit, 6'sı `:latest` | `tunnel.rs`, `landing.rs` | Tedarik zinciri |
| D-2 | Kayıt defteri aynası (mirror) bu 10 imgeye **hiç uygulanmıyor** | `commands.rs:12668` | **Politika deliği** |
| E-1 | `env.schema.json`'daki 43 tüketicinin 39'u var olmayan dosya | `contracts/env.schema.json` | Bayat sözleşme |
| E-2 | `SUPPORTED_LANGUAGES_RUST_DEFAULT`: sözleşme `1.62`, kod `1.84` | `env.schema.json:494` | **Sözleşme yalanı** |
| F-1 | 39 İngilizce katalog cümlesi iki dilli arayüze ham gidiyor | `quickcmd/oauth/tooling` | i18n deliği |
| G-1 | Genel Bakış her projeye `/var/www/html` diyor — Node/Go'da `/app` | `OverviewPane.vue:86` | **Kullanıcıya görünen hata** |
| H-1 | Önceki raporun bir maddesi yanlış: `appsAvailable()` ölü değil | `PreferencesPane.vue:32` | Düzeltme |

---

## A. Arayüzde olması gereken ama yalnız kodda olan veri

Bu bölüm sorunun ikinci yarısının doğrudan cevabı.

### A-1. Sunulan sürüm **listeleri** hiçbir yerden düzenlenemiyor

`config::SETTINGS` iki tür anahtar taşıyor ve ikisi arayüzde eşit muamele görmüyor:

| Anahtar ailesi | Ne demek | Arayüz denetimi |
| --- | --- | --- |
| `SUPPORTED_LANGUAGES_*_DEFAULT` | Hangi sürüm seçili gelsin | ✅ `PhpPane.vue` — altı `v-select` |
| `SUPPORTED_LANGUAGES_*_VERSIONS` | **Hangi sürümler sunulsun** | ❌ hiçbiri |

Ölçüldü: `SETTINGS`'in 36 anahtarından **7'si** `src/` ağacının tamamında bir kez bile
geçmiyor — altı `_VERSIONS` listesi ve `SUPPORTED_LANGUAGES_PHP_EXTENSIONS_DEFAULT`.

```
src-tauri/src/config.rs:81   SUPPORTED_LANGUAGES_PHP_VERSIONS     = 5.6 … 8.5
src-tauri/src/config.rs:85   SUPPORTED_LANGUAGES_PYTHON_VERSIONS  = 2.7 … 3.14
src-tauri/src/config.rs:87   SUPPORTED_LANGUAGES_GO_VERSIONS      = 1.11 … 1.23
src-tauri/src/config.rs:89   SUPPORTED_LANGUAGES_RUBY_VERSIONS    = 2.4 … 3.3
src-tauri/src/config.rs:91   SUPPORTED_LANGUAGES_RUST_VERSIONS    = 1.70 … 1.84
src-tauri/src/config.rs:93   SUPPORTED_LANGUAGES_NODEJS_VERSIONS  = 16,18,20,21,22,23
```

`config.rs`'in kendi belge yorumu bu tasarımın gerekçesini yazıyor ve gerekçe doğru:
listeler `.env`'den ikilik dosyaya taşındı, "yeni PHP sürümü → yeni build → yeni liste".
Eksik olan cümlenin ikinci yarısı: **iki build arasında kullanıcının yapabileceği bir şey
yok.** Bir sürüm listeyi kaçırdıysa tek çare `.env`'i elle açmak; uygulama o dosyayı
kendi yazdığı halde bu anahtarları hiçbir panelde göstermiyor.

Ve listeler şimdiden geride: Go `1.23`'te duruyor (1.24 ve 1.25 çoktan çıktı), Ruby
`3.3`'te (3.4 çıktı), Node `23`'te (24 LTS oldu), Rust `1.84`'te (2025 Ocak sürümü).
Burada asıl mesele hangi sürümün eksik olduğu değil — **eksikliğin ancak bir yayın
koşusuyla kapanabilmesi.**

> **Yapılacak:** `PhpPane`'e altı `v-combobox` (`multiple chips`) — `PHP_DEFAULT_TOOLS` ve
> `PHP_DEFAULT_APT_PACKAGES` için zaten tam olarak bu bileşen kullanılıyor
> (`PhpPane.vue:158,168`), yani desen ağaçta hazır. `useEnvEditor`'ın `setList`'i de hazır.
> Yaklaşık 30 satır, ve bir yayına bağımlılığı ortadan kaldırıyor.

### A-2. Bir klasörü "kabul et" yolu ayarların hiçbirini okumuyor

`detected_spec` (`src-tauri/src/commands.rs:7405`) benimsenen bir dizinden manifest üretiyor.
İmzası şu: `fn detected_spec(name: &str, detected: &detect::Detected)` — **`Env` almıyor**,
yani kullanıcının kaydettiği hiçbir ayarı okuyamaz. Sonuç dört sabit değer:

| Satır | Yazılan | Ayarın söylediği | Sonuç |
| --- | --- | --- | --- |
| `commands.rs:7409` | `format!("{name}.loc")` | `DEFAULT_TLD_SUFFIX` = `stackvo.loc` | Etki alanı ayarı yok sayılıyor |
| `commands.rs:7442` | `"8.4"` | `SUPPORTED_LANGUAGES_PHP_DEFAULT` | PHP seçimi yok sayılıyor |
| `commands.rs:7436` | `detected.server` → `detect.rs:205,261,337,410` hepsi `"nginx"` | `SUPPORTED_SERVERS_DEFAULT` | Sunucu seçimi yok sayılıyor |
| `commands.rs:7419` | `"22"` (node) | `SUPPORTED_LANGUAGES_NODEJS_DEFAULT` | — (bugün aynı, yarın değil) |

Yani: **Ayarlar → Varsayılan PHP sürümü**'nü 8.2 yapan bir kullanıcı, yeni proje
sihirbazından proje açtığında 8.2 alıyor; var olan bir klasörü benimsettiğinde 8.4 alıyor.
İkisi de "yeni proje", ikisi arasında hiçbir açıklama yok.

`.loc` için sözleşmede bir gerekçe var (`env.schema.json` → `DEFAULT_TLD_SUFFIX`: "Project
domains do NOT use it"), ve o savunulabilir. Diğer ikisi için gerekçe yok — `detected_spec`
`Env` almadığı için tercih değil, **erişim eksikliği**.

> **Yapılacak:** `detected_spec(name, detected, &env)`. Üç çağrı yeri var
> (`commands.rs:5357`, `:6976`, `:7204`) ve üçünün de elinde `root` zaten var.

### A-3. Editör / terminal / tarayıcı / veritabanı istemcisi listesi kapalı

`apps.rs` sekiz platform-koşullu sabit dizi taşıyor (`:51,:77,:90` terminaller, `:121`
editörler, `:166,:208,:218` tarayıcılar, `:393` DB istemcileri) ve her satır sabit bir
uygulama demeti yolu: `/Applications/Zed.app`, `/Applications/TablePlus.app`…

Modül başlığı tasarımı açıkça anlatıyor ve haklı: eskiden serbest metin kutusu vardı,
kullanıcının başlatıcı adını bilmesi gerekiyordu, tespit bundan iyi. Ama **serbest metin
kutusu kaldırılırken yerine "diğer…" seçeneği konmamış.** `PreferencesPane.vue:52-86`'daki
üç `v-select`'in öğeleri yalnız `api.appsAvailable()`'dan geliyor; `apps.rs` içinde
`custom` diye bir varyant yok. Listede olmayan bir editör kullanan biri — Helix, Neovim,
Emacs, JetBrains'in listelenmeyen sekiz IDE'sinden biri — bugün **hiçbir şey seçemiyor.**

> **Yapılacak:** listeye sabit bir `{ id: "custom" }` satırı ve seçildiğinde açılan bir
> komut alanı. Tespit varsayılan kalır, kapı açılır.

### A-4. Hızlı komutlar kataloğu derlenmiş

`quickcmd.rs:119`: *"Everything on offer. Adding a row here is the only way to add a
command."* — açık ve bilinçli, ve bir güvenlik gerekçesi var (`argv`, asla kabuk dizgesi).
Ama bu karar hiçbir kullanıcıya dönük belgede yazmıyor, ve "kendi komutumu ekleyeyim"
bir yerel geliştirme aracından beklenen ilk isteklerden biri. `hooks.rs` ve `cron.rs`
kullanıcının yazdığı komutları zaten çalıştırıyor, yani yasak katalog için geçerli, kabuk
için değil. Kararın kendisi savunulabilir; **yazılı olmaması** savunulabilir değil.

### A-5. "Varsayılan sunucu" ayarı tek bir yerde tüketiliyor, geri kalanı `"nginx"`

`SUPPORTED_SERVERS_DEFAULT` üretim kodunda **tam olarak bir** yerde okunuyor:
`commands.rs:1073` — `Catalog.default_server` alanını doldurmak için. O alanın tek
tüketicisi `NewProjectDrawer.vue:206`, yani sihirbazda seçili gelen değer.

Manifestte `server` alanı yoksa render yolu ne yapıyor:

```
src-tauri/src/generator.rs:771    manifest.server.as_deref().unwrap_or("nginx")
src-tauri/src/generator.rs:1285   manifest.server.as_deref().unwrap_or("nginx")
src-tauri/src/generator.rs:1675   manifest.server.as_deref().unwrap_or("nginx")
src-tauri/src/commands.rs:12514   m.server.as_deref().unwrap_or("nginx")
src-tauri/src/commands.rs:12516   m.server.as_deref().unwrap_or("nginx")
```

Beş yerde sabit. `SUPPORTED_SERVERS_DEFAULT=caddy` yazan bir kullanıcı için: sihirbazdan
açılan proje Caddy, benimsenen proje nginx, `server` alanı elle silinmiş bir manifest
nginx.

---

## B. Aynı değerin birden çok yerde sabitlenmesi

### B-1. Dokuz nginx yönergesi üç ayrı yerde — ve iki tanesi **sıra numarasıyla** okunuyor

Aynı dokuz ayar üç bağımsız listede yaşıyor:

| Nerede | Ne taşıyor |
| --- | --- |
| `src-tauri/src/config.rs:98-108` | anahtar + **varsayılan** |
| `src-tauri/src/generator.rs:823` `NGINX_DIRECTIVES` | anahtar + nginx adı + **varsayılan** (ikinci kopya) |
| `src/components/settings/ServerLimitsPane.vue:27,35` | anahtar + biçim + ikon (elle yazılmış) |

İkisi de `"1m"`, `"60"`, `"75"`, `"on"`, `"off"` değerlerini ayrı ayrı yazıyor ve
**hiçbir test ikisini karşılaştırmıyor** (`grep -rn NGINX_DIRECTIVES` üretim dışında sıfır
sonuç). `value_of` bir değeri varsayılanına eşitse yazmıyor — yani iki kopya ayrışırsa
sonuç bir hata değil, **sessizce fazladan ya da eksik yazılan bir yönerge**.

Daha keskini `generator.rs:946` ve `:952`:

```rust
if let Some(size) = self.value_of(&NGINX_DIRECTIVES[0]) {   // client_max_body_size
    …  request_body { max_size … }
}
if self.value_of(&NGINX_DIRECTIVES[4]).is_some_and(|v| v == "on") {   // gzip
    …  encode gzip
}
```

Caddy'nin iki yönergesi diziye **sıra numarasıyla** erişiyor. Dizi zaten alfabeye ya da
konuya göre dizilmiş değil; araya bir yönerge eklemek — bu dosyada en olası düzenleme —
`[0]`'ı `client_max_body_size`'dan başka bir şeye kaydırır ve Caddy `max_size 60` yazar.
Derleme geçer, test yok, hata çalışma anında ortaya çıkar.

Aynı panelde `SERVER_SUPPORT` (`ServerLimitsPane.vue:54`) beş sunucuyu adıyla sayıyor;
`manifest.rs:30` `SERVERS` de beşini. Altıncı bir sunucu eklendiğinde harita `undefined`
döndürür ve yeni sunucu arayüzde sessizce "desteklenmiyor" görünür.

> **Yapılacak:** (1) `NGINX_DIRECTIVES[0]`/`[4]` yerine `by_key("SERVER_MAX_BODY_SIZE")`.
> (2) `NGINX_DIRECTIVES`'in `default` alanını silip `config::SETTINGS`'ten okumak — ya da
> en azından ikisini eşitleyen bir test. (3) Panelin anahtar listesini `contracts:check`'e
> bağlamak; bu on anahtarın `env.schema.json`'a eklenmesi zaten öneri listesinde (P2-5) ve
> eklendiğinde şema tek kaynak olabilir.

### B-2. Beş varsayılanın literal kopyaları

`Env::parse` (`config.rs:436`) `EMBEDDED`'i dosyanın **altına** seriyor, yani `env.get(K)`
bir anahtar için asla `None` dönmüyor. Buna rağmen çağrı yerleri varsayılanı ikinci kez
yazıyor:

| Değer | Kanonik yeri | Literal tekrar |
| --- | --- | --- |
| `stackvo.loc` | `config.rs:97` + `certs.rs:31` **adlandırılmış sabit** | 14 (üretimde `commands.rs`'te 8) |
| `stackvo-net` | `config.rs:113` `DOCKER_DEFAULT_NETWORK` | 9 |
| `nginx` | `config.rs:79` `SUPPORTED_SERVERS_DEFAULT` | 7 |
| `20` | `config.rs:115` `PHP_TOOL_NODEJS_VERSION` | 3 |
| `latest` | `config.rs:114` `PHP_TOOL_COMPOSER_VERSION` | 3 |

`certs.rs:31`'de `pub const FALLBACK_SUFFIX: &str = "stackvo.loc"` **zaten var** ve tam
olarak bunun için yazılmış; `commands.rs` sekiz kez görmezden geliyor.

Bunlar bugün zararsız — çünkü `unwrap_or` kolu normal yolda hiç çalışmıyor. Zararsız
olmaları asıl sorun: **ölü oldukları için ayrıştıkları gün hiçbir test kırılmaz.**
Varsayılan `.loc`'tan başka bir şeye çevrildiğinde bu sekiz satır sessizce eskiyecek.

Aynı yapıda üç kopya daha: `ToolchainOptions` bloğu `commands.rs:7370`, `:12232` ve
`:12418`'de kelimesi kelimesine tekrarlanıyor (`"latest"` + `"20"` dahil).

### B-3. Varsayılan PHP sürümü dört yerde, biri uyuşmuyor

| Yer | Değer |
| --- | --- |
| `src-tauri/src/config.rs:82` | `8.4` |
| `contracts/env.schema.json:411` | `8.4` |
| `src-tauri/src/commands.rs:7442` (kabul yolu) | `8.4` — sabit, ayarı okumuyor (A-2) |
| `tools/validate-contracts.mjs:489` | **`8.2`** |

Sözleşme denetleyicisinin kendi yedeği iki minör sürüm geride. `.env` bulunmayan bir
kontrolde (`--allow-no-manifests`, yani CI'daki hal) uzantı matrisi 8.4 yerine 8.2 kurallarına
göre doğrulanıyor: 8.3'te kaldırılmış bir uzantı sessizce geçer.

### B-4. Varsayılan Python: `3.14` mü `3.13` mü

```
src-tauri/src/config.rs:86     SUPPORTED_LANGUAGES_PYTHON_DEFAULT = "3.14"   → katalog, Ayarlar paneli
src-tauri/src/manifest.rs:120  lang_defaults("python").version    = "3.13"   → benimseme, eksik manifest
```

`manifest::lang_defaults` bir manifestte `python` bloğu yoksa devreye giriyor
(`manifest.rs:1275`) ve benimseme yolunda manifeste yazılıyor (`commands.rs:7420`). Yani
Ayarlar 3.14 diyor, benimsenen proje 3.13 çalışıyor. İkisini bağlayan bir test yok.

(Go `1.23`/`1.23` ve Ruby `3.3`/`3.3` bugün uyuşuyor — yani bu kopyalar **çalışıyor**,
Python'da çalışmıyor. Rust'ta `"1"` bilinçli olarak farklı ve gerekçesi yazılı.)

### B-5. 28 proje şablonu dört elle yazılmış listede

| Liste | Yer | Girdi |
| --- | --- | --- |
| `Template::ALL` | `scaffold.rs:85` | 28 |
| `TEMPLATE_GROUPS` | `NewProjectDrawer.vue:46` | 28 + `empty` |
| `TEMPLATE_RUNTIME` | `NewProjectDrawer.vue:70` | 28 |
| `TEMPLATE_ICONS` | `NewProjectDrawer.vue:100` | 28 + `empty` |

Karşılaştırıldı: **dördü de bugün birebir uyuşuyor.** Tek sorun, uyuşmalarını sağlayan
şeyin disiplin olması. Karşı örnek aynı depoda: `IMPORT_SOURCES` (`Projects.vue:689`) de
elle yazılmış bir kopya ama `foreign_import.rs` onu Rust listesine bağlıyor, ve
`Projects.vue:684`'teki yorum bunu neden yaptığını anlatıyor — *"an id here the backend
refuses is a button that errors"*. Şablonlar için aynı akıl yürütme yapılmamış: yeni bir
şablon dört dosyada düzenleme demek ve üçü unutulursa arayüz ya şablonu göstermez ya da
`mdi-folder-outline` ile gösterip yanlış runtime'a bağlar.

### B-6. Ön yüz beş runtime biliyor, arka uç sekiz üretebiliyor

```
src-tauri/src/commands.rs:989   IMPLEMENTED_RUNTIMES = php, node, python, go, ruby, rust, bun, deno   (8)
src/composables/useCatalog.js:53  RUNTIME_DEFAULTS   = python, go, ruby, rust, node                   (5)
```

`build_catalog` özellikle bun ve deno'yu `.env` bilmese de kataloğa ekliyor
(`commands.rs:1037-1047`) ve yorumu neden yaptığını anlatıyor: *"What the app can build is
a fact about this binary, not about a file on disk."* Ama Ayarlar panelinin ızgarası
`RUNTIME_DEFAULTS` üzerinde dönüyor (`PhpPane.vue:57`), ve o liste beş satır. **Bun ve
Deno'nun varsayılan sürümü arayüzden hiç görünmüyor** — kataloğun onlar için doğru cevabı
üretmek üzere yazılmış olmasına rağmen. `SETTINGS`'te `SUPPORTED_LANGUAGES_BUN_DEFAULT`
diye bir anahtar da yok, yani bu ızgara düzeltilse bile yazacak yeri olmayacak.

---

## C. Tema sabitleri: kullanıcının seçtiği renk baypas ediliyor

`lib/appearance.js` örnek bir tasarım: 20 accent tonu, üç durum paleti (Okabe-Ito dahil),
altı nötr aile, hepsi veri. Sorun bu sistemin dışında kalan üç yerde.

### C-1. Grafik renkleri `primary`'yi izlemiyor

```
src/composables/useContainerStats.js:34,35   memoryPie   #1976D2 / #2A313C
src/composables/useContainerStats.js:45,46   networkPie  #1976D2 / #2A313C
src/composables/useContainerStats.js:57,63   diskPie     #1976D2 / #2A313C
src/components/project/IndicatorPane.vue:49  sparkline   ['#1976D2', '#4CAF50']
```

`#1976D2` `DEFAULT_APPEARANCE.primary`'nin (`appearance.js:22`) **kopyası**, `#2A313C` da
graphite temasının `surface-variant`'ının (`appearance.js:123`) kopyası. Kullanıcı accent'i
mora çevirdiğinde uygulamanın tamamı morarıyor, üç pasta grafiği ve sparkline mavi kalıyor.
Açık temada `#2A313C` (koyu kömür) beyaz kartın üstünde ikinci dilim olarak duruyor —
temayla hiç ilgisi olmayan bir renk.

Vuetify teması çalışma anında okunabilir (`useTheme().current.value.colors.primary`), yani
düzeltme birkaç satır.

### C-2. Isı ızgarasının beş yeşili

`src/styles/project-panes.css:225-238` — `.heat-cell.l0` … `.l4` sabit beş yeşil ton
(`#0e2a16` → `#66bb6a`). Ne temaya, ne accent'e, ne `statusPalette` seçimine bağlı.
`ACCESSIBILITY.md:83` renk körlüğü paletini uygulamanın bir özelliği olarak sayıyor; bu
ızgara o sistemin dışında. (Yoğunluk rampası ton değil parlaklık taşıdığı için 1.4.1'i
ihlal etmiyor — hücrelerde tooltip ve altında "az/çok" göstergesi var. Bulgu erişilebilirlik
değil, **tema tutarlılığı**: kullanıcı temayı değiştirdiğinde bu kart değişmiyor.)

### C-3. `#12121a` iki yerde

`TerminalPane.vue:71` (xterm teması, JS) ve `:193` (CSS arka planı) aynı rengi ayrı ayrı
yazıyor. İkisi ayrışırsa terminalin kenarında bir çerçeve belirir.

---

## D. Kapsayıcı imgeleri: hareketli etiket + politika deliği

### D-1. Uygulamanın kendi 10 imgesi sabit, altısı `:latest`

```
src-tauri/src/tunnel.rs:214,307   cloudflare/cloudflared:latest
src-tauri/src/tunnel.rs:331       ngrok/ngrok:latest
src-tauri/src/tunnel.rs:356       tailscale/tailscale:latest
src-tauri/src/tunnel.rs:376       openziti/zrok:latest
src-tauri/src/tunnel.rs:394       localxpose/localxpose:latest
src-tauri/src/tunnel.rs:424       kroniak/ssh-client:latest      (SSH_IMAGE)
src-tauri/src/tunnel.rs:274       node:22-alpine                 (localtunnel)
src-tauri/src/landing.rs:43       nginx:alpine
src-tauri/src/tunnelid.rs:97      nginx:alpine                   (GUARD_IMAGE)
src-tauri/src/perf.rs:627         alpine:3                       (HELPER_IMAGE)
```

Aynı depo `pkg.rs:50`'de bir kural taşıyor:

> `MOVING_TAGS: ["latest", "stable", "edge", "main", "master"]` — *"A moving tag is
> forbidden… an image that changes under a fixed manifest has no digest the manifest can
> pin, so it has no place in the chain of trust."*

Bu kural **yalnız üçüncü taraf paketlere** uygulanıyor. Uygulamanın kendi imgeleri tam da
yasakladığı etiketleri kullanıyor; kırık bir `cloudflared:latest` yayınlandığı gün her
kullanıcının tünelleri bozulur ve **sabitlenecek bir yer yoktur** — bu on değer arayüzde
de, `.env`'de de, politika dosyasında da görünmüyor.

### D-2. Kayıt defteri aynası bu on imgeye hiç uygulanmıyor

`policy.rs` kurumsal bir aynayı destekliyor (`registryPrefix`) ve `commands.rs:12667`'deki
yorum kapsamın "her yer" olduğunu söylüyor:

> *"Every image reference in the workspace passes through this function on its way to disk,
> so one pass here covers the project Dockerfiles, both compose files and every service
> template at once."*

Cümledeki belirleyici kelime **`to disk`**. `policy::rewrites` (`policy.rs:824`) yalnız
`Dockerfile`, `*.yml`, `*.yaml` uzantılarına `true` diyor. Yukarıdaki on imge ise hiçbir
dosyaya yazılmıyor — doğrudan `docker run` argümanına gidiyorlar
(`tunnel.rs:841,859,888,899,915,943,987,1021,1044` → `args.push(provider.image.into())`;
`landing.rs:107`; `tunnelid.rs:459`; `perf.rs:428,482`).

Sonuç: `registryPrefix` ayarlanmış, Docker Hub'a erişimi olmayan bir kurumsal makinede
projeler ayağa kalkar, **tüneller, karşılama sayfası, tünel muhafızı ve performans
yardımcısı sessizce Docker Hub'a gitmeye çalışır ve başarısız olur.** `policy.rs`'in
muafiyet listesi (`:789`) yalnız yerel derlenen `stackvo-*` imgelerini sayıyor; bu on tanesi
orada da anılmıyor, yani muafiyet değil **atlanma**.

> **Yapılacak:** `docker run` argümanını kuran her yerde `policy::mirror(prefix, image)`.
> Fonksiyon zaten `pub` ve tam bu iş için yazılmış; bugün onu yalnız `rewrite` çağırıyor.

---

## E. Sözleşmenin bayat yarısı

`contracts/` bu deponun en güçlü fikri — ve bir yarısı ölçülmüyor.

### E-1. 43 "tüketici"nin 39'u var olmayan dosya

`env.schema.json` her anahtarın altında `consumers` listesi taşıyor. Ölçüldü: 43 farklı
yolun **39'u diskte yok**, çünkü hepsi silinmiş Bash ağacına ait:

```
core/cli/lib/generators/traefik.sh                    (4 anahtarda)
core/templates/services/mysql/docker-compose.mysql.tpl
core/ui/server/routes/supported-languages.js          (14 anahtarda)
core/ui/server/services/DockerService.js
… 35 tane daha
```

Var olan dört yol: `src-tauri/src/config.rs`, `idle.rs`, `template.rs`, `workspace.rs`.

Bunu üreten araç da çalışamıyor: `tools/measure-env-usage.mjs:34` `core/` dizini yoksa
`"No core/ under … — is that a StackVo checkout?"` deyip çıkıyor. Yani `statusLegend`'in
*"Measured by tools/measure-env-usage.mjs, not by hand — the hand-run version of this was
wrong for 12 keys"* iddiası **bu depoda ölçülemez durumda**; sayılar 2024'ün ağacında
donmuş. `validate-contracts.mjs` `consumers` alanına hiç bakmıyor.

### E-2. `SUPPORTED_LANGUAGES_RUST_DEFAULT`: sözleşme `1.62`, kod `1.84`

```
contracts/env.schema.json:494    "default": "1.62",  "status": "conflicting"
src-tauri/src/config.rs:92       ("SUPPORTED_LANGUAGES_RUST_DEFAULT", "1.84")
```

63 şema anahtarının varsayılanı `config::SETTINGS` ile karşılaştırıldı; **tek gerçek
uyuşmazlık bu** (diğer beş fark şemanın `default` alanını hiç doldurmamasından geliyor:
`*_PHP_EXTENSIONS`, `*_PHP_EXTENSIONS_DEFAULT`, `*_GO_VERSIONS`, `*_RUBY_VERSIONS`,
`*_RUST_VERSIONS`). Denetleyici bunu göremiyor çünkü yalnız *"anahtar `.env`'de var ama
şemada yok"* sorusunu soruyor, *"şemadaki varsayılan kodun varsayılanıyla aynı mı"*
sorusunu sormuyor — ve bu ikincisi tam olarak sözleşmenin var oluş nedeni.

> **Yapılacak:** `contracts:check`'e H suiti: her şema `default`'unu `config::SETTINGS`
> ile karşılaştır. `generate-types.mjs` `commands.rs`'i zaten ayrıştırıyor, yani ayrıştırıcı
> hazır. Bu tek denetim B-3, B-4 ve E-2'yi birden yakalar.

### E-3. On `SERVER_*` anahtarı (önceki raporun P2-5'i doğrulandı)

Ölçüm bu turda tekrarlandı: `contracts:check` 12 uyarının 10'unu bu aileye veriyor ve
uyarılar doğru. Ek olarak: bu on anahtar B-1'deki üç kopyanın sebebi — şema onları tanımasa
`generator.rs` ve `ServerLimitsPane.vue` varsayılanı kendileri yazmak zorunda.

---

## F. i18n: katalog düzyazısı iki dilli arayüze ham gidiyor

Şablon taraması temiz çıktı: 259 `.vue` dosyasının şablonlarında çevrilmemiş yalnız **6**
metin var, hepsi meşru (`Stack`, `GitHub`, `StackVo`, `MIT`, `/var/www/html`,
`.stackvo/context.json`). `en.js`/`tr.js` 2179 anahtarda birebir; birebir aynı kalan dört
uzun dize de doğru (`Compass, TablePlus, psql` gibi ürün adları). `hints.rs` kataloğu
çevrilmiş ve `hint_translations.rs` ile bağlanmış.

Kapatılmamış olan, **Rust kataloglarındaki düzyazı alanları**. Üretim kodunda ölçüldü:

| Modül | İngilizce cümle | GUI'ye ulaşıyor mu |
| --- | --- | --- |
| `quickcmd.rs` `about` | 26 | ✅ `ProjectDetail.vue:616`, `ReplPane.vue:168` |
| `oauth.rs` `note` | 9 | ✅ `OAuthPane.vue:117` |
| `tooling.rs` `why` | 4 | ✅ `ToolingPane.vue:329` |
| `mcp.rs` | 34 | ❌ protokol yüzeyi — İngilizce doğru |
| `cli.rs` | 40 | ❌ CLI — İngilizce (bilinçli, ama yazılı değil) |

Yani **39 cümle** iki dilli bir pencerede İngilizce görünüyor. `hints.rs`'in başlığı bu
sorunun tam olarak neden önemli olduğunu zaten yazmış: *"The suggestion is the worst one to
leave untranslated. It is the sentence that tells someone what to do."* `php artisan
migrate` satırının altındaki *"Run pending migrations."* aynı sınıftan bir cümle.

**İkinci yarısı 3.1.2.** `ErrorAlert.vue` Rust'ın yazdığı İngilizceyi `lang="en"` ile
işaretliyor ve `language-of-parts.spec.js` bunu koruyor. Yukarıdaki beş bileşende
`lang="` hiç geçmiyor — kontrol edildi, sekiz dosyanın hiçbirinde yok. Test bir **elle
yazılmış liste** (`PASSAGES`) üzerinde çalıştığı için bu boşluğu göremiyor; testin kendi
yorumu bunu kabul ediyor (*"A list rather than a sweep, and that is the honest shape"*),
ama liste bu beş yeri hiç düşünmemiş.

---

## G. Sabit değerin doğrudan sebep olduğu bir hata

### G-1. Genel Bakış her projeye `/var/www/html` diyor

```
src/components/project/OverviewPane.vue:85-92
    <span class="field-key">{{ t('projectDetail.containerPath') }}</span>
    <code class="field-mono">/var/www/html</code>
    … @click="copy('/var/www/html', 'cpath')"
```

Satırın **`v-if`'i yok** — hemen üstündeki PHP ve Node satırlarının ikisi de koşullu,
bu değil. Oysa üreteç iki farklı yol yazıyor:

```
src-tauri/src/generator.rs:388,439,529,535   WORKDIR /var/www/html     (PHP: nginx/apache/caddy/frankenphp/swoole)
src-tauri/src/generator.rs:598               WORKDIR /app              (render_node_dockerfile)
src-tauri/src/generator.rs:723               WORKDIR /app              (render_lang_dockerfile — python/go/ruby/rust/bun/deno)
```

Bir Node, Python, Go, Ruby, Rust, Bun veya Deno projesinin sayfasında "Kapsayıcı yolu"
satırı `/var/www/html` gösteriyor ve kopyala düğmesi onu panoya koyuyor. Bu yol o
kapsayıcıda **yok**. `docker exec` ile oraya `cd` etmeye çalışan kullanıcı hata alır ve
hatanın kaynağını uygulamanın kendi ekranında bulamaz.

Kök neden tam olarak bu raporun konusu: değer türetilebilirken sabit yazılmış. Manifest bu
bilgiyi taşımıyor, IPC yüzeyinde de yok — yani düzeltme `runtime`'a bakan bir `computed`
(iki satır) ya da manifest'e bir `containerPath` alanı.

`/var/www/html` üretim Rust kodunda ayrıca **63 kez** literal olarak geçiyor; adlandırılmış
bir sabiti yok.

---

## H. Önceki raporun bir maddesi yanlış

**P2-5'teki *"`api.appsAvailable()` tanımlı, hiçbir view veya store çağırmıyor — silinmeli"*
maddesi hatalı, ve öneri uygulanırsa Tercihler paneli bozulur.**

Çağrı yeri: `src/components/settings/PreferencesPane.vue:32-34`

```js
apps.value = await api
  .appsAvailable()
  .catch(() => ({ terminals: [], editors: [], browsers: [] }));
```

Uyarıyı üreten denetim `tools/validate-contracts.mjs:779`:

```js
const used = new RegExp(`\\bapi\\.${method}\\b`).test(consumers);
```

Tek satırlık bir düzenli ifade. Prettier `api` ile `.appsAvailable()` arasına satır sonu
koyduğu için eşleşme olmuyor. Yani bu bir **denetleyici hatası**, ölü kod değil — ve
yanlış yönde: var olan bir çağrıyı yok sayıyor.

> **✅ Yapıldı.** `validate-contracts.mjs`'in `UNUSED_API` düzeni artık `api` ile noktanın
> arasında boşluk ve satır sonu kabul ediyor (`api\s*\.\s*<method>`), ve neden orada
> olduğu satırın yanında yazılı. `[F] reachability` temiz; `contracts:check` 0 hata /
> **11** uyarı. Aynı düzeltme, bugün var olmayan ama yarın Prettier'ın böleceği her çağrıyı
> da koruyor. P2-5'teki madde üstü çizilerek kapatıldı.

---

## I. Bilerek yapılmış ve doğru olan sabitler

Rapor adil olsun diye: aşağıdakiler hardcode ve **öyle kalmalı**. Hiçbiri düzeltilecek bir
şey değil, ve her birinin gerekçesi kodun içinde yazılı.

- **`signing.rs:76` `PINNED`** — kayıt defterinin içerik anahtarı. `key_ceremony.rs`
  updater anahtarıyla aynı olan bir build'i reddediyor. Yapılandırılabilir olması güven
  zincirini ortadan kaldırırdı.
- **`market.rs:714` `OFFICIAL_RAW`** — resmî katalog adresi, çözülmüş biçimde yazılmış ki
  karşılaştırma kesin olsun.
- **`tauri.conf.json` `pubkey` ve `endpoints`** — updater kimliği.
- **`config.rs:295-323` yer tutucu parolalar** (`root`, `minioadmin`) — dosyanın kendisi
  bunların neden sır olmadığını açıklıyor ve `no_real_credential_is_compiled_into_the_binary`
  gerçek bir parolanın buraya yapıştırılmasını engelliyor.
- **`qr.rs:55-75`** — QR standardının kendi tabloları.
- **`ports.rs:51` `STRIDE = 10`** — okunabilirlik için seçilmiş ve gerekçesi yazılı.
- **`dns.rs:95-97` `PORT`** — platforma göre 53/15353, koşullu derleme.
- **`hosts.rs:47` `/etc/hosts`** — işletim sisteminin sözleşmesi.

Ve şu üç mekanizma bu raporda önerilen her düzeltmenin şablonu, çünkü sorunu **zaten
çözülmüş** hâlde gösteriyorlar:

| Desen | Nerede |
| --- | --- |
| Elle yazılmış ön yüz listesini Rust listesine bağlayan test | `foreign_import.rs` ↔ `IMPORT_SOURCES` |
| Katalogtan türetilen seçim listesi | `useCatalog.js` ↔ `catalog_get` |
| Katalog + çeviri anahtarı + parite testi | `hints.rs` ↔ `en.js`/`tr.js` ↔ `hint_translations.rs` |
| Slug listesi ↔ dosya paritesi (103/103, iki dilde de tam) | `help.js` ↔ `docs/help/{en,tr}` ↔ `help-topics.spec.js` |

---

## J. Önerilen sıra

**Ucuz ve bugün yapılabilir (yaklaşık yarım gün):**

1. `NGINX_DIRECTIVES[0]`/`[4]` indeks erişimini anahtar aramasıyla değiştir. *(B-1, gizli hata)*
2. `OverviewPane.vue`'nun kapsayıcı yolunu runtime'dan türet. *(G-1, kullanıcıya görünen hata)*
3. `validate-contracts.mjs:779` düzenli ifadesini çok satırlı çağrıyı görecek hale getir ve
   P2-5'ten `appsAvailable` maddesini çıkar. *(H-1)*
4. Python varsayılanını tek kaynağa indir (`3.14`/`3.13`). *(B-4)*
5. `validate-contracts.mjs:489`'daki `'8.2'` yedeğini `8.4` yap. *(B-3)*

**Yayından önce:**

6. `detected_spec`'e `Env` geçir — benimseme yolu PHP ve sunucu ayarlarını okusun. *(A-2)*
7. `docker run` imgelerini `policy::mirror`'dan geçir. *(D-2, kurumsal kurulumu bozuyor)*
8. `contracts:check`'e "şema varsayılanı = kod varsayılanı" denetimi ekle. *(E-2, ve B-3/B-4'ü de yakalar)*

**Yayından sonra:**

9. Altı `*_VERSIONS` listesine `PhpPane`'de `v-combobox`. *(A-1)*
10. Uygulama seçicilerine "diğer…" seçeneği. *(A-3)*
11. Grafik ve ısı ızgarası renklerini temadan oku. *(C-1, C-2)*
12. Şablon listelerini `scaffold.rs`'e bağlayan test — `foreign_import.rs` deseniyle. *(B-5)*
13. `quickcmd`/`oauth`/`tooling` düzyazısını `hints.rs` desenine taşı, ya da en az
    `lang="en"` ile işaretle. *(F-1)*
14. `stackvo.loc`, `stackvo-net`, `nginx` literallerini adlandırılmış sabitlere indir;
    `certs::FALLBACK_SUFFIX` zaten duruyor. *(B-2)*
15. `env.schema.json`'ın `consumers` alanını bu ağaca göre yeniden ölç ya da alanı sil —
    ölçülemeyen bir alan sözleşmenin geri kalanına olan güveni aşındırıyor. *(E-1)*
16. Tünel imgelerini sürüme sabitle; `pkg::MOVING_TAGS` kuralını uygulamanın kendisine de
    uygula. *(D-1)*

---

## K. Ölçüm özeti

| Ne | Sayı |
| --- | --- |
| Okunan/taranan kaynak dosya | 275 |
| JS/Vue satır | 47.794 |
| Rust satır | 111.843 (113 modül) |
| Şablonlarda çevrilmemiş metin | **6** (hepsi meşru) |
| `.env` anahtarı elle yazılmış ön yüz dosyası | 27 anahtar — **hepsi** `config::SETTINGS`'te var |
| Arayüz denetimi olmayan `SETTINGS` anahtarı | **7 / 36** |
| Üretimde `unwrap`/`expect` | 20 (önceki raporla aynı) |
| Şema varsayılanı ↔ kod varsayılanı uyuşmazlığı | **1** (`RUST_DEFAULT`) |
| `env.schema.json` tüketicisi: diskte yok | **39 / 43** |
| Hareketli etiketli kendi imgesi | 6 (+4 sabit ama sürümsüz) |
| Aynayı baypas eden imge | **10 / 10** |
| GUI'ye ham giden İngilizce katalog cümlesi | 39 |
| `vitest` | 84/84 dosya, 1275 test geçti |
| `contracts:check` | 0 hata / 12 uyarı (1'i yanlış pozitif — H-1) |

---

# Ek Rapor — Rakip Analizi (17 ürün) ve Yerel Geliştirme Uygulaması Olarak Eksikler

**Tarih:** 2026-08-27 (üçüncü tur)
**Taban:** `6e56a2f` (main) + çalışma ağacı
**Sorulan soru:** Bu kategorideki 17 üründe olan ya da olmayan neye bakılırsa, "bir yerel
geliştirme uygulaması olarak bunun da olması gerekir" denir?

**Yöntem.** İki taraf ayrı ayrı ölçüldü ve sonra karşılaştırıldı.

*Rakip tarafı:* 17 adresin **14'ü doğrudan okundu**. Üçü kapalı döndü ve dolaylı
kaynaklardan tamamlandı — `kettlecode.org` ve `larabox.org` HTTP 403, `forgekit.tools`
kökü yalnız başlık; bunlar için ürün dokümantasyonunun alt sayfaları
(`forgekit.tools/docs/other/faq`), karşılaştırma sayfaları ve arama sonuçları kullanıldı.
`devilbox.org` sertifika uyuşmazlığı verdi, `devilbox.readthedocs.io` okundu.
**Bu rapor rakiplerin *iddia ettiği* yüzeyi okur; hiçbirini kurup ölçmedi** — bir özelliğin
sayfada yazması onun çalıştığı anlamına gelmez ve aşağıdaki her satır o kayıtla okunmalıdır.

*StackVo tarafı:* ağaçtan ölçüldü, belgeden değil — 113 Rust modülü / 111.843 satır,
309 IPC komutu, 73 olay, 41 CLI komutu, 34 MCP aracı, 28 scaffold şablonu, 16 tespit
işareti, 31 servis paketi, 7 içe aktarma kaynağı, 9 tünel sağlayıcı, 5 web sunucusu,
8 çalışma zamanı, 26 panel.

---

## L1. Kategori haritası — 17 ürün

| # | Ürün | Motor | Platform | Lisans / fiyat | Konum |
| --- | --- | --- | --- | --- | --- |
| 1 | **Laradock** | Docker Compose, GUI yok | Linux / mac / Win | MIT | 130+ servisli "kütüphane"; 10 yıl, 450 katkıcı |
| 2 | **Laravel Herd** | Yerel ikili (konteyner yok) | Win / mac — **Linux yok** | Ücretsiz / Pro **$99·yıl** / Teams $299 | Kategori lideri, ticari |
| 3 | **Lerd** | **Rootless Podman + systemd** | Linux / mac / WSL2 | MIT | Herd'in açık kaynak yanıtı |
| 4 | **ServBay** | Yerel ikili | mac 12+ / Win 10+ | Ücretsiz / Pro **$59·yıl** / Team $399 | "AI-native"; en geniş dil yelpazesi |
| 5 | **Lando** | Docker + JS eklenti çerçevesi | Linux / mac / Win | Ücretsiz — 501(c)(3) vakıf | DevOps çerçevesi |
| 6 | **Yerd** | **~8 MB tek Rust daemon** | mac / Linux | MIT | Minimalist Valet |
| 7 | **DDEV** | Docker + Go | Linux / mac / Win / **bulut** | Apache-2.0 | Kurumsal ve CMS lideri |
| 8 | **dde** | Docker + Traefik v3 + dnsmasq | mac / Linux | AGPL-3.0 | Worktree + eklenti |
| 9 | **EnvKit** | Yerel ikili | Win 10/11, mac | Ücretsiz (public beta) | Laragon göçmeni avcısı |
| 10 | **Laragon** | Yerel, **taşınabilir** | Windows | Ücretsiz + ticari lisans (2025) | Eski hacim lideri |
| 11 | **FlyEnv** | Yerel ikili | Win / mac / Linux | Açık kaynak | Çok dilli + MCP |
| 12 | **ForgeKit** | Yerel, **kapalı kaynak** | Windows | Ücretsiz, hesapsız | Çoklu izole instance |
| 13 | **Kettle Code** | Yerel, `~/.kettle` | macOS | Tamamen ücretsiz | Apache 2.4 + **18 MCP aracı** |
| 14 | **XAMPP** | Yerel | Win / Linux / mac | Ücretsiz | **Donmuş** — PHP 8.2, 2023 |
| 15 | **Larabox** | Yerel | Windows | Ücretsiz | "Larabox Shell" + zamanlayıcı |
| 16 | **Devilbox** | Docker | Linux / mac / Win | Açık kaynak | LAMP/MEAN + intranet paneli |
| 17 | **Laraflare** | **Rust + Tauri**, yerel | Windows (mac/Linux yolda) | Ücretsiz, telemetrisiz | StackVo'nun teknoloji ikizi |

**StackVo:** Docker + Rust/Tauri, mac / Linux / Win, MIT, ücretsiz.

### Bu tablodan çıkan dört gerçek

**1. Motor tercihi kategoriyi ikiye bölmüş ve büyüyen yarı konteynersiz.** 17 üründen
**11'i** konteyner kullanmıyor; altısı kullanıyor (Laradock, Lando, DDEV, Devilbox, dde ve
— Docker değil ama konteyner — Lerd). Konteyner tarafındakilerin üçü kategorinin en
eskileri (Laradock 2015, Devilbox, Lando), biri kurumsal/CMS'e konumlanmış (DDEV), ikisi
yeni ve niş (dde, Lerd). Son üç yılda çıkan yeni ürünlerin biri hariç hepsi — Herd,
ServBay, Yerd, EnvKit, ForgeKit, Kettle Code, Larabox, Laraflare — **yerel ikili** ile
geldi ve hepsi aynı cümleyi satıyor: *"konteyner yok, VM yok, sistem kirlenmiyor."*
Tek istisna Lerd ve o da konteyneri gizlemiyor, **rootless Podman**'i ürünün başlığı
yapıyor: *"no daemon, no sudo, no system pollution."*

**2. MCP artık giriş bileti.** 17 üründen en az yedisi MCP sunucusu sevk ediyor — Lerd
(11 araç), Kettle Code (18 araç), ServBay (50+ servise erişim), EnvKit, FlyEnv, Herd
(Laravel Boost üzerinden), ve StackVo (34 araç). Bu, `mcp.rs`'in başlığındaki *"in 2026 it
is the price of entry rather than a differentiator"* cümlesini doğruluyor.
**StackVo bu alanda araç sayısıyla önde** (34 vs 18 vs 11) ve *hangi aracın hangi sözleşme
komutunu uyguladığını build'de doğrulayan* tek ürün.

**3. Ücretli özellik hep aynı şey: telemetri penceresi.** Herd Pro ($99), ServBay Pro ($59)
ve ücretsiz rakiplerin hepsinin öne çıkardığı özellik `dump()`/`dd()` yakalama, sorgu
kaydı, N+1 tespiti, iş/görünüm/olay akışı ve mail kutusu. Bu, StackVo'nun **zaten
sahip olduğu** yüzeyin büyük kısmı — ve L5-R11'de tam olarak nerede yarım kaldığı yazılı.

**4. Bir göç savaşı var ve iki büyük taban serbest.** XAMPP PHP 8.2'de donmuş (2023 sonu,
sayfası hâlâ 8.2.12 diyor); Laragon 2025'te ticarileşti. Kategorinin geri kalanı bu iki
tabanı topluyor: ForgeKit "XAMPP, WAMP veya Laragon'dan doğrudan göç", EnvKit "Laragon'dan
toplu içe aktarma", Kettle Code MAMP/Herd/ServBay karşılaştırma sayfaları,
ForgeKit `/guides/best-local-php-development-environments-windows`.
**StackVo bu savaşta en geniş silaha sahip ve savaşa girmemiş** — bkz. L5-R14.

---

## L2. StackVo'nun bugünkü yüzeyi, rakip yüzeyine karşı

### Tablo A — Temel (yerel-ikili rakipler)

| | **StackVo** | Herd | Lerd | ServBay | EnvKit | Laragon | Kettle Code |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Motor | **Docker** | yerel | Podman | yerel | yerel | yerel | yerel |
| Windows | CI'da yeşil¹ | ✅ | WSL2 | ✅ | ✅ | ✅ tek | ❌ |
| macOS | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ tek |
| Linux | ✅ | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ |
| Fiyat | **ücretsiz** | $99·yıl | ücretsiz | $59·yıl | ücretsiz | ticari | ücretsiz |
| PHP sürümleri | 5.6–8.5 (12) | 7.4–8.5 | 7.4–8.5 | 5.6–8.5 | 7.4+ | 8.2–8.4 | 8.0–8.5 |
| Site başına PHP | ✅ | ✅ | ✅ | ✅ | ✅ | profil | ✅ |
| **PHP değiştirme maliyeti** | **imaj derleme** | anında | anında | anında | anında | anında | anında |
| Varsayılan TLD | **`.loc`** | `.test` | `.test` | seçilir | `.test` | `.test` | `.test` |
| Yerel CA / HTTPS | ✅ mkcert | ✅ | ✅ mkcert | ✅ + ACME | ✅ | ✅ | ✅ mkcert |
| DNS sunucusu | ✅ `dns.rs` | ✅ | ✅ | ✅ dnsmasq | ✅ | hosts | ✅ dnsmasq |
| Taşınabilir kurulum | ❌ | ❌ | ❌ | ❌ | ❌ | ✅ | ❌ |

¹ `.github/workflows/ci.yml:24` matrisi `windows-latest` içeriyor ve iş yeşil. Ama
`README.md` hâlâ *"those blocks have never been compiled"* diyor — bkz. L5-R1.

### Tablo B — Temel (Docker tabanlı rakipler)

| | **StackVo** | DDEV | Lando | Laradock | Devilbox | dde |
| --- | --- | --- | --- | --- | --- | --- |
| GUI | ✅ masaüstü | pano | ❌ | ❌ | web paneli | ❌ |
| CLI | ✅ 41 komut | ✅ | ✅ | ❌ | ❌ | ✅ |
| TUI | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| MCP | **✅ 34 araç** | ❌ | ❌ | ❌ | ❌ | ❌ |
| Ters vekil | Traefik | nginx-proxy | nginx | nginx | nginx | **Traefik v3** |
| Servis sayısı | 31 paket | eklenti kaydı | reçete | **130+** | ~15 | 4 |
| Bulut/Codespaces | ❌ | **✅** | ❌ | ❌ | ❌ | ❌ |
| Sağlayıcı pull/push | mekanizma² | **✅ 4 reçete** | ✅ | ❌ | ❌ | ❌ |
| Eklenti/özel komut | ❌² | **✅** | **✅ JS** | ❌ | ❌ | **✅** |
| Git worktree | **✅ + DB + env** | ❌ | ❌ | ❌ | ❌ | ✅ host + TLS |
| Paket yöneticisiyle kurulum | ❌ | brew/apt/dnf | brew | git clone | git clone | brew/apt/apk/pacman |

² Mekanizma bitmiş, katalog boş — L5-R5 ve L5-R6.

### Tablo C — Geliştirici araçları

| | **StackVo** | Herd Pro | Lerd | Yerd | EnvKit | ServBay |
| --- | --- | --- | --- | --- | --- | --- |
| `dump()`/`dd()` penceresi | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| Sorgu kaydı | ✅ | ✅ | ✅ | ✅ | ✅ | — |
| N+1 tespiti | ✅ | — | ✅ | — | ✅ | — |
| **İş / kuyruk olayları** | **❌** | ✅ | ✅ | ✅ | ✅ | — |
| **Görünüm render** | **❌** | ✅ | — | ✅ | ✅ | — |
| **Giden HTTP** | **❌** | — | — | ✅ | ✅ | — |
| **Cache hit/miss** | **❌** | — | — | ✅ | — | — |
| Xdebug adım hata ayıklama | ✅ tek anahtar | ✅ | ✅ | — | ✅ | ✅ |
| Örnekleyici profilleyici (SPX) | ✅ | — | ✅ | — | — | — |
| Alev grafiği | ✅ gerçek yığın | — | ✅ | — | — | — |
| **Zaman ekseni (dump+sorgu korelasyonu)** | **✅ tek** | ❌ | ❌ | ❌ | ❌ | ❌ |
| REPL / Tinker | ✅ | Tinkerwell (ayrı ürün) | ✅ Monaco | — | — | — |
| Mail yakalayıcı | ✅ + relay | ✅ | ✅ | ✅ SMTP | ✅ Mailpit | ✅ tam sunucu |
| IDE hata ayıklama yapılandırması | ✅ yazıyor | tablo | tablo | — | — | tablo |

### Tablo D — StackVo'nun tek ya da neredeyse tek olduğu yerler

| Özellik | StackVo | Kategorinin geri kalanı |
| --- | --- | --- |
| **İmzalı servis paketi kaydı** (minisign → sha256 → sha256, anahtar rotasyonu, geri çekme) | ✅ | Hiçbiri. DDEV'in eklenti kaydı imzasız |
| **Kurumsal politika dosyası** (MDM, kilitli anahtar, registry aynası) | ✅ | Yalnız Herd Teams (lisans yönetimi, politika değil) |
| **Sırların OS keystore'una taşınması** | ✅ | Hiçbiri |
| **Worktree'ye kendi veritabanı + ortam** | ✅ | dde host+TLS veriyor, DB/env vermiyor |
| **Zaman ekseni** — dump ve sorgunun tek eksende | ✅ | Hiçbiri |
| **Devcontainer ihracı** | ✅ | Hiçbiri (DDEV Codespaces'te *koşuyor*, ihraç etmiyor) |
| **7 kaynaktan içe aktarma** | ✅ | ForgeKit 3, EnvKit 1 |
| **9 sağlayıcılı tünel + parola muhafızı + kalıcı adres** | ✅ | Herd Expose (1), Laragon ngrok (1), ServBay 3 |
| **Erişilebilirlik uyum beyanı (EN 301 549)** | ✅ | Hiçbiri |
| **İki dilli arayüz + 103 yardım konusu iki dilde** | ✅ | EnvKit "multi-language" (kapsam yazılı değil) |
| **Sözleşme ile doğrulanan IPC/MCP/CLI yüzeyi** | ✅ | Hiçbiri |
| **41 CLI + TUI + MCP + pencere: dört yüzey, tek çekirdek** | ✅ | Lerd (GUI+TUI+MCP) en yakını |

Bu sütun kısa değil ve rapor bunu abartmıyor: **kategoride hiçbir ürünün tedarik zinciri,
kurumsal dağıtım ve erişilebilirlik tarafında StackVo'ya yakın bir yüzeyi yok.** Sorun bu
sütunun boş olması değil; L5'teki on altı satırın ona rağmen açık olması.

---

## L3. Servis kataloğu — 31 pakete karşı ne var

Ölçüldü (`stackvo-service-packages/packages/`):

```
databases   cassandra clickhouse mariadb mongo mysql postgres        (6)
cache       dragonfly memcached redis valkey                          (4)
search      elasticsearch kibana meilisearch solr typesense           (5)
queue       kafka rabbitmq soketi                                     (3)
storage     minio                                                     (1)
monitoring  grafana graylog prometheus                                (3)
admin-uis   adminer kafbat mongo-express pgadmin phpcacheadmin phpmyadmin (6)
devtools    blackfire mailhog mailpit                                 (3)
```

Laradock'un 130+ servisiyle yarışmak **kasıtlı olarak reddedilmiş** ve gerekçesi
`vector_capability.rs`'de yazılı: *"Laradock's 130 services is already written down as a
fight not to have."* Bu doğru bir karar. Ama listede reddedilmemiş, sadece düşünülmemiş
olanlar var:

| Eksik | Neden önemli | Sınıf |
| --- | --- | --- |
| **MSSQL / SQL Server** | `config.rs:83` PHP uzantı listesi `sqlsrv` ve `pdo_sqlsrv` sunuyor. Bu uzantıları açan kullanıcının bağlanacağı **hiçbir servis yok**. Laradock'ta var, DDEV .NET/Umbraco için destekliyor | **İç çelişki** |
| **Beanstalkd** | Laravel'in kutudan gelen beş kuyruk sürücüsünden biri (`QUEUE_CONNECTION=beanstalkd`). `detect.rs` `QUEUE_CONNECTION` anahtarını okuyor ama beanstalkd'yi eşleyemiyor | Boşluk |
| **Varnish** | PHP/Magento/Drupal dünyasında yaygın; Laradock ve Devilbox'ta var | Boşluk |
| **OpenSearch** | Elasticsearch lisans göçünün varış noktası. `env.schema.json` adı anıyor, paket yok | Boşluk |
| **LocalStack** | AWS S3/SQS/SNS taklidi. `minio` yalnız S3'ü karşılıyor | Boşluk |
| **Keycloak** | OIDC/SAML sağlayıcı. `oauth.rs` yönlendirme URI'sini veriyor ama karşısında bir sağlayıcı yok | Boşluk |
| **NATS, Mosquitto (MQTT), CouchDB, Neo4j, Percona** | Laradock'ta var, talep dar | Düşük |

**Ollama ve Qdrant kasıtlı olarak dışarıda** ve gerekçesi ölçülmüş: `vector_capability.rs`
Apple Silicon'da konteynerli Ollama'nın **3–5× yavaş** olduğunu ve bunun kapanmayacak bir
fark olduğunu yazıyor, ve kararı Ağustos 2026'da yeniden ölçtüğünü söylüyor. Bu, bu
depodaki en iyi rekabet kaydı ve L5-R12'nin şablonu.

---

## L4. Şablon ve tespit listeleri birbirini tanımıyor

Ölçüldü: `scaffold.rs:85` **28 şablon** üretiyor, `detect.rs:215-410` **16 işaret**
tanıyor, ve iki liste arasında hiçbir bağ yok.

| | Şablon var | Tespit var |
| --- | --- | --- |
| laravel, wordpress, symfony, drupal, cakephp, codeigniter, slim | ✅ | ✅ |
| nextjs, nuxt, astro, nest | ✅ | ✅ |
| **magento, statamic** | ❌ | ✅ |
| **remix, sveltekit** | ❌ | ✅ |
| **yii, laminas, typo3, prestashop** | ✅ | ❌ |
| **django, flask, fastapi, rails, sinatra, gin, echo, rocket** | ✅ | ❌ |
| **tina, angular, vue, react, svelte** | ✅ | kısmen (`vite`) |

İki yönlü sonuç:

- Var olan bir **Yii, TYPO3, Laminas veya PrestaShop** klasörünü benimseten kullanıcı
  jenerik bir PHP projesi alıyor — oysa aynı çatı için bu uygulamada sıfırdan proje
  açabiliyor. Bu, kullanıcıya "bu araç Yii'yi bilmiyor" gibi görünüyor; biliyor.
- **Django, Rails, Flask, FastAPI, Gin, Echo, Sinatra, Rocket** için de aynısı — bunlar
  Python/Go/Ruby/Rust olarak doğru tespit ediliyor ama çatısı bilinmiyor, yani
  `manage.py`/`Gemfile`/`main.go` olan bir klasör doğru çalışma zamanını alıp yanlış
  giriş noktasıyla geliyor olabilir.

Karşılaştırma: **Lerd dokuz PHP çatısı tespit ediyor** (Laravel, Symfony, WordPress,
Drupal, Magento, CakePHP, Statamic, CodeIgniter, Tempest) — StackVo'nun PHP tespiti de tam
dokuz ve neredeyse aynı liste; fark Tempest'in olmaması ve Slim'in fazladan olması. Yani
**tespit tarafı rakiple başa baş, şablon tarafı üç katı, ve ikisi birbirinden habersiz.**

> **Yapılacak:** `Template::ALL` ile `detect::MARKERS` arasına `foreign_import.rs`
> desenindeki bağ testini koy — ama parite değil, **kapsama raporu** olarak: hangi şablonun
> tespit karşılığı yok, hangi tespitin şablonu yok. İkisinin eşit olması gerekmiyor
> (`vite` bir çatı değil); listelerin **habersiz** olmaması gerekiyor.

---

## L5. Bulgular — rakiplerde var, StackVo'da yok

Sıra önem sırasına göre; her satır ölçülmüş bir yerden geliyor.

### R-1 (P0). Windows: hacim orada, ve README hâlâ "hiç derlenmedi" diyor

17 üründen **dördü yalnız Windows** (Laragon, ForgeKit, Larabox, Laraflare — sonuncusu
mac/Linux'u "yolda" diyor), üçü Windows + macOS (Herd — kendi sayfası Windows'u birincil
hedef olarak anıyor —, ServBay, EnvKit), altısı üç platform (Laradock, Lando, DDEV,
Devilbox, FlyEnv, XAMPP). **Windows'u destekleyen 13 / 17.** Windows'u desteklemeyen
üç ürün — Yerd, dde, Kettle Code — kategorinin en küçükleri. Bu kategoride Windows
azınlık değil, **kütlenin bulunduğu yer**.

Ağaçtaki durum: `.github/workflows/ci.yml:24` matrisi `windows-latest` içeriyor, git
geçmişinde `0b1ab2d Fix the four Windows failures one run could finally show at once`
var, yani **Windows blokları artık derleniyor ve test ediliyor.**

`README.md` ise hâlâ şunu diyor:

> *"What is **not** verified: the handful of `#[cfg(target_os = "windows")]` blocks in
> `engine.rs`, `hosts.rs` and `pty.rs`. … they have never been compiled."*

İki cümle ayrışmış ve **belge kodun gerisinde**. Bu, P0-5'in (README üreteç bölümü)
aynı sınıftan ikinci örneği ve P1-2'nin (README iddia denetimi yok) doğrudan sonucu.

Ama derlenmek çalışmak değildir. Rakiplerin sattığı Windows deneyimi — hosts dosyası,
adlandırılmış boru, PowerShell yükseltme istemi, `.test` çözümlemesi — bir CI koşusunun
göremeyeceği şeyler.

> **Yapılacak:** (1) README'nin Windows paragrafını CI'nin bugünkü haline göre yeniden
> yaz — "derlenmedi" yerine "derleniyor ve birim testleri geçiyor; gerçek bir makinede
> el ile doğrulanmadı". (2) Yayın öncesi bir Windows makinesinde `preflight` → proje
> oluştur → `up` → tarayıcıda aç turunu **elle** koş ve sonucu yaz. Bu, kategorinin en
> büyük yarısına girip girmediğinin cevabı.

### R-2 (P0). Docker zorunluluğu ve onun iki görünür maliyeti

Kategorinin **11/17'si konteynersiz** ve hepsi aynı şeyi satıyor. StackVo'nun Docker
tercihi mimarinin temeli ve geri alınacak bir şey değil — ama bedeli ölçülmemiş ve
yazılmamış:

| Ne | StackVo | Yerel rakipler |
| --- | --- | --- |
| İlk kurulum | Docker Desktop + imaj çekme (GB) | tek kurucu (~100 MB) |
| Bir projenin ilk `up`'ı | imaj derleme (dakikalar) | saniyeler |
| **PHP sürümünü değiştirme** | manifest → `generator::render` → `docker compose build` | anında |
| Boşta RAM | Docker VM + Traefik + servisler | Laraflare `< 50 MB`, Laragon 4–10 MB |

**"PHP sürümünü değiştirmek yeniden derleme demek"** cümlesi bu ürünün en görünür
farkı ve hiçbir belgede yazmıyor. Rakiplerin tamamı bunu ilk özellik olarak satıyor.

**Podman desteklenmiyor.** `engine.rs:59-134` soketleri sınıflandırıyor ve Colima ile
OrbStack'i tanıyor (ikisi de Docker API uyumlu), ama Podman'ın rootless soketi
(`$XDG_RUNTIME_DIR/podman/podman.sock`) listede yok. Lerd tüm ürününü *"rootless Podman,
zero daemon"* üzerine kurmuş ve Linux kullanıcıları için bu gerçek bir talep.
Bu, önceki raporun "8. Şunlar da olsaydı" listesinde 6. madde olarak duruyor ve orada
kalması yeterli değil: Podman'ın Docker uyumlu soketi zaten var, iş bir yol eklemek ve
bir uyumluluk testi yazmak.

> **Yapılacak:** (1) `engine.rs`'in soket arama listesine Podman'ın rootless soketini
> ekle ve `daemon.rs`'in cevabı yorumlamasını Podman'a karşı bir fikstürle test et.
> (2) Yukarıdaki tabloyu README'ye koy. Bir kullanıcının bu ürünü seçme ya da seçmeme
> kararını en çok etkileyen dört satır bu ve bugün hiçbiri yazılı değil.

### R-3 (P1). Varsayılan TLD `.loc` — ayrılmış bir ad değil

Ölçüldü: `config.rs:97` `DEFAULT_TLD_SUFFIX = "stackvo.loc"`, `commands.rs:7409` benimseme
yolunda `format!("{name}.loc")`, `certs.rs:31` `FALLBACK_SUFFIX = "stackvo.loc"`.

Rakiplerin varsayılanları: Herd `.test`, Lerd `.test`, Yerd `.test`, dde `.test`,
EnvKit `.test`, Laragon `.test`, Kettle Code `.test`, Larabox `.test`, Laraflare `.test`,
ForgeKit `.test`/`.local`/`.localhost`, DDEV `.ddev.site` (gerçek bir alan adı,
127.0.0.1'e çözümleniyor).

**`.test` RFC 6761'de özel amaçlı üst düzey alan adı olarak ayrılmıştır ve asla delege
edilmeyecektir.** `.loc` ayrılmamıştır. Bugünkü sonucu iki tane:

- Bir StackVo alan adı çözümlenemediğinde (hosts satırı yok, DNS sunucusu kapalı) sorgu
  **yukarı çıkar** ve kullanıcının ISS'sinin çözümleyicisine `shop.loc` diye bir soru
  gider. `.test` bunu tanımı gereği yapmaz.
- ICANN `.loc`'u bir gün delege ederse her StackVo alan adı gerçek bir siteyle çakışır.

`env.schema.json` `DEFAULT_TLD_SUFFIX` için *"Project domains do NOT use it"* diyor, yani
proje alan adları `<ad>.loc` biçiminde ve bu anahtardan bağımsız — dolayısıyla ayarı
`.test` yapan bir kullanıcı bile projelerinde `.loc` alıyor. Bu, A-2'nin (kabul yolu
ayarları okumuyor) daha geniş hali.

> **Yapılacak:** varsayılanı `.test`e çevir; var olan kurulumlarda `.loc`'u koru (bir
> alan adını değiştirmek herkesin yer imlerini bozar) ve `doctor`'a bir satır ekle.
> Yeni kurulumun `.test` alması geriye dönük uyumluluk sorunu değil.

### R-4 (P1). Şablon ve tespit listeleri habersiz

Tam ölçüm L4'te. Özet: 28 şablon, 16 işaret, sıfır bağ; dört PHP çatısı ve sekiz
PHP-dışı çatı yalnız bir yönde biliniyor.

### R-5 (P1). Sağlayıcı mekanizması bitmiş, katalog boş

`provider.rs` DDEV'in `pull`/`push`'unu **daha sıkı** bir tehdit modeliyle uygulamış:
argv dizisi (kabuk yok), yalnız konteyner (host varyantı yok ve olmayacak), yön başına
onam, keystore'dan gelen sırlar, sembolik bağ reddi, `push` denetim kaydına yazılıyor.
Bu tasarım DDEV'inkinden iyi ve modülün başlığı bunu gerekçesiyle yazmış.

Ama reçeteler **projenin kendi `stackvo.json`'ından** okunuyor (`provider.rs:242` —
`json.get("providers")`) ve depoda **sevk edilen tek bir reçete yok.** DDEV'de
`.ddev/providers` her projede Upsun, Acquia ve Lagoon reçeteleriyle **hazır geliyor**;
Pantheon, git, rsync ve yerel dosya için örnekler var.

Sonuç: StackVo'da bu özelliği kullanmak için önce reçete formatını öğrenip elle yazmak
gerekiyor. *"Kategorinin bulduğu en büyük boşluk"* diye nitelenen özellik, kullanıcının
karşısına boş bir alanla çıkıyor.

> **Yapılacak:** üç reçeteyi sevk et — SSH+mysqldump (en yaygın, hiçbir sağlayıcıya
> bağlı değil), Upsun ve Pantheon. Servis paketi kaydı zaten imzalı dağıtım için var;
> reçeteler de oradan gelebilir, bu da katalogun ikinci içerik türü olur.

### R-6 (P1). Kullanıcı ve paket düzeyinde komut/eklenti yüzeyi yok

Karşılaştırma:

| Ürün | Kullanıcı kendi komutunu nasıl ekler |
| --- | --- |
| **DDEV** | `.ddev/commands/{host,web}/<ad>` — dosya bırak, komut olsun; ayrıca eklenti kaydı |
| **Lando** | `.lando.yml` `tooling:` bloğu + JS eklenti çerçevesi |
| **dde** | proje-yerel ve global eklentiler, özel komutlar |
| **Laragon** | `Procfile` ile herhangi bir servis (MeiliSearch örneği), oto-başlatma |
| **StackVo** | Proje `stackvo.json` → `commands` ✅ … ve başka hiçbir yol |

StackVo'nun proje-başına `commands` alanı doğru tasarlanmış ve README'de gerekçesi yazılı
(*"they run in the project's container and nowhere else"*). Kapalı olan iki yüzey:

- **Makine geneli komut yok.** `quickcmd.rs:119` — *"Adding a row here is the only way to
  add a command."* Bu, önceki raporun A-4'ü ve orada "bilinçli ama yazılı değil" olarak
  sınıflandırılmış. Rekabet bağlamında sınıf değişiyor: dört rakip bunu **ilk sırada**
  satıyor.
- **Bir paket komut getiremiyor.** Servis paketi kaydı bir compose parçası ve
  yapılandırma taşıyabiliyor; bir komut taşıyamıyor. DDEV'in eklenti kaydının cazibesinin
  yarısı bu — `ddev-redis` sadece Redis'i değil, `ddev redis-cli`'yi de getiriyor.

Güvenlik gerekçesi geçerli ve terk edilmemeli: kabuk dizgesi yok, argv var. Ama
`hooks.rs` ve `cron.rs` kullanıcının yazdığı komutları **zaten çalıştırıyor**, yani yasak
kataloğun kapalılığından geliyor, argv kuralından değil.

> **Yapılacak:** (1) `<root>/commands.json` — makine geneli, `stackvo.json`'ın `commands`
> şemasının aynısı, aynı argv kuralı, aynı konteyner sınırı. Yeni bir tehdit modeli
> gerekmiyor, var olan iki tanenin birleşimi. (2) Paket manifestine `commands` alanı,
> aynı imza zincirinden geçerek.

### R-7 (P2). Servis kataloğunda dört gerçek eksik

Tam ölçüm L3'te. Öncelikli: **MSSQL** (PHP uzantı listesi `sqlsrv` sunuyor, bağlanacak
servis yok — iç çelişki), **Beanstalkd** (Laravel'in kutudan gelen kuyruk sürücüsü,
`detect.rs` `QUEUE_CONNECTION`'ı okuyor ama eşleyemiyor), **OpenSearch**, **Varnish**.

### R-8 (P2). Telemetri penceresi yarım — ve alan zaten açılmış

`debugbridge.rs:144` `Event` bir `kind` alanı taşıyor ve `timeline.rs:12` o alanın doğuş
gerekçesini alıntılıyor: *"so queries and jobs do not need a second file and a second
reader when they arrive."*

Ölçüldü: üretim kodunda `kind` alanına yazılan tek değer **`"dump"`**
(`timeline.rs:235`, `debugbridge.rs:727` — köprü yalnız `dump`, `dd` ve
`__stackvo_emit`'i sarıyor). Yani alan hazır, mekanizma hazır, tek olay türü var.

Rakiplerin akıttığı, StackVo'nun akıtmadığı: **kuyruk işleri** (Herd Pro, Lerd, Yerd,
EnvKit), **görünüm render** (Herd Pro, Yerd, EnvKit), **giden HTTP istekleri** (Yerd,
EnvKit), **cache hit/miss** (Yerd), **olay gönderimi** (EnvKit).

Bunlar Herd Pro'nun $99'unun ve EnvKit'in tüm konumlanmasının içeriği. StackVo'da
`worker.rs` kuyruk **işçisini** yönetiyor ve `SchedulerPane` zamanlanmış işleri
gösteriyor — ama bir işin *ne zaman alındığı, ne kadar sürdüğü, başarısız olup olmadığı*
akmıyor.

> **Yapılacak:** `__stackvo_emit`'in yanına Laravel için dört olay dinleyicisi
> (`JobProcessed`, `QueryExecuted` zaten var, `RequestHandled`, `MessageSent`) —
> `kind` alanı ve `timeline.rs`'in ekseni ikisi de bunu bekliyor. Bu, tek bir modülde
> ücretli rakiplerin ana satış kalemini kapatıyor.

### R-9 (P2). Paket yöneticisiyle kurulum yok

`tools/check-installers.mjs:64-69` altı biçim tanıyor: `deb`, `rpm`, `AppImage`, `dmg`,
`msi`, `nsis`. Hepsi doğrudan indirme.

| Rakip | Kurulum yolu |
| --- | --- |
| dde | `brew`, `apt`, `apk`, `pacman` |
| DDEV | `brew`, `apt`, `yum`/`dnf`, WSL2 |
| Lando | `brew` |
| StackVo | **yalnız indirme** |

Homebrew cask ve winget, ikisi de bir manifest PR'ı ve otomatik güncellenebiliyor.
Kategoride kurulumun kolaylığı satılan ilk şey ("one liner install" — Lando'nun kendi
ifadesi).

> **Yapılacak:** yayın etiketlendikten sonra Homebrew cask + winget manifesti. `release.yml`
> zaten altı hedefin sha256'sını üretiyor, yani girdi hazır.

### R-10 (P2). Java, .NET ve iki web sunucusu yok — ve karar yazılı değil

| | StackVo | ServBay | FlyEnv | Laradock |
| --- | --- | --- | --- | --- |
| Çalışma zamanları | php, node, python, go, ruby, rust, bun, deno (8) | + **Java 7–24**, **.NET 2.0–10**, Mono | + Java | + Java (Tomcat) |
| Web sunucuları | nginx, apache, caddy, frankenphp, swoole (5) | + Tomcat | + Tomcat | + **OpenResty**, **RoadRunner**, Tomcat |

İki not:

- **RoadRunner özellikle eksik.** Laravel Octane'ın iki sürücüsü Swoole ve RoadRunner;
  StackVo Swoole'u destekliyor, RoadRunner'ı desteklemiyor. Bu bir çiftin yarısı ve
  Octane kullanan bir Laravel projesi için ikili bir seçim.
- **Java/.NET bir konum sorusu.** StackVo kendini "PHP ve Node projeleri" olarak
  tanımlıyor (ARCHITECTURE.md §1) ama sekiz çalışma zamanı üretiyor. Sekiz mi, on mu
  olacağı bir karar; kararın **yazılı olmaması** sorun. `manifest::LANG_RUNTIMES`'in
  başlığı hangi çalışma zamanının neden listede olmadığını söylemiyor.

### R-11 (P2). Takım paylaşımı yarım kalıyor

| Ürün | Takıma ne gidiyor |
| --- | --- |
| DDEV | `.ddev/config.yaml` repoda — klonlayan aynı ortamı alıyor |
| Lando | `.lando.yml` repoda |
| Herd | `herd.yml` repoda (Pro) |
| dde | proje dosyası repoda |
| **StackVo** | `stackvo.json` repoda ✅ **+** `preset` — ama preset repoda değil |

`preset.rs` doğru sorunu bulmuş ve gerekçesi kusursuz: klonlayan `stackvo.json`'ı zaten
alıyor, almadığı şey **hangi servislerin hangi sürümde açık olduğu**, çünkü o `.env`'de.
`preset::save` bunu bir dosyaya yazıyor ve `apply_file` uyguluyor.

Eksik olan tek şey **yerleşim kuralı**: preset dosyasının nereye konacağı ve klonlayanın
onu nasıl bulacağı hiçbir yerde yazmıyor. DDEV'de bu soru yok — dosya `.ddev/` içinde ve
`ddev start` onu okuyor. StackVo'da kullanıcı dosyayı dışa aktarıyor, bir yere koyuyor,
takım arkadaşı buluyorsa açıyor.

> **Yapılacak:** kurallı bir yol — `<proje>/stackvo.preset.json`, ve proje açılırken
> uygulanmamış bir preset varsa bir satırlık bildirim. `MigrationGate`/`CatalogueGate`
> deseni zaten ağaçta.

### R-12 (P3). Koddaki üç rekabet iddiası bayat

Bu depo iddialarını testle koruyor — ama bir dış dünya iddiası test edilemez, tarihlenir.
`vector_capability.rs` bunu doğru yapıyor: *"Re-measured August 2026, against a
competitive review that listed Ollama in Laradock, ServBay and FlyEnv."* Diğer üçü
tarihsiz ve üçü de artık yanlış:

| Yer | İddia | Bugün |
| --- | --- | --- |
| `worktree.rs:9` | *"the one thing in the competitive review that nothing else in this space does"* | **dde** her worktree'ye kendi hostname'i ve TLS sertifikasını veriyor. StackVo'nunki hâlâ daha ileri (kendi veritabanı + ortam değişkenleri) ama "nothing else" yanlış |
| `imports.rs:3` | *"**Two** of them, and the reason is a window rather than a feature list"* | `ALL` **yedi** kaynak taşıyor: Xampp, Laragon, Mamp, Valet, Sail, Herd, **Ddev** |
| `mcp.rs:3` | *"**Five of the eight** competitors ship one"* | Bu turda ölçülen 17 üründen en az yedisi MCP sevk ediyor; "sekiz rakip" tabanı da artık dar |

> **Yapılacak:** üçünü de `vector_capability.rs` desenine getir — iddiayı **tarihle**
> yaz ("Ağustos 2026'da ölçüldü: …"). Tarihli bir iddia eskidiğinde yanlış olmaz,
> *eski* olur; tarihsiz olan sessizce yanlışa döner. Bu, §7a'nın "gerekçe kodun yanında
> yaşar" kararının doğal uzantısı.

### R-13 (P3). Göç savaşı kazanılabilir durumda ve görünmüyor

StackVo **yedi kaynaktan** içe aktarıyor — kategorinin en genişi. En yakını ForgeKit (3),
sonra EnvKit (1, yalnız Laragon). Herd yalnız kılavuz yayınlıyor.

Ve `imports.rs`'in tasarımı rakiplerinkinden dikkatli: kopyalama varsayılan, taşıma
seçenek, **diğer kuruluma tek bayt yazılmıyor** — modülün kendi ifadesiyle *"EnvKit takes
Laragon out of PATH as part of importing it; that is a decision about somebody else's
machine made on their behalf, and it is exactly what this module does not do."*

README'de bu özellikten **tek satır yok**. Karşılaştırma sayfası yok. Ekran görüntüsü yok.
Rakiplerin hepsinde var: Kettle Code'un `/compare/servbay`, `/compare/mamp`,
`/compare/laravel-herd` sayfaları; ForgeKit'in `/guides/best-local-php-development-
environments-windows`'u; EnvKit'in Herd/Laragon/AppServ/XAMPP karşılaştırma tablosu.

Bu bir pazarlama notu değil, **keşfedilebilirlik** notu: bu kategoride kullanıcı ürünü
karşılaştırma sayfasından buluyor ve XAMPP'tan/Laragon'dan çıkmak isteyen kişi tam olarak
bunu arıyor. P0-4 (README bir son kullanıcıya hitap etmiyor) bu maddeyle birleşiyor.

### R-14 (P3). Bulut/uzak geliştirme ortamı desteklenmiyor

DDEV **GitHub Codespaces** ve Gitpod içinde çalışıyor, devcontainer desteğiyle geliyor ve
bunu ana sayfasında "cloud environments" olarak sayıyor.

StackVo devcontainer **ihraç ediyor** (`devcontainer.rs`) — bu, DDEV'in yapmadığı bir şey
ve doğru yönde. Ama StackVo'nun kendisi bir masaüstü uygulaması: bir Codespace içinde
çalışmıyor ve çalışamaz. `websurface.rs` (loopback HTTP yüzeyi) ve `cli.rs` teoride
başsız bir kullanımı mümkün kılıyor, ama bu bir ürün olarak konumlanmamış.

Bu bir eksik değil, bir **sınır** — ve sınırın yazılı olması gerekiyor, çünkü kurumsal
alıcının ilk sorularından biri.

### R-15 (P3). Sürdürülebilirlik modeli yazılı değil

| Ürün | Model |
| --- | --- |
| Herd | Pro $99·yıl, Teams $299 |
| ServBay | Pro $59·yıl, Team $399·yıl |
| Laragon | 2025'te ticarileşti |
| DDEV | Sponsorluk — "iki tam zamanlı bakımcıyı finanse ediyor", 100+ kurumsal sponsor |
| Lando | 501(c)(3) kâr amacı gütmeyen kuruluş |
| Laradock | Sponsorluk ($478/ay, hedef $1.000) |
| **StackVo** | MIT, ücretsiz — **model yazılı değil** |

Ücretsiz olmak bir avantaj ve bu raporun onu küçültmesi gerekmiyor. Ama DDEV ve Lando'nun
sponsorluk/vakıf cümlelerini ana sayfalarına koymalarının sebebi var: kurumsal bir alıcı
"bu proje iki yıl sonra duruyor mu" diye soruyor ve cevabı olmayan projeyi almıyor.
P4'ün (`SUPPORT.md`, `CODE_OF_CONDUCT.md`) eksik komşusu bu.

### R-16 (P4). Taşınabilir kurulum yok — ve olamaz

Laragon (tek klasör, kayıt defteri kaydı yok, kopyalayıp taşı), ForgeKit (her şey kendi
dizininde), Laraflare (portable .zip), XAMPP. Bu, Windows tarafında satılan gerçek bir
özellik.

Docker mimarisinde imkânsız: imajlar ve volume'ler Docker'ın kendi deposunda. `STACKVO_ROOT`
yarı bir cevap (workspace taşınabilir, motor değil).

Bilinçli ve doğru bir sınır; **yazılı olması** gereken bir sınır.

---

## L6. Rakiplerin sattığı, StackVo'nun ölçüp reddettiği

Rapor adil olsun diye: aşağıdakiler eksik değil, **verilmiş karar** — ve her birinin
gerekçesi kodun içinde, ölçümle birlikte yazılı.

| Ne | Nerede reddedildi | Gerekçe |
| --- | --- | --- |
| **Ollama / yerel LLM** | `tests/vector_capability.rs:16-29` | Docker Desktop macOS'ta Apple GPU'yu konteynere geçiremiyor; konteynerli Ollama Apple Silicon'da **3–5× yavaş**. "İstenen paket hiçbir şey yapmamaktan ölçülebilir biçimde kötü olurdu." M1'den M5'e kadar geçerli, Ağustos 2026'da yeniden ölçüldü |
| **Qdrant** | aynı dosya | Zaten dört veritabanı var; beşincisi bir kategori açar |
| **pgvector ayrı servis olarak** | aynı dosya | PostgreSQL'in kendisi — aynı protokol, port, volume, istemci. **Sürüm** olarak çözüldü (`16-pgvector`), servis olarak değil |
| **Laradock'un 130 servisi** | aynı dosya | "a fight not to have" |
| **Mutagen paketleme** | `perf.rs:15` | DDEV'in yaptığı; ikinci bir ikili ve ikinci bir hata yüzeyi |
| **Sağlayıcı reçetesine ssh-agent** | `provider.rs:33-39` | DDEV geliştiricinin ssh agent'ını konteynere bağlıyor. Burada kural: "depo tarafından tanımlanmış bir konteyner host yolu almaz, ve bir ssh agent imza atan bir host yoludur." **Not:** projenin *kendi* konteyneri için ssh agent iletimi **var** (`site.rs:184-263`) ve macOS/Windows'ta VM sorununu doğru çözüyor. Reddedilen yalnız sağlayıcı reçetesi |
| **Serbest metin editör/terminal kutusu** | `apps.rs` başlığı | Tespit daha iyi — ama "diğer…" seçeneği konmamış (A-3, hâlâ açık) |

Bu tablo, önceki iki raporun "I. Bilerek yapılmış ve doğru olan sabitler" bölümünün
rekabet karşılığı ve aynı işi görüyor: **bir eksik ile bir karar arasındaki fark, kararın
yazılı olmasıdır.**

---

## L7. Ölçüm özeti

| Ne | Sayı |
| --- | --- |
| İncelenen rakip ürün | 17 |
| Doğrudan okunan site | 14 (3'ü 403/boş → doküman ve arama ile tamamlandı) |
| Konteynersiz rakip | **11 / 17** (konteynerli 6, biri Podman) |
| Windows'u destekleyen rakip | **13 / 17** (dördü yalnız Windows) |
| MCP sevk eden rakip | ≥ 7 |
| Ücretli katman taşıyan rakip | 3 (Herd, ServBay, Laragon) |
| StackVo'nun tek olduğu ölçülen özellik | 12 (Tablo D) |
| **Rakipte var, StackVo'da yok** | **16 bulgu** (R-1 … R-16) |
| Bunların P0'ı | 2 (Windows doğrulaması, Docker maliyetinin yazılmaması) |
| StackVo'nun ölçüp reddettiği | 7 (L6) |
| Bayat rekabet iddiası (kodda) | **3** (`worktree.rs`, `imports.rs`, `mcp.rs`) |
| Şablon (28) ↔ tespit (16) arasında bağ | **0** |
| Servis paketi | 31 |
| Katalogda olmayan ve gerçekten istenen servis | 4 (MSSQL, Beanstalkd, OpenSearch, Varnish) |
| Sevk edilen sağlayıcı reçetesi | **0** (DDEV: 4 + 4 örnek) |
| `debugbridge` olay türü — tanımlı alan / kullanılan | 1 (`"dump"`) |

---

## L8. Önerilen sıra

**Yayından önce (rekabet bağlamında P0):**

1. **Windows'u gerçek bir makinede elden geçir** ve README'nin "hiç derlenmedi"
   paragrafını CI'nin bugünkü haline göre yeniden yaz. *(R-1 — kategorinin 13/17'si orada)*
2. **Docker'ın dört maliyetini README'ye yaz** — kurulum boyutu, ilk `up`, PHP sürümü
   değiştirme, boşta RAM. Bir kullanıcının bu ürünü seçme kararını en çok etkileyen
   satırlar ve hiçbiri yazılı değil. *(R-2)*
3. **Varsayılan TLD'yi `.test` yap** (var olan kurulumları koruyarak). Bir satırlık
   varsayılan değişikliği, gerçek bir DNS sızıntısını kapatıyor. *(R-3)*
4. **Yedi kaynaktan içe aktarmayı README'nin görünür yerine koy** ve bir karşılaştırma
   sayfası aç. Kategorinin en geniş göç silahı bugün belgede yok. *(R-13, P0-4 ile birlikte)*
5. **Koddaki üç bayat rekabet iddiasını tarihle.** `vector_capability.rs` deseni.
   *(R-12)*

**Yayından hemen sonra:**

6. Şablon ↔ tespit kapsama testi; Magento ve Statamic'e şablon, Yii/TYPO3/Laminas/
   PrestaShop'a tespit. *(R-4)*
7. Üç sağlayıcı reçetesi sevk et (SSH+mysqldump, Upsun, Pantheon). *(R-5)*
8. `debugbridge`'e kuyruk işi ve istek olayları — `kind` alanı ve zaman ekseni ikisi de
   bekliyor; ücretli rakiplerin ana satış kalemi. *(R-8)*
9. Homebrew cask + winget manifesti. *(R-9)*
10. MSSQL ve Beanstalkd paketleri; MSSQL'inki iç çelişkiyi de kapatıyor. *(R-7)*

**Sonra:**

11. `<root>/commands.json` — makine geneli komut, aynı argv kuralı; ve paket manifestine
    `commands` alanı. *(R-6)*
12. Podman soketi + uyumluluk testi. *(R-2 ikinci yarısı)*
13. `<proje>/stackvo.preset.json` yerleşim kuralı. *(R-11)*
14. RoadRunner sürücüsü — Octane'ın ikinci yarısı. *(R-10)*
15. Sürdürülebilirlik cümlesi (`SUPPORT.md` ile birlikte) ve taşınabilirlik/bulut
    sınırlarının yazılması. *(R-15, R-14, R-16)*

---

## L9. Tek cümlelik hüküm

Bu ürün, rakiplerinin hiçbirinde bulunmayan bir tedarik zinciri, kurumsal dağıtım ve
erişilebilirlik yüzeyi taşıyor ve kategorinin en geniş göç, tünel ve MCP silahlarına
sahip — onu bugün rakiplerinin gerisine düşüren şey bir özellik açığı değil,
**kategorinin kütlesinin bulunduğu platformda elden doğrulanmamış olması, Docker
tercihinin bedelinin hiçbir yerde yazılı olmaması ve elindeki en güçlü on iki özelliğin
kullanıcının göreceği tek bir belgede sayılmaması** — üçü de kod yazmayı değil, söylemeyi
gerektiren işler.

---

# Ek Rapor — Artı Kazandıracak Özellikler: Rakiplerde Olmayan ve Bu Mimarinin Ucuza Çıkardığı İşler

**Tarih:** 2026-08-27 (dördüncü tur)
**Taban:** `6e56a2f` (main)
**Sorulan soru:** Rakiplerde *olmayan*, bu ürüne artı kazandıracak hangi özellikler var?

---

## M0. Bu raporun filtresi ve stratejik çerçevesi

Bir özellik fikri ucuzdur; işe yarayanı bulmak değil. Aşağıdaki her madde **üç kapıdan**
geçti ve geçemeyen yazılmadı:

| Kapı | Ne soruyor |
| --- | --- |
| **Rakip kanıtı** | 17 üründen hiçbirinde var mı? Nasıl doğrulandı? |
| **Ağaç kanıtı** | Bunu ucuza çıkaran mekanizma **bugün hangi dosyada** duruyor? |
| **Hendek** | Rakip bunu kopyalayabilir mi, yoksa **mimarisi mi engelliyor**? |

Üçüncü kapı bu raporun asıl fikri. Önceki tur şunu ölçtü: kategorinin 11/17'si
konteynersiz ve hepsi aynı cümleyi satıyor. Buna verilecek doğru cevap **konteynersiz
olmak değil** — bunu bu depo zaten yazmış. `release.rs`'in başlığı, üretim imajı özelliğini
anlatırken:

> *"the container lineage makes it possible and **no native-binary competitor can
> follow**."*

Bu cümle bir modül notu olarak yazılmış; bu rapor onu bir **stratejiye** çeviriyor.
Docker bir maliyet (R-2) ve o maliyet yazılmalı — ama bir maliyet ancak karşılığında
alınamayacak bir şey varsa savunulabilir. Aşağıdaki sekiz maddeyi Herd, ServBay, EnvKit,
Laragon, Yerd, Kettle Code, Larabox, Laraflare, ForgeKit ve FlyEnv **isteseler de
yapamazlar**, çünkü PHP'yi host'ta çalıştırıyorlar: paylaşılan bir dosya sistemi, tek bir
MySQL instance'ı ve izole edilemeyen bir süreç ağacı.

---

## M1. Dış dünyanın 2026'da nerede olduğu — beş ölçüm

**1. AI ajan izolasyonu kategorinin çözülmemiş problemi ve pazarı bulutta.** 2026 başında
Cloudflare, Vercel, Ramp ve Modal sandbox özelliği çıkardı; tam geliştirme ortamı için
E2B ve Daytona microVM satıyor. Şikayet **fiyat**: bazı ekipler sandbox'a tüm hesaplama
bütçesinden fazla harcıyor. Sektörün kendi ifadesi: *"isolation is the unsolved problem
when using AI agents — everyone focuses on making agents smarter, few on making them safe
to run."*
**Yerel, ücretsiz, tek tık bir ajan kum havuzu yok. Hiçbir yerde.**

**2. Kategori lideri AI stratejisi olmadığını yazıyor.** DDEV'in Şubat 2026 bülteni %72
Drupal pazar payını duyuruyor; 2026 plan yazısı ise şunu diyor: *"considering a general AI
strategy for DDEV users."* Yani **düşünüyor.** Aynı plan iki şey daha içeriyor ve bunlar
kapanan pencereler: **mkcert CA yerine gerçek sertifikalar** ve **ek servisler için ayrı
port yerine alt alan adları**.

**3. Dal başına ortam bir bulut kategorisi ve kimse memnun değil.** Preview environment
araçları (Preevy, Okteto, Northflank, Qovery) Kubernetes ya da VPS istiyor. Port'un 2025
anketi: geliştiricilerin **%6'sı** ortam açma araçlarından memnun. **Yerel karşılığı yok
— ve StackVo'nun `worktree.rs`'i tam olarak odur.**

**4. Bir numaralı şikayet hâlâ aynı ve hiçbir ürün ölçmüyor.** "Bende çalışıyor" ve
onboarding: node sürümü uyuşmazlığı, PostgreSQL çalışmıyor, Redis bağlantı reddi.
Konteyner bunu *azaltıyor* ama DDEV/Lando dahil hiçbiri **iki makinenin farkını
gösteremiyor.**

**5. İstek tekrarı / geriye doğru hata ayıklama bu kategoride hiç yok.** PHP'de bunu
yapan tek şey `dontbug` adlı bağımsız bir ters hata ayıklayıcı. Herd, DDEV, Lando ve
diğer on dördünün hiçbirinde yok; hepsi ileri adımlayan Xdebug'da duruyor.

---

## M2. Hendek haritası

| Sınıf | Neye dayanıyor | Rakip neden kopyalayamaz | Fikir |
| --- | --- | --- | --- |
| **K — Kale** | Konteyner soyağacı | Yerel ikilide izolasyon yok: paylaşılan FS, tek MySQL, izole edilemeyen süreç ağacı | 8 |
| **Z — Zincir** | İmzalı kayıt + politika + keystore | Hiçbirinde imza zinciri, MDM katmanı ya da keystore yok | 3 |
| **S — Sözleşme** | `contracts/ipc.json` + MCP çapraz denetimi | MCP'leri var, hangi aracın hangi komutu uyguladığını doğrulayan gate yok | 4 |
| **U — Ucuz ve tek** | Var olan modüllerin birleşimi | Kopyalanabilir — ama bugün kimsede yok, ilk olan alır | 6 |

---

## M3. Kale sınıfı — konteyner soyağacının mümkün kıldığı

### K-1. Ajan kum havuzu ⭐ *(en yüksek etki)*

**Ne.** Bir AI ajanına "şu hatayı düzelt" dendiğinde migration koşuyor, tablo düşürüyor,
paket kuruyor, `.env` yazıyor. Bugün bunların hepsi geliştiricinin **canlı veritabanında**
oluyor. `stackvo sandbox <ad>` bunun yerine şunu verir: kendi dizini, kendi hostname'i,
**ana veritabanının bir kopyası**, kendi ortam değişkenleri, ve o worktree'ye kapsanmış
bir MCP yüzeyi. Ajan ana veritabanını göremez — konteynerinde yok. Başarılıysa çıktı bir
`git diff`; değilse `worktree_remove`.

**Rakip.** Bulut sandbox'ları var (Cloudflare, Vercel, Modal, E2B, Daytona) ve hepsi
ücretli ve uzak. Bu kategorideki 17 üründen **hiçbirinde** ajan izolasyonu yok — Lerd 11
araç, Kettle Code 18 araç, ServBay 50+ servis veriyor ve **hepsi canlı ortam üzerinde
çalışıyor.**

**Ağaçta zaten var — bu maddenin tamamı bir fiil eksikliği:**

| Parça | Nerede |
| --- | --- |
| Dal → kendi dizin + hostname + DB + env | `worktree.rs` (`worktree_create`, `worktree_env_set`, 6 IPC komutu) |
| Veritabanı kopyalama | `db.rs:1115` `copy_database`, `db.rs:983` `create_database` |
| Geri dönüş noktası | `snapshot.rs` (`db_snapshot_take`/`_restore`) |
| Konteyner içindeki ajana bağlam | `agentctx.rs` |
| Host yolu yok / host portu yok kuralı | `sidecar.rs` |
| Tek özne tek işlem | `inflight.rs` |
| Geri alınamayanın kaydı | `audit.rs` |

**Hendek.** Yerel ikili rakipte PHP host'ta koşuyor, dosya sistemi paylaşılıyor ve MySQL
tek instance. Ajanı bundan izole etmenin yolu yok. **Kopyalanamaz.**

**Maliyet.** Yeni modül yok. `worktree::create` + `db::copy_database` + bir TTL + MCP
kapsamı. `worktree.rs` zaten `prune`'u (`:851`) taşıyor.

> Bu tek özellik ürünün konumunu değiştirir: *"yerel geliştirme ortamı"*dan
> *"ajanların güvenle koşabildiği yerel geliştirme ortamı"*na. 2026'da ikincisinin
> yerelde sahibi yok.

### K-2. Ortam kilit dosyası — `stackvo.lock` ⭐

**Ne.** Bugün bir depoyu klonlayan `stackvo.json`'ı alıyor: `php: 8.4`, `mysql: 8.0`.
Yarın `8.4` demek `8.4.7`; geçen ay `8.4.3` demekti. Kilit dosyası **imaj digest'lerini,
uzantı sürümlerini, servis paketi hash'lerini** dondurur: `stackvo up --locked` iki
makinede **byte olarak aynı** konteyneri kurar.

**Rakip.** Hiçbirinde yok. DDEV'in satış cümlesi *"your dev setup committed to Git"* —
ama commit edilen bir **yapılandırma**, kilit değil. Lando `.lando.yml`, Herd `herd.yml`
aynı sınıf. Kategoride "npm ci" karşılığı **yok**.

**Ağaçta zaten var — ve bu maddenin en zor yarısı çoktan bitmiş:**

| Parça | Nerede |
| --- | --- |
| Üreteç belirlenimci ve **byte-for-byte doğrulanmış** | `tests/fixtures_differential.rs`, `npm run diagnose` |
| Hareketli etiket yasağı | `pkg.rs:50` `MOVING_TAGS` |
| Manifest başına + dosya başına sha256 | `market.rs`, `pkg::verify` |
| İmzalı indeks → manifest → dosya zinciri | `signing.rs` `PINNED` |
| Paylaşılabilir stack yarısı | `preset.rs` (`SHAREABLE` allow-list) |

**Hendek.** Yerel ikilide kilitlenecek bir şey yok: host'a kurulmuş PHP'nin digest'i
yoktur ve kullanıcı onu `brew upgrade` ile değiştirebilir. **Kopyalanamaz.**

**Maliyet.** Manifest'e digest alanları + `stackvo lock` + `up --locked`. Doğrulama
mekanizması (`pkg::verify`) aynen kullanılıyor.

> Bu, M1'in dördüncü ölçümünün — onboarding ve "bende çalışıyor" — kategorideki **tek
> gerçek cevabı**. Ve D-1'i (kendi imgelerinin altısı `:latest`) kapatmayı zorunlu kılıyor,
> yani iki bulguyu birden kapatıyor.

### K-3. Ortam farkı — "bende çalışıyor"un ölçülmüş cevabı

**Ne.** İki geliştiricinin teşhis paketini karşılaştır ve farkı say: *"sende Docker 27,
onda 25; senin `php.ini`'n `memory_limit`'i ezmiş; redis digest'iniz farklı; onda
`SERVICE_REDIS_ENABLE=false`."*

**Rakip.** Hiçbiri. Bu, kategorinin en eski şikayeti ve **hiçbir ürün ölçmüyor** — hepsi
"konteyner bunu çözer" diyerek geçiyor, ki çözmüyor (aynı compose iki farklı Docker
sürümünde iki farklı şey).

**Ağaçta zaten var.** `diagnostics.rs` her şeyi tek dosyada paketliyor (log + preflight +
doctor + motor durumu + sürüm, iki kez maskelenmiş). `doctor.rs` adlandırılmış suçlular
üretiyor. `preflight.rs` ön koşulları. Eksik olan **iki paketi karşılaştıran fonksiyon**.

**Hendek.** Kopyalanabilir — ama teşhis paketi zaten olan tek ürün bu.

**Maliyet.** Bir karşılaştırma fonksiyonu + bir panel. Yeni ölçüm yok.

### K-4. İstek tekrarı (request replay)

**Ne.** Bir isteği yakala — yöntem, yol, başlıklar, gövde, oturum — ve kod
değiştikten sonra konteynere **yeniden çal**. "Bu hata yalnız o sepette oluyordu"nun
sonu.

**Rakip.** Bu kategoride hiçbir üründe yok (ölçüldü: M1-5). PHP dünyasında karşılığı
`dontbug` adlı bağımsız bir ters hata ayıklayıcı; hiçbir yerel ortam ürünü onu
paketlemiyor.

**Ağaçta zaten var — ve anahtar parça beklenmedik bir yerde:**

| Parça | Nerede |
| --- | --- |
| **Bir isteği adlandıran tek artefakt** | `spx::Report` — `GET /checkout`, başlangıç anı, süre (`explain.rs` bunu "the key" diye adlandırıyor) |
| Veritabanının o isteğe ne sorduğu | `querylog.rs` (+ N+1 tespiti) |
| Dump'ın hangi istekte olduğu | `debugbridge.rs` `request` alanı |
| Üçünü tek eksende birleştiren | `timeline.rs`, `explain.rs` (`request_explain`, `request_timeline` IPC komutları) |

`explain.rs`'in başlığı bu maddenin gerekçesini zaten yazmış: *"no new measurement is
needed."* Eksik olan **fiil**.

**Hendek.** Yerel ikilide de teknik olarak mümkün — ama kimse yapmadı ve StackVo'da
gereken dört parçanın dördü de duruyor.

### K-5. Anlık görüntüye bağlı tekrar — hata raporunun tekrarlanabilir hali

**Ne.** K-4 + `snapshot.rs`: kaydedilen isteğin **başında** otomatik bir anlık görüntü
alınır; tekrar, veritabanının tam o andaki hâline karşı koşar. "Bende üretemiyorum"un
sonu — ve bir hata raporunun eki artık bir metin değil, **bir durum**.

**Rakip.** Hiçbiri. Bulut tarafında bile nadir.

**Ağaçta.** `snapshot.rs` (adlandırılmış anlık görüntü + zamanlanmış olan + "bir insanın
adlandırdığını asla silme" kuralı), `db::restore` güvenlik ağıyla (`provider.rs`:
*"restoring the wrong file is recoverable"*).

### K-6. Ortamla birlikte bisect

**Ne.** `git bisect` kodu geri alıyor; ortamı almıyor. Üç ay önceki commit'te PHP 8.3
vardı ve bugünkü konteynerde 8.4 var — yani bisect'in yarısı yalan. K-2'nin kilidi varsa
`stackvo bisect` her commit'in **ortamını da** kurar.

**Rakip.** Hiçbiri — çünkü hiçbirinde kilit yok (K-2).

**Hendek.** K-2'ye bağımlı, dolayısıyla aynı hendeği devralıyor. **Kopyalanamaz.**

### K-7. Çok dilli tek repo (polyglot monorepo)

**Ne.** Bir depoda `api/` (Go), `web/` (Next.js), `worker/` (Python) → üç konteyner, üç
hostname, tek proje, tek `up`.

**Rakip.** ServBay ve FlyEnv çok dil destekliyor ama hepsinde birim **"site"** ve bir
sitenin bir çalışma zamanı var. Monorepo'yu tek özne olarak ele alan yok.

**Ağaçta zaten var.** `manifest::LANG_RUNTIMES` (6) + php + node = **8 çalışma zamanı**,
`generator.rs` her biri için Dockerfile üretiyor, `sidecar.rs` proje-kapsamlı konteyner
kuralını taşıyor, Traefik router üretimi hostname başına çalışıyor.

**Hendek.** Yerel ikilide bir dizinin iki çalışma zamanı olamaz. **Kopyalanamaz.**

### K-8. Çıkış görünürlüğü (egress)

**Ne.** Hangi konteyner internete konuştu, nereye. Kurumsal alıcının sorusu ve
`policy.rs`'in doğal tamamlayıcısı — `registryPrefix` ile aynayı zorlayan bir yönetici,
aynayı **kimin baypas ettiğini** de görmek ister (ki D-2'de on imgenin baypas ettiği
ölçüldü).

**Rakip.** Hiçbiri.

**Hendek.** Yerel ikilide konteyner yok, dolayısıyla ağ isimlendirmesi de yok — host
süreçlerinin trafiğini ayırmak için işletim sistemi seviyesinde iş gerekir.
**Kopyalanamaz.**

---

## M4. Zincir sınıfı — imzalı kayıt ve politikanın mümkün kıldığı

### Z-1. Projenin kendi bağımlılıkları için tedarik zinciri raporu

**Ne.** `pkg.rs` **paketleri** doğruluyor; aynı disiplin projenin kendi
`composer.lock` / `package-lock.json`'una uzatılabilir: *"bu projenin üç güvenlik
danışmanı var, ikisi doğrudan bağımlılıkta."*

**Rakip.** Hiçbiri. Herd, ServBay, DDEV — hiçbiri projenin bağımlılıklarına bakmıyor.

**Ağaçta.** `doctor.rs` deseni (adlandırılmış suçlu + önerilen düzeltme), `market.rs`
`HttpSource` (`http://` reddi dahil), `signing.rs`.

**Uyarı.** Bu bir **ağ çağrısıdır** ve `PRIVACY.md` telemetri yokluğunu bir testle
(`privacy_claims.rs`) koruyor. Dolayısıyla: kullanıcının açık isteğiyle, hangi hostun
çağrıldığı `PRIVACY.md`'nin "erişilebilir hostlar" listesine yazılarak. Sözü bozmadan
yapılabilir — `market.rs`'in kataloğu çekmesi zaten aynı sınıfta.

### Z-2. Sır sızıntı taraması

**Ne.** `secrets.rs` bir parolayı `.env`'den keystore'a taşıyor. **Ters yön yok:**
"`.env`'inde keystore'da olmayan bir AWS anahtarı var" ve daha sertçe *"bu anahtar
`git log`'da görünüyor."*

**Rakip.** Hiçbiri.

**Ağaçta.** `config::Env::is_secret` (sonek eşlemesi), `logging.rs`'in maskeleme kuralları
(zaten her satırda çalışıyor), `git.rs`, `doctor.rs`'in bulgu şekli.

**Not.** `preset.rs` bu işin **doğru şeklini** zaten göstermiş: deny-list değil
allow-list, *"a key added upstream tomorrow called `SERVICE_FOO_APIKEY` would sail
straight through"* gerekçesiyle. Tarayıcı da aynı asimetriyi almalı.

### Z-3. Politika uyum raporu

**Ne.** Bir yönetici `policy.json` yazıyor ve bugün öğrenebileceği tek şey dosyanın
ayrıştığı. Eksik olan: *"bu makinede politikanın hangi maddesi fiilen uygulanıyor"* —
kilitli anahtarlar tutuyor mu, ayna hangi imgelere uygulandı, hangileri baypas etti
(D-2'de ölçülen on tanesi), hangi paket imzasız kabul edildi.

**Rakip.** Hiçbirinde politika katmanı **yok**, dolayısıyla raporu da yok. En yakını
Herd Teams ve o bir lisans yönetimi.

**Ağaçta.** `policy.rs`, `audit.rs`, `market.rs`'in doğrulama sonucu, `doctor.rs` biçimi.

---

## M5. Sözleşme sınıfı — IPC/MCP disiplininin mümkün kıldığı

### S-1. Ajan eylem defteri ve geri alma ⭐

**Ne.** Her MCP **yazma** aracı ne yaptığını *ve telafi eylemini* kaydetsin.
*"14:32'de `stackvo_stack_down` çağrıldı, Claude Code tarafından"* → **tek tık geri al.**

**Rakip.** Hiçbiri. Lerd 11 araç, Kettle Code 18, ServBay 50+ servis erişimi veriyor ve
**hiçbirinde kim ne yaptı sorusunun cevabı yok.** README'nin kendi uyarısı bu boşluğun
adını koyuyor: `--allow-writes` bir ajana **tüm stack'i durdurma** yetkisi veriyor.

**Ağaçta.** `audit.rs` tam olarak bu şey için yazılmış — döndürülmeyen, geri alınamayan
işlerin kaydı, *"whoever has to account for the machine"* için. `contracts/ipc.json` her
komutun `query` mi `mutation` mı olduğunu **zaten biliyor** ve üç test bunu MCP tarafında
çapraz denetliyor.

**Ve bir bulgu:** `audit.rs` yazıyor ama **okunamıyor.** 309 IPC komutu içinde adı `audit`
geçen **sıfır** komut var. Yani bugün bu defter yalnızca dosyayı elle açan biri için var.
Bu maddenin ilk adımı yeni bir özellik değil, **var olanın önünü açmak**.

### S-2. Kapsamlı ajan yetkisi

**Ne.** Bugün `--allow-writes` hepsi-ya-hiç: 12 araç birden açılıyor ve içinde
`stack_down` ile `service_stop` var. Yerine: *"bu ajan yalnız `shop` projesini yeniden
başlatabilir, 30 dakika boyunca."*

**Rakip.** Hiçbirinde kapsam yok — hepsinde MCP ya açık ya kapalı.

**Ağaçta.** `websurface.rs` bu sorunun **aynısını zaten çözmüş**: loopback + koşu başına
üretilen ve workspace'e hiç yazılmayan bir token + salt-okuma. Üç kural ve her birinin
neden diğer ikisi yetmediği için orada olduğu yazılı. Kapsamlı yetki aynı desenin
genişletilmesi.

### S-3. Yapılandırılmış "son hata + önerilen düzeltme", MCP kaynağı olarak

**Ne.** Bir ajan bugün başarısızlığı **stderr'den tahmin ediyor.** Oysa `explain.rs`,
`doctor.rs` ve `hints.rs` yapılandırılmış teşhis üretiyor: kod, suçlu, ve **ne yapılacağını
söyleyen cümle** (`hint_key`, iki dilde).

**Ağaçta.** Üçü de cevabı zaten üretiyor. `error.rs`'in tek şekli (`code`, `message`,
`hint`, `hint_key`, `details`) bunun için biçilmiş kaftan.

### S-4. Ölçüm araçlarının MCP'de olmaması — bugünkü en somut boşluk

**Ölçüldü.** 34 MCP aracı arasında `stackvo_hotspots` ve `stackvo_profiler` var. Ama IPC
yüzeyinde duran şu komutların MCP karşılığı **yok**:

| IPC komutu | Ne cevaplıyor | MCP'de |
| --- | --- | --- |
| `request_explain` | "Bu istek neden yavaştı" — üç enstrüman bir istek etrafında | ❌ |
| `request_timeline` | dump + sorgu, tek eksende | ❌ |
| `query_log` | Veritabanına ne soruldu, aynı soru kaç kez (N+1) | ❌ |
| `profiler_flame` / `profiler_tree` | Gerçek yığınlardan alev grafiği | ❌ |
| `worktree_list` / `worktree_create` | Dal başına ortam | ❌ |
| `release_plan` / `release_build` | Üretim imajı | ❌ |

Yani **ürünün en farklılaşmış dört enstrümanı** — zaman ekseni, açıklama, sorgu kaydı,
alev grafiği — bir ajanın erişemeyeceği yerde duruyor. README *"why is shop.loc not
loading?"* sorusunu MCP'nin cevaplayabildiğini anlatıyor; *"neden yavaş"* sorusunu
cevaplayamıyor, oysa cevabı üreten kod yazılmış ve test edilmiş durumda.

**Rakip.** Lerd MCP'sinden SPX'e erişilebiliyor. Ama **hiçbir rakipte** dump+sorgu
korelasyonu yok, dolayısıyla MCP'sinde de olamaz.

**Maliyet.** Araç başına bir satır dispatch + bir şema. Üç test zaten çapraz denetliyor.
Bu, bu raporun **en ucuz maddesi** ve etkisi K sınıfına yakın.

---

## M6. Ucuz ve tek — kopyalanabilir, ama bugün kimsede yok

### U-1. Kaynak bütçesi ve proje başına atıf

**Ne.** *"`shop` bugün 4,2 GB·saat ve 38 dakika CPU tüketti."* Ve bir bütçe: bir projenin
eşiği aşması bir bildirim.

**Rakip.** Hiçbiri. Docker tabanlılar dahil — DDEV, Lando, Laradock hiçbiri kaynak
muhasebesi yapmıyor.

**Ağaçta.** `stats_store.rs` konteyner başına örnek geçmişini **yeniden başlatmalar
arasında** tutuyor (ve zamanın yalanı konusunu çözmüş: yaşa göre filtreli yükleme).
`idle.rs` kullanılmayanı zaten durduruyor ve sinyal olarak Traefik erişim kaydını
kullanıyor — *"a router that has served nothing for an hour is a fact rather than an
inference."*

**Neden önemli.** Bu, **R-2'yi savunmaya çevirir.** Docker pahalı; pahalı olduğunu ölçen
ve yöneten tek ürün olmak, ölçmeyip inkâr etmekten iyi bir konumdur.

### U-2. Odak modu

**Ne.** *"Yalnız bu projenin ihtiyacı olanı çalıştır, gerisini durdur."* Dizüstü
bilgisayarda en çok istenen tek fiil.

**Ağaçta.** `manifest.services` projenin ihtiyacını **beyan ediyor**, `idle.rs`
durdurmayı biliyor, `inflight.rs` çakışmayı engelliyor. Eksik olan tek şey fiil.

**Rakip.** Herd/ServBay servisleri tek tek açıp kapatıyor; "bu projenin ihtiyacı" diye
bir kavram yok çünkü manifest yok.

### U-3. Paylaşılabilir teşhis bağlantısı

**Ne.** Bir meslektaşın açacağı, parolalı, geçici bir URL — teşhis paketinin okunabilir
hâli. Zip'i e-postayla göndermenin yerine.

**Ağaçta üç parça da var:** `diagnostics.rs` (paket, iki kez maskelenmiş), `tunnel.rs`
(9 sağlayıcı), `tunnelid.rs` (**parola muhafızı ve kalıcı adres**), `landing.rs`
(sayfa üretimi). Birleştiren fiil yok.

**Rakip.** Hiçbiri.

### U-4. Onboarding doğrulaması

**Ne.** Depo *"bu projede çalışmak için şunlar gerekir"* diyor; `stackvo verify` *"senin
kurulumun beyan edilene uyuyor mu"* diye cevap veriyor — ve uymuyorsa **hangi satır**.

**Rakip.** DDEV `.ddev/config.yaml` + hooks ile işin **kurma** yarısını yapıyor;
**doğrulama** yarısını hiçbiri yapmıyor.

**Ağaçta.** `preset.rs` (`plan_file`/`apply_file` — plan zaten ayrı bir kavram!),
`doctor.rs`, `manifest::validate`. K-2 ile birleşince cevap kesinleşiyor.

### U-5. Sürüm başına yükseltme notu ve ikinci kanal

**Ne.** Önceki raporun "8. Şunlar da olsaydı" listesinin 3. ve 5. maddeleri; rekabet
bağlamında sınıf değişiyor: Herd ve ServBay ikisi de sürüm notunu ürünün içinde
gösteriyor, `channel.rs` zaten yazılmış ve `tauri.conf.json` tek uç tanımlıyor.

### U-6. İlk açılış turu

**Ne.** 309 komutluk, 26 panelli bir yüzeyde keşif rakiplerinkinden **daha zor**, çünkü
yüzey daha geniş. `BootstrapGate`, `RequirementsGate`, `MigrationGate`, `CatalogueGate`
var — ama bunlar engel, tanıtım değil.

**Rakip.** Herd, ServBay ve EnvKit'in hepsi karşılama akışı gösteriyor.

---

## M7. Zaten var, sayılmıyor — sıfır kod, tam kazanç

Bu bölüm en yüksek getirili olanı, çünkü maliyeti yazmak.

| Özellik | Ağaçta | Rakip durumu | Kullanıcıya görünürlük |
| --- | --- | --- | --- |
| **Üretim imajı üretimi** | `release.rs` + **7 IPC komutu** (`release_build`, `release_push`, `release_push_plan`, `release_recipe`, `release_plan`, `release_save`, `release_load`) | **Hiçbiri.** Modülün kendi başlığı: *"no native-binary competitor can follow"* | README'de **sıfır satır** — "release" kelimesinin 7 geçişi de uygulamanın kendi yayın süreciyle ilgili |
| **Dal başına tam ortam** | `worktree.rs` + 6 IPC komutu — kendi hostname, **kendi veritabanı**, kendi env | dde host+TLS veriyor, DB/env vermiyor. Geri kalanı bulut kategorisi ve ücretli (M1-3) | README'de sıfır satır |
| **İstek açıklaması + zaman ekseni** | `request_explain`, `request_timeline`, `explain.rs`, `timeline.rs`, `trace.rs` (gerçek yığın alev grafiği) | Hiçbirinde korelasyon yok | README'de sıfır satır |
| **7 kaynaktan içe aktarma** | `imports.rs`, `ALL: [Source; 7]` | En yakını 3 | R-13 |
| **Denetim kaydı** | `audit.rs` | Hiçbiri | **Okuma yüzeyi bile yok** (S-1) |
| **Devcontainer ihracı** | `devcontainer.rs` | Hiçbiri | README'de sıfır satır |

Altı özellik, hepsi yazılmış ve test edilmiş, ve **dördü hiçbir kullanıcı belgesinde
geçmiyor.** Bu, R-13'ün (göç savaşı görünmüyor) genelleştirilmiş hâli ve bu raporun en
ucuz maddesi: **kod yazmayı değil, saymayı gerektiriyor.**

---

## M8. Rakibin ilan ettiği plan — kapanan pencereler

DDEV'in 2026 plan yazısı üç şey söylüyor ve üçü de bu rapor için birer saat:

| DDEV planı | StackVo'da bugün | Pencere |
| --- | --- | --- |
| *"considering a general AI strategy"* | 34 MCP aracı, 6 kural dosyası, 8 istemci kaydı, `agentctx.rs` | **Açık ve geniş** — %72 pazar paylı lider henüz düşünüyor. K-1 ve S-1 bu pencereden girer |
| *"exploring using real certificates instead of mkcert CA"* | mkcert wildcard; `tunnel.rs` var, ACME yok | **Kapanıyor.** ServBay Pro bunu zaten satıyor (Let's Encrypt, ZeroSSL, Google Trust) |
| *"subdomains for extra ports/services instead of separate ports"* | Traefik + `routes.rs` bunu yapabilecek yerde | **Kapanıyor** — ve StackVo'nun altyapısı zaten uygun |

Ve Herd'in 2026 sürümleri (servis klonlama, `herd php:update all`, RustFS, JSON çıktılı
CLI) bir şey daha söylüyor: **lider artırımlı gidiyor.** Kategoride sıçrama yapacak yer
boş.

---

## M9. Etki × maliyet

| | **Düşük maliyet** | **Orta maliyet** | **Yüksek maliyet** |
| --- | --- | --- | --- |
| **Çok yüksek etki** | **M7** (say, kod yazma)<br>**S-4** (dört enstrümanı MCP'ye) | **K-1** ajan kum havuzu<br>**S-1** ajan defteri + geri al | **K-2** kilit dosyası |
| **Yüksek etki** | **S-3** yapılandırılmış hata<br>**U-2** odak modu | **K-3** ortam farkı<br>**S-2** kapsamlı yetki<br>**U-1** kaynak bütçesi | **K-4** istek tekrarı |
| **Orta etki** | **U-5** sürüm notu<br>**U-3** teşhis bağlantısı | **Z-2** sır taraması<br>**U-4** onboarding doğrulama<br>**U-6** açılış turu | **K-5** anlık görüntülü tekrar<br>**K-7** monorepo<br>**Z-1** tedarik zinciri raporu |
| **Niş / uzun vadeli** | | **Z-3** politika uyum raporu | **K-6** ortamla bisect<br>**K-8** egress |

Sol üst köşe bu raporun cevabı: **M7 ve S-4 hem en ucuz hem en yüksek etkili, ve ikisi de
yeni özellik değil — var olanın önünü açmak.**

---

## M10. Önerilen sıra

**Sıfır risk, yayınla birlikte (kod yazmayı gerektirmeyen):**

1. **M7'nin altı satırını say.** `release.rs`, `worktree.rs`, `request_explain`,
   `devcontainer.rs`, `imports.rs`, `audit.rs` — dördü hiçbir kullanıcı belgesinde
   geçmiyor. Bu, R-13 ve P0-4 ile aynı düzenlemede yapılır.
2. **Dal başına ortamı adıyla konumlandır.** Bulut tarafında bu kategorinin adı *preview
   environment* ve ücretli; StackVo'da yerel, ücretsiz ve yazılmış. Kullanılan kelime
   ürünün nerede aranacağını belirliyor.

**Yayından hemen sonra (ucuz, yüksek etki):**

3. **S-4 — dört enstrümanı MCP'ye koy.** `request_explain`, `request_timeline`,
   `query_log`, `profiler_flame`. Araç başına bir dispatch satırı + bir şema; üç test
   zaten çapraz denetliyor. Ürünün en farklılaşmış yüzeyi bugün ajana kapalı.
4. **S-1'in ilk yarısı — `audit`'e bir okuma komutu.** Bugün defter yazılıyor ve
   okunamıyor. Bir IPC komutu + bir panel.
5. **S-3** — son hatayı yapılandırılmış olarak MCP kaynağı yap. `error.rs` şekli hazır.
6. **U-2 odak modu** — tek fiil, `manifest.services` + `idle.rs`.

**Bir sonraki büyük iş — sırayla, ikisi birbirine bağlı:**

7. **K-1 ajan kum havuzu.** `worktree::create` + `db::copy_database` + TTL + kapsamlı MCP.
   Yeni modül yok. 2026'da yerelde sahipsiz olan tek konum.
8. **S-1'in ikinci yarısı — telafi eylemi ve geri alma.** K-1'i güvenli yapan şey; ikisi
   bir arada "ajanların güvenle koşabildiği ortam" cümlesini tamamlıyor.
9. **S-2 kapsamlı yetki.** `websurface.rs`'in token deseni.

**Orta vade:**

10. **K-2 kilit dosyası.** En zor yarısı (belirlenimci üreteç, digest zinciri) çoktan
    bitmiş. D-1'i (kendi imgelerinin altısı `:latest`) kapatmayı zorunlu kılıyor.
11. **K-3 ortam farkı.** `diagnostics.rs` üzerine bir karşılaştırma.
12. **U-1 kaynak bütçesi.** Docker'ın maliyetini ölçülen bir şeye çevirir.
13. **K-4 istek tekrarı**, sonra **K-5** anlık görüntüyle.

**Uzun vade / fırsat penceresine göre:**

14. ACME seçeneği — DDEV planında yazılı, ServBay satıyor (M8).
15. **K-7** monorepo, **Z-1/Z-2** tedarik zinciri ve sır taraması, **K-6** bisect,
    **K-8** egress.

---

## M11. Ölçüm özeti

| Ne | Sayı |
| --- | --- |
| Önerilen özellik | **21** (K 8, Z 3, S 4, U 6) |
| Yerel-ikili rakibin **kopyalayamayacağı** (kale + zincir) | **11** |
| Yeni modül gerektirmeyen (var olanın birleşimi ya da önünün açılması) | **14 / 21** |
| Sıfır kod — yalnız belgede sayılmayı bekleyen özellik | **6** (M7) |
| Ajana kapalı olan farklılaşmış enstrüman | **4** (`request_explain`, `request_timeline`, `query_log`, `profiler_flame`) |
| `audit.rs`'i okuyan IPC komutu | **0 / 309** |
| Toplam IPC komutu | 309 |
| Toplam MCP aracı | 34 (12'si yazma) |
| DDEV'in ilan ettiği ve StackVo'nun önde olduğu alan | AI stratejisi (*"considering"*) |
| DDEV'in ilan ettiği ve StackVo'da olmayan | gerçek sertifika (ACME), alt alan adları |

---

## M12. Tek cümlelik hüküm

Bu ürünün elinde, kategorinin **hiçbir rakibinin mimarisinin izin vermediği** on bir
özellik için gereken parçaların neredeyse tamamı zaten yazılmış ve test edilmiş
durumda — ve 2026'nın en büyük boşluğu, yani **bir AI ajanının canlı veritabanına
dokunmadan çalışabileceği yerel ve ücretsiz bir ortam**, bu depoda `worktree.rs`,
`db::copy_database`, `snapshot.rs` ve `agentctx.rs` olarak halihazırda duruyor ve
yalnızca **onları birleştiren bir fiil ile onları sayan bir paragraf** eksik.

---

# Birleşik Yol Haritası — Dört Raporun Tek Sıraya İndirilmiş Hâli

**Tarih:** 2026-08-28 (birleştirme turu)
**Taban:** `d465c68` (main)
**Kaynak:** Bu belgedeki dört raporun tamamı — §9, §J, §L8, §M10

Dört tur da kendi içinde sıralı, ama **birbirlerinden habersiz sıralanmışlar.** Aynı iş dört
farklı yerde farklı önceliklerle geçiyor, bir madde bir başkasını yanlışlıyor, iki madde
birbirinin önkoşulu. Bu bölüm dört sırayı tek sıraya indiriyor; bulgu numaraları korunuyor
ki yukarıdaki gerekçelere geri dönülebilsin.

Bloklar teslimat fazı numarasıyla değil **ne oldukları** ile adlandırıldı. Gerekçesi §0'ın
kendisi: numaralı bir kuyruk okuyanda "bu konu kapanmış" izlenimi bırakıyor ve
`no_dangling_docs.rs::nothing_still_dates_itself_by_a_delivery_phase` bu adlandırmayı zaten
bir kapı olarak tutuyor. Blok içindeki sıra bağımlılıktan geliyor, takvimden değil.

---

## N. Durum — 28 Ağustos 2026

Bu blok tek bakışta nerede olunduğunu söylüyor. Her satırın gerekçesi kendi maddesinde;
burada yalnız sayı var, çünkü bir yol haritasının en çok bayatlayan yeri bu ve bayat bir
"tamamlandı" işareti hiç işaret olmamasından kötü.

**Maddelerin işaretlenişi.** Her madde numarasından hemen sonra bir **işaretle** başlar ve
başlığının sonunda **parantez içinde hükmünü** taşır — göz, satırın başında durumu görüyor,
ve okuyan cümlenin sonunda onu okunabilir hâliyle buluyor.

| İşaret | Hüküm | Başlığın üstü | Ne demek |
| --- | --- | --- | --- |
| ✅ | `(yapıldı)` | **çizili** | Tam bitti |
| ⚠️ | `(yarım)` | çizili değil | Bir yarısı bitti; hangi yarının **neden** kalmadığı maddenin içinde yazılı |
| ❌ | `(yapılmadı)` | çizili değil | Hiç başlanmadı — çoğu dışsal bir şeyi bekliyor |
| 🚫 | `(yapılmayacak)` | **çizili** | Ölçüldü ve bırakıldı: bir eksik değil, **alınmış bir karar**, gerekçesi maddenin içinde |

Üstü çizili olmak "bu madde kapandı" demek; işaret hangi biçimde kapandığını söylüyor.

| Blok | Yapıldı | Yarım | Açık | Yapılmayacak |
| --- | --- | --- | --- | --- |
| **N1** — yarım günlük blok | 7 / 7 | — | — | — |
| **N2** — yayın bloklayıcıları | 9 | 1 (#4 README) | — | 1 (#8 TLD) |
| **N3** — yayın koşusu | — | 1 (#0 anahtarlar) | 5 | — |
| **N4** — yayından sonra | 10 | — | 1 (#6) | — |
| **N5** — bakım borcu | 4 | — | 1 | — |
| **N6** — arayüz/hardcode borcu | 11 | 3 (#7, #12, #14) | — | — |
| **N7** — stratejik | 5 (#1, #2, #3, #5, #6) | 4 (#4, #7, #8, #10) | — | 1 (#9 ACME) |

**Ölçülen durum.** Rust 1786 → **1918** test, vitest 1275 → **1349**,
`contracts:check` 12 uyarı → **1** (beklenen `NO_MANIFESTS`), IPC yüzeyi 309 → **324**
komut (`machine_commands`, `crash_reports`, `crash_reports_seen`, `audit_undo`,
`diagnostics_compare`, `usage_report`, `request_replay`,
`project_verify`, `leaks_scan`,
`env_untrack`),
MCP 34 → **38** araç,
servis kataloğu 31 → **33**, şablon 28 → **29**, Rust modülü **119** (bu turlarda
`queuelog`, `images`, `grant`, `undo`, `usage`, `verify` ve `leaks` eklendi),
köprünün `kind` alanı 1 değer → **3**, web sunucusu 5 → **6** (RoadRunner), tanınan motor
3 → **4** (Podman).
Kapsam: Rust %64,05 → **%68,05** (taban 60 → **65**), ön yüz %89,65 → **%92,20**
(taban 85 → **90**). Paket: eager 1547 → **1248,8 KB** (tavan 1700 → **1400**),
toplam 3001,5 → **2705,9 KB** — ikon stil dosyası 408 → **32 KB**; son 2,4 KB
alt kümeleyicinin `src-tauri/src`'yi de okumasının bedeli, ve karşılığı üç
seçicinin boş kare çizmemesi.

**Yayını tutan şey hâlâ kod değil.** N1 ve N2'nin kod tarafı bitti; N3'ün beş maddesi
dışsal — kimlikler, bir sürüm numarası kararı, bir Publish tuşu ve bir Windows makinesi.

**Bu turların bulduğu ve raporun bulmadığı hatalar** — her biri kendi maddesinde yazılı,
burada yalnız sınıfları: bir ölü tespit kuralı (`statamic/cms`, `artisan`'ın altında
kalmış), bir yanlış çatı cevabı (PrestaShop → symfony), var olmayan bir belge kökü
(Magento `pub/`, CakePHP `webroot/`), yarım düşen bir hata yolu (`details` ve `hintKey`
düşüyordu), iki farklı ana bilgisayar adı üreten iki benimseme yolu, ve rotasyonun
ortaya çıkardığı elle kopyalanmış ikinci bir sabitlenmiş anahtar. Üçü rapordaki teşhisin
**yanlış** olduğu yerlerdi ve her biri maddesinde ölçümüyle duruyor.

Son tur bunlara **dört** tane daha ekledi, hepsi N4 #11'in içinde yazılı: kuyruk
işçisi sidecar'ının hata ayıklama köprüsünü hiç taşımaması (yani bir kuyruk işinin
içindeki `dump()` hiçbir yere gitmiyordu, ve iki ayrı belge çalıştığını söylüyordu),
`spl_autoload_register`'ın PHP 8'de herkesin yanıtına bir notice basması, ilk işi
yutan bir imleç tohumu, ve zaman çizelgesinin bilinmeyen bir kaynağı sorgu olarak
çizmesi. **Ve bir tanesi rapordaki reçetenin uygulanamaz olduğu yerdi:** R-8'in
istediği dört Laravel dinleyicisi `auto_prepend_file`'dan kurulamıyor, çünkü Composer
kendi otomatik yükleyicisini önden kaydediyor.

---

## N0. Raporlar arası çakışmalar

Bunlar tek iş sayılmazsa aynı dosya dört kez açılır.

| Birleşen maddeler | Tek iş |
| --- | --- |
| P0-4 + R-13 + R-2 (tablo) + M7 (6 özellik) | **Tek bir README yeniden yazımı** |
| P0-5 + R-1 (Windows paragrafı) + P1-2 | README iddiaları ve onları koruyan gate |
| P2-5 + E-3 | Aynı on `SERVER_*` anahtarı |
| B-3 + B-4 + E-2 | **Tek bir `contracts:check` denetimi** üçünü birden yakalıyor |
| A-4 + R-6 | Makine geneli komut yüzeyi |
| §8.6 + R-2 (ikinci yarı) | Podman |
| §8.3 + §8.5 + U-5 | Kanal ve sürüm başına yükseltme notu |
| §8.2 + U-6 | İlk açılış turu |
| D-1 + K-2 | D-1, K-2'nin **önkoşulu** — kilitlenecek şeyin sabit bir etiketi olmalı |
| A-2 + R-3 | Benimseme yolu `.loc`'u ayrıca sabitliyor; TLD değişimi buna bağlı |
| R-4 + B-5 | Şablon listelerini bağlayan **tek** kapsama testi |

**Ve bir çelişki.** P2-5 *"`api.appsAvailable()` ölü kod, silinmeli"* diyor; H-1 bunu ölçüp
yanlışlıyor — çağrı `PreferencesPane.vue:32`'de duruyor, Prettier satırı böldüğü için
`validate-contracts.mjs`'in tek satırlık regex'i göremiyordu. **P2-5 olduğu gibi
uygulansaydı Tercihler paneli bozulacaktı.** Bu yüzden silme değil regex düzeltmesi olarak
sıraya girdi — ve **kapatıldı** (N1 #3). Geriye kalan on bir uyarının onu `SERVER_*`
ailesi, biri beklenen.

**Durum: on bir birleşmenin sekizi kapandı.** README yeniden yazımı (satır 1) bir madde
eksikle, iddia gate'i (2), `SERVER_*` (3), `contracts:check` denetimi (4), şablon kapsama
testi (11) ve `.loc` kararı (10) yapıldı; A-4+R-6 (makine geneli komut), Podman,
kanal/sürüm notu, ilk açılış turu ve D-1→K-2 N6 ile N7'ye taşındı — birleşmeleri geçerli,
işleri açık.

---

## N1. Yarım günlük blok — önkoşulsuz, hata sınıfı

Kendi içinde sıra: gizli hata → görünen hata → yalan söyleyen denetleyici → temizlik.

1. ✅ ~~**`NGINX_DIRECTIVES[0]` / `[4]` indeks erişimini anahtar aramasıyla değiştir.**~~ (yapıldı)
   `generator::directive(key)` eklendi, Caddy iki yönergesini anahtarla
   alıyor. Üç test: konum yerine anahtar (gövde zaman aşımını `max_size`'a kaydıran
   sürüm kırılıyor), her anahtarın kendi satırına çözülmesi, ve **tablonun dokuz
   varsayılanının `config::SETTINGS` ile eşitliği** — B-1'in ikinci yarısı, daha önce
   hiçbir şey karşılaştırmıyordu. *(B-1)*
2. ✅ ~~**`OverviewPane.vue`'nun kapsayıcı yolunu runtime'dan türet.**~~ (yapıldı)
   `containerPath`, `render_dockerfile`'ın kendi dağıtımını aynalıyor (PHP değilse `/app`),
   yani dokuzuncu bir çalışma zamanı iki tarafta da düzenleme istemiyor.
   `tests/project-overview.spec.js`, 9 test. *(G-1)*
3. ✅ ~~**`validate-contracts.mjs` regex'ini `api\s*\.\s*<method>` yap** ve P2-5'ten
   `appsAvailable` maddesini çıkar.~~ (yapıldı) — `[F] reachability` temiz,
   `contracts:check` 0 hata / 11 uyarı. *(H-1)*
4. ✅ ~~**`validate-contracts.mjs`'deki `'8.2'` yedeğini `8.4` yap.**~~ (yapıldı) — ama
   beşinci bir literal yazarak değil: yedek artık `envSpec`'ten, yani şemanın kendi
   `default`'undan okunuyor, ve şema onu taşımıyorsa bu bir **hata**. Dört kopya üçe indi.
   *(B-3)*
5. ✅ ~~**Python varsayılanını tek kaynağa indir.**~~ (yapıldı) — `lang_defaults`
   python/go/ruby için `config::SETTINGS`'i okuyor (`settings_version`); rust/bun/deno
   gerekçesi yazılı literallerini koruyor. İki test: üçünün tabloyu okuduğu, ve diğer
   üçünün **okumadığı** — sonraki bir "tutarlılık" turu Deno'yu var olmayan bir etikete
   bağlamasın diye. *(B-4)*
6. ✅ ~~**Kökteki 0 baytlık `version` dosyasını sil.**~~ (yapıldı) — `git rm`; hiçbir
   Rust, betik ya da iş akışı onu okumuyordu (arandı). *(P3-2)*
7. ✅ ~~**Paket üstverisi.**~~ (yapıldı) — `package.json`'a `repository`/`bugs`/`homepage`/
   `author`/`engines`, `Cargo.toml`'a `repository`/`homepage`/`keywords`/`categories`,
   `authors` LICENSE'taki ada çevrildi, `.nvmrc` = 22 (CI ile aynı). İki test
   `version_agreement.rs`'e eklendi: krediyi lisansla, deponun adresini iki manifest
   arasında bağlıyor. N4 #5'in (Dependabot) önkoşulu artık karşılandı. *(P3-3)*

---

## N2. Yayın bloklayıcıları — kod işi

Kendi içinde sıra: **önce gate, sonra düzeltme** — yoksa aynı sınıf hata üçüncü kez döner.

1. ✅ ~~**Bağlantı/iddia denetimini genelleştir.**~~ (yapıldı)
   `every_link_points_at_a_file_that_exists` artık `LINKED_DOCUMENTS` üzerinde: altı
   belge (`ARCHITECTURE`, `README`, `SECURITY`, `CONTRIBUTING`, `PRIVACY`,
   `ACCESSIBILITY`). Ayrıştırıcı koruması belge başına değil **toplam** üzerinde,
   çünkü ikisi meşru olarak hiçbir yere bağlanmıyor. *(P1-2)*
2. ✅ ~~**README üreteç bölümü.**~~ (yapıldı) — bölüm "devralma nasıl bitti" olarak
   yeniden yazıldı, tablo iki davranışı gösteriyor, `verify`'ın anlam değiştirdiği
   söyleniyor. Kendine bağlanan cümle de düzeltildi: `stackvo/stackvo` bağlantısı
   okuyucuyu aynı sayfaya geri getiriyordu. **Yeni gate:**
   `the_readme_names_the_generator_default_the_enum_actually_carries` — `#[default]`'ı
   taşıyan varyantı ayrıştırıyor, README başka bir varsayılan adlandıramıyor. *(P0-5)*
3. ✅ ~~**README Windows paragrafı.**~~ (yapıldı) — derlendiği ve birim testlerinin
   geçtiği yazıldı; doğrulanmamış olan üç şey (UAC üzerinden hosts yazımı, gerçek Docker
   Desktop'a karşı adlandırılmış boru, tarayıcıda alan adı çözümlemesi) ayrıca sayıldı.
   **Yeni gate:** `the_readme_does_not_deny_a_windows_build_the_matrix_performs` —
   `ci.yml` `windows-latest` içeriyorsa README "hiç derlenmedi" diyemiyor. *(R-1a)*
4. ⚠️ **README'yi son kullanıcıya çevir** (yarım) — dört bölümün dördü yazıldı, iki
   madde açık kaldı:
   - ✅ **Installing it** — altı kurulum biçimi, platform başına sistem gereksinimi,
     Docker gereksinimi. Yayın henüz yok, ve bunu söylüyor. *(P0-4)*
   - ✅ **What Docker costs you** — dört maliyet tablosu, karşılığında ne alındığı, ve
     "bu takas sana yanlış geliyorsa yanlıştır; kapanacak bir açık değil, mimarinin
     kendisi" cümlesi. *(R-2)*
   - ✅ **Coming from something else** — yedi kaynak adıyla, ve diğer kuruluma tek bayt
     yazılmadığı. *(R-13)*
   - ✅ **What it does that gets missed** — altı özellik tabloyla. *(M7)*
   - ❌ **Ekran görüntüsü ve rozet yok.** Bir ekran görüntüsü çalışan bir yapı ve bir
     insan gerektiriyor; buradan üretilemez. **Açık, ve sahibi kullanıcı.**
   - ❌ **Türkçe README yok.** Yazılabilir ama ikinci bir README ikinci bir bayatlama
     yüzeyi: `readme_claims.rs` yalnız İngilizcesini denetliyor, ve denetlenmeyen bir
     çeviri altı ay sonra farklı bir ürünü anlatır. Yazılacaksa gate'in iki dosyayı da
     sayması gerekiyor. **Açık.**

   **Yeni gate:** `the_readme_counts_the_surfaces_it_advertises` — yedi `release_*`, altı
   `worktree_*` ve `imports::ALL`'un yedi kaynağını ağaçtan sayıp README'yle
   karşılaştırıyor. `imports.rs`'in başlığı "**Two** of them" derken `ALL` yediyi
   taşıyordu (R-12); aynı sınıfın README tarafı artık kapalı.
5. ✅ ~~**Geçici dosya stage'ini sertleştir.**~~ (yapıldı) — `elevate::staging_dir`
   çağrı başına `0700` bir dizin açıyor (süreç kimliği + sayaç), `create_dir_all` değil
   `create_dir`: var olan bir adı **benimsemek**, onu yaratanı benimsemektir — saldırının
   tam olarak hamlesi buydu. `hosts.rs` ve `dns.rs` ikisi de oraya taşındı, temizlik
   `remove_dir_all`. Üç test: çağrı başına ayrı dizin, `0700` (yazılmadan **önce**),
   ve dolu adın reddi. *(P1-1)*
6. ✅ ~~**`docker run` imgelerini `policy::mirror`'dan geçir.**~~ (yapıldı)
   `policy::run_image` eklendi (aynayı okuyan tek nokta), on üç çağrı yeri ona bağlandı:
   `tunnel.rs` ×9, `tunnelid.rs`, `landing.rs`, `perf.rs` ×2. Üç test: `run_image`'in
   `mirror`'ın **her** muafiyetini koruduğu, politikasız makinede kimlik olduğu, ve
   **kapsama gate'i** — `_IMAGE` sabiti taşıyan her üretim modülü `run_image`'i çağırmak
   zorunda, yani beşinci bir modül eklendiğinde bu birinin hava boşluklu makinesinde
   değil burada görünüyor. *(D-2)*
7. ✅ ~~**`detected_spec`'e `Env` geçir.**~~ (yapıldı) — imza `(name, detected, env)`,
   üç çağrı yeri de `Env::load`'u geçiyor (`adopt_many` toplu iş için bir kez).
   PHP, sunucu ve Node sürümü artık ayardan; **klasörün beyanı ayarı yenmeyi
   sürdürüyor** (bir `package.json` karar, ayar yalnız karar vermemiş klasörün cevabı).
   `detected.server` kaldırıldı — `detect.rs`'in dört yerinde de `"nginx"`'ti, yani
   tespit değil, değiştirilemeyecek bir yere yazılmış ayardı. Üç test, biri **alan adının
   bilerek okunmadığını** kilitliyor: `DEFAULT_TLD_SUFFIX` kendini proje alan adlarının
   kullanmadığı bir anahtar olarak tanımlıyor, ve sonraki bir "işi bitirme" turu onu
   uygulatmasın. *(A-2)*
8. 🚫 ~~**Varsayılan TLD'yi `.test` yap.**~~ (yapılmayacak) — **karar verildi, `.loc` kalıyor.**

   Bu bir eksik değil, **verilmiş bir karar**, ve L6'nın kendi tablosuyla aynı sınıfa
   giriyor. Gerekçesi burada duruyor ki dördüncü bir tur maddeyi yeniden açmasın.

   **R-3'ün iki gerekçesi de ölçünce dar çıktı:**

   | İddia | Ölçüm |
   | --- | --- |
   | "Çözümlenemeyen sorgu ISS'e sızar" | Proje alan adları `hosts_apply` ile `/etc/hosts`'a yazılıyor, ve bir hosts satırı DNS sunucusu kapalıyken de çözümlüyor. Sızma yalnız **joker takma adlarda** kalıyor (`*.shop.loc`) — `OverviewPane`'in yorumu bunu zaten söylüyor: joker sertifikaya ve router'a ulaşır, `/etc/hosts`'a ulaşamaz |
   | "`.test` RFC 6761'de ayrılmış" | Doğru, ama garanti tam değil: metin *"SHOULD NOT attempt to look up"* diyor, MUST değil. Çoğu çözümleyici yine yukarı soruyor, yani fark pratikte küçük |

   **Ve alan adı zaten proje bazında serbest.** `project.schema.json`'da `domain` herhangi
   bir geçerli ad olabilir, `formToSpec` yazılanı olduğu gibi alıyor, ayar yalnız
   yazılmadığında devreye giren varsayılan. `.test` isteyen bugün yazıyor — kapalı bir kapı
   yok.

   **Karşı tarafta duran maliyet ise gerçek ve ölçüldü:** `.env` yalnız geçersiz kılmaları
   tutuyor ve `Env::load`'un yorumu bunu söylüyor — *"having none is the normal state of a
   fresh workspace"*. TLD ayarına hiç dokunmamış her mevcut kurulum bu anahtarı dosyada
   taşımıyor, `EMBEDDED`'den çözüyor. `SETTINGS`'i çevirmek onları sessizce taşırdı:
   `certs::suffix` değişir, joker sertifika `*.stackvo.loc`'u kapsıyorken artık eşleşmez, ve
   **her projenin HTTPS'i sertifika yeniden düzenlenene kadar bozulur.** Doğru yapılışı bir
   göç — varsayılanı çevirmeden önce mevcut workspace'lerin `.env`'ine mevcut son eki
   yazmak, artı sertifika yenilemesi — ve `MigrationGate` `handover`'a bağlı, genel bir
   çerçeve değil.

   Yani: düşük olasılıklı bir gelecek riski ve dar bir joker sızıntısı karşılığında her
   kurulumu göç ettirmek. Takas tutmuyor. **`.loc` kalıyor.**

9. ✅ ~~**`contracts:check`'e "şema varsayılanı = kod varsayılanı" suiti.**~~ (yapıldı)
   suite D'ye `DEFAULT_DISAGREES`. `EMBEDDED_VALUES` `SETTINGS`'i zaten anahtar **ve**
   değeriyle ayrıştırıyordu, yani maliyeti bir döngü. **İlk koşuşunda E-2'yi yakaladı:**
   şema `1.62`, kod `1.84` — `"status": "conflicting"` bir insana not, kapı değil. Şema
   `1.84`/`active` yapıldı. Gate boş geçmediği kasıtlı bir ayrışmayla doğrulandı. *(E-2)*
10. ✅ ~~**On `SERVER_*` anahtarını `env.schema.json`'a ekle.**~~ (yapıldı) — yeni
    `serverLimits` grubu. Anahtarlar, nginx adları ve varsayılanlar **elle yazılmadı**:
    `generator.rs`'den ve `config::SETTINGS`'ten okunup ikisinin eşit olduğu doğrulanarak
    üretildi. `contracts:check` **12 uyarıdan 1'e** düştü ve `[D] env keys — clean`;
    kalan tek uyarı beklenen olan (`--allow-no-manifests`). *(P2-5 / E-3)*
11. ✅ ~~**Koddaki üç bayat rekabet iddiasını tarihle.**~~ (yapıldı) — üçü de "Ağustos
    2026'da ölçüldü" biçiminde yeniden yazıldı ve iddia daraltıldı (worktree'nin önü
    "hiçbiri" değil, dde'nin yapmadığı iki yarım; imports yedi; MCP ≥7/17).
    **Ve gate on modül daha buldu** — rapor üç diyordu. Ölçmediğim için tarihleyemedim;
    `UNDATED` listesinde **beyan edilmiş borç** olarak duruyorlar (`published_urls.rs`
    deseni: her zaman geçen dar bir denetim yerine gerekçeli bir liste). Listeden çıkan
    gerçek bir tarih taşımak zorunda, listeye girmeyen yeni modül de. *(R-12)*

---

## N3. Yayın koşusu — kod değil, sıraya koyma

Bu blok bilerek kod değil. Ama içindeki bir madde bu turda **kısmen** ilerledi, ve
ilerlemeyen kısmın neden ilerlemediği de yazılı.

0. ⚠️ **İmzalama kimlikleri** (yarım)
   - ✅ **StackVo'nun kendi iki anahtarı yapıldı.** `tools/keys.sh generate`, ikisi ayrı:
     güncelleyici anahtarı ikiliyi imzalıyor, içerik anahtarı paket indeksini. Ayrı
     olmalarının bedeli iki sır, karşılığı birinin sızmasının **bir** sahtecilik olması —
     sahte kurulum **ya da** sahte paket, asla sahte kurulum içinde sahte paket.
   - ✅ **İçerik anahtarı döndürüldü ve indeks imzalandı.** Eskisinin özel yarısı başka bir
     makinedeydi. Döndürmek mümkündü çünkü **v0.1.0 hiçbir ikili taşımıyor** — eski anahtarı
     sabitleyen kurulu tek bir kopya yok. **Bu pencere, içinde bir varlık olan ilk sürüme
     kadar açık;** ondan sonra rotasyon bir yayın işi olur, bir düzeltme değil.
     Rotasyon ayrıca elle kopyalanmış **ikinci** bir sabitlenmiş anahtar ortaya çıkardı
     (`stackvo-service-packages/tools/verify-signature.mjs`) — doğru imzalanmış bir indeksi
     yanlış anahtarla imzalanmış diye raporluyordu. Kopya kaldı (uygulamanın sabitini okumak
     çevrimdışı çalışması gereken bir gate'e ağ bağımlılığı katardı), ama satır artık
     döndürmenin orayı da kapsadığını söylüyor.
   - ❌ **Apple Developer Program ve Authenticode alınmadı.** Bunlar kurumsal kayıt, kimlik
     doğrulama ve ödeme — günler alıyor, dışsal, ve buradan başlatılamaz. **Takvimin kritik
     yolu hâlâ bu.** *(P0-3)*
1. ❌ Sürümü yükselt (ör. `0.2.0`), `CHANGELOG.md`'de sürüm başlığını aç, 4300 satırlık (yapılmadı)
   `Unreleased`'i oraya taşı, etiketle. **Numarayı seçmek kullanıcının kararı**; seçildiği
   an üç dosyadaki yükseltme ve CHANGELOG bölümlemesi buradan yapılabilir. *(P0-1)*
2. ❌ Kullanıcıya dönük **kısa** sürüm notu yaz — mevcut CHANGELOG bir mühendislik günlüğü ve (yapılmadı)
   sürüm notu olarak kullanılamaz. Bu buradan yazılabilir, #1'e bağlı.
3. ❌ Yayın koşusunu rehearsal'da uçtan uca doğrula, sonra **Publish** et; `releaseDraft: true` (yapılmadı)
   olduğu için basılmadıkça `latest.json` 404 verir. **Publish bir insanın tuşu.** *(P0-2)*
4. ❌ `npm run updates:check` ile ucu doğrula. #3'ten sonra anlamlı. (yapılmadı)
5. ❌ **Windows makinede elle tur:** (yapılmadı) — `preflight` → proje oluştur → `up` → tarayıcıda aç.
   Kategorinin 13/17'si Windows'ta ve bir CI koşusu bu soruyu cevaplamıyor. **Bir Windows
   makinesi gerekiyor.** *(R-1b)*

---

## N4. Yayından hemen sonra — ucuz, yüksek etki

Kendi içinde sıra: var olanın önünü açanlar → dağıtım → katalog boşlukları.

1. ✅ ~~**Dört enstrümanı MCP'ye koy.**~~ (yapıldı) — `stackvo_explain_request`,
   `stackvo_timeline`, `stackvo_query_log`, `stackvo_flame`. Araç sayısı 34 → **38**.
   İkisinin mantığı Tauri `State`'ten ayrıldı (`explain_request`, `build_timeline`),
   `verify_generator`'ın deseniyle — MCP'de kopyalamak ikinci bir kopya olurdu. README'nin
   araç sayısını **mevcut gate yakaladı** (34→38) ve düzeltildi. Yeni gate: dört komutun
   da bir MCP aracı tarafından uygulandığını araç adıyla değil **komut adıyla** doğruluyor,
   çünkü araç yeniden adlandırılabilir ve boşluk aynı boşluk olurdu. *(S-4)*
2. ✅ ~~**`audit`'e bir okuma komutu + panel.**~~ (yapıldı) — `audit::tail_of`,
   `audit_trail` IPC komutu (310. komut), `AuditPane.vue`, sözleşme kaydı, iki dilde
   yardım belgesi. Okuma için ayrı bir `Record` şekli: `Entry::action` bilerek
   `&'static str` ("aynı fiil sonsuza kadar aynı dize") ve derleyici bu sözü böyle
   tutuyor, dolayısıyla tek yapı iki işi görsün diye onu `String`'e genişletmek yazma
   tarafının dayandığı bir değişmezi okuma tarafının ihtiyacı olmayan bir tanım için
   takas etmek olurdu. Beş Rust + beş Vue testi. **Testler kendi panelimde gerçek bir
   hata buldu:** hata durumunda hem hata hem "hiçbir şey yapılmadı" görünüyordu —
   "bakamadım" ile "hiçbir şey yok" farklı cümleler. *(S-1 birinci yarısı)*
3. ✅ ~~**Son hatayı yapılandırılmış MCP kaynağı yap.**~~ (yapıldı) — **ama raporun çerçevesi
   değil.** MCP `resources` protokolü bu ağaçta hiç yok (`resources/list` → method not
   found), yani "kaynak" olarak sunmak bir dispatch satırı değil protokol işi. Asıl boşluk
   başka yerdeydi ve ölçüldü: hata yolu `error.rs`'in dört alanından **ikisini
   düşürüyordu** — `details` (adlandırılmış suçlu: hangi soket, hangi anahtarlar) ve
   `hint_key` (çeviri anahtarı). Colima ile Docker Desktop'ın ikisi de kurulu bir makinede
   "Docker çalışmıyor" cevap değil; **hangisi** cevap. Tek `failure()` fonksiyonu eklendi
   ve **loopback yüzeyi de ona bağlandı** — o `hint`'i de düşürüyordu, yani tek araca iki
   yoldan ulaşmak teşhisin ne kadarını aldığın konusunda farklı cevap veriyordu. Ayrıca
   `format!("{:?}", e.code)` silindi: `Code` zaten `Serialize`, o üçüncü yazımdı. İki
   test. *(S-3)*
4. ✅ ~~**Odak modu.**~~ (yapıldı) — `focus.rs` (saf mantık, 8 test), `focus_plan` +
   `focus_apply` IPC komutları (311. ve 312.), `ProjectDetail`'de plan diyaloğu, iki dilde
   metin. Plan/uygula ayrımı `preset`/`worktree`/`release` deseniyle, ve **plan uygula
   tarafında yeniden yapılıyor** (`provider`'ın kuralı: "düğmeyi sunan ekran dakikalar
   önce olabilir"). Kararlar yazılı: yalnız **gerekli** bağımlılıklar izleniyor (isteğe
   bağlı olan zaten odağın durdurmak istediği şey), bir servisin **her** örneği korunuyor
   (manifest 8.0 ile 8.4 arasında seçim yapamaz ve yanlış tahmin projenin bağlı olduğu
   veritabanını durdurur), ve **hiçbir servis beyan etmeyen proje reddediliyor** —
   `services`'in boş hâli "beyan yok", "ihtiyaç yok" değil, ve ona göre davranmak tüm
   workspace'i doldurulmamış bir alan uğruna durdururdu. *(U-2)*
5. ✅ ~~**Açık kaynak dosyaları.**~~ (yapıldı) — `dependabot.yml` (üç ekosistem, gruplu,
   haftalık; **major'lar gruptan çıkarıldı** çünkü Vuetify 4 ve Pinia 4 bump değil göç,
   ve altı yama güncellemesiyle karışmış bir PR ne incelenebilir ne geri alınabilir),
   `CODE_OF_CONDUCT.md`, `SUPPORT.md`, `ISSUE_TEMPLATE/config.yml` (boş issue kapalı,
   güvenlik advisory'ye yönlendirildi) + `feature_request.yml`, `.editorconfig`.
   `.nvmrc` N1'de yapılmıştı.

   **R-15 karşılandı:** `SUPPORT.md`'de "What this project can promise" — ücretsiz, MIT,
   tek kişi, **fon yok, destek sözleşmesi yok, yanıt süresi taahhüdü yok**. Düz yazıldı,
   çünkü alternatifi birinin tahmin etmesi. Kurumsal alıcının sorduğu soru bu.

   **Ve iki belge bağlantı gate'ine eklendi** (`LINKED_DOCUMENTS` 6 → 8), README'nin
   sonuna dördünü sayan bir bölüm kondu — yoksa GitHub bulur, okuyucu bulmaz. Aynı
   turda README'nin bayat `contracts:check` sayısı da düzeltildi: "altı uyarı" diyordu,
   bire düşmüştü. *(P4 + R-15)*
6. ❌ **Homebrew cask + winget manifesti** (yapılmadı) — bilerek sıradan sonra. `release.yml`
   altı hedefin sha256'sını zaten üretiyor, ama bir cask **gerçek bir indirme URL'si ve
   gerçek bir sha256** istiyor: ikisi de N3 #3 etiketlendikten sonra var olur. Şimdi
   yazılan bir manifest, doğrulanamayacak iki alan taşıyan bir dosya olurdu. *(R-9)*
7. ✅ ~~**`installers:check` script'ini düzelt.**~~ (yapıldı) — **ama teşhis düzeltilerek.**
   Ölçüldü: CI aracı **doğrudan** çağırıyor (`release.yml:534`), npm script'ini değil.
   Yani script'in hiç çağıranı yok ve kırık olan CI değil, script'in kendisi.

   Ve `--target`'ı varsayılana bağlamak **yanlış düzeltme** olurdu: aracın var oluş
   sebebi ana makineye sessizce düşmüş bir çapraz derlemeyi yakalamak, bir varsayılan
   üçlü onu yakalaması gereken hatayla anlaştırırdı. Bunun yerine çıkış eyleme
   dönüştürüldü — `rustc -vV`'den bu makinenin üçlüsünü basıyor ve npm'in yuttuğu `--`'yi
   söylüyor, yani çıkmaz değil kopyalanabilir bir satır. `package.json`'daki girinti
   farkı da düzeltildi; o zaten satırın hiç koşturulmadığının işaretiydi. *(P2-3)*
8. ✅ ~~**Şablon ↔ tespit kapsama testi.**~~ (yapıldı) — **ve teşhisin bulmadığı iki hata
   çıktı.** `tests/scaffold_coverage.rs` (3 test) iki katalogu birbirine bağlıyor;
   `detect::FRAMEWORKS` yazıldı ve `detect.rs`'te bir **erişilebilirlik testi** ile
   tutuluyor: her ada, `infer`'i o adı döndürmeye zorlayan gerçek bir checkout şekli.

   **Bulunan birinci hata — ölü kural.** `statamic/cms` kuralı `artisan` kuralının
   *altındaydı*, ve her gerçek Statamic sitesi Laravel'in `artisan`'ını taşır. Yani
   kural yalnızca Statamic **olmayan** bir depoda ateşleyebilirdi: destek gibi okunuyordu,
   hiçbir şeyin desteğiydi. PrestaShop aynı şekilde Symfony'nin `bin/console`'unun
   arkasındaydı ve **symfony olarak** cevaplanıyordu. İkisi de artık "üzerine kurulduğu
   çatıdan önce sorulan dağıtımlar" bloğunda. Bu sınıf hatayı başka hiçbir test bulamaz:
   parmak izleri elle kuruluyor, dolayısıyla bir kural için yazılmış test tam olarak o
   kuralın alanlarını dolduruyor ve komşusuyla hiç çarpışmıyor. Yalnız **hepsini birden**
   sormak buluyor.

   **Bulunan ikinci hata — yanlış belge kökü.** Tespit tablosu tek bir `public` yedeğine
   düşüyordu; Magento `pub/`'dan, CakePHP `webroot/`'tan servis eder ve ikisinde de
   `document_root` dayanacak bir şey bulamaz. Yani var olmayan bir dizin adlandırılıyordu
   — kurulan, başlayan ve 404 veren proje, yani hiçbir yerde hatası olmayan hata. Artık
   her satır kendi kökünü taşıyor; dizinin söylediği hâlâ kazanıyor (kendi kökünden
   servis eden bir Drupal 7 kurulumu oradaki `index.php` ile cevaplanıyor, güncel
   major'ün geleneğiyle değil).

   **Eklenen tespitler:** `yiisoft/yii2` → yii (`web`), `typo3/cms-core` → typo3,
   `laminas/laminas-mvc` → laminas, `prestashop/prestashop` → prestashop. Sonuncusu
   `composer.json`'ın **`name`** alanından okunuyor — yeni bir parmak izi alanı
   (`composer_name`), çünkü `create-project` paketin kendi manifestini kopyalar ve
   PrestaShop'un require'ları yarım kalmış bir kurulumda henüz yazılmamış olabilir.

   **Eklenen şablon:** Statamic (29 şablon). **Magento eklenmedi ve nedeni yazıldı:**
   `repo.magento.com` kullanıcının kendi Adobe hesabında ürettiği bir anahtar çiftiyle
   kimlik doğruluyor, yani düğme kullanıcı başka bir yere gidip kimlik bilgisi alana
   kadar **her zaman** başarısız olurdu — üstelik tek dönen şey, adı hiç geçmemiş bir
   alan adından gelen 401 composer hatası. Benimseme çalışıyor; oluşturma ekosistemin
   işi. `remix` de aynı şekilde beyan edilmiş boşluk: `create-remix` artık React Router
   v7 kuruyor, ama insanların elindeki checkout'lar hâlâ `@remix-run/dev` taşıyor, o
   yüzden **tespit kalıyor, şablon gelmiyor**.

   **Ve dört elle yazılmış liste bağlandı:** Rust enum'u, `contracts/ipc.json` union'ı,
   drawer'ın üç haritası (`TEMPLATE_GROUPS`/`TEMPLATE_RUNTIME`/`TEMPLATE_ICONS`) ve iki
   locale dosyası. Her biri farklı şekilde sessizce bozuluyor: drawer'da olmayan şablon
   seçilemez, enum'da olmayan şablon kullanıcı listeden seçtikten *sonra* "bilinmeyen
   şablon" ile reddedilir, locale'de olmayan şablon yeni kullanıcının gördüğü ilk ekranda
   çeviri anahtarı olarak görünür. Bölüm bazlı arama, dosya bazlı değil — `astro`
   drawer'ın düzyazısında da geçiyor ve tüm dosyayı tarayan bir kontrol bir yorumla
   tatmin olurdu. *(R-4 + B-5)*
9. ✅ ~~**Üç sağlayıcı reçetesi sevk et.**~~ (yapıldı) — **ama üçü rapordakiler değil, ve
   nedenleri ölçüldü.** `provider::RECIPES` (üç reçete), `provider_recipes` ve
   `provider_recipe_add` IPC komutları (313. ve 314.), `ProvidersPane`'de başlangıç
   noktaları bölümü, iki dilde metin ve yardım belgesi. 5 Rust + 1 manifest + 6 Vue testi.

   **Sevk edilenler:** `mysql-remote` ve `postgres-remote` (iki yön de var; parola
   `MYSQL_PWD`/`PGPASSWORD` ile ortamdan okunuyor, yani hiçbir zaman argüman olmuyor) ve
   `upsun` (yalnız çekme). Üçü de **ölçülerek** seçildi: `ghcr.io/upsun/cli:latest`
   çalıştırıldı, `db:dump --directory --file` doğrulandı, `UPSUN_CLI_TOKEN`'ı ortamdan
   okuduğu görüldü; `mysqldump --result-file`, `mysql --execute=source`, `pg_dump --file`
   ve `psql --file` resmi imajlarda tek tek kontrol edildi.

   **SSH+mysqldump sevk edilemedi ve sebep eksik kabuk değil.** Modülün kendi başlığındaki
   kural: depo tarafından beyan edilen konteyner **hiçbir host yolu almıyor**, yani ajan
   soketi de anahtar dosyası da yok. `ssh` ise ya bir *dosyayla* ya bir *ajanla* kimlik
   doğruluyor; buradaki sır bir ortam değişkeni olarak geliyor ve birini diğerine çeviren
   bir argv yok. (`mysqldump --result-file` zaten boru istemiyor — engel oradaki değildi.)
   Kapatmak, bir tarifin sırrının scratch dizininde **dosya olarak** maddileştirilmesini
   istemesine izin vermek demek; bu modülün tehdit modeliyle ilgili bir karar, bir reçete
   değil, ve burada verilmedi. **Açık soru olarak yazıldı.**

   **Pantheon iki bağımsız sebeple sevk edilemedi:** kontrol edilen hiçbir registry'de
   Terminus imajı yok (DDEV CLI'ı kendi web konteynerine kuruyor, imaj çalıştırmıyor), ve
   `terminus backup:get` indirilmesi gereken bir URL döndürüyor — ikinci bir komut,
   dolayısıyla bir kabuk.

   **Kart artık hiçbir şey beyan etmeyen projede de görünüyor.** Eskiden gizleniyordu ve
   gerekçesi *"'sağlayıcı yok' diyen bir panel kimsenin sormadığı soruya cevap verir"*
   idi — sunacak bir şey yokken doğru, olduğu anda yanlış. Beyan eden proje bu soruyu
   geçmiş olduğu için ona gösterilmiyor.

   **Eklemek hiçbir şeyi onaylamıyor.** Tarif manifeste yazılıyor ve elle yazılmış bir
   tarifle aynı digest'ten geçiyor; her sevk edilen reçete bir yer tutucu taşıdığı için
   onaylanan sürüm tanımı gereği eklenen sürüm değil, ve o farkı kapsayan digest ikisini
   iki ayrı eylem yapan şey. Yazma `manifest::write` üzerinden — form kaydının kullandığı
   serileştirici — çünkü dosyaya JSON eklemek tek dosya için ikinci bir yazıcı olurdu ve
   ilk ayrışma kimsenin yazmadığı bir `Problem` olarak görünürdü. Aynı adı taşıyan bir
   tarif **değiştirilmiyor, reddediliyor**: diskteki kopya tanımı gereği düzenlenmiş
   (çalışabilmiş olan tek sürüm o) ve "ekle" yazan bir düğmeye karşılık birinin bulup
   yazdığı sunucu adını sessizce atmak olurdu. *(R-5)*
10. ✅ ~~**MSSQL ve Beanstalkd paketleri.**~~ (yapıldı) — paketler
    `stackvo-service-packages`'te (33 servis, 122 sürüm), uygulama tarafında sözcük
    dağarcığı, sürücü eşlemesi ve **iç çelişkiyi kural hâline getiren yeni bir gate**.

    **MSSQL:** `mcr.microsoft.com/mssql/server`, 2022 ve 2025. Ölçüldü, README'den
    okunmadı: bir M serisi Mac'te öykünmeyle **başlıyor** ve `SELECT @@VERSION`
    (16.0.4265.3) cevap veriyor — yavaş, bozuk değil. Sağlık kontrolü `sqlcmd` değil
    **TCP** yoklaması, ve bu bir güvenlik tercihi: healthcheck komutu konteynerin kendi
    yapılandırmasında duruyor, yani `docker inspect` çalıştırabilen her şey SA parolasını
    okuyabilirdi. Daha zayıf yoklama ucuz olan takas — SQL Server 1433'ü bağlantı kabul
    edene kadar açmıyor.

    **Beanstalkd:** `ghcr.io/beanstalkd/beanstalkd`. Yukarı akış tek etiket yayımlıyor
    (`latest`) ve ikilisi `-v`'ye `unknown` diyor, dolayısıyla sürüm dizini doğrulayamadığı
    bir sürümle değil imajın derlendiği tarihle adlandırıldı: `2025.03`. **Bu ağaçtaki ilk
    digest ile sabitlenmiş paket** — `latest`'i adlandırmayı güvenli kılan da bu: etiket
    kayabilir, baytlar kayamaz. İmaj yalnız x86-64; not olarak yazıldı, çünkü alternatifi
    bir yığın izinden öğrenilmesi.

    **`eol.mjs`'e sürüm adı takma tablosu.** endoflife.date SQL Server'ı yıla göre değil
    numaraya göre anıyor — dünyanın 2022 dediği içeride 16.0 — ve eşleme olmadan satır
    "kontrol edilmedi" olarak raporlanıyordu. Yani deponun kendi standardı
    (*"'destekleniyor' bir ölçüm olmalı, bir görüş değil"*) tam da en net takvimi yayımlayan
    sağlayıcı için tutulmayacaktı. Artık ölçülüyor: 2033-01-11 ve 2036-01-06.

    **Uygulama tarafı:** `env.schema.json` sözcük dağarcığına iki id (31 → 33),
    `detect.rs`'te `DB_CONNECTION=sqlsrv` → mssql ve `QUEUE_CONNECTION=beanstalkd` →
    beanstalkd, `agentctx.rs`'e iki konteyner portu. İkisi de aynı şekildeydi: **kural
    buradaydı, servis yoktu** — ve kataloğun taşımadığı bir servise çözülen değer zaten
    düşürülüyordu (`every_service_a_rule_can_produce_is_in_the_catalog`'un kendi
    gerekçesi). Yani SQL Server'ı ya da Beanstalkd'yi çoktan seçmiş bir projeyi benimsemek,
    hiç veritabanı ve hiç kuyruk beyan etmemiş bir proje olarak okunuyordu.

    **Ve çelişki bir kural oldu:** `every_php_driver_that_needs_a_server_has_one_in_the_catalogue`
    — PHP imajının sunduğu her sürücü için katalogda bağlanacak bir şey olmalı. Yalnız
    **sunucu** isteyen çiftler; `pdo_sqlite` servis istemiyor ve uzantı başına paket isteyen
    bir kural kimsenin tutamayacağı bir kural olurdu.

    **Kalan tek adım kullanıcıda:** `registry.json` yeniden üretildi (sıra 19) ama
    **imzalanması gerekiyor** — anahtar bilerek CI sırrı değil, elle imzalanıyor
    (`tools/keys.sh sign registry.json`). İmzalanmazsa her kullanıcının kataloğu
    düşürülmez, **reddedilir**. Yan iş olarak `schema/` kopyalarındaki mevcut sapma da
    giderildi (uygulama deposu ADR atıflarını temizlemişti, kopyalar eskimişti) — o
    gate zaten kırmızıydı. *(R-7)*
11. ✅ ~~**`debugbridge`'e kuyruk işi ve istek olayları.**~~ (yapıldı) — **ama raporun
    reçetesi ölçülünce uygulanamaz çıktı, ve uygulamaya çalışmak iki gerçek hata
    buldu.** `kind` artık üç değer taşıyor (`dump`, `request`, `job`),
    `timeline::Source` beş, ve pane'de sinyal başına bir süzgeç var.

    **İstek olayları — köprüden, çerçevesiz.** Bir `register_shutdown_function`,
    prepend anında kaydediliyor: her çalıştırma için bir satır, HTTP durumu ve
    süresiyle. Bir çerçevenin "istek karşılandı" olayına bağlanmamasının sebebi
    yedeklilik değil — PHP'nin kendi kancası **ölümcül hatada ve `exit()`'te de**
    koşuyor, ve hiçbir şeyin yüklenmiş olmasını istemiyor. Yani satır Laravel için
    de, Symfony için de, WordPress için de, elle yazılmış bir `index.php` için de
    çıkıyor. Saat `REQUEST_TIME_FLOAT`, yani süre otoyükleyiciyi ve çerçevenin
    açılışını da kapsıyor. Konteynerde ölçüldü: `200`, kasıtlı bir `503`, ve
    `dd()`'nin `exit(1)`'inden **sonra** yine yazılan satır.

    **Raporun dört dinleyicisi mümkün değil, ve bu ölçüldü.** `auto_prepend_file`
    konteyner var olmadan koşuyor, dolayısıyla bağlanılacak bir olay yok. Otomatik
    yükleyiciyi gözlemek yol gibi görünüyor ve değil: **Composer kendi yükleyicisini
    `$prepend = true` ile kaydediyor**, yani prepend dosyasının kaydettiği her şeyin
    *önüne* geçiyor ve çözebildiği bir sınıf asla arkadakine ulaşmıyor. Gerçek bir
    Laravel 12 checkout'unda `spl_autoload_functions()`
    `[Composer\Autoload\ClassLoader::loadClass, <köprü>]` döndü, bu sırayla, ve
    kuyruk olayları hiçbir şey ateşlemedi.

    **Bu yüzden iş yarısı ana bilgisayardan geliyor — `querylog`'un argümanıyla.**
    Yeni `queuelog.rs`, `queue:work`'ün kendi iki sütunlu çıktısını okuyor
    (`App\Jobs\Hello ... 40.80ms DONE`) ve satırları aynı olay dosyasına
    `kind: "job"` olarak ekliyor — yani pane, zaman ekseni, MCP araçları ve istek
    açıklayıcısı tek okuyucuda kalıyor. Saat konteynerin bastığı değil **motorun
    `--timestamps` öneki**: içerideki damga imajın derlendiği saat diliminde ve
    aynı eksendeki bir dump'tan saatlerce uzağa düşerdi. Uçtan uca doğrulandı:
    boş bir işçide tohum, sonra üç satır (`ok`, `failed`, `failed`), sonra
    tekrar çağrıldığında **0** — ve yeni bir iş sonrası yine üç.

    **Bulunan birinci hata — işçi köprüyü hiç taşımıyordu.** Köprü web
    konteynerine compose kaplamasıyla ulaşıyor, ama kuyruk sidecar'ı `docker run`
    ile kalkan ayrı bir konteyner ve tek bir `-v` alıyordu. Yani **bir kuyruk
    işinin içindeki `dump()` hiçbir yere yazılmıyordu** — üstelik hem modülün
    kendi başlığı hem `debug_bridge_set`'in sözleşme notu bunun çalıştığını
    söylüyordu. Aynı iş, mount'lu ve mount'suz iki konteynere karşı koşularak
    ölçüldü. `run_args` artık üçünü de veriyor, ini bir **dosya** mount'u olarak
    (dizini bağlamak imajın her ini'sini, profilcinin kendisininkini de gölgeler).

    **Bulunan ikinci hata — `spl_autoload_register`'ın `$do_throw`'u.** `false`
    geçmek PHP 8'de bir notice bastırıyor, ve bir prepend dosyasının herkesin
    yanıtına yazdığı notice bu dosyanın yapabileceği en görünür şey olurdu. İlk
    konteyner koşusunda stdout'ta çıktı.

    **Ve bir üçüncüsü, kendi kodumda, koşturarak bulundu:** yakalama açıldıktan
    hemen sonra çalıştırılan **ilk iş yutuluyordu**. İmleç yokken tohum "işçinin
    en yeni satırı" idi, ve o satır tam da az önce çalıştırılan işin satırıydı.
    Hiç çıktı basmamış bir işçi artık **sıfırdan** tohumlanıyor — "geçmişten
    sonra başla" ile "hiçbir şeyden sonra başla" farklı cümleler — ve tohum
    anahtarın kendisinde atılıyor, bir poll sonra değil.

    **Söylenemeyen şey yazıldı:** `FAIL` her **denemede** basılıyor ve konsol
    bir sonraki denemenin gelip gelmeyeceği hakkında hiçbir şey söylemiyor, o
    yüzden üç satır üç satır olarak duruyor; ve okunan, bu uygulamanın başlattığı
    işçi — kendi terminalinde `queue:work` koşturan biri satır üretmiyor.
    *(R-8)*

---

## N5. Bakım borcu

**Durum: beş maddenin dördü kapandı, biri açık.** Hiçbiri yayını tutmuyor; hepsi
yayından sonra ölçülebilir hâle gelen ya da yayın sonrası maliyeti düşüren işler.

1. ✅ ~~**Rust kapsam tabanını ölçülen değere yaklaştır.**~~ (yapıldı) — **ve önce yeniden
   ölçüldü, çünkü asıl arıza oradaydı.** Kayıtlı sayı 7 Ağustos'ta donmuştu (%64,05) ve
   ağaç o gün bu güne dört puan kazanmıştı: gerçek ölçüm **%68,05**, yani taban ölçümün
   dört değil **sekiz** puan altındaydı — bu bir gerileme alarmı değil, süstür.
   Taban **%65**'e çekildi, ve mesafe artık aritmetik: ölçüm eksi bir puan platform
   (macOS/Ubuntu farkının deponun kendi tarihindeki değeri) eksi iki puan
   "testleri henüz yazılmamış 1.200 satırlık bir modül" (59,5k satırın üzerinden).
   Ön yüz de aynı şekilde: %89,65 → **%92,20** ölçüldü, taban 85 → **90**, dal 72 → **78**.

   **Ve kapıya bayatlama uyarısı eklendi.** Bu maddeyi doğuran hata "taban çok düşük"
   değil, *"kayıtlı ölçüm bayatladı ve kimse fark etmedi"*ydi. Rapor kayıtlı sayının iki
   puan üstüne çıktığında artık bunu söylüyor — kazanç olduğu için hata değil uyarı, ama
   söylenmemesi çiftin sessizce ayrılmasının yoluydu. Uyarının kendisi kasıtlı bir bayat
   değerle ateşlenerek doğrulandı. *(P2-1)*
2. ✅ ~~**Docker'lı nightly duman testi.**~~ (yapıldı) — `tests/docker_smoke.rs` +
   `.github/workflows/nightly.yml`. Test geçici bir workspace kuruyor, manifesti yazıyor,
   **uygulamanın kendi `write_generated`'ı** ile üretiyor, `docker compose up --build`
   koşuyor ve sayfayı iki yerden istiyor: konteynerin içinden (`nginx` + `php-fpm` +
   bağlama + `document_root`, `PHP_SAPI`'yi basarak — nginx kaynağı düz metin verirse
   test düşüyor) ve Traefik üzerinden HTTPS ile (etiketin gerçekten router olduğu).

   **Kendi makinesinde koşturulamadı ve bu bir kapı hâline getirildi.** Üretilen compose
   projesinin adı her makinede `stackvo`, konteynerleri `stackvo-*`, ağı `stackvo-net` —
   yani ikinci bir yığın ikinci bir yığın değil, *aynısı*. `--remove-orphans` ile geliştirici
   kendi kurulumunun konteynerlerini sildirirdi. Test bu yüzden **atlamıyor, reddediyor**,
   ve reddetme yolu bu makinenin canlı yığınına karşı koşturularak doğrulandı.

   Koşulamayan yarının **kurulum yarısı** normal takıma alındı (`--build`'e ihtiyacı yok):
   servis üretiliyor mu, etiket doğru alan adını mı taşıyor, derleme bağlamında Dockerfile
   var mı, ve `compose_file_list`'in vereceği her dosya diskte mi. Gece yarısı bir dosya
   adı yüzünden kırmızıya düşmek yerine commit anında düşüyor.

   **İki sürüklenme kapısı:** nightly'nin apt listesi CI'ınkinin üst kümesi olmak zorunda
   (`workflow_parity.rs`, release ile aynı kural bir halka ötede), ve iş akışının aradığı
   "atlandı" cümlesi testin bastığı cümle olmak zorunda — yoksa yeşil bir job "koştu" mu
   "hiç koşmadı" mı söylemez, ki `driver` job'ının kendi yorumu bu yanığı anlatıyor.
   İkisi de kasıtlı bozmayla doğrulandı. *(P2-2)*
3. ✅ ~~**Paket bütçesine pay yarat.**~~ (yapıldı) — **ve tavan aşılmıştı, dar değildi.**
   Ölçünce `total` **3001,5 KB / 3000** çıktı, yani build o adımda zaten kırmızıydı.
   Dosyanın kendi sözü ikinci kez "kabul et ve yükselt" demeyi kapatıyordu, o yüzden
   kırpıldı.

   **Kırpılan şey `totalKb` yorumunun adıyla saydığı şeydi:** *"kazara gelen bir bağımlılık
   — tam bir ikon fontu"*. Material Design Icons 7.447 cam kuralı, **408 KB**, ve bu
   uygulama 313 tanesini adlandırıyor. Build artık yalnız erişilebilen kuralları basıyor:
   stil dosyası 408 → **30 KB**, eager 1547 → **1246 KB**, total 3001,5 → **2700,9 KB**.
   Font'a dokunulmadı (394 KB, bir font zinciri ister) ve bu bir sonraki adım olarak
   yazıldı. Eager tavanı 1700 → **1400**'e *indirildi*: 1246'nın üzerinde 1700 %36 boşluk
   olurdu, ve bir bütçenin alarm olmaktan çıktığı yer o boşluğun içidir.

   **Ve kırpma iki gerçek hata ortaya çıkardı:** `mdi-discord` iki major sürümdür ikon
   setinde yok (marka işaretleri kaldırıldı) ve `mdi-format-textdirection-r-to-l` yeniden
   adlandırılmış — yani Discord bağlantısının ve Yerelleştirme panelinin yön başlığının
   ikonu **bugün boş kare çiziyordu**. Eksik cam, bir ekranın yapabileceği en sessiz hata:
   hiçbir şey uyarmıyor, ve boşluk bir aralık tercihi gibi okunuyor.

   Kullanılan ikon listesi tek kaynakta (`tools/mdi-icons.mjs`), üç okuyucusu var: kırpan
   eklenti, derlenmiş stil dosyasını listeye karşı tutan bütçe kapısı (derleme sonrası —
   emitted dosyanın var olduğu tek an), ve listeyi ikon setine karşı tutan test. **Vuetify'ın
   kendi 26 ikonu** (onay kutusu, sıralama oku, sayfalama) `src/`'de hiç geçmiyor; yalnız
   bu ağacı tarayan bir liste onay kutuları boş bir uygulama üretirdi — ilk sürümü tam
   olarak bunu yaptı. *(P2-4)*
4. ✅ ~~**`commands.rs`'i alt sistemlere böl.**~~ (yapıldı) — **ama dosyayı bölerek değil,
   ölçerek.** ARCHITECTURE bu maddeyi zaten tartmış ve reddetmişti: *"dosyanın boyutu
   rahatsız edici, zararlı değil, ve bölme her komuta aynı anda dokunur."* O cümle
   yazıldığından beri dosya 12,7k → **16,07k** büyümüştü, yani karar yeniden ölçülmeyi
   hak ediyordu.

   **Ölçüm kararı doğruladı, ama başka bir şeyi de gösterdi.** Dosyada 391 fonksiyon var
   ve ortalaması **27 satır** — yani gerçekten "ince fonksiyonlar dizini", ve üç yüz ince
   fonksiyonu bir dizine taşımak her komuta dokunup karşılığında yalnız daha kısa bir
   dosya verirdi. Ama en uzun maddeleri **komut değildi**: yüz ile üç yüz satır arasında
   beş özel yardımcı, `&Path` alıp düz değer dönen, aralarında tek bir Tauri tipi
   olmayan. Yani **konusundan bir bant yukarıda duran alan mantığı**, ve bedeli kuralın
   kendi vaadiydi — yukarıdaki bir fonksiyon ne testten, ne `diagnose` örneğinden, ne MCP
   yüzeyinden çağrılabilir. Beşinin üçünün `mcp.rs`, `cli.rs` ve `examples/diagnose.rs`'te
   **yukarı doğru** `commands`'a uzanan çağrıcıları vardı; bant oku o yöne çizilmiyor.

   **1.220 satır konusunun yanına indi:** üretilen ağacın tamamı (render + verify + write,
   577 satır) → `generator`, göç önizlemesi (207) → `handover`, worktree planı (381) →
   `worktree`, ve "bir projenin dizini ve manifesti" → `workspace`. `commands.rs`
   16.070 → **14.850** satır, 308 komutun 308'i yerinde, ve taşınan tek satır Tauri state
   almıyor — bant kapısı iki yönden de tutuyor.

   **Her biri ulaşılabilir olduğu için test kazandı.** Üçü de yazılamayacak testlerdi:
   göç etmiş bir workspace'in ikinci kez planlanmaması (paket ağacı gerçek olmak zorunda —
   ilk sürümü boş katalogla yazıldığı için reddi silince de geçiyordu, o yüzden atıldı ve
   fikstürlü dosyaya taşındı), reddedilen bir worktree planının yine de adını ve alan adını
   taşıması, ve `scope_includes`/`service_source`'un kendi modüllerindeki testleri.
   Üçü de kasıtlı bozmayla kırıldı, sonra geri alındı. *(P3-1)*
5. ❌ **Yedi major bağımlılık geçişi** (yapılmadı) — Dependabot açıldıktan **sonra**, yayın
   öncesi değil. *(§7)*

---

## N6. Arayüz ve hardcode borcu

**Durum: on dört maddenin on biri ✅ yapıldı (üstü çizili), üçü ⚠️ yarım — ve üç yarımın
üçü de bir eksik değil, yazılı bir sebep:**

* **#7** — makine geneli `commands.json` yapıldı; **paketin komut getirmesi** yapılmadı,
  çünkü imza zincirinden geçen ayrı ve daha büyük bir parça.
* **#12** — sürüm notu **zaten gösteriliyordu** (ölçüldü; yapılan tek şey notu `lang=""`
  ile işaretlemek oldu, çünkü o metin yayıncının ve dili bilinmiyor). İkinci kanal ilk
  yayına bağlı ve gerekçesi `channel.rs`'e yazıldı: bugün bir ayar eklemek, kimsenin
  yayımlamadığı bir kanalı seçen ve güncellemeleri **sessizce durduran** bir ayar kurmak
  olurdu.
* **#14** — çökme **bildirimi** yapıldı (asıl eksik oydu: uygulama çöküyor ve söylemiyor);
  **göndermek** yapılmadı, çünkü `PRIVACY.md` "telemetri yok, çökme raporlama servisi yok,
  sunucu yok" diye söz veriyor ve gelecekteki her şeyin önce orada yazılı olacağını
  söylüyor. Gönderilecek yer eklemek kod değil ürün kararı.

Kendi içinde sıra: kullanıcının bugün yapamadığı şeyler → tema tutarlılığı → sürüklenme
kapıları → yazılmamış kararlar.

1. ✅ ~~**Altı `*_VERSIONS` listesine `PhpPane`'de `v-combobox`.**~~ (yapıldı) — ve ilk iş
   raporun teşhisini düzeltmek oldu. *"`src/` ağacında bir kez bile geçmiyor"* doğru ama
   *"okunmuyor"* değil: `build_catalog` anahtarı `format!("SUPPORTED_LANGUAGES_{key}_VERSIONS")`
   ile **kuruyor**, yani anahtar-anahtar arama sıfır gösterirken zincir tıkır tıkır
   işliyordu — `.env` → `build_catalog` → `catalogGet` → `PhpPane`'in seçicisi. Yani bu
   altı liste her seçicinin ne sunduğuna karar veriyor, ve bayat olduklarında kullanıcının
   çıkışı yoktu.

   `RUNTIME_DEFAULTS`'ın yanına `VERSION_LISTS` kondu (altı anahtar, PHP dahil — onun
   varsayılanı bir grup aşağıda kendi seçicisinde), ve çalışma zamanları grubuna bir
   açılır bölüm: her liste için `multiple chips closable-chips` bir `v-combobox`,
   `PHP_DEFAULT_TOOLS`'un deseniyle. Kayıt defterinde olmayan bir sürüm imaj derlenirken
   motorun kendi mesajıyla düşüyor — aynı takas.

   **Ve bir kapı**, çünkü asıl hata bu maddenin kendisiydi: Rust'a yedinci bir çalışma
   zamanı eklendiğinde `build_catalog` onu anında okur, seçici anında sunar, ve elle
   yazılmış ön yüz listesinde olmadığı için **yine kimse düzenleyemez**.
   `tests/version-lists.spec.js` `config.rs`'i metin olarak okuyup iki listeyi eşitliyor;
   sahte bir yedinci anahtarla kırıldığı doğrulandı. Panel testi de öyle — ve panelin
   kapalı açılır bölümü içeriği mount etmediği için testin onu **açması** gerekti; açmayan
   bir test kontroller silindikten sonra da geçerdi. *(A-1)*
2. ✅ ~~**Uygulama seçicilerine "diğer…" seçeneği.**~~ (yapıldı) — dört listenin dördüne de.
   Önce raporun örneklerini düzeltmek gerekti: **Neovim ve Vim zaten listede** (`EDITORS`'ın
   son iki satırı), yani üç örneğin ikisi yanlıştı. Helix, Emacs ve listelenmeyen JetBrains
   IDE'leri doğru: onlar için seçilecek hiçbir şey yoktu.

   `apps::CUSTOM` her listenin sonuna eklenen gerçek bir satır, her zaman seçilebilir,
   ve karşılığı `preferences.json`'da kendi anahtarında duruyor — `editorCustom`,
   `terminalCustom`, `browserCustom`, `dbClientCustom`. Seçim ile komut **ayrı iki anahtar**:
   biri "hangisi", öbürü "ne çalıştırılacak"; tek dizgede tutulsaydı "VS Code'a geri dön"
   demek özel komutu ezmek olurdu.

   İki özellik bilerek korundu. **Otomatik seçim asla özel satır olamaz** — `offer()`
   önce `mark_default`'ı çalıştırıp satırı ondan sonra ekliyor, ve `open_in_editor`'ün
   kurulu editörleri gezen yolu onu atlıyor; kimsenin yazmadığı bir komutu yedek olarak
   başlatmak yedeksizlikten kötüdür. **Ve kabuk yok**: `split_command` yalnız tırnak
   ayırıyor, kelimeler `Command::new`'e doğrudan gidiyor, yani `$HOME`, `&&`, boru ve
   yönlendirme düz metin. Ters bölü kaçış *değil*, çünkü Windows'ta yazılacak ilk şey
   tırnaklı bir mutlak yol ve `C:\Program Files\...` üç kaçış dizisi olarak okunsaydı
   özelliğin var olma sebebi bozulurdu.

   Terminalin kendi tipi var (`apps::Terminal`), çünkü terminale bir yol değil bir **komut
   satırı** veriliyor ve her emülatör onu başka bayrakla alıyor. Tek kural: kullanıcının
   kelimeleri önce, komut tek bir son argüman olarak sonra. Bayrak tahmin edilmiyor —
   `-e`, `--`, `start --`, `/K` hepsi farklı, ve yanlış tahmin **yanlış şeyi çalıştıran**
   bir terminal açar; o yüzden bayrak da kutuya yazılıyor.

   **Ve iki ölçülen hata**, ikisi de bunu yazarken çıktı:

   * **On sekiz ikon yalnız Rust'ta adlandırılıyor** ve geçen turun alt kümeleyicisi
     `src/`'yi okuduğu için hepsini siliyordu — yani terminal, editör ve tarayıcı
     seçicilerindeki **her ikon** derlenmiş uygulamada boş kare çiziyordu. `apps.rs` bir
     katalog: `mdi-apple`, `mdi-firefox`, `mdi-powershell` ve on beş tanesi daha hiçbir
     `.vue` dosyasında geçmiyor. Tarama `src-tauri/src`'yi de okuyor artık; eager sete
     maliyeti **2,4 KB**, ölçüldü.
   * Geniş taramanın ilk koşusu **`mdi-vim`**'i buldu: Neovim ve Vim satırlarının ikonu,
     ve **böyle bir ikon hiç olmadı**. Yazıldığı günden beri iki satır boş kare çiziyor.
     Okunmayan bir dosyada bulunamazdı.

   Rust tarafı `.rs` için dar okunuyor — orada ikon adı hep dizgi değişmezi, etrafındaki
   düzyazı ise ikon olmayan adlarla dolu (`mdi-vim` hatanın kendisi olarak yazılı,
   `mdi-icons.mjs` bu dosyanın adı); ilk geniş koşu ikisini de topladı. *(A-3)*
3. ✅ ~~**Grafik ve ısı ızgarası renklerini temadan oku.**~~ (yapıldı) — üçü de.

   **C-1.** Üç pastanın renkleri `useContainerStats`'tan tamamen çıktı ve boyamayı
   `IndicatorPane` yapıyor; sebebi "composable renk bilmesin" değil, buraya ait olan
   rengin **o anki temanın** rengi olması ve temanın ancak bileşenin içinden okunabilmesi.
   Ölçülen slice `primary`, kalan `surface-variant` — konuma göre, anahtara göre değil,
   çünkü üç pasta da *ölçülen* ile *kalan*: kullanılan/boş, indirme/yükleme, okuma/yazma.
   Sparkline gradyanı `[primary, success]`; `success` olması ayrıca önemli, çünkü o üç
   rengin biri ve **renk körlüğü paleti onu yeniden yazıyor** — sabit yeşil, o seçimi tam
   olarak renkten ibaret olan grafikte yok sayıyordu.

   **C-2.** Beş sabit yeşil gitti; hücre artık temanın `success`'i ve yoğunluk **opaklıkla**
   taşınıyor. Beş ton yerine opaklık olmasının sebebi tam da açık tema: hücre kartın kendi
   yüzeyinin üstünde duruyor, yani rampa yüzey ne ise ondan başlayıp tam renge çıkıyor. Beş
   ton bunu yapamaz — hepsi tek bir arka plana göre seçilir. `color-mix` değil `opacity`,
   çünkü bu uygulama makinede hangi WebView varsa onun içinde koşuyor ve `ci.yml` motor
   hakkında iddia kurmayı zaten reddediyor; `opacity` böyle bir iddia gerektirmiyor.

   **C-3.** `#12121a` tek sabit oldu (`appearance::CONSOLE_BACKGROUND`) ve iki yarı da onu
   okuyor. Ayrıca yarısı koşulluydu: `darkConsoles` kapalıyken xterm kendi varsayılanına
   düşüyor, CSS çerçevesi ise `#12121a` kalmaya devam ediyordu — yani 8 piksellik çerçeve
   bulgunun tarif ettiği hâliyle **zaten oradaydı**. Artık ikisi de aynı `computed`'i
   okuyor, kapalıyken çerçeve `transparent`.

   Kapı: `tests/theme-colours.spec.js`, dördü de mutasyonla kırıldı. Kapının kendisi bir
   şey ölçtü — ilk koşusunda **kendi açıklama yorumlarında** düştü, çünkü bu depoda bir
   kuralın gerekçesi kuralın yaşadığı dosyaya yazılıyor ve o gerekçe `#1976D2`'yi anıyor.
   Kapı artık yorumları soyup koda bakıyor. *(C-1, C-2, C-3)*
4. ✅ ~~**`quickcmd`/`oauth`/`tooling` düzyazısını `hints.rs` desenine taşı.**~~ (yapıldı)
   ve sayı düzeltildi. Raporun *"39 cümle, üç katalog"* ölçümü iki yerden yanlış: o üç
   katalogda **37** var (`oauth.rs` 9 değil **7** taşıyor), ama bakılmayan iki katalogda
   **11** tane daha var — `tooling::OWN` (2, `ToolingPane:196`) ve `provider::RECIPES`
   (3 `about` + 6 `edit`, `ProvidersPane:225,233`). Gerçek toplam **47 cümle**.

   Anahtar, kataloğun **zaten taşıdığı id**: `Spec.id`, `Provider.id`, `Tool.id`,
   `Recipe.name`. Yani yazılacak ikinci bir ad yok ve düzeyde tutulacak bir eşleme yok —
   `hints.rs`'in "her iki yarı da tek yerde" pazarlığı, ikinci yarıyı yerel dosyaya
   koyarak. İngilizce Rust'ta kalıyor: CLI'nin bastığı, MCP istemcisinin okuduğu ve
   günlüğe düşen o; pencere çeviriyi gösteriyor, çeviri yoksa İngilizceye düşüyor.

   Tek istisna `provider::Edit`: bir tarifin `edit` listesinin kendi id'si yok ve **konum
   ad değildir** — iki veritabanı tarifi aynı cümleye ihtiyaç duyuyor, konuma göre
   anahtarlansaydı tek cümlenin iki çevirisi olurdu. O yüzden `edit` artık `hints.rs`'in
   tam şekli: `{ key, english }`.

   Kapı `hint_translations.rs`'e eklendi (4 test → 8), dört mutasyonla kırıldı: çevrilmemiş
   satır, İngilizce bırakılmış Türkçe, Rust'tan sapan İngilizce, ve karşılığı olmayan öksüz
   anahtar. Okuyucunun kendisi de bir şey ölçtü — quick command id'lerinin yarısı tireli
   (`migrate-status`, `optimize-clear`), yani Prettier onları **tırnaklı** yazıyor ve eski
   ayrıştırıcı yalnız çıplak tanımlayıcıyı biliyordu: yirmi altının on üçünü sessizce
   düşürüp kalanı eksiksiz diye rapor ederdi.

   **İkinci yarısı — `lang="en"` — uygulanmadı, çünkü artık yanlış olurdu.** Rapor "en az
   işaretle" diyordu; çevirmek daha iyi olanıydı ve yapılan o, dolayısıyla o metinler artık
   pencerenin kendi dilinde. Ama tarama bir şey buldu: `SidecarsPane:77` **başka** sınıftan
   bir metin oluşturuyor — sidecar açıklaması projenin kendi `stackvo.json`'ından geliyor,
   yani dili bilinmiyor. `lang=""` aldı, `LogView` ve `DumpValue` ile aynı kural, ve
   `language-of-parts.spec.js`'in listesine eklendi. *(F-1)*
5. ✅ ~~**Literalleri adlandırılmış sabitlere indir.**~~ (yapıldı) — ve mesele tekrar değil,
   **anlaşmazlıktı**. Üretim kodunda ölçüldü: `stackvo.loc` 13, `stackvo-net` 5, `nginx` 25
   (rapordaki 14/9/7 testleri de sayıyordu). Ama önemli olan sayı değil: her üçünün de
   **birbiriyle çelişen iki türetmesi** vardı.

   `certs::suffix` kırpıp küçültüyor ve boşu yok sayıyordu; on çağrı noktası düz
   `unwrap_or("stackvo.loc")` diyordu. Ölçüldü:

   ```
   DEFAULT_TLD_SUFFIX=            çağrı noktaları ""          certs::suffix "stackvo.loc"
   DEFAULT_TLD_SUFFIX=Shop.LOC    çağrı noktaları "Shop.LOC"  certs::suffix "shop.loc"
   ```

   Boş olanı kullanıcıya ulaşıyordu: `Env::parse` dosyadaki boş değeri gömülü varsayılanın
   önüne koyuyor, yani anahtarı yazıp karşısını boş bırakan bir `.env` her projeye **çıplak
   noktayla biten** bir alan adı veriyor, sertifika ise hâlâ `stackvo.loc` için kesiliyordu.

   Artık üç sabit (`config::DEFAULT_TLD_SUFFIX`, `DEFAULT_NETWORK`, `DEFAULT_SERVER`),
   `EMBEDDED` onlara işaret ediyor, ve üç tek türetme: `Env::tld_suffix` (küçültür),
   `Env::docker_network` (küçültmez — Docker ağ adları büyük/küçük harfe duyarlı) ve
   `Env::default_server`.

   **A-5 gerçekten kapandı.** `SUPPORTED_SERVERS_DEFAULT` beş render noktasına ulaşıyor;
   `Manifest::server_or` seçim yapmamış projenin cevabını veriyor. Ve bunu yaparken
   **rapordaki teşhisin ötesinde bir hata** çıktı: `detect.rs`'in dört `server: "nginx"`'i
   hiç tespit değildi, ama `imports.rs` orayı **gerçek kanıtla** yazıyor (DDEV'in
   `webserver_type`'ı). `detected_spec` ayarı tespitin önünde okuduğu için, Apache'ye
   yapılandırılmış bir DDEV projesi ayarın söylediği sunucuyla içe aktarılıyordu — kanıt
   kaydedilip yok sayılıyordu. `Detected.server` artık `Option`, ve sıra o fonksiyondaki
   diğer her alanla aynı: klasör ne diyorsa o kazanır, ayar hiçbir şey demeyene cevap verir.

   Kapı `tests/named_constants.rs`: iki değer için "tek dosya, tek kez", `nginx` içinse
   **yedek deyimi** yasak — çünkü `"nginx"` aynı zamanda bir sunucunun *adı* ve
   `Server::parse`'ta, geçerli adlar listesinde, nginx'e özgü render'da meşru olarak
   geçiyor; onu topyekûn yasaklayan bir kapı daha kötü bir kod isterdi. Üç mutasyonla
   kırıldı.

   Bir de tooling gerilemesi yakalandı: `validate-contracts.mjs` `EMBEDDED`'ı metin olarak
   okuyor ve satırlar sabit referansına dönünce üçünü birden "`.env`'de yok" diye
   raporladı — aynı regex'in sessizce ölmesinin **üçüncü** kezi. Artık adlandırılmış sabiti
   de çözüyor. *(B-2, A-5)*
6. ✅ ~~**`env.schema.json`'ın `consumers` alanını yeniden ölç ya da sil.**~~ (yapıldı)
   ölçüldü, silinmedi. Sayı 43/39 değil **45/39**: kırk beş yolun otuz dokuzu `core/` altında,
   yani Rust'ın yerini aldığı Bash/Node uygulamasında. Altısı canlı ağacı gösteriyordu.

   Aracın kendisi de `core/`'a bakıyordu, yani her koşuda `exit 2` verip bunu **kimseye
   söylemiyordu** — alanın ölmesini engellemek için yazılmış araç, tam olarak o şekilde öldü.

   `measure-env-usage.mjs` artık bu depoyu okuyor (`src-tauri/src`, `src`, `skeleton`,
   `tools`) ve iki kuralı var. **Bildirim tüketim değildir:** her anahtar `config.rs`'in
   tablolarında yazılı, o dosya sayılsaydı yetmiş ikisi de "aktif" çıkar ve ölçüm hiçbir şey
   söylemezdi. **Yorumlar soyuluyor** — ve bu süs değil: bu depo bir anahtarın ölü olduğunu
   *o anahtarın adını anan bir cümlede* yazıyor, `skeleton.rs` `DOCKER_REMOVE_ORPHANS` ile
   `HOST_PORT_ADMINER`'ı tam da öyle anıyor, ve soyulmamış tarama ikisini de canlı okudu.
   (Aynı yanlış pozitif sınıfının bu turda **üçüncü** görülüşü.)

   Sonuç: 72 anahtarın 72'sinin etiketi ölçümle uyuşuyor, `consumers` 35 yol taşıyor ve
   **hepsi diskte var**. Kapı `tests/env-consumers.spec.js` (6 test): yollar var mı, listeler
   taze taramayla aynı mı, etiketler ölçümle uyuşuyor mu, bildirim sitesi sayılmıyor mu.

   Ve şemanın başlığında **iki bayat iddia** daha bulundu: `source` alanı `.env.example`
   diyordu — **öyle bir dosya yok**, o da Bash uygulamasıyla gitti — ve "159 anahtar"
   diyordu; belgelenen sayı **72**. İlk paragrafı yanlış olan bir sözleşmenin gerisine kimse
   güvenmez, o yüzden ikisi de düzeltildi ve sayı kapıya bağlandı.

   `npm run env:usage` okur, `npm run env:usage:fix` yazar ve ardından Prettier'i koşturur —
   `JSON.stringify(…, 2)` ile Prettier kısa dizilerde ayrışıyor. *(E-1)*
7. ⚠️ **`<root>/commands.json`** (yarım) — İlk yarısı yapıldı, ikinci yarısı (paket
   manifestine `commands` alanı) açık. (Üstü çizilmedi: madde tam bitmedi.)

   **Yapılan.** `<root>/commands.json`, `stackvo.json`'ın `commands` bloğunun **aynı**
   şeması, aynı argv kuralı, aynı konteyner sınırı — ve *aynı ayrıştırıcı*: `parse_from`
   hangi dosyayı okuduğunu bilen tek okuyucu. İkinci bir ayrıştırıcı, `host` reddinin iki
   yerde ayrışması demekti. Yeni bir tehdit modeli yok; zaten alınmış iki kararın birleşimi.

   **Katmanlar arası kural, ve asimetrisi bilinçli: proje kazanır, makine dosyasına
   söylenir.** Projenin komutunu reddetmek, yazarının hiç görmediği kişisel bir dosya
   yüzünden işlenmiş ve paylaşılmış bir dosyayı reddetmek olurdu; sessizce ezmek ise
   kataloğun zaten reddettiği "etiketi yalan söyleyen düğme" hatası. `shadowed()` gölgede
   kalan satırı raporluyor, ve her satır `because` alanında geldiği dosyayı taşıyor.

   Yüzey: `machine_commands` IPC komutu, `MachineCommandsPane` (salt okunur — dosyanın
   kendisi arayüz, DDEV/Lando/dde/Laragon'da olduğu gibi; bir form aynı JSON'u yazmanın
   ikinci yolu olurdu ve bir editör kullanılan ilk anda çelişirdi), iki dilde yardım konusu,
   README bölümü. `stackvo commands` ve `stackvo run` bedavaya aldı — ikisi de zaten
   `for_project`/`resolve` üzerinden geçiyor.

   **Ve bir mutasyon boşluk buldu.** `declared_for`'daki birleştirmeyi silmek — özelliğin
   tamamını hiçbir şey yapmaz hâle getiren tek satır — altı testin altısını da yeşil
   bıraktı, çünkü hepsi iki katmanı elle kuruyordu. Tek arıza biçimi "parçalar hiç
   birleştirilmedi" olan bir özellik, diski okuyan bir teste ihtiyaç duyar; yazıldı ve
   mutasyon artık yakalanıyor.

   **Kalan.** Bir paketin komut getirmesi (`ddev-redis`'in `ddev redis-cli`'yi de
   getirmesi) — imza zincirinden geçen ayrı ve daha büyük bir parça, ve bu maddede
   bilerek ayrı tutuldu. *(A-4 / R-6)*
8. ✅ ~~**`<proje>/stackvo.preset.json` yerleşim kuralı.**~~ (yapıldı) — `preset.rs`'in doğru
   sorunu çözdüğü doğruydu; eksik olan **dosyanın olacağı yerdi**. Dışa aktarma bir dosya
   yazıyor, içe aktarma bir dosya okuyor, ve ikisinin arasında birinin "onu şuraya koydum"
   demesi gerekiyordu — bu özelliğin ortadan kaldırmak için var olduğu cümlenin ta kendisi.
   DDEV'de böyle bir soru yok: dosya `.ddev/config.yaml` ve `ddev start` onu okuyor.

   `preset::CONVENTIONAL_FILE` = `stackvo.preset.json`, manifestin yanında, depoda — yani
   klon onu getiriyor. `.stackvo/` dizini içinde değil, çünkü ona eşlik edecek ikinci bir
   dosya yok: tek dosya için bir dizin, tutacağı bir şey olmayan bir kuraldır.

   Gereksinimler kartı artık bunu görüyor: proje bir ön ayar taşıyorsa ve uygulamak hâlâ
   bir şeyi değiştirecekse bir satır çıkıyor, düğmesi `preset_apply`'a gidiyor — Ayarlar'ın
   içe aktarımıyla **aynı** plan-sonra-uygula komutu, çünkü başkasının klonuyla gelen bir
   dosya, siz bir sayfayı açtınız diye yığınınızı yeniden yazmamalı. Yığın uyuştuğunda satır
   kayboluyor; kalıcı bir afiş üçüncü ziyarette kimsenin okumadığı bir satırdır ve bunun
   önem taşıdığı an klondan sonraki ilk açılıştır.

   **Ve bir ayrışma bulundu:** dışa aktarma `<ad>.stackvo-preset.json` öneriyordu, yani
   aynı fikrin iki yazımı ve yalnız biri okunuyordu — hiçbir okuyucunun bakmadığı bir dosya
   yazan bir dışa aktarma, sessizce çalışmayan bir özelliktir. Tek yazıma indi ve
   `tests/preset-convention.spec.js` iki tarafı eşit tutuyor.

   (Kapı yorumları soyup koda bakıyor — aynı yanlış pozitif sınıfının bu depodaki
   **dördüncü** görülüşü: bir adın neden yanlış olduğunu anlatan cümle o adı içeriyor.)
   *(R-11)*
9. ✅ ~~**Podman rootless soketi** + bir uyumluluk fikstürü.~~ (yapıldı) — ve iş tam da
   bulgunun dediği kadardı: Podman zaten Docker API'sini bir unix soketinde konuşuyor,
   yani o satırın üstündeki her şey çalışıyordu ve eksik olan tek şey **yoldu**.

   İki soket, rootless önce — çünkü rootless Podman'in amacı ve Lerd'in tüm ürününü
   üzerine kurduğu durum:

   ```
   $XDG_RUNTIME_DIR/podman/podman.sock   rootless, kullanıcı başına
   /run/podman/podman.sock               rootful, sistem geneli
   ```

   `XDG_RUNTIME_DIR` **okunuyor**, varsayılmıyor. `/run/user/<uid>` neredeyse her Linux'ta
   doğru, ve sorun "neredeyse"de: dizini tanımlayan şey o değişken, ve onu başka yere koyan
   bir oturum var olmayan bir yol alırken soket hiç bakılmayan bir yerde dururdu.

   **Uyumluluk fikstürü** Podman'ın compat `/version` cevabı, ve içindeki ayrım şu:
   `Version` Podman'ın **kendi** sürümü (`5.1.1`, bir Docker sürümü değil), `ApiVersion`
   ise taklit ettiği Docker API seviyesi (`1.41`). Hiçbir yerde ikisi de bir asgariyle
   karşılaştırılmıyor, ve **Docker olmayan bir motoru çalıştıran şey tam olarak bu**: bir
   "Docker >= 20" kontrolü, API'yi gayet iyi konuşan bir motoru başka bir ürünün sürüm
   dizgesine bakarak reddederdi.

   Motor adı bir **etiket, bir dal değil**: bu uygulamada hiçbir şey dördünden hangisinin
   cevapladığına göre başka türlü davranmıyor.

   **Yazılı bir sınır:** `podman-docker` Podman'ın soketini Docker'ın adı altına kuruyor —
   o paketin amacı bu — dolayısıyla `Engine` olarak okunuyor ve çalışıyor. Orada platformu
   doğru raporlamak, sunucuya ne olduğunu sormak demekti ve cevap uygulamanın sonrasında
   yaptığı hiçbir şeyi değiştirmezdi.

   (Bulgunun ikinci yarısı — karşılaştırmanın README'ye konması — motor paragrafına girdi.)
   *(R-2 ikinci yarısı / §8.6)*
10. ✅ ~~**RoadRunner sürücüsü**~~ (yapıldı) — Octane'ın tam olarak iki sürücüsü var ve bu
    uygulama birini gönderiyordu — Octane kullanan bir proje için bu "eksik özellik" değil,
    kaybedebileceği bir yazı tura: yarısı burada olmayan yarıyı seçmiş.

    Şekil Swoole'unki: `php-cli` imajı, kendi HTTP sunucusu 8000'de, FPM yok, dizin listesi
    yok, Traefik 80'e değil 8000'e yönlendiriliyor. **Ayrım maliyette**: Swoole yorumlayıcıya
    derlenen bir PHP uzantısı (PECL modülü + iki geliştirme kütüphanesi), RoadRunner ise
    PHP ile boru üzerinden konuşan bir **Go ikilisi** — yani PHP derlemesi hakkında hiçbir
    şey değişmiyor. Blok bir indirme ve bir `chmod`.

    `rr get-binary` betiği, sabitlenmiş bir sürüm bağlantısı değil: imajın mimarisine uygun
    derlemeyi kendisi çözüyor, ve bu imaj geliştiricinin makinesi neyse onun üzerinde
    derleniyor.

    **Yedek yolu Swoole'unkinden daha önemli.** `artisan` yoksa Octane yok, ve Octane'sız
    RoadRunner bir `.rr.yaml` ile bir PSR-7 işçisi istiyor — bu uygulamanın kimse adına
    yazamayacağı iki şey. O yüzden Laravel olmayan bir proje aynı portta PHP'nin kendi
    geliştirme sunucusunu alıyor: çalışan bir site, çıkan bir konteyner değil. Swoole on iki
    satırlık bir betiğe düşebiliyor çünkü uzantının kendisi bir HTTP sunucusu; Go ikilisinin
    bir işçi olmadan sunacak bir şeyi yok.

    Ve **hiçbir yapılandırma dosyası üretmiyor**: Octane bir `.rr.yaml` yayımlıyor, o dosya
    projenin; ikincisini üretmek uygulamanın çerçevenin kendi dosyasıyla tartışması olurdu.
    *(R-10)*
11. ✅ ~~**Yazılmamış kararları yaz.**~~ (yapıldı) — beşi de, ve her biri kararın
    yaşadığı yere:

    * **Çalışma zamanı sınırı neden altı+iki** — `manifest::LANG_RUNTIMES` başlığı. Java ve
      .NET bilerek yok: ikisi de `COPY . .` + başlatma komutu değil. Java bir yapı aracının
      (Maven/Gradle) bu uygulamanın modellemediği yaşam döngüsünden bir artefakt üretmesi,
      .NET ise çıktı düzeni SDK sürümüne ve hedef çerçeveye bağlı bir `publish` adımı.
      Eklemek yedinci ve sekizinci satır değil, **ikinci bir üreteç şekli** olurdu. Yedinci
      satırın barajı popülerlik değil şekil: üç komutla kurulup derlenip tek portta başlayan
      bir dil buraya ait ve neredeyse bedava.
    * **CLI neden yalnız İngilizce** — `cli.rs` başlığı. Bir CLI'nin çıktısı insanlar kadar
      **makineler** tarafından okunuyor: `grep`'e giriyor, bir issue'ya yapıştırılıyor, geçen
      sene yazılmış bir CI adımıyla eşleşiyor. Çevirmek bunların hepsini yerel ayara bağlardı
      ve arıza sessiz olurdu. `git`, `docker` ve `kubectl` de bu yüzden İngilizce, ve bu depo
      `consoleLocale`'i zaten tam bu sebeple bir *ayar* olarak taşıyor. İngilizce olmayan
      şey hata kataloğu: `hints.rs` her öneriyi çeviriyor, masaüstü çeviriyi gösteriyor,
      CLI aynı `Hint`'in taşıdığı İngilizce yedeği basıyor.
    * **quickcmd kataloğu neden kapalı** — `CATALOG`'un başlığı. Eski cümle ("buraya satır
      eklemek komut eklemenin tek yolu") **iki kez** doğruluğunu yitirdi; hâlâ doğru olan
      kısım yazıldı: tablo *webview'e* kapalı. Kataloğun kendisinin kapalı kalmasının sebebi
      ayrı: bu on bir satır uygulamanın **başkasının çerçevesi hakkında iddia** ettiği
      şeyler. "Ne yazdıysam onu çalıştır" farklı bir şey, ve iki bildirim katmanı tam onun
      için.
    * **Taşınabilir kurulum neden imkânsız** ve **bulut/Codespaces sınırı** — README, Docker
      takasının anlatıldığı yere. İkisi de eksik değil sınır, ve "bu eksik mi karar mı?"
      sorusunun cevabı kurulumdan **önce** okunmalı.
12. ⚠️ **Kanal ve sürüm başına yükseltme notu** (yarım) — Sürüm notu zaten yapılmıştı
    (ölçüldü), ikinci kanal ilk yayına bağlı ve gerekçesi artık yazılı. (Üstü çizilmedi:
    madde tam bitmedi.)

    **Sürüm notu zaten gösteriliyor** — `Settings.vue`'nun güncelleme kartı `update.notes`'u
    basıyor, `updates.js` onu manifestten okuyor. Bulgunun bu yarısı ölçüldüğünde kapalıydı.
    Bulunan tek şey metnin **işaretsiz** olmasıydı: o not yayıncının, uygulamanın değil, ve
    dili bilinmiyor — `lang=""` aldı ve `language-of-parts.spec.js`'in listesine girdi.

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
    sorusu karşısında gerçek bir cevabı olan gerçek bir soru hâline geliyor. *(§8.3, §8.5,
    U-5)*
13. ✅ ~~**İlk açılış turu**~~ (yapıldı) — Dört Gate vardı ve **hepsi engel**: bir şey eksik
    ya da kurulmamış, halledilene kadar uygulama kullanılamaz. Hiçbiri bir şey tanıtmıyor,
    ve içeri girerken dört kez durdurulmak tanıtım değil.

    `WelcomeTour` bunun tersi bir ekran ve öyle davranıyor: yığın **ayağa kalktıktan sonra**
    geliyor, her noktada bırakılabiliyor, ve bırakmak bir hata durumu değil. Beş tam pencere
    ekranının sonuncusu ve gate olmayan tek ekran — ulaşamayacağınız bir özellikle
    tanıştırılmak, hiç tanıştırılmamaktan kötü.

    **İçeriği M7'nin listesi**, ve seçim ölçülmüş: kimsenin kendi başına bulamayacağı altı
    şey — yedi araçtan içe aktarma, dal başına tam ortam, yavaş isteğin açıklaması, üretim
    imajı, devcontainer ihracı, denetim kaydı. Altısı da yazılmış ve test edilmiş, ve
    **dördü hiçbir kullanıcı belgesinde geçmiyor**. Panoyu tanıtan bir tur, kendini zaten
    tanıtan kısmı tanıtırdı.

    Her kartın gideceği bir yer var: yalnız anlatan bir tur, insanların kapattığı turdur.

    **Bir kez, ama geri alınabilir.** `tourSeen` oturum bayrağında değil tercihlerde, çünkü
    "bir kez"in yeniden başlatmayı atlatması gerekiyor yoksa bir kez değil. Ve Ayarlar'dan
    yeniden açılabiliyor: ilk dakikasında atlanan tek atışlık bir ekran, bir daha asla
    geri gelmeyen bir ekrandır — tek şansı olan her karşılama akışının arıza biçimi bu.

    Yazarken bir sıra hatası yakalandı: tercihler boot'tan **sonra** okunuyor, yani
    `prefs.value?.tourSeen !== true` optional chain'i tercihler inmeden `undefined !== true`
    → doğru cevaplıyordu ve tur, çoktan kapatmış biri için her açılışta bir an parlardı.
    Koşul artık tercihlerin **yüklenmiş olmasını** şart koşuyor. *(§8.2, U-6)*
14. ⚠️ **Çökme raporu** (yarım) — Bildirme yapıldı, gönderme yapılmadı ve sebebi
    aşağıda yazılı. (Üstü çizilmedi: madde tam bitmedi.)

    Ölçüm iki şey buldu. **Rapor zaten teşhis paketine giriyor** — `diagnostics::parts`
    günlük dizinini gezip `crash-` ile başlayanları `crashes/` klasörüne koyuyor; bulgunun
    "iliştirilir" dediği yarı zaten kapalıydı.

    **Göndermek yapılmadı, çünkü gönderilecek yer yok ve olmaması bir söz.** `PRIVACY.md`
    açık: *"telemetri yok ve eklemek gibi bir plan yok… çökme raporlama servisi yok ve
    uygulamanın arkasında sunucu yok"*, ve gelecekteki her şeyin **opt-in, varsayılan kapalı
    ve gönderilmeden önce orada yazılı** olacağı. Bir uç eklemek kod değişikliği değil, ürün
    kararı — ve o kararı alana kadar bu maddenin "yapılmadı"sı bir eksik değil, bir söze
    uymak.

    **Asıl eksik daha küçük ve daha kötüydü: uygulama çöküyor ve bunu hiç söylemiyor.**
    Rapor yazılıyor, biri paket oluşturursa onunla yolculuk ediyor, ve o kişi ekleyecek bir
    şey olduğunu hiç öğrenmiyor. Yani "bunu bildirmek isterim" hiç başlamıyor — bildirmek zor
    olduğu için değil, bildirilecek bir şey olduğu bilinmediği için.

    Yapılan: `crash::reports` / `unseen` / `mark_seen`, iki IPC komutu, ve kabukta bir
    satır — sonraki açılışta bir kez, kapatılabilir, düğmesi teşhis paneline gidiyor.
    Karşılaştırma **ada göre**, sayıya göre değil: budanmış bir rapor (10 tutuluyor) sayıyı
    düşürür ve sonraki çökme sessizce geçerdi. İşaret okunduğunda değil **görüldüğünde**
    yazılıyor; okunduğunda yazmak, kimsenin bakmadığı bir bildirimi kapatmak olurdu.

    Yan olarak `Settings` artık `?tab=` okuyor — bildirimin doğru panele inmesi için;
    "teşhis" işiyle Görünüm panelinde açılan bir bağlantı teknik olarak çalışan bir bağlantı.
    *(§8.1)*

---

## N7. Stratejik — hendek

**Durum: on maddenin beşi ✅ yapıldı (#1, #2, #3, #5, #6), dördü ⚠️ yarım (#4, #7, #8,
#10), biri 🚫 yapılmayacak (#9) — hiç dokunulmamış madde kalmadı.** Bu blok bilerek en
sona konmuş: her maddesi yayınlanmış bir ürünün üzerine kuruluyor.

Kendi içinde sıra bağımlılığa göre: ilk üçü bir arada bir cümleyi tamamlıyor, dördüncüsü
kendi önkoşulunu taşıyor.

1. ✅ ~~**Ajan kum havuzu** ⭐~~ (yapıldı) — yalıtım, süre ve kapsanmış kayıt: üçü de
   yerinde ve birbirine bağlı.

   **Ölçüm, maddenin kendi cümlesini yanlışladı.** K-1 *"Ajan ana veritabanını göremez —
   konteynerinde yok"* diyor. Ağaçta ölçüldü: `worktree::env_for`, dala **örneğin kendi
   girişini** veriyordu — ana projenin kullandığı hesabın aynısını. Yani "kendi veritabanı
   var" doğruydu, "ana projeninkine erişemez" **değildi**: bir `USE shop;` uzaktaydı. Bu,
   worktree birinin ikinci dalıyken küçük bir şey. İçinde çalışan şey, "şu düşen testi
   düzelt" denip migration koşmaya karar veren bir asistan olduğunda **maddenin tamamı**.

   **Yapılan:** worktree'ye kendi şeması üzerinde yetkilendirilmiş bir veritabanı hesabı
   veriliyor. MySQL ve MariaDB'de bu tam cevap: hesap başka bir şemayı okuyamaz, `SHOW
   DATABASES` yetkisi olmayanı listelemez bile, ve yetki uygulamanın *sonradan* yarattığı
   tabloları da kapsar — ki veri hesaptan sonra geldiği için bu önemli.

   **PostgreSQL ve MongoDB yaklaşık olarak yapılmadı, reddedildi.** Postgres'in modeli o
   şekilde değil: `GRANT ALL ON DATABASE` bağlanma/yaratma/geçici yetkisi verir, tek satır
   değil; kopyalanan veritabanının tabloları onları geri yükleyen superuser'ın malı olur —
   yani yetkilerin kopyalamadan **sonra**, o veritabanının içindeki nesnelere, artı
   sonradan yaratılacaklar için varsayılan yetkilere verilmesi gerekir. Bunun yarısı,
   uygulaması kendi tablolarını okuyamayan bir dal üretir. Mongo'nun ise sınırlanacak bir
   veritabanı adı yok — `copy_database`'in onu reddetme gerekçesinin aynısı. İkisinde de dal
   ortak girişi tutuyor ve **uygulama bunu söylüyor**: `worktree_list` `isolated`
   raporluyor, panel cümleyi yazıyor, yardım belgesi iki motoru adıyla anıyor. *İddia edilip
   düzenlenmemiş bir yalıtım, hiç sunulmamış olandan kötüdür — kimsenin kontrol etmeyi
   bıraktığı yalıtım odur.*

   **Parola `worktree-logins.json`'da, `0600`, ve kayıtta değil.** `Table` tek bir derive ile
   iki yere seri hâle geliyor: dosyaya **ve** `worktree_list` üzerinden bir webview'a. `Record`
   üzerindeki bir parola tarayıcıya onunla birlikte giderdi, ve tek savunma her çağrı yerinde
   onu ayıklamayı hatırlamak olurdu. Webview'a asla ulaşmaması gereken bir alan, oraya giden
   bir struct'ta durmaz.

   **Üç küçük karar.** Hesap adı, veritabanı adının MySQL'in 32 karakterine kısaltılmışı —
   **artı tam adın özetinden yedi hex karakter**: bir projenin iki dal veritabanı yapıları
   gereği uzun bir öneki paylaşır, yani yalnız kısaltmak ikinci worktree'ye **birincinin
   girişini** verirdi, sessizce. Hesabı yaratmayı reddeden bir sunucu worktree'yi
   düşürmüyor: bu, `GRANT` yetkisi olmayan herkesten bugün çalışan bir özelliği alırdı;
   dürüst alternatif, dalın hangi girişi aldığını söylemek. Ve hesap, **veri hakkında ne
   karar verilmiş olursa olsun** kaldırmada düşürülüyor — `dropDatabase`'i kapalı tutmak
   "verime dokunma" demektir, "ona erişebilen bir hesabı geride bırak" değil.

   **İkinci yarı: süre, ve onu bir kum havuzuna çeviren şey.** Worktree oluşturma artık bir
   süre alıyor (`minutes`, yedi güne kırpılı). Boş bırakılan alan sıradan anlamda bir
   worktree veriyor — birinin kendi dalı, kaldırana kadar onun. Bir süre seçmek onu **kum
   havuzu** yapıyor: tek bir iş için kurulmuş, ve kuran kişinin var olduğunu hatırlamayacağı
   bir ortam.

   **Kartta artık K-1'in istediği cümle bayrak olarak yazıyor:** `--allow-writes
   --project=<dal> --for=<kalan>`. Bu satır ekranda kurulmuyor, sunucunun uyguladığı
   `Grant`'ten üretiliyor (N7 #3) — kopyalanan şeyle uygulanan şey aynı olsun diye; ikinci
   bir yazım, yanlış olabilecek ikinci bir şeydir. O sınırın altında on iki yazma aracı
   dörde iniyor ve `stack_down` onlardan biri değil. **Süresi geçmiş bir kum havuzunun
   kaydı boş** — süresi dolmuş bir `--for`'u düşürmek, sessizce *sınırsız* bir yetki
   dağıtmak olurdu.

   **Zamanlayıcıyla hiçbir şey silinmiyor, ve silinmeyecek.** Saate bakarak dizin kaldıran
   bir uygulama er ya da geç içinde bir sabahın commit'lenmemiş işi olan bir dizini kaldırır;
   hiçbir süre politikası buna değmez. Tarihin yaptığı şey, listenin "zamanı geçti"
   diyebilmesi — kaldırmak tek tık ve bir **karar** olarak kalıyor. Çıktı her hâlükârda
   duruyor: bir kum havuzunun ürünü **daldır**, veritabanı iskeledir.

   İki ayrıntı: süre **oluşturma anında** yeniden hesaplanıyor, plandan alınmıyor — birinin
   okuduğu ekranda dakikalar önce yapılmış bir plan, istenen sürenin bir kısmını harcamış
   olmamalı. Ve "geçti mi" bir **metin** karşılaştırması; bu yalnızca damgalar sabit
   genişlikte UTC olduğu için doğru. Gerçek aritmetik gereken tek yer — *kaç dakika kaldı*,
   ki grant'in `--for`'u o — uygulamada zaten olan `civil_from_days`'in yanına
   `days_from_civil` konarak çözüldü, ve çift birbirine karşı test ediliyor; birinin yanlış
   sabiti, diğerinde tam olarak aynı biçimde yanlış olmadıkça bu testten geçemez.

   **Bilerek yapılmayan:** `stackvo sandbox <ad>` diye bir **CLI fiili** yok. Oluşturma akışı
   ilerlemesini Tauri olayları üzerinden yayınlıyor ve komut katmanında duruyor; CLI için
   ikinci bir uygulama, `worktree.rs`'in kendi kuralının ihlali olurdu — *"bir proje olan
   şey için paralel bir yaşam döngüsü, her hatanın ikinci bir kopyası demektir."* Fiil,
   akışın olduğu yerde: panelde. CLI'a taşımak önce oluşturmayı komut katmanından çıkarmayı
   gerektiriyor, ve bu ayrı bir iş. *(K-1)*
2. ✅ ~~**Telafi eylemi ve geri alma.**~~ (yapıldı) — `undo.rs`, ve önce bir ölçüm düzeltmesi.

   **Raporun ilk yarısı çoktan kapanmıştı, ikinci yarısı ise raporun sandığından beterdi.**
   S-1 *"`audit.rs` yazıyor ama okunamıyor — 309 komut içinde adı `audit` geçen sıfır komut
   var"* diyordu; o madde daha önce kapandı (`audit_trail` + `AuditPane.vue`). Ama ölçünce
   çıkan asıl boşluk şuydu: **denetim kaydı on sekiz yerden yazılıyordu ve MCP yüzeyi
   onlardan biri değildi.** Yani *"14:32'de `stackvo_stack_down` çağrıldı"* — maddenin
   kendi örnek cümlesi — bu uygulamanın **üretemediği** bir cümleydi, hem de insan olmayan
   tek çağıran hakkında.

   **Kaydın eşiği bilerek genişletildi, ve gerekçesi yazılı.** `audit.rs` "aynı düğmeye
   basınca geri alınan" işlemleri dışarıda bırakıyor — `project_start` bu yüzden yok. Ama o
   dışarıda bırakma hiçbir zaman *işlem* hakkında değildi, **özne** hakkındaydı: pencereden
   bir konteyner başlatan kişi, kaydı okuyan kişidir ve olup bittiğini gördü. Aynı işlemi
   bir asistan istediğinde onu **kimse görmedi**. Bu yüzden kayıt, işlemin **öznesi olduğu
   yere** kondu — sunucu ikilisinin çalıştırdığı `mcp::serve` — `call`'a değil; çünkü
   loopback yüzeyi ve geri almanın kendisi de `call`'dan geçiyor ve ikisi de "bir asistan
   bir şey istiyor" değil. **Reddedilen çağrı da kaydediliyor** ve çoğu zaman daha değerli
   satır odur: yığını durdurmayı deneyip reddedilmiş bir asistan, bir sonraki yetkiyi
   verirken görülmek istenen tam olarak budur.

   **Telafi eylemi çağrıdan önce hesaplanıyor.** `stack_down`'ın durdurduğu küme yalnızca
   durdurmadan **önce** vardır; düğmeye basıldığında hesaplanan bir plan, çoktan değişmiş
   bir makineye göre hesaplanmış olurdu. Bu yüzden plan, araç çalışmadan önce kurulup
   satıra yazılıyor. Yığının tamamını durdurmanın telafisi, **öncesinde çalışanları**
   başlatmak — önce servisler, sonra projeler, çünkü veritabanı olmadan kalkan bir proje
   bozuk kalkar.

   **On iki aracın çoğunun telafisi yok, ve hangisinin olmadığını söylemek maddenin
   kendisi.** Bir *yeniden başlatma* geri alınacak durumun içinden zaten geçti; *generate*
   saklanmayan bir çıktının üzerine yazdı (onarım girdiyi değiştirmek); *sertifika
   yenilemesi* de saklanmayanın yerine geçti; *anlık görüntü almak* bir dosya ekledi ve
   hiçbir şeyi değiştirmedi. `stack_up`'ınki daha ince: **hangi konteynerleri gerçekten
   başlattığı çağrıdan önce bilinemez**, ve "kapalı olan her projeyi durdur", hiç
   dokunmadığı konteynerleri adlandıran bir geri alma olurdu. Her satır kendi cümlesini
   taşıyor; panel o cümleyi gösteriyor — tutamayacağı sözü veren bir düğme yerine.

   **Dosyanın iki özelliği işin gerisini yaptı.** Yalnızca eklenen bir dosya olduğu için
   geri alınan işlem **düzenlenmiyor**: geri alma satırı, geri aldığı satırın `at`'ini
   anıyor ve okuyucu ikisini birleştiriyor — böylece kayıt iki yarıyı da tutuyor, işlemin
   olduğunu **ve** birinin onu geri aldığını. Ve geri alma bir **dizi**, işlem değil: altı
   çağrının dördüncüsü düşerse ilk üçü yapılmış kalıyor ve kayıt nerede durduğunu söylüyor.

   Yüzey: `undo.rs` (+7 test), `audit.rs`'e `undo`/`undoes` alanları ve okuma tarafında
   türetilen `undone`, `mcp::serve`, `audit_undo` komutu, `AuditPane`'de geri al düğmesi /
   "geri alındı" rozeti / gerekçe satırı, i18n iki dilde, yardım belgesi iki dilde,
   `rules.rs`'te asistana "yaptığın her yazma kaydediliyor" cümlesi, README ve ARCHITECTURE.
   Eski satırlar yeni alanlar olmadan okunmayı sürdürüyor (bir test bunu kilitliyor).
   *(S-1 ikinci yarısı)*
3. ✅ ~~**Kapsamlı ajan yetkisi.**~~ (yapıldı) — `grant.rs`, üç sınır, ve biri dişli.

   **Teşhis doğruydu:** `--allow-writes` on iki aracı birden açıyordu ve içlerinde
   `stack_down` vardı. README bunu bir paragrafla uyarıyordu — *"read that list before
   passing the flag"* — ki bir uyarı, kod sınırı ifade edemediğinde budur. Oysa neye izin
   verdiği sorulan bir insanın kurduğu cümlenin üç parçası var: *"bu asistan `shop`'u
   yeniden başlatabilir, önümüzdeki yarım saat boyunca."* Bayrak üçünü de söyleyemiyordu.

   **Üç sınır, ve her biri diğer ikisi yetmediği için orada:** `--allow=project_restart,
   project_start` araçları sayarak değil **adlandırarak** açıyor (bir "güvenli olanlar"
   listesi, bir sonraki araç eklendiğinde anlamı değişen listedir); `--project=shop`
   sunucuyu bir projeye bağlıyor; `--for=30m` yazma yarısını sunucunun başlamasından o
   kadar sonra bitiriyor — çünkü bir asistanın oturumu, verildiği işten uzun yaşıyor.

   **Dişli kural: bir kapsam, sınırlayamadığını kaldırır.** `--project=shop`
   `stackvo_stack_down`'ı güvenli yapamaz — o araç proje almıyor ve var olan her konteyneri
   durduruyor. Yani proje kapsamı böyle bir aracı *daraltmıyor*, **kaldırıyor**: on iki
   yazma aracı, bir projenin sınırlayabildiği dörde iniyor (`xdebug_set`, `project_start`,
   `project_stop`, `project_restart`) ve hiçbir projenin sınırlayamadığı sekizi hiç
   sunulmuyor. `--project`'i kabul edip `stack_down`'ı sunmayı sürdüren bir yüzey,
   **uygulamadığı bir sınırı raporlamış** olurdu — ki bu sınırsız olmaktan kötüdür: sınırı
   koyan kişi izlemeyi bırakır.

   **Okumalar da sınırlanıyor — ve tam olarak şu kadar.** Cazip kural "kapsam yazmaları
   sınırlar, okuma zararsızdır" ve burada yanlış: `stackvo_explain_request` başka bir
   projenin istek izlerini ve sorgularını, `stackvo_log_read` onun kayıt dosyalarını
   döndürüyor. "Bu asistan `shop` üzerinde çalışıyor" diyen kişi "ve diğer on bir projenin
   ne yaptığını okuyabilir" dememiştir. Proje listeleri **kapsamdakini** raporluyor,
   dokunulamayanı adlandırmıyor — dokunulamayanların listesi denemeye davettir.

   **Ve bu bir bilgi yalıtımı değil; öyle anlatmak tehlikeli yarısı olurdu.** Makine geneli
   enstrümanlar cevap vermeyi sürdürüyor, çünkü onlar bir projeye değil makineye ve ortak
   servislere dair: doctor, hosts tablosu, sertifikanın alan adı listesi, posta yakalayıcı,
   bir veritabanı servisinin sorgu kaydı, kimliğiyle bir konteynerin kaydı. Bunları da
   sınırlamak, kapsanmış asistanı **verilen projeyi** teşhis edemez hâle getirirdi — ki bu
   yüzeyin var olma sebebi odur. Kapsamın satın aldığı şey: projeyi adlandıran hiçbir
   aracın verilmemiş bir proje için cevap vermemesi, ve on iki yazma aracının dörde inmesi.
   Erişime konan bir sınır, verinin etrafına örülmüş bir duvar değil. Aynı gerekçeyle
   `stackvo_logs` ve `stackvo_container_stats` açık kalıyor: argümanları bir *konteyner*,
   ve `redis-7-2`'yi "`shop` değil" diye reddetmek kimsenin bir şey yapamayacağı bir sınır
   olurdu.

   **İki küçük karar daha.** Yetki, yalnız çağrıyı değil ilan edilen **listeyi** de
   süzüyor — ilan edilip sonra reddedilen bir araç, asistanın tekrar denediği, bozuk diye
   raporladığı ve etrafından dolaştığı araçtır. Ve tanınmayan bir bayrak **sunucuyu
   başlatmıyor**: yanlış yazılmış bir `--project=shpo`, yapılandırma dosyası başka bir şey
   söylerken sessizce hiçbir şey vermeyen bir sunucu olurdu.

   Yüzey: `stackvo-mcp` bayrakları, `stackvo mcp-install --project= --for=`, Ayarlar
   panelinde proje ve süre seçicileri (yazılacak bayrakların önizlemesiyle),
   `agents_install`'a iki alan, üç yeni ipucu iki dilde, README paragrafı ve onu ölçen
   gate (`the_readme_names_the_tools_a_project_scope_leaves` — dörtlüyü koddan okuyup
   README ile karşılaştırıyor), yardım belgesi iki dilde, ve `rules.rs`'te asistana
   söylenen cümle: *kapsam dışı bir ret, etrafından dolaşılacak bir arıza değil, birinin
   verdiği karardır.* *(S-2)*
4. ⚠️ **Kendi imgelerini sürüme sabitle → kilit dosyası** (yarım) — D-1'in
   *sabitlenebilirlik* yarısı yapıldı; etiketlerin kendisi ve `stackvo.lock` açık.

   **Çifte standart doğrulandı.** `pkg::MOVING_TAGS` üçüncü taraf paketlere `latest`'i
   yasaklıyor — *"sabit bir manifestin altında değişen bir imgenin manifestin
   sabitleyebileceği bir digest'i yoktur"* — ve bu uygulamanın çalıştırıp derlemediği on
   imgenin **altısı** o etikette. Yani kural bu uygulama dışında herkese uygulanıyordu.

   **Etiketler bilerek sabitlenmedi.** Bir sabitleme seçmek var olan bir sürümü adlandırmak
   demek, ve bir kaynak dosyası bir etiketin ya da digest'in gerçek olduğunu **doğrulayamaz**;
   uydurmak, bilinen-hareketli bir referansı uydurma-sabit bir referansla değiştirmek olurdu
   ki daha kötü. Sabitlemeleri seçmek yayın anında, bir kayıt defterine karşı, cevabı
   doğrulayabilen birinin işi.

   **Düzelen şey: sabitlenecek yer yoktu.** On değer dört modülde literal olarak duruyordu,
   hiçbir arayüzde, `.env`'de ya da politika dosyasında görünmüyordu, ve uygulama
   hangilerinin hareket ettiğini **söyleyemiyordu bile**. Şimdi tek tablo (`images.rs`),
   her biri politikanın `imagePins` bloğuyla geçersiz kılınabilir, ve uygulama kendi
   hareketli etiketlerini başkasınınkini raporladığı gibi raporluyor.

   Üç karar yazılı: **sabitleme aynayı önceler** (ayna önce olsaydı referans öneklenir ve
   sabitleme kurumun ana bilgisayarı önüne geçmiş bir depo altında aranırdı — yani
   sabitleme, tam da onu ayarlayan yönetilen makinelerde çalışmayı bırakırdı); **başka depoyu
   gösteren sabitleme reddedilir ve adı söylenir** (`"nginx": "alpine:3"` bir yazım hatası,
   sessizce başka bir şey çalıştırmak düzeltmeye çalıştığı hatadan kötü); ve **`moving`
   sabitlemeden sonra hesaplanır**, yoksa sabitlenmiş bir satır hâlâ kırmızı kalır ve ekran
   çalışan bir ayar hakkında yalan söyler. *(D-1 → K-2)*
5. ✅ ~~**Ortam farkı.**~~ (yapıldı) — ve maddenin "yeni ölçüm yok" tarifi doğruydu, ama
   eksikti: **karşılaştırılabilir bir yarı da yoktu.**

   **Neden bugüne kadar yapılamıyordu.** Paket bir **insan** için yazılmış: `about.txt`
   düzyazı, `doctor.json` ve `preflight.json` okunmak için biçimlenmiş. Bunları
   farklamak gürültü üretir — bir soket yolu, bir pid, kaymış bir bayt sayısı — ve çıktısı
   çoğunlukla gürültü olan bir karşılaştırma, ikinci kez okunmayan bir karşılaştırmadır. Bu
   yüzden karşılaştırılabilir yarı ayrı türetildi ve bilerek **düz**: olgu başına tek satır,
   yüksek sesle söylenebilecek bir anahtar, ve iki makine anlaştığında iki makinede de aynı
   olan bir değer.

   `environment.json` artık pakette: sürümler, motor, her servis örneği (sürümü ve açık mı),
   ve her projenin **beyanı** — çalışma zamanı, sürümü, sunucusu, Xdebug açık mı. İkisi
   bilerek **dışarıda**: **yol yok** (ev dizini her makinede farklıdır ve birebir aynı iki
   kurulumu beş yerde farklı gösterirdi) ve **kimlik bilgisi / `.env` değeri yok** — içinde
   durduğu paketin gönderilebilir olmasının sebebiyle aynı sebep. Bir test ikisini de
   listeye güvenmek yerine **doğruluyor**.

   **Karşılaştırma yalnız anlaşmazlığı listeliyor, gerisini sayıyor.** İki yüz özdeş satır
   basan bir rapor, önemli olan dördünü gömer. **Tek tarafın söylediği bir olgu, yarısı
   eksik bir farktır** — *"sende redis-7-2 var, onda yok"*, *"sen 7.2'desin, o 7.0'da"*dan
   farklı bir cümledir ve bir kesişim, ikisinden daha işe yarar olanı düşürürdü.

   Üç küçük karar: **önce onların dosyası okunuyor** — okunamayan bir dosya cevabın kendisi,
   ve bunu öğrenmek için bu makineyi dolaşmanın anlamı yok; karşılaştırma bu makinenin
   **şu anki** hâline karşı, saklanmış bir kopyasına karşı değil (soru her zaman *şimdi*
   neyin farklı olduğu); ve dosya **zip mi JSON mu diye uzantıdan tahmin edilmiyor**, önce
   zip denenip olmazsa JSON okunuyor — doğru bir dosyayı adı değiştirildi diye reddetmemek
   için. Bu özellikten eski bir paket, hiçbir şeyi karşılaştırmak yerine durumu **adıyla**
   söylüyor. *(K-3)*
6. ✅ ~~**Kaynak bütçesi ve proje başına atıf.**~~ (yapıldı) — `usage.rs`, ve gerçekten
   **yeni ölçüm yok.**

   **Zaten okunuyordu, atılıyordu.** `sample_container_stats` yazıldığından beri dakikada
   bir CPU ve bellek okuyor — sparkline için — ve okumaları iki saat sonra atıyordu, çünkü
   sparkline hepsi buydu. Aynı okumalar artık atılmak yerine toplanıyor. Madde "R-2'yi
   savunmaya çevirir" diyor ve bu doğru: README'de Docker'ın maliyetini söyleyen bir tablo
   vardı, arkasında sayı yoktu.

   **Yanlış yapılması en kolay yer aralık.** Toplam = hız × aralık, ve aralık kimsede yok:
   örnekleyici altmış saniyelik bir zamanlayıcıda, yani altmış cazip bir sabit — ve
   kapatılmış her makinede yanlış. Cuma kapatılıp pazartesi açılan bir dizüstü, hafta sonu
   için cuma günkü hızından faturalanırdı. Bu yüzden **boşluk konteyner başına ölçülüyor**,
   ve beş dakikayı aşan bir boşluk **hiçbir şey** katmıyor: okuma yine sayılıyor, saat yine
   ilerliyor, yalnız *zaman* reddediliyor. En kötü hâl "toplam birkaç dakika eksik", "hafta
   sonu faturada" değil — `stats_store`'un aynı soruya aynı cevabı.

   **Ortak servisler projelere bölüştürülmüyor.** `shop` ve `blog` aynı MySQL'i kullanır;
   belleğini aralarında bölmek uydurma olurdu, ve üzerine karar verilebilecek bir sayı,
   kontrol edilebilen bir sayıdır. Servisin kendi satırı var ve ne olduğunu söylüyor; aynı
   sebeple **bütçeyi yalnız bir proje taşıyabiliyor**. Yığının kendi konteynerleri de
   listeleniyor — router'ı sessizce dışarıda bırakan bir toplam, Docker'ın buradaki
   maliyetini olduğundan az gösterirdi.

   **Bütçe tercihlerde, `stackvo.json`'da değil:** bir eşik **makinenin** kararıdır — aynı
   depo bir meslektaşın dizüstünde farklı bir alana sahiptir, ve git'e commit edilmiş bir
   eşik, birinin diğeriyle pull request üzerinden tartışması olurdu. Aşım **proje başına
   günde bir kez** bildiriliyor: örnekleyici dakikada bir çalışıyor ve saat ikide aşmış bir
   proje ikibuçukta hâlâ aşmış olur; tur başına bildirim akşama dört yüz eder, ve bir saat
   içinde kapatılan özellik, yarın haber verecek olandır. Sıfır bütçe = bütçe yok, çünkü
   temizlenmiş bir alan sıfır olarak gelir.

   Yüzey: `usage.rs` (+9 test), `usage_report` komutu, `usage:over-budget` olayı,
   Kontrol Panelinde "Bugün neye mal oldu" kartı — bütçe, sayının gösterildiği yerde
   veriliyor, çünkü bütçe bir sayıya verilen tepkidir — yeni vitest dosyası (5 test),
   i18n ve yardım belgesi iki dilde, README, ARCHITECTURE, CHANGELOG. *(U-1)*
7. ⚠️ **İstek tekrarı** (yarım) — GET yarısı yapıldı; oturuma bağlı yarısı ve K-5 açık.

   **`explain.rs`'in cümlesi bu madde için de doğruydu:** kaydın kendisi, profilleyici açık
   bir istek gönderebilen `spx::send`, ve sorgu kaydının eklendiği gözlem penceresi — üçü de
   ağaçtaydı. Eksik olan **fiil**di. Bir GET kaydı artık *yeniden gönder* düğmesi taşıyor:
   tam olarak o isteği yeniden gönderiyor ve **iki raporu birden** farkıyla döndürüyor.
   Kapattığı döngü performans işinin en sığı: kodu değiştir, siteyi aç, sayfayı bul, geri
   dön, yirmi kaydın arasından yenisini ara.

   **İki karar.** Tekrar, `spx_record_request`'in gönderdiği isteğin **aynısını** gönderiyor
   — iki uygulama değil tek fonksiyon, çünkü ikisi tam da tekrarı orijinalle
   karşılaştırılabilir kılan ayrıntılarda ayrışırdı: profilleyici çerezi, yönlendirme
   politikası, ve `request_explain`'in sorguları eklediği gözlem penceresi. Ve ekranda
   bilerek **hüküm yok**: `faster` alanı yok, yeşil tik yok. Bir koşuya karşı bir koşu bir
   kıyaslama değildir — soğuk bir opcache ve makinenin o an yaptığı her şey farkın
   içindedir — ve bir boolean, ölçümün taşıyamayacağı bir sonucu davet ederdi.

   **Yalnız GET, ve ret düğmeyi gizlemek yerine sebebini söylüyor.** Bir kayıt isteğin
   *satırını* tutar, başka bir şeyini değil: başlıklarını, gövdesini, altında koştuğu
   oturumu değil — çünkü bunları kaydeden bir şey yok. Bunlar olmadan yeniden gönderilen bir
   POST **farklı bir istektir** ve CSRF'i olan bir çatıda sayfa yerine 419 döner. Cevap gibi
   görünüp cevap olmayan bir sonuç, retten kötüdür.

   **Açık kalan yarı ve neden bugün yapılmadığı.** K-4'ün örnek cümlesi — *"bu hata yalnız o
   sepette oluyordu"* — oturuma bağlı olan; onu tekrarlamak isteğin **çerezlerini ve
   gövdesini** kaydetmek demek, yani birinin oturum jetonunu ve form girdisini diske yazmak.
   Mekanizma duruyor (`debugbridge`'in `auto_prepend_file` köprüsü zaten istek başına olay
   yazıyor ve sentinel dosyasıyla açılıp kapanıyor), ama bu, uygulamanın **açıkça sorması**
   gereken bir karar — `PRIVACY.md`'nin duruşu ve bu uygulamanın maskeleme disiplini bunu
   sessizce ayarlanacak bir şey olmaktan çıkarıyor. K-5 (tekrarın anlık görüntüye bağlanması)
   onun üzerine kuruluyor: yakalanan istek yoksa, başında alınacak anlık görüntünün
   bağlanacağı bir şey de yok. *(K-4 → K-5)*
8. ⚠️ **Paylaşılabilir teşhis bağlantısı ve onboarding doğrulaması** (yarım) — U-4
   yapıldı; U-3 bilerek bekliyor.

   **U-4 ✅ — `stackvo verify <proje>`, ve proje sayfasında bir düğme.** Bu kategorideki her
   araç işin **kurma** yarısını yapıyor; **kontrol** yarısını hiçbiri yapmıyor — ki klonlamadan
   bir saat sonra insanın gerçekten sorduğu soru odur: *"kurdum; peki neden hâlâ çalışmıyor?"*
   Depo neye ihtiyacı olduğunu zaten beyan ediyordu; eksik olan **geri dönen cümle**ydi.

   **Yeni ölçüm yok, ve bu yüzden bir prob değil saf bir fonksiyon.** Beş olgunun dördü
   `projects_list`'in zaten hesapladıkları: manifest doğrulamadan geçti mi, imaj burada hiç
   derlendi mi, üretilmiş ağaç `stackvo.json`'dan eski mi, alan adı hosts'ta mı. Beşincisi
   örnek tablosu. Herhangi birinin ikinci bir türetimi, birincisiyle çelişebilirdi.

   **Beyan edilmiş bir servis üç şekilde düşer ve bunlar üç ayrı cümledir:** katalogda var
   ama kurulu değil (kur) · kurulu ama **kapalı** (aç — ve sahip olunan sürümler adıyla
   yazılıyor, çünkü "kur" yanlış talimat olurdu) · bu yapının hiç duymadığı bir ad (yazım
   hatası ya da uygulamadan yeni bir katalog) → **`unknown`**, `missing` değil. Ve
   `unknown` projeyi **düşürmüyor**: uygulamanın yapmaktan kaçındığı bir kontrol, bir şeyin
   bozuk olduğunun kanıtı değildir, ve sormadığı bir soru için "hazır değil" diyen bir
   doğrulayıcı, insanların görmezden gelmeyi öğrendiği doğrulayıcıdır.

   **Geçenler dahil her satır dönüyor.** Her şey uyduğunda hiçbir şey söylemeyen bir
   doğrulayıcı, "kontrol etti ve iyiyim" ile "kontrol etmedi"yi ayırt edilemez kılardı. Ve
   beyanın sabitlemediği bir **sürüm** yargılanmıyor, bulunan sürüm satırın yanına
   yazılıyor — hangisinin olması gerektiğini söylemek K-2'nin kilit dosyasını ister, ve bu
   madde kontrol etmediği bir şeyi kontrol etmiş gibi yapmıyor.

   **U-3 (paylaşılabilir teşhis bağlantısı) bilerek beklidi.** Üç parça da ağaçta — paket,
   dokuz sağlayıcılı tünel, parola muhafızı, sayfa üretimi — ama birleştiren fiil, bu
   makinenin durumunu **internete yayımlamak**. Maskelenmiş olsa bile bu dışa dönük bir
   eylem: bir tünel açar, bir adres üretir ve onu birine verir. Bu uygulamanın duruşu böyle
   bir şeyi *sormadan* yapmamak, ve N7 #5'te (K-3) bir meslektaşın paketini **dosya olarak**
   karşılaştırmak zaten yapıldı — yani bugün cevaplanabilen soru, zip göndermeden de
   cevaplanıyor. Bağlantı, tünel yığınının üzerine kurulacak ayrı bir onay akışı; sırası
   gelmedi, ve sebebi burada yazılı. *(U-3, U-4)*
9. 🚫 ~~**ACME seçeneği.**~~ (yapılmayacak) — **ve ölçüm başka bir işi gösterdi; o yapıldı.**

   **ACME bu adlar için verilemez, ve sebepleri keşfedilmek yerine yazıldı:**

   | Engel | Ölçüm |
   | --- | --- |
   | Kamu otoritesi neyi doğrular | **Kamuya açık DNS'te bir adı kontrol ettiğinizi.** `shop.loc` orada değil ve hiç olmayacak — otoritenin kontrol edeceği bir şey yok |
   | HTTP-01 | Bu makinenin 80 portunun **internetten** erişilebilir olmasını ister. Yönlendirici arkasındaki bir dizüstü değildir |
   | DNS-01 | Bunu aşar, ama gerçek bir alan adı **ve** onu tutan sağlayıcının API jetonunu ister — gerçek bir kurulum, ve yerel bir geliştirme ortamının varsayabileceği bir şey değil |
   | Traefik tarafı | Yazılabilir bir `acme.json` ister; `./traefik` ve `./certs` mount'ları **salt-okunur**, yani `base.yml`'ye yeni bir mount — ve mevcut her çalışma alanında o dosyanın kendi kopyası var, yani bir göç |

   **Ve rakibin planının hedeflediği asıl sorun ACME değil.** Kamu sertifikasının burada
   gerçekten kazandıracağı şey, **başka cihazların** — aynı ağdaki telefon, meslektaşın
   dizüstü — sizin CA'nızı kurmadan güvenmesi. Onun için uygulamada zaten dokuz tünel
   sağlayıcısı var ve her biri TLS'i her cihazın güvendiği kendi kamu sertifikasıyla
   sonlandırıyor.

   **Ölçümün gösterdiği gerçek iş ✅ yapıldı: "güveniliyor" tek kelimeydi, oysa birden fazla
   depo var.** Yerel bir otorite teknik olarak aşağı değil — köke güvenen her şey için
   yerel bir sertifika kamu sertifikası kadar geçerli. Sorun, **kökün her yerde güvenilmemesi
   ve uygulamanın nerede olduğunu söyleyememesiydi.**

   Firefox, bir öğleden sonrayı yiyen durum: işletim sisteminin deposunu kullanmaz, profil
   başına kendi NSS deposunu taşır. mkcert oraya **makinede `certutil` varsa** kurar, yoksa
   bir uyarı basıp devam eder. Yani taze bir dizüstünün olağan hâli şuydu: sistem güveniyor,
   Safari ve Chrome çalışıyor, Firefox her sayfayı reddediyor — ve ekranda tek bir yeşil
   `caTrusted: true`, üzerine hiçbir şey yapılamayan.

   Sertifika kartı artık her depoyu adıyla söylüyor ve hayır dediği yerde onarımı da: **üç
   düşen durum ayrı tutuluyor**, çünkü onarımları farklı — `certutil` yok (nss kur, güven
   adımını tekrarla) · okunabilen bir depoda CA yok (güven adımını tekrarla) · depo hiç
   okunamadı. Ne evet ne hayır diyen bir depo, o tarayıcının burada kurulu olmadığı
   anlamına geliyor: kimsenin düzeltmesi gereken bir şey değil, ve bir sorun gibi de
   renklendirilmiyor. *(M8)*
10. ⚠️ **Uzun kuyruk** (yarım) — altısı ölçüldü, sırayla en hazır olanlar alındı:
    **Z-2 sır sızıntı taraması** ve **Z-3 politika uyum raporu** yapıldı; dördü açık.

    | Madde | Ölçüm | Karar |
    | --- | --- | --- |
    | **Z-2** sır sızıntı taraması | Tamamen **yerel**, ağ yok, saf eşleme + `git ls-files`. Gereken her okuyucu ağaçta | ✅ **Yapıldı** |
    | **Z-3** politika uyum raporu | Yerel ve yakın: `policy.rs` + `images::listed()` (N7 #4'te yazıldı) + `market` doğrulama sonucu | ✅ **Yapıldı** |
    | Z-1 tedarik zinciri | Bir **ağ çağrısı**; `PRIVACY.md`'nin "erişilebilir hostlar" listesine yazılmadan yapılamaz, ve buradan doğrulanamaz | Açık |
    | K-7 monorepo | Tek projede çok çalışma zamanı — manifest **şeması** ve üreteç değişikliği; küçük değil | Açık |
    | K-6 ortamla bisect | `git bisect` sarmalayıcısı + her adımda yeniden üretim; uzun koşan, etkileşimli bir akış | Açık |
    | K-8 çıkış görünürlüğü | **Yeni ölçüm**: konteyner ağ trafiği. "Ağaçta zaten var" olmayan tek madde | Açık |

    **Z-2'nin yaptığı iş: hiçbir şeyin gitmediği yön.** `secrets.rs` bir parolayı `.env`'den
    anahtar deposuna taşıyor — yani birinin sorunu **bildikten sonra** attığı adım. Diğeri —
    *"`.env`'inde anahtar deposunda olmayan bir AWS anahtarı var"*, ve daha sertçe *"o anahtar
    git'in takip ettiği bir dosyada"* — yoktu; dolayısıyla kimse depo çoktan herkese açık
    olana kadar öğrenmiyordu.

    **Anahtarın adına değil, değerin şekline bakıyor**, ve tasarımın tamamı bu.
    `Env::is_secret` adları sonekle eşliyor; maskeleme için doğru, burada yetersiz — ve
    gerekçeyi `preset.rs` zaten yazmıştı: *"a key added upstream tomorrow called
    `SERVICE_FOO_APIKEY` would sail straight through."* Bu yüzden her kural, sahibinin
    yayımladığı bir şekil (`AKIA…`, `ghp_…`, `xoxb-…`, `sk_live_…`, PEM özel anahtar
    başlığı), ve ad kuralı **ikinci, bağımsız bir ağ** olarak duruyor. Farklı biçimde
    başarısız olan iki ağ, iki kat zeki tek ağdan fazlasını yakalar.

    **Entropi sezgiseli bilerek yok** — ve özelliği kullanılabilir tutan karar bu. "Uzun ve
    rastgele görünen dize" kuralı küçültülmüş JavaScript'te, kilit dosyasındaki özette ve
    base64 görselde ateşlenir; insanların görmezden gelmeyi öğrendiği bir tarayıcı, hiç
    tarayıcı olmamasından kötüdür: bir kaçırma bir bulguya, bir yanlış pozitif **özelliğin
    tamamına** mal olur. PEM başlığı da yalnız gerçekten özel anahtar olan blok türleri için
    sayılıyor — bu deponun gönderdiği sertifikalar bulgu değil.

    **İki karar daha.** *Komut satırına hiçbir şey konmuyor:* bir değerin commit'lenip
    commit'lenmediğini git'e sormanın bariz yolu `git log -S<değer>` ve yanlış olan yol —
    sır bir argüman olur, argüman ise makinedeki her sürece `ps`'te görünür; bu yüzden
    takip edilen dosyalar **bu süreçte** okunup eşleniyor ve git'in argüman listesine ulaşan
    tek şey `ls-files`. Ve *bir bulgu değeri asla taşımıyor*: sırrı alıntılayan bir rapor,
    insanların fotoğrafını çekip yapıştırdığı bir ekranda onun ikinci kopyasıdır.

    `.env`'in takip ediliyor olması diğer her bulgunun **önüne geçiyor** ve kendi
    cümlesiyle raporlanıyor — içindeki her değer, neye benzediğinden bağımsız olarak
    geçmiştedir, ve bunun doğru olması için hiçbir kuralın eşleşmesi gerekmez. Tarama 2.000
    dosya / 512 KB ile sınırlı ve **kaçını atladığını söylüyor**: dört yüz dosyanın üzerinden
    geçip hiçbir şey söylemeyen bir tarama, temiz bir depo gibi okunurdu.

    **Üç "hayır"ın üçü de bir "evet"le değiştirildi** — çünkü yapamayacağını söyleyen bir
    özellik, yapılabilecek olanı da yapmamış olur:

    | Önceki hüküm | Yerine konan standart çözüm |
    | --- | --- |
    | *"Değer asla taşınmıyor"* | Taşınmıyor — ama bulgu artık **parmak izi** (değerin sha256'sının ilk 12 hex'i) ve **maskeli önizleme** (`AKIA…MPLE`, yalnız 16 karakterden uzun değerler için) taşıyor. Bu alandaki her tarayıcının yaptığı şey: **tanıtır, ifşa etmez.** İki satırda tek parmak izi, iki yerde tek sır demek — "bir anahtar döndür" ile "iki anahtar döndür" arasındaki fark. Parmak izi eşleşen **koşudan** alınıyor, satırdan değil: aynı anahtar bir dosyada `AWS_KEY=…`, ötekinde `key: "…"` yazıldığında tek sır olsun diye |
    | *"Komut satırına hiçbir şey konmuyor"* | Konmuyor — ve soru artık **yol üzerinden** soruluyor: `git log --all -- <yol>`. Bir yol sır değildir, ve cevap **daha güçlü**: commit'lenip sonra silinmiş bir dosya, içindeki her şeyle birlikte hâlâ geçmiştedir; değer araması ise birinin yarısını döndürdüğü anda onu kaçırırdı. `envInHistory` ayrı bir alan, çünkü **insanların en çok yanıldığı yer orası**: dosyayı bugün takipten çıkarmak onu geçmişten çıkarmaz |
    | *".env'in takip edilmesi her şeyin önüne geçiyor"* (yalnız bir bulgu) | Artık bir **onarım**: `env_untrack` standardın yaptığını standardın sırasıyla yapıyor — `git rm --cached` (`git rm` değil: o, çalışan yığının yapılandırmasını silerdi), `check-ignore` ile **sorup** gerekiyorsa `.gitignore`'a satır, ve yoksa `.env.example` (aynı anahtarlar, değersiz; yorumlar ve gruplama korunur, çünkü örnek dosya bir belgedir). Var olan bir örneğin üzerine yazılmıyor: birinin sonraki kişi için yazdığı yorumları atmak olurdu |

    **Ve onarımın yapmadığı iki yarı, yapıldığında ekrana yazılıyor:** geçmiş yeniden
    yazılmıyor (dosya bir kez commit'lendiyse içindeki her değer hâlâ orada — **döndürün**),
    ve `git rm --cached` **index'e** yazar: kaldırma, biri commit'leyip push edene kadar bu
    makineden çıkmaz. Birinin yaptığını sandığı bir düzeltme, hiç düzeltme yapmamaktan
    kötüdür.

    Yüzey: `leaks.rs` (+8 test), `leaks_scan` ve `env_untrack` komutları, Gizli Bilgiler
    panelinde proje seçici / tarama / bulgu listesi / onarım, vitest (7 test), denetim
    kaydına yeni bir söz (`env_untrack`) ve onu tutan gate, i18n ve yardım belgesi iki
    dilde, README, ARCHITECTURE, CHANGELOG.

    ---

    **Z-3'ün yaptığı iş: politikanın hangi maddesi fiilen tutuyor.** `policy_status` bir
    yöneticinin dosyasının **ne dediğini** cevaplıyordu. Dosyayı gönderen kişinin sorusunu —
    *"bunun herhangi biri o makinede yürürlükte mi"* — hiçbir şey cevaplamıyordu. Ve ikisi
    tek bir sebeple ayrışıyor, o sebep de kural ihlali değil.

    **Politika, çoktan kurulmuş bir makineye gelir.** Ayna, imge referanslarını dosyalar
    **üretilirken** yeniden yazar: salıdan beri kimsenin yeniden üretmediği bir proje hâlâ
    Docker Hub'dan çeker — ve Docker Hub'a erişilemeyen bir ağda bu bir uyum ayrıntısı
    değil, yığının neden kalkmadığıdır. `allowedPackages` bir paket **kurulurken**
    denetlenir: geçen ay kurulmuş bir servis, onu reddedecek liste bugün gelince de kurulu
    kalır. `requireSignature` **bir sonraki** yenilemenin neyi kabul edeceğine karar verir,
    önbellekte duran dizin hakkında hiçbir şey söylemez. `allowOverrides: false` yeni bir
    üstünyazım oluşturulmasını durdurur, diskte olanlara dokunmaz — onlar yayımlanmış
    paketin önünde okunmaya devam eder. Bunların her birinin bir onarımı var (yeniden üret,
    kaldır, yenile, sil) ve şimdiye kadar hiçbir şey kime, nerede olduğunu söylemiyordu.

    **Dört durum, ve raporu dürüst yapan üçüncüsü.** `holding` · `bypassed` · `silent` ·
    `unmeasured`. `market` bloğundaki her liste boşken *görüş yok* demektir, asla "hiçbiri"
    değil — dolayısıyla sessizliği yeşil bir tike katlayan bir rapor, **hiç politikası
    olmayan** bir makineyi tam uyumlu diye puanlardı; bir uyum raporunun yapabileceği en
    yanıltıcı şey budur. `unmeasured` ise iki şeyi birden kapsıyor: uygulamanın göremediği
    bir olgu (üretilen ağaç okunamadı, bir paket manifestosu yüklenemedi) **ve** uygulanacak
    bir şeyi olmayan bir madde — bu yapının hiç çalıştırmadığı bir depoyu adlandıran
    `imagePins` girdisi hiçbir şey yapmayan bir satırdır, ki bu `policy.rs`'in kendi
    kurmadığı bir anahtarı kilitlemek için zaten koyduğu adın aynısı. İkisi de uyum kanıtı
    değil.

    **`attestable`, `compliant` değil.** `verify.rs` aynı şekle sahip ve **ters** karar
    veriyor: onun `unknown`'ı `ready`'yi tutmuyor, çünkü sormaktan kaçındığı bir soru
    projenin bozuk olduğunun kanıtı değil. Bu, *"çalışabilir miyim?"* için doğru. Burada
    sorulan soru başka: *"biri bunun altına imzasını atabilir mi?"* — ve orada sorulmamış
    soru tam da yutulmaması gereken şeydir; bu yüzden `bypassed` **ya da** `unmeasured` olan
    her şey hükmü geri tutuyor. Adın kendisi de kasıtlı: raporlanan katman bir güvenlik
    sınırı değil — `policy.rs` bunu ilk paragrafında söylüyor — ve sertifika bu uygulamanın
    vereceği bir şey değil.

    **Hiçbir yeni ölçüm yok ve motora hiç sorulmuyor:** yazıldığı hâliyle `.env`
    (`Env::parse`, `load` değil — `load` politikayı en sona uygular, yani yüklenmiş ortama
    "politika tuttu mu" diye sormak cevabı yapısı gereği `true` olan bir soru sormaktır),
    üretilen ağaç, paket dizini, hatırlanan katalog kaynağı, üstünyazım dosyaları, proje
    manifestoları. Docker kapalıyken çalıştırılamayan bir uyum raporu, en çok ihtiyaç
    duyulduğunda çalıştırılamayan rapordur. Ayna maddesini **aynanın kendisi** cevaplıyor —
    bir dosyanın kendi baytlarına `policy::rewrite` uygulamak onları değiştirecekse, ayna o
    dosyaya hiç ulaşmamıştır — burada ikinci bir tarayıcı yazmak yerine: derleme aşamaları
    ve zaten kayıt defteri adı taşıyan referanslar hakkında ikinci bir görüş olurdu ve
    yanlış olan bu olurdu.

    Yüzey: `compliance.rs` (+8 test), `policy_compliance` komutu, `Policy::image_pins`,
    Tanılama panelinde yalnız yönetilen makinede görünen uyum kartı, i18n ve yardım belgesi
    iki dilde, README, ARCHITECTURE, CHANGELOG. *(Z-2 ✅, Z-3 ✅; K-7, Z-1, K-6, K-8 açık)*

---

## N8. Kritik yol

Yayına giden zincir kısa: **imzalama kimliği (gün sayılı, dışsal) ∥ N1 + N2 → N3.**
N2'nin içinde de yalnız iki gerçek bağımlılık var — gate (#1) diğerlerinden önce,
`detected_spec` (#7) TLD değişiminden (#8) önce.

§10'un hükmü doğru: yayını tutan şey kodun kalitesi değil. Ama bu listede **üç gerçek kod
işi** var ve üçü de "yayından sonra" diye etiketlenmemeli — `NGINX_DIRECTIVES` indeksi
(sessiz yanlış yapılandırma), `OverviewPane` yolu (kullanıcıya görünen tek hata) ve
`policy::mirror` (kurumsal kurulumu tamamen bozuyor). İlki ve ikincisi toplam yirmi dakika.

**Durum: zincirin kod tarafı bitti.** N1'in yedisi, N2'nin dokuzu (biri karar, biri yarım)
ve yukarıdaki üç kod işi kapandı. Kalan kritik yol tamamen dışsal ve sırası şu:

1. **Apple Developer Program + Authenticode** — gün sayılı, paralel yürüyebilir, ve hiçbir
   şey onu beklemeden ilerleyemiyor değil: 2 ve 3 bugün yapılabilir.
2. **Sürüm numarası kararı** → üç dosyada yükseltme, CHANGELOG bölümlemesi, sürüm notu.
3. **Publish** → `updates:check` → **Windows'ta elle tur**.

İmzalama anahtarlarının kendi töreni bu turda tamamlandı ve **içerik anahtarının
döndürülebildiği pencere ilk varlıklı sürümde kapanıyor** — yani rotasyon gerekiyorsa
1'den önce yapılmalı, sonra değil.
