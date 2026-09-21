# Başlarken

Insiders, StackVo'yu `main`'den çalıştıranlardır: bir sonraki sürümün
kesileceği ağaç, [değişiklik günlüğünün](changelog.md) *Unreleased* altında
listelediği her düzeltmeyle ve beklemeden. Ayrı bir derleme, açılacak bir
kilit yok: depo herkese açık ve bu sayfa sizi bir klondan uygulamanın
makinenizde çalışmasına götürür.

Aşağıdakilerin hepsi Docker kurulu bir dizüstünde çalışır. Araçlar için on
dakika, ilk Rust derlemesi için birkaç dakika daha; sonrasında uygulama
saniyeler içinde açılır.

## Gereksinimler

| Ne | Sürüm | Nerede sabitlenmiş |
| --- | --- | --- |
| Node.js | 22 | `.nvmrc` |
| Rust | 1.96.1, `rustfmt` ve `clippy` ile | `src-tauri/rust-toolchain.toml` |
| Tauri 2 sistem kütüphaneleri | platforma göre | [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites/) |
| Docker | güncel herhangi bir motor | uygulamanın yönettiği şeyleri çalıştırmak için |

`rustup` toolchain dosyasını okur ve tam o sürümü kendisi kurar. Node, nvm
varsa `nvm use` ile, yoksa nodejs.org'dan gelir.

## Kodu alın

```bash
git clone https://github.com/stackvo/stackvo.git
cd stackvo
npm install
```

`npm install` Tauri CLI'ı da getirir; global hiçbir şey kurulmaz.

## Çalıştırın

```bash
npm run tauri:dev
```

İlk derleme Rust tarafını derler ve birkaç dakika sürer. Sonrasında Vue
tarafındaki bir değişiklik yerinde yenilenir; Rust tarafındaki bir değişiklik
ikiliyi yeniden derleyip pencereyi yeniden açar.

Uygulamanın yönetecek bir çalışma alanına ihtiyacı var: kurulu bir kopyanın
kullandığı `~/.stackvo`. Önce `STACKVO_ROOT` ile, sonra alışılmış yerlerde
aranır; **Ayarlar → Çalışma alanı** içinden de seçebilirsiniz.

!!! warning "Aynı anda tek StackVo"

    Geliştirme derlemesi, kurulu kopya ile aynı çalışma alanını ve aynı
    konteynerleri yönetir. Birini başlatmadan önce diğerinden çıkın.

## Kontrolleri çalıştırın

```bash
npm run lint               # eslint + prettier
npm run test:js            # vitest, ön yüz
npm test                   # yukarıdaki artı cargo test
npm run test:e2e           # Playwright, erişilebilirlik dahil
npm run contracts:check    # IPC sözleşmesi koda karşı
npm run audit              # cargo-deny + npm audit
```

Push etmeden önce hepsini bir seferde, CI'ın yaptığı gibi çalıştırın:

```bash
tools/before-push.sh          # bu makinenin cevaplayabildikleri
tools/before-push.sh --all    # artı Linux ve Windows yarıları, bir konteynerde
```

Kapı betiği her kontrol için yeşil, kırmızı ya da *atlandı* yazar ve
atlananların nedenini söyler. **Hiçbir dal bu betik çalışmadan push
edilmez.** CI aynı listeyi Linux, macOS ve Windows'ta `cargo clippy -D
warnings` ve `cargo fmt --check` ile sorar; oradaki bir kırmızı, buradakinden
çok daha pahalıdır.

## Kurulum paketlerini üretin

```bash
npm run tauri:build
```

Bir sürümün ürettiği paketlerin aynısını, imzasız üretir. İmzasız bir
derlemede her işletim sisteminin istediği ek tıklama
[Kurulum](../getting-started/installation.md) sayfasında.

## Neler nerede

- `src/` — Vue 3 ön yüz: görünümler, bileşenler, composable'lar ve arka ucu
  çağıran tek yer olan `lib/ipc.js`.
- `src-tauri/src/` — Rust arka uç, düz, her konu için bir modül;
  `commands.rs` IPC yüzeyidir.
- `contracts/` — manifest şeması, `.env` anahtarları, IPC yüzeyi.
- `docs/` — bu site ve yardım kartları; `docs/README.md` sitenin nasıl
  derlendiğini anlatır.
- `tests/` ve `src-tauri/tests/` — vitest, Playwright ve Rust takımları.

[ARCHITECTURE.md](https://github.com/stackvo/stackvo/blob/main/ARCHITECTURE.md)
haritadır: bilinmeye değer tek akış, katmanlar ve iki yarının neden bir tip
değil bir sözleşme paylaştığı. Testlerin uyguladığı kurallar (önce sözleşme,
üretilen dosyalar elle düzenlenmez)
[Pull request açma](../community/contributing/making-a-pull-request.md)
sayfasında; bir düzeltmenizi geri göndermeye değer bulduğunuz gün için.

## Bu site

```bash
python3 -m venv .venv && . .venv/bin/activate
pip install -r docs/requirements.txt
npm run docs:serve:tr       # http://127.0.0.1:8001/stackvo/tr/
```

Python kurmak istemiyorsanız temanın Docker imajı da olur. O komut ve siteyle
ilgili diğer her şey `docs/README.md` içinde.
