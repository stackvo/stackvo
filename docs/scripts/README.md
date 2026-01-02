# Changelog Scripts

Bu dizin, Stackvo projesinin changelog yönetimi için kullanılan scriptleri içerir.

## generate-changelog.sh

Git commit geçmişinden otomatik changelog oluşturur.

### Kullanım

**Manuel Kullanım** (Lokal test için):
```bash
./docs/scripts/generate-changelog.sh [versiyon]
```

**Otomatik Kullanım** (GitHub Actions):
- GitHub'da yeni bir tag oluşturduğunuzda otomatik çalışır
- Workflow: `.github/workflows/changelog.yml`

### Örnekler

```bash
# Unreleased olarak işaretle
./docs/scripts/generate-changelog.sh

# Belirli versiyon için
./docs/scripts/generate-changelog.sh 1.2.0
```

### Çıktılar

- `docs/tr/changelog.md` - Türkçe changelog
- `docs/en/changelog.md` - İngilizce changelog

### Conventional Commits

Script, aşağıdaki commit tiplerini tanır:

- `feat:` → Eklenenler / Added
- `fix:` → Düzeltmeler / Fixed
- `docs:` → Dokümantasyon / Documentation
- `refactor:` → Yeniden Yapılandırma / Refactored
- `perf:` → Performans / Performance
- `test:` → Testler / Tests
- `chore:` → Diğer / Chore

### GitHub Release İş Akışı

1. **Kodunuzu geliştirin** ve commit edin (Conventional Commits formatında)
   ```bash
   git commit -m "feat: yeni özellik eklendi"
   git commit -m "fix: hata düzeltildi"
   ```

2. **GitHub'da yeni bir release oluşturun**
   - Releases → Draft a new release
   - Tag: `1.2.0` (v prefix olmadan!)
   - Title: `1.2.0`
   - Description: İsteğe bağlı
   - Publish release

3. **GitHub Actions otomatik olarak**:
   - Changelog'u günceller
   - Değişiklikleri commit eder
   - GitHub Release'e changelog ekler

### Tag Formatı

> [!IMPORTANT]
> Tag oluştururken **"v" prefix kullanmayın**. Doğru format: `1.2.0`, `1.0.5` gibi.

**Doğru**:
- ✅ `1.0.0`
- ✅ `1.2.5`
- ✅ `2.0.0`

**Yanlış**:
- ❌ `v1.0.0`
- ❌ `v1.2.5`

## Notlar

- Bu scriptler dokümantasyon amaçlıdır
- Ana kullanım GitHub Actions üzerinden yapılır
- Manuel kullanım sadece test/geliştirme amaçlıdır
- Tüm commit'ler Conventional Commits formatında olmalıdır
