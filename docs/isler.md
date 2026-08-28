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

> **Yapıldı.** `validate-contracts.mjs`'in `UNUSED_API` düzeni artık `api` ile noktanın
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

---

## N1. Yarım günlük blok — önkoşulsuz, hata sınıfı

Kendi içinde sıra: gizli hata → görünen hata → yalan söyleyen denetleyici → temizlik.

1. ~~**`NGINX_DIRECTIVES[0]` / `[4]` indeks erişimini anahtar aramasıyla değiştir.**~~
   **Yapıldı** — `generator::directive(key)` eklendi, Caddy iki yönergesini anahtarla
   alıyor. Üç test: konum yerine anahtar (gövde zaman aşımını `max_size`'a kaydıran
   sürüm kırılıyor), her anahtarın kendi satırına çözülmesi, ve **tablonun dokuz
   varsayılanının `config::SETTINGS` ile eşitliği** — B-1'in ikinci yarısı, daha önce
   hiçbir şey karşılaştırmıyordu. *(B-1)*
2. ~~**`OverviewPane.vue`'nun kapsayıcı yolunu runtime'dan türet.**~~ **Yapıldı** —
   `containerPath`, `render_dockerfile`'ın kendi dağıtımını aynalıyor (PHP değilse `/app`),
   yani dokuzuncu bir çalışma zamanı iki tarafta da düzenleme istemiyor.
   `tests/project-overview.spec.js`, 9 test. *(G-1)*
3. ~~**`validate-contracts.mjs` regex'ini `api\s*\.\s*<method>` yap** ve P2-5'ten
   `appsAvailable` maddesini çıkar.~~ **Yapıldı** — `[F] reachability` temiz,
   `contracts:check` 0 hata / 11 uyarı. *(H-1)*
4. ~~**`validate-contracts.mjs`'deki `'8.2'` yedeğini `8.4` yap.**~~ **Yapıldı** — ama
   beşinci bir literal yazarak değil: yedek artık `envSpec`'ten, yani şemanın kendi
   `default`'undan okunuyor, ve şema onu taşımıyorsa bu bir **hata**. Dört kopya üçe indi.
   *(B-3)*
5. ~~**Python varsayılanını tek kaynağa indir.**~~ **Yapıldı** — `lang_defaults`
   python/go/ruby için `config::SETTINGS`'i okuyor (`settings_version`); rust/bun/deno
   gerekçesi yazılı literallerini koruyor. İki test: üçünün tabloyu okuduğu, ve diğer
   üçünün **okumadığı** — sonraki bir "tutarlılık" turu Deno'yu var olmayan bir etikete
   bağlamasın diye. *(B-4)*
6. ~~**Kökteki 0 baytlık `version` dosyasını sil.**~~ **Yapıldı** — `git rm`; hiçbir
   Rust, betik ya da iş akışı onu okumuyordu (arandı). *(P3-2)*
7. ~~**Paket üstverisi.**~~ **Yapıldı** — `package.json`'a `repository`/`bugs`/`homepage`/
   `author`/`engines`, `Cargo.toml`'a `repository`/`homepage`/`keywords`/`categories`,
   `authors` LICENSE'taki ada çevrildi, `.nvmrc` = 22 (CI ile aynı). İki test
   `version_agreement.rs`'e eklendi: krediyi lisansla, deponun adresini iki manifest
   arasında bağlıyor. N4 #5'in (Dependabot) önkoşulu artık karşılandı. *(P3-3)*

---

## N2. Yayın bloklayıcıları — kod işi

Kendi içinde sıra: **önce gate, sonra düzeltme** — yoksa aynı sınıf hata üçüncü kez döner.

1. ~~**Bağlantı/iddia denetimini genelleştir.**~~ **Yapıldı** —
   `every_link_points_at_a_file_that_exists` artık `LINKED_DOCUMENTS` üzerinde: altı
   belge (`ARCHITECTURE`, `README`, `SECURITY`, `CONTRIBUTING`, `PRIVACY`,
   `ACCESSIBILITY`). Ayrıştırıcı koruması belge başına değil **toplam** üzerinde,
   çünkü ikisi meşru olarak hiçbir yere bağlanmıyor. *(P1-2)*
