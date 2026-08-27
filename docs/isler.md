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
- **1 ölü kod:** `api.appsAvailable()` tanımlı, hiçbir view veya store çağırmıyor.
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
