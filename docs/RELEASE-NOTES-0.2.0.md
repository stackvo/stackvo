# StackVo 0.2.0

**2 September 2026** · [Türkçe](#stackvo-020-türkçe)

StackVo manages Docker-based local development environments from a desktop
window. Every project gets its own PHP version, its own database, its own
domain and its own HTTPS certificate — without typing `docker compose` or
installing a single PHP on your machine.

This is the first release with a changelog behind it. The engineering log lives
in [`CHANGELOG.md`](../CHANGELOG.md) and is long on purpose; this page is the
short version.

## What you get

- **A project per repository, generated rather than hand-written.** Compose
  files and Dockerfiles are rendered from `stackvo.json` every time, so you edit
  the manifest and never the output. The app tells you when what is on disk has
  drifted from what it would write.
- **A monorepo can be one project.** `api/` in Go, `web/` in Next.js, `worker/`
  in Python — one entry, one start, one certificate.
- **A full environment per git branch.** Each worktree gets its own hostname,
  its own database and a login that reaches only that database, so a branch
  cannot read the one it was branched from.
- **"Why was this request slow?"** The profiler, the query log and your `dump()`
  calls on one axis around a single request — and the request that actually
  failed can be replayed, bound to the snapshot it ran against.
- **An MCP server for AI assistants.** 38 tools, read-only by default. Writes
  sit behind an explicit flag, can be bound to one project and can be given a
  time limit that ends by itself.
- **What Docker costs you, measured.** CPU-minutes and GB-hours per project,
  per day, rather than argued about.
- **Import from seven other local environments** — XAMPP, Laragon, MAMP,
  Laravel Valet, Laravel Sail, Laravel Herd and DDEV. It never writes a byte
  into the installation it imports from.

## Installing

Downloads are on the [Releases page](https://github.com/stackvo/stackvo/releases).
You need Docker Desktop (or Podman, rootless) and about 8 GB of RAM.

**These builds are not code-signed, and that is a decision rather than an
oversight.** What it means in practice:

- **macOS** will say the app is damaged and offer to move it to the bin. It is
  not damaged; that is Gatekeeper's message for "unsigned". Right-click the app
  and choose **Open**, or run
  `xattr -dr com.apple.quarantine /Applications/StackVo.app`.
- **Windows** SmartScreen will warn before installing. Choose **More info** →
  **Run anyway**.

What you *can* verify: the checksums published beside each artifact, and the
updater's own signature — in-app updates are signed with a key of their own and
the app refuses an update that is not.

## Known limitations

- The Windows and Linux ARM builds have not been walked through by hand yet.
  They compile in CI; that is not the same claim.
- There is no second update channel. `beta.json` becomes real once there is a
  release to put in it.
- Crash reports are shown to you and sent nowhere. There is no telemetry, no
  crash-reporting service and no server behind this app — see
  [`PRIVACY.md`](../PRIVACY.md).

## Reporting something

Bugs and requests: [Issues](https://github.com/stackvo/stackvo/issues).
Security: [privately](https://github.com/stackvo/stackvo/security/advisories/new),
never as a public issue — see [`SECURITY.md`](../SECURITY.md).

---

# StackVo 0.2.0 (Türkçe)

**2 Eylül 2026**

StackVo, Docker tabanlı yerel geliştirme ortamlarını bir masaüstü penceresinden
yönetir. Her proje kendi PHP sürümünü, kendi veritabanını, kendi alan adını ve
kendi HTTPS sertifikasını alır — `docker compose` yazmadan ve makinenize tek bir
PHP kurmadan.

Bu, arkasında bir değişiklik günlüğü olan ilk sürüm. Mühendislik günlüğü
[`CHANGELOG.md`](../CHANGELOG.md) dosyasında ve bilerek uzun; bu sayfa kısa
olanı.

## Neler var

- **Depo başına bir proje, elle yazılan değil üretilen.** Compose dosyaları ve
  Dockerfile her seferinde `stackvo.json`'dan üretilir; siz manifest'i
  düzenlersiniz, çıktıyı asla. Diskteki dosya üreticinin yazacağından
  ayrıldığında uygulama bunu söyler.
- **Monorepo tek proje olabilir.** Go ile `api/`, Next.js ile `web/`, Python ile
  `worker/` — tek giriş, tek başlatma, tek sertifika.
- **Git dalı başına tam ortam.** Her worktree kendi alan adını, kendi
  veritabanını ve yalnızca o veritabanına erişen bir kullanıcıyı alır; yani dal,
  türediği veritabanını okuyamaz.
- **"Bu istek neden yavaştı?"** Profiler, sorgu günlüğü ve `dump()`
  çağrılarınız tek bir isteğin etrafında aynı eksende — ve gerçekten düşen istek,
  koştuğu snapshot'a bağlanarak yeniden oynatılabilir.
- **Yapay zekâ asistanları için MCP sunucusu.** 38 araç, varsayılan salt okunur.
  Yazma yetkisi açık bir bayrağın arkasında, tek projeye bağlanabilir ve
  kendiliğinden biten bir süre sınırı alabilir.
- **Docker'ın size maliyeti, ölçülerek.** Proje başına, gün başına CPU-dakikası
  ve GB-saat — tartışılarak değil.
- **Yedi başka yerel ortamdan içe aktarma** — XAMPP, Laragon, MAMP, Laravel
  Valet, Laravel Sail, Laravel Herd ve DDEV. İçe aktardığı kuruluma tek bayt
  yazmaz.

## Kurulum

İndirmeler [Releases sayfasında](https://github.com/stackvo/stackvo/releases).
Docker Desktop (ya da rootless Podman) ve yaklaşık 8 GB RAM gerekiyor.

**Bu yapılar kod imzalı değil, ve bu bir gözden kaçma değil bir karar.**
Pratikte anlamı şu:

- **macOS** uygulamanın bozuk olduğunu söyleyip çöpe taşımayı önerecek. Bozuk
  değil; Gatekeeper "imzasız"ı böyle söylüyor. Uygulamaya sağ tıklayıp **Aç**
  deyin, ya da
  `xattr -dr com.apple.quarantine /Applications/StackVo.app` çalıştırın.
- **Windows** SmartScreen kurulumdan önce uyaracak. **Daha fazla bilgi** →
  **Yine de çalıştır**.

Doğrulayabilecekleriniz: her dosyanın yanında yayımlanan sağlama toplamları, ve
güncelleyicinin kendi imzası — uygulama içi güncellemeler ayrı bir anahtarla
imzalanır ve uygulama imzasız bir güncellemeyi reddeder.

## Bilinen sınırlar

- Windows ve Linux ARM yapıları henüz elle baştan sona denenmedi. CI'da
  derleniyorlar; bu aynı iddia değil.
- İkinci bir güncelleme kanalı yok. `beta.json`, içine konacak bir sürüm
  olduğunda gerçek olacak.
- Çökme raporları size gösterilir ve hiçbir yere gönderilmez. Telemetri yok,
  çökme raporlama servisi yok, uygulamanın arkasında sunucu yok — bkz.
  [`PRIVACY.md`](../PRIVACY.md).

## Bildirim

Hatalar ve istekler: [Issues](https://github.com/stackvo/stackvo/issues).
Güvenlik: [özel olarak](https://github.com/stackvo/stackvo/security/advisories/new),
asla açık bir konu olarak değil — bkz. [`SECURITY.md`](../SECURITY.md).