2. ~~**README üreteç bölümü.**~~ **Yapıldı** — bölüm "devralma nasıl bitti" olarak
   yeniden yazıldı, tablo iki davranışı gösteriyor, `verify`'ın anlam değiştirdiği
   söyleniyor. Kendine bağlanan cümle de düzeltildi: `stackvo/stackvo` bağlantısı
   okuyucuyu aynı sayfaya geri getiriyordu. **Yeni gate:**
   `the_readme_names_the_generator_default_the_enum_actually_carries` — `#[default]`'ı
   taşıyan varyantı ayrıştırıyor, README başka bir varsayılan adlandıramıyor. *(P0-5)*
3. ~~**README Windows paragrafı.**~~ **Yapıldı** — derlendiği ve birim testlerinin
   geçtiği yazıldı; doğrulanmamış olan üç şey (UAC üzerinden hosts yazımı, gerçek Docker
   Desktop'a karşı adlandırılmış boru, tarayıcıda alan adı çözümlemesi) ayrıca sayıldı.
   **Yeni gate:** `the_readme_does_not_deny_a_windows_build_the_matrix_performs` —
   `ci.yml` `windows-latest` içeriyorsa README "hiç derlenmedi" diyemiyor. *(R-1a)*
4. **README'yi son kullanıcıya çevir** — **büyük ölçüde yapıldı**, tek eksikle:
   - ✅ **Installing it** — altı kurulum biçimi, platform başına sistem gereksinimi,
     Docker gereksinimi. Yayın henüz yok, ve bunu söylüyor. *(P0-4)*
   - ✅ **What Docker costs you** — dört maliyet tablosu, karşılığında ne alındığı, ve
     "bu takas sana yanlış geliyorsa yanlıştır; kapanacak bir açık değil, mimarinin
     kendisi" cümlesi. *(R-2)*
   - ✅ **Coming from something else** — yedi kaynak adıyla, ve diğer kuruluma tek bayt
     yazılmadığı. *(R-13)*
   - ✅ **What it does that gets missed** — altı özellik tabloyla. *(M7)*
   - ❌ **Ekran görüntüsü ve rozet yok.** Görüntü üretilemiyor; bu madde açık kalıyor.
   - ❌ **Türkçe README yok.** Açık.

   **Yeni gate:** `the_readme_counts_the_surfaces_it_advertises` — yedi `release_*`, altı
   `worktree_*` ve `imports::ALL`'un yedi kaynağını ağaçtan sayıp README'yle
   karşılaştırıyor. `imports.rs`'in başlığı "**Two** of them" derken `ALL` yediyi
   taşıyordu (R-12); aynı sınıfın README tarafı artık kapalı.
5. ~~**Geçici dosya stage'ini sertleştir.**~~ **Yapıldı** — `elevate::staging_dir`
   çağrı başına `0700` bir dizin açıyor (süreç kimliği + sayaç), `create_dir_all` değil
   `create_dir`: var olan bir adı **benimsemek**, onu yaratanı benimsemektir — saldırının
   tam olarak hamlesi buydu. `hosts.rs` ve `dns.rs` ikisi de oraya taşındı, temizlik
   `remove_dir_all`. Üç test: çağrı başına ayrı dizin, `0700` (yazılmadan **önce**),
   ve dolu adın reddi. *(P1-1)*
6. ~~**`docker run` imgelerini `policy::mirror`'dan geçir.**~~ **Yapıldı** —
   `policy::run_image` eklendi (aynayı okuyan tek nokta), on üç çağrı yeri ona bağlandı:
   `tunnel.rs` ×9, `tunnelid.rs`, `landing.rs`, `perf.rs` ×2. Üç test: `run_image`'in
   `mirror`'ın **her** muafiyetini koruduğu, politikasız makinede kimlik olduğu, ve
   **kapsama gate'i** — `_IMAGE` sabiti taşıyan her üretim modülü `run_image`'i çağırmak
   zorunda, yani beşinci bir modül eklendiğinde bu birinin hava boşluklu makinesinde
   değil burada görünüyor. *(D-2)*
7. ~~**`detected_spec`'e `Env` geçir.**~~ **Yapıldı** — imza `(name, detected, env)`,
   üç çağrı yeri de `Env::load`'u geçiyor (`adopt_many` toplu iş için bir kez).
   PHP, sunucu ve Node sürümü artık ayardan; **klasörün beyanı ayarı yenmeyi
   sürdürüyor** (bir `package.json` karar, ayar yalnız karar vermemiş klasörün cevabı).
   `detected.server` kaldırıldı — `detect.rs`'in dört yerinde de `"nginx"`'ti, yani
   tespit değil, değiştirilemeyecek bir yere yazılmış ayardı. Üç test, biri **alan adının
   bilerek okunmadığını** kilitliyor: `DEFAULT_TLD_SUFFIX` kendini proje alan adlarının
   kullanmadığı bir anahtar olarak tanımlıyor, ve sonraki bir "işi bitirme" turu onu
   uygulatmasın. *(A-2)*
8. ~~**Varsayılan TLD'yi `.test` yap.**~~ **Yapılmayacak — karar verildi, `.loc` kalıyor.**

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

9. ~~**`contracts:check`'e "şema varsayılanı = kod varsayılanı" suiti.**~~ **Yapıldı** —
   suite D'ye `DEFAULT_DISAGREES`. `EMBEDDED_VALUES` `SETTINGS`'i zaten anahtar **ve**
   değeriyle ayrıştırıyordu, yani maliyeti bir döngü. **İlk koşuşunda E-2'yi yakaladı:**
   şema `1.62`, kod `1.84` — `"status": "conflicting"` bir insana not, kapı değil. Şema
   `1.84`/`active` yapıldı. Gate boş geçmediği kasıtlı bir ayrışmayla doğrulandı. *(E-2)*
10. ~~**On `SERVER_*` anahtarını `env.schema.json`'a ekle.**~~ **Yapıldı** — yeni
    `serverLimits` grubu. Anahtarlar, nginx adları ve varsayılanlar **elle yazılmadı**:
    `generator.rs`'den ve `config::SETTINGS`'ten okunup ikisinin eşit olduğu doğrulanarak
    üretildi. `contracts:check` **12 uyarıdan 1'e** düştü ve `[D] env keys — clean`;
    kalan tek uyarı beklenen olan (`--allow-no-manifests`). *(P2-5 / E-3)*
11. ~~**Koddaki üç bayat rekabet iddiasını tarihle.**~~ **Yapıldı** — üçü de "Ağustos
    2026'da ölçüldü" biçiminde yeniden yazıldı ve iddia daraltıldı (worktree'nin önü
    "hiçbiri" değil, dde'nin yapmadığı iki yarım; imports yedi; MCP ≥7/17).
    **Ve gate on modül daha buldu** — rapor üç diyordu. Ölçmediğim için tarihleyemedim;
    `UNDATED` listesinde **beyan edilmiş borç** olarak duruyorlar (`published_urls.rs`
    deseni: her zaman geçen dar bir denetim yerine gerekçeli bir liste). Listeden çıkan
    gerçek bir tarih taşımak zorunda, listeye girmeyen yeni modül de. *(R-12)*

---

## N3. Yayın koşusu — kod değil, sıraya koyma

0. **İmzalama kimliklerini bugün başlat** — Apple Developer Program + Authenticode günler
   alıyor; takvimin kritik yolu bu ve N1/N2 ile **paralel** yürümeli. *(P0-3)*
1. Sürümü yükselt (ör. `0.2.0`), `CHANGELOG.md`'de sürüm başlığını aç, 4300 satırlık
   `Unreleased`'i oraya taşı, etiketle. *(P0-1)*
2. Kullanıcıya dönük **kısa** sürüm notu yaz — mevcut CHANGELOG bir mühendislik günlüğü ve
   sürüm notu olarak kullanılamaz.
3. Yayın koşusunu rehearsal'da uçtan uca doğrula, sonra **Publish** et; `releaseDraft: true`
   olduğu için basılmadıkça `latest.json` 404 verir. *(P0-2)*
4. `npm run updates:check` ile ucu doğrula.
5. **Windows makinede elle tur:** `preflight` → proje oluştur → `up` → tarayıcıda aç.
   Kategorinin 13/17'si Windows'ta ve bir CI koşusu bu soruyu cevaplamıyor. *(R-1b)*

---

## N4. Yayından hemen sonra — ucuz, yüksek etki

Kendi içinde sıra: var olanın önünü açanlar → dağıtım → katalog boşlukları.

1. ~~**Dört enstrümanı MCP'ye koy.**~~ **Yapıldı** — `stackvo_explain_request`,
   `stackvo_timeline`, `stackvo_query_log`, `stackvo_flame`. Araç sayısı 34 → **38**.
   İkisinin mantığı Tauri `State`'ten ayrıldı (`explain_request`, `build_timeline`),
   `verify_generator`'ın deseniyle — MCP'de kopyalamak ikinci bir kopya olurdu. README'nin
   araç sayısını **mevcut gate yakaladı** (34→38) ve düzeltildi. Yeni gate: dört komutun
   da bir MCP aracı tarafından uygulandığını araç adıyla değil **komut adıyla** doğruluyor,
   çünkü araç yeniden adlandırılabilir ve boşluk aynı boşluk olurdu. *(S-4)*
2. ~~**`audit`'e bir okuma komutu + panel.**~~ **Yapıldı** — `audit::tail_of`,
   `audit_trail` IPC komutu (310. komut), `AuditPane.vue`, sözleşme kaydı, iki dilde
   yardım belgesi. Okuma için ayrı bir `Record` şekli: `Entry::action` bilerek
   `&'static str` ("aynı fiil sonsuza kadar aynı dize") ve derleyici bu sözü böyle
   tutuyor, dolayısıyla tek yapı iki işi görsün diye onu `String`'e genişletmek yazma
   tarafının dayandığı bir değişmezi okuma tarafının ihtiyacı olmayan bir tanım için
   takas etmek olurdu. Beş Rust + beş Vue testi. **Testler kendi panelimde gerçek bir
   hata buldu:** hata durumunda hem hata hem "hiçbir şey yapılmadı" görünüyordu —
   "bakamadım" ile "hiçbir şey yok" farklı cümleler. *(S-1 birinci yarısı)*
3. ~~**Son hatayı yapılandırılmış MCP kaynağı yap.**~~ **Yapıldı — ama raporun çerçevesi
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
4. ~~**Odak modu.**~~ **Yapıldı** — `focus.rs` (saf mantık, 8 test), `focus_plan` +
   `focus_apply` IPC komutları (311. ve 312.), `ProjectDetail`'de plan diyaloğu, iki dilde
   metin. Plan/uygula ayrımı `preset`/`worktree`/`release` deseniyle, ve **plan uygula
   tarafında yeniden yapılıyor** (`provider`'ın kuralı: "düğmeyi sunan ekran dakikalar
   önce olabilir"). Kararlar yazılı: yalnız **gerekli** bağımlılıklar izleniyor (isteğe
   bağlı olan zaten odağın durdurmak istediği şey), bir servisin **her** örneği korunuyor
   (manifest 8.0 ile 8.4 arasında seçim yapamaz ve yanlış tahmin projenin bağlı olduğu
   veritabanını durdurur), ve **hiçbir servis beyan etmeyen proje reddediliyor** —
   `services`'in boş hâli "beyan yok", "ihtiyaç yok" değil, ve ona göre davranmak tüm
   workspace'i doldurulmamış bir alan uğruna durdururdu. *(U-2)*
5. ~~**Açık kaynak dosyaları.**~~ **Yapıldı** — `dependabot.yml` (üç ekosistem, gruplu,
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
6. **Homebrew cask + winget manifesti** — `release.yml` altı hedefin sha256'sını zaten
   üretiyor. *(R-9)*
7. ~~**`installers:check` script'ini düzelt.**~~ **Yapıldı — ama teşhis düzeltilerek.**
   Ölçüldü: CI aracı **doğrudan** çağırıyor (`release.yml:534`), npm script'ini değil.
   Yani script'in hiç çağıranı yok ve kırık olan CI değil, script'in kendisi.

   Ve `--target`'ı varsayılana bağlamak **yanlış düzeltme** olurdu: aracın var oluş
   sebebi ana makineye sessizce düşmüş bir çapraz derlemeyi yakalamak, bir varsayılan
   üçlü onu yakalaması gereken hatayla anlaştırırdı. Bunun yerine çıkış eyleme
   dönüştürüldü — `rustc -vV`'den bu makinenin üçlüsünü basıyor ve npm'in yuttuğu `--`'yi
   söylüyor, yani çıkmaz değil kopyalanabilir bir satır. `package.json`'daki girinti
   farkı da düzeltildi; o zaten satırın hiç koşturulmadığının işaretiydi. *(P2-3)*
8. ~~**Şablon ↔ tespit kapsama testi.**~~ **Yapıldı — ve teşhisin bulmadığı iki hata
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
9. ~~**Üç sağlayıcı reçetesi sevk et.**~~ **Yapıldı — ama üçü rapordakiler değil, ve
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
10. ~~**MSSQL ve Beanstalkd paketleri.**~~ **Yapıldı** — paketler
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
11. **`debugbridge`'e kuyruk işi ve istek olayları** — `kind` alanı ve `timeline.rs`'in
    ekseni ikisi de bekliyor; bugün tek değer `"dump"`. Ücretli rakiplerin ana satış kalemi.
    *(R-8)*

---

## N5. Bakım borcu

1. Rust kapsam tabanını ölçülen değere yaklaştır — %64,05 ölçülüyor, taban %60, yani dört
   puanlık bir gerileme bugün sessizce geçer. *(P2-1)*
2. Docker'lı nightly duman testi — Linux runner'da Docker hazır geliyor; "Docker açıkken
   proje kalkıyor mu" sorusunu CI'da soran hiçbir şey yok. *(P2-2)*
3. Paket bütçesine pay yarat ya da tavanı gerekçeli yükselt — tavana %1,1 kalmış. *(P2-4)*
4. `commands.rs`'i alt sistemlere böl — 15.6k satır, 303 IPC komutunun tamamı, 113 modülün
   110'u konusuna göre ayrılmışken. *(P3-1)*
5. Yedi major bağımlılık geçişi — Dependabot açıldıktan **sonra**, yayın öncesi değil. *(§7)*

---

## N6. Arayüz ve hardcode borcu

Kendi içinde sıra: kullanıcının bugün yapamadığı şeyler → tema tutarlılığı → sürüklenme
kapıları → yazılmamış kararlar.

1. **Altı `*_VERSIONS` listesine `PhpPane`'de `v-combobox`** — `SETTINGS`'in 36 anahtarından
   yedisi `src/` ağacında bir kez bile geçmiyor, ve listeler şimdiden geride (Go 1.23, Ruby
   3.3, Node 23, Rust 1.84). Asıl mesele eksikliğin **yalnız bir yayınla** kapanabilmesi.
   Desen `PhpPane.vue:158,168`'de hazır, yaklaşık 30 satır. *(A-1)*
2. **Uygulama seçicilerine "diğer…" seçeneği** — Helix, Neovim, Emacs ya da listelenmeyen
   sekiz JetBrains IDE'sinden birini kullanan bugün hiçbir şey seçemiyor. *(A-3)*
3. **Grafik ve ısı ızgarası renklerini temadan oku** — `#1976D2` dört yerde sabit; accent
   moraldığında üç pasta ve sparkline mavi kalıyor. `useTheme().current.value.colors.primary`
   çalışma anında okunabilir. *(C-1, C-2, C-3)*
4. **`quickcmd`/`oauth`/`tooling` düzyazısını `hints.rs` desenine taşı** — 39 İngilizce cümle
   iki dilli pencereye ham gidiyor; en az `lang="en"` ile işaretle (beş bileşende hiç yok).
   *(F-1)*
5. **Literalleri adlandırılmış sabitlere indir** — `stackvo.loc` ×14, `stackvo-net` ×9,
   `nginx` ×7; `certs::FALLBACK_SUFFIX` zaten bunun için yazılmış ve `commands.rs` sekiz kez
   görmezden geliyor. Aynı turda `SUPPORTED_SERVERS_DEFAULT`'un beş yerdeki
   `unwrap_or("nginx")` baypası. *(B-2, A-5)*
6. **`env.schema.json`'ın `consumers` alanını yeniden ölç ya da sil** — 43 yolun **39'u
   diskte yok** ve `measure-env-usage.mjs` `core/` olmadığı için çalışamıyor. Ölçülemeyen bir
   alan sözleşmenin geri kalanına olan güveni aşındırıyor. *(E-1)*
7. **`<root>/commands.json`** — makine geneli komut, `stackvo.json`'ın `commands` şemasının
   aynısı, aynı argv kuralı, aynı konteyner sınırı; + paket manifestine `commands` alanı,
   aynı imza zincirinden. Dört rakip bunu ilk sırada satıyor. *(A-4 / R-6)*
8. **`<proje>/stackvo.preset.json` yerleşim kuralı** — `preset.rs` doğru sorunu çözmüş;
   eksik olan tek şey dosyanın nereye konacağı ve klonlayanın onu nasıl bulacağı. *(R-11)*
9. **Podman rootless soketi** `engine.rs:59-134` listesine + bir uyumluluk fikstürü.
   *(R-2 ikinci yarısı / §8.6)*
10. **RoadRunner sürücüsü** — Octane'ın iki sürücüsünden ikincisi; Swoole var, çift yarım.
    *(R-10)*
11. **Yazılmamış kararları yaz** — çalışma zamanı sınırı neden sekiz
    (`manifest::LANG_RUNTIMES` başlığı), CLI neden yalnız İngilizce (§8.4), bulut/Codespaces
    sınırı (R-14), taşınabilirliğin neden imkânsız olduğu (R-16), quickcmd kataloğunun neden
    kapalı olduğu (A-4). Bir eksik ile bir karar arasındaki fark, kararın yazılı olmasıdır.
12. **Kanal ve sürüm başına yükseltme notu** — `channel.rs` yazılmış, `tauri.conf.json` tek
    uç tanımlıyor, `updates.js` manifesti zaten okuyor. *(§8.3, §8.5, U-5)*
13. **İlk açılış turu** — 309 komut, 26 panel; dört Gate var ama hepsi engel, tanıtım değil.
    *(§8.2, U-6)*
14. **Kullanıcı isteğiyle çökme raporu gönderme** — `crash.rs` kaydı zaten tutuyor,
    `diagnostics.rs` paketine iliştirilir, `PRIVACY.md`'nin sözü bozulmaz. *(§8.1)*

---

## N7. Stratejik — hendek

Kendi içinde sıra bağımlılığa göre: ilk üçü bir arada bir cümleyi tamamlıyor, dördüncüsü
kendi önkoşulunu taşıyor.

1. **Ajan kum havuzu** ⭐ — `worktree::create` + `db::copy_database` + TTL + kapsamlı MCP.
   **Yeni modül yok**; yedi parçanın yedisi de ağaçta. 17 rakibin hiçbirinde ajan izolasyonu
   yok ve yerel ikili mimaride kopyalanamaz. Ürünün konumunu değiştiren tek madde. *(K-1)*
2. **Telafi eylemi ve geri alma** — K-1'i güvenli yapan şey; `contracts/ipc.json` her komutun
   `query` mi `mutation` mı olduğunu zaten biliyor. *(S-1 ikinci yarısı)*
3. **Kapsamlı ajan yetkisi** — bugün `--allow-writes` 12 aracı birden açıyor ve içinde
   `stack_down` var. `websurface.rs` bu sorunun aynısını çözmüş: koşu başına üretilen token +
   loopback + salt-okuma. *(S-2)*
4. **Kendi imgelerini sürüme sabitle → kilit dosyası.** Önce D-1 (altısı `:latest`, ve
   `pkg::MOVING_TAGS` bunu üçüncü taraflara **yasaklıyor**), sonra `stackvo.lock`. En zor
   yarısı — belirlenimci üreteç, digest zinciri, imzalı doğrulama — çoktan bitmiş.
   Kategoride "npm ci" karşılığı yok. *(D-1 → K-2)*
5. **Ortam farkı** — `diagnostics.rs` üzerine bir karşılaştırma fonksiyonu; yeni ölçüm yok.
   *(K-3)*
6. **Kaynak bütçesi ve proje başına atıf** — `stats_store.rs` + `idle.rs` hazır. Stratejik
   değeri: **R-2'yi savunmaya çevirir** — Docker pahalı, ama pahalılığını ölçen tek ürün
   olmak, ölçmeyip inkâr etmekten iyi bir konumdur. *(U-1)*
7. **İstek tekrarı**, sonra anlık görüntüye bağlı hâli. `explain.rs`'in başlığı zaten
   *"no new measurement is needed"* diyor. *(K-4 → K-5)*
8. **Paylaşılabilir teşhis bağlantısı** ve **onboarding doğrulaması** — ikincisi K-2'den
   sonra kesinleşiyor. *(U-3, U-4)*
9. **ACME seçeneği** — DDEV'in planında yazılı, ServBay satıyor; kapanan pencere. *(M8)*
10. **Uzun kuyruk** — monorepo *(K-7)*, tedarik zinciri raporu *(Z-1)*, sır sızıntı taraması
    *(Z-2)*, politika uyum raporu *(Z-3)*, ortamla bisect *(K-6)*, çıkış görünürlüğü *(K-8)*.

---

## N8. Kritik yol

Yayına giden zincir kısa: **imzalama kimliği (gün sayılı, dışsal) ∥ N1 + N2 → N3.**
N2'nin içinde de yalnız iki gerçek bağımlılık var — gate (#1) diğerlerinden önce,
`detected_spec` (#7) TLD değişiminden (#8) önce.

§10'un hükmü doğru: yayını tutan şey kodun kalitesi değil. Ama bu listede **üç gerçek kod
işi** var ve üçü de "yayından sonra" diye etiketlenmemeli — `NGINX_DIRECTIVES` indeksi
(sessiz yanlış yapılandırma), `OverviewPane` yolu (kullanıcıya görünen tek hata) ve
`policy::mirror` (kurumsal kurulumu tamamen bozuyor). İlki ve ikincisi toplam yirmi dakika.
