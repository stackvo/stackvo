# Pull request açma

Bir fork'tan birleştirilmiş bir değişikliğe giden yol ve testlerin uyguladığı
ev kuralları; CI'dan önce siz karşılayın diye.

## Başlamadan önce

- **Hata düzeltmesi izin istemez.** Varsa issue'ya atıf yapın.
- **Özellik önce bir cümle ister.** Ne yapmayı planladığınızı ve kabaca
  nasıl yapacağınızı [istekte](requesting-a-change.md) ya da bir
  [tartışmada](https://github.com/stackvo/stackvo/discussions) söyleyin.
  Oradaki on dakika baştan yazılan bir pull request'ten kurtarır.
- **Pull request başına bir değişiklik.** Tek pull request'teki bir düzeltme
  ve bir refactor iki olarak incelenir, hiçbiri olarak birleştirilir.
  Değiştirmediğiniz kodu yeniden biçimlendirmeyin; toolchain'i, sabitlenmiş
  bir action'ı ya da bir bağımlılığı başka konudaki pull request'te
  yükseltmeyin.

## Adımlar

### 1. Fork ve dal

```bash
gh repo fork stackvo/stackvo --clone
cd stackvo
git switch -c fix/snapshot-restore-after-rename
```

Dal adları dalın ne yaptığını söyler: `fix/…`, `feat/…`, `docs/…`, `ci/…`.

### 2. Kurulum

[Başlarken](../../insiders/getting-started.md): Node 22, sabitlenmiş Rust toolchain'i,
`npm install`, `npm run tauri:dev`.

### 3. Değişikliği testiyle yapın

Her düzeltme, onu yakalayacak olan testle gelir ve test koruduğu davranışa
göre adlandırılır. Bir test gerçek bir hatayı koruyorsa yorumu hatanın ne
olduğunu söyler. Yorumlar neyi değil *neden*i açıklar: bir satır tuhaf
görünüyor ama doğruysa, onsuz neyin bozulacağını yazın.

### 4. Kapı betiğini çalıştırın

```bash
tools/before-push.sh          # bu makinenin cevaplayabildikleri
tools/before-push.sh --all    # ve Linux ile Windows yarıları, bir konteynerde
```

**Hiçbir dal bu betik çalışmadan push edilmez.** CI'ın sorduğu her şeyi sorar
(lint, vitest, cargo test, `-D warnings` ile clippy, fmt, sözleşme kontrolü,
denetimler) ve her kontrol için yeşil, kırmızı ya da *atlandı* yazar.
Platforma özgü koda dokunan her şey için `--all` kullanın: `engine.rs`,
`hosts.rs`, `pty.rs` ve benzerleri yalnızca Mac'ten derlenemez.

### 5. Push edip taslak açın

```bash
git push -u origin fix/snapshot-restore-after-rename
gh pr create --draft --fill
```

Taslak, sizin söylemenize gerek kalmadan "henüz değil" der. Yönü görmek
istiyorsanız erken açın; kapı betiği yeşil olunca hazır işaretleyin.

### 6. Şablonu doldurun

Pull request şablonu değişikliğin ne yaptığını ve nedenini sorar, bir kontrol
listesiyle biter:

- :material-checkbox-blank-circle-outline: `npm test` geçiyor (vitest + cargo test)
- :material-checkbox-blank-circle-outline: `npm run lint` geçiyor
- :material-checkbox-blank-circle-outline: `npm run contracts:check` temiz
- :material-checkbox-blank-circle-outline: Davranış ile `contracts/` uyuşmuyorsa sözleşme de güncellendi
- :material-checkbox-blank-circle-outline: Uygulamanın yönettiği çalışma alanına dokunulmadı

### 7. İnceleme

CI takımı Linux, macOS ve Windows'ta çalıştırır. Geliştirici inceler;
`CODEOWNERS` sözleşmeleri, araçları, iş akışlarını ve güvenlikle ilgili Rust
modüllerini bilinçli bir bakışa yönlendirir. Bir inceleme yorumuna geçmişi
üzerine yazarak değil, bir commit push ederek cevap verin; inceleyen, son
seferden bu yana olan farkı okur.

### 8. Birleştirme

CI yeşil olup inceleme bitince geliştirici birleştirir. Sonrasında dalınızı
silin.

## Ev kuralları

Bunlar test edilir; yani incelemede değil, kendi makinenizde düşer.

**Önce sözleşme.** `contracts/ipc.json` iki yarının paylaştığı her komutu ve
olayı adlandırır. Orada olmayan bir komut ne `lib.rs`'te kaydedilebilir ne de
CLI'dan sürülebilir. Davranış ile sözleşme uyuşmuyorsa biri hatalıdır; kod
yazmadan önce hangisi olduğuna karar verin ve ikisini aynı pull request'te
değiştirin.

**Üretilen dosyalar elle düzenlenmez.** `generated/` her seferinde
manifestten üretilir. Rust üreticisi Bash olanla bayt bayt karşılaştırılır;
karşılaştırmayı silerek "sadeleştirmeyin".

**Bir kartın işaretlemesi ve stilleri birlikte taşınır.** `<style scoped>`
yalnızca kendi bileşenine ulaşır; `tests/pane-styles.spec.js` bunu denetler.

**Belgeler ağaca karşı test edilir.** ARCHITECTURE.md'deki bir sayı,
PRIVACY.md'deki bir sunucu adı, bir yardım kartındaki konu adı: her birinin
dosyayı okuyan ve kod uyuşmayınca düşen bir testi var. Belgeyi ve kodu
birlikte değiştirin.

**Çalışma alanını koddan asla değiştirmeyin.** Uygulamanın yönettiği klasör
kullanıcınındır. Uygulama onu okur ve içine yalnızca açık bir kullanıcı
eylemiyle yazar.

**Bağımlılık bir karardır.** `cargo deny` ve bağımlılık incelemesi her pull
request'te lisansları ve güvenlik uyarılarını denetler. Yeni bir crate ya da
paketi, neden gerektiğiyle birlikte açıklamada adlandırın.
