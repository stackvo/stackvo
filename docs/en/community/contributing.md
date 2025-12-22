# Contributing Guide

Stackvo'a katkıda bulunmak için teşekkürler! 🎉 Bu kılavuz, repository fork etmeden pull request göndermeye, conventional commits kullanımından code style'a, bug report ve feature request açmaktan testing ve CI/CD süreçlerine kadar katkıda bulunma sürecinin tüm adımlarını detaylı olarak açıklamaktadır. Kod, dokümantasyon, testing ve community desteği gibi farklı katkı alanları hakkında bilgi içerir.

## Hızlı Başlangıç

### 1. Repository'yi Fork Edin

```bash
# Fork edin: https://github.com/stackvo/stackvo/fork

# Clone edin
git clone https://github.com/YOUR_USERNAME/stackvo.git
cd stackvo
```

### 2. Development Environment Kurun

```bash
# Dependencies
docker --version
docker compose --version

# CLI kur
./cli/stackvo.sh install

# Test et
stackvo doctor
```

### 3. Branch Oluşturun

```bash
# Feature branch
git checkout -b feat/my-feature

# Bugfix branch
git checkout -b fix/bug-description
```

### 4. Değişikliklerinizi Yapın

```bash
# Kod değişiklikleri
nano .env

# Test edin
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

### 5. Commit Edin

**Conventional Commits** formatını kullanın:

```bash
git commit -m "feat(mysql): add MySQL 8.1 support"
git commit -m "fix(traefik): resolve SSL certificate issue"
git commit -m "docs(readme): update installation guide"
```

**Commit Types:**
- `feat`: Yeni özellik
- `fix`: Bug düzeltme
- `docs`: Dokümantasyon
- `style`: Kod formatı
- `refactor`: Kod refactoring
- `perf`: Performance
- `test`: Test
- `chore`: Diğer

### 6. Push ve Pull Request

```bash
# Push
git push origin feat/my-feature

# GitHub'da Pull Request oluşturun
```

---

## Contribution Checklist

Pull Request göndermeden önce:

- [ ] Kod değişiklikleri test edildi
- [ ] Dokümantasyon güncellendi
- [ ] Conventional commits kullanıldı
- [ ] Conflict yok
- [ ] CI/CD testleri geçti

---

## Katkı Alanları

### 1. Kod Katkıları

- **Yeni Servisler:** PostgreSQL 16, Redis 7.2, vb.
- **Yeni Özellikler:** Monitoring, backup, vb.
- **Bug Fixes:** Issue'lardaki bugları düzeltin
- **Performance:** Optimizasyon yapın

### 2. Dokümantasyon

- **Guides:** Yeni kılavuzlar yazın
- **Examples:** Örnek projeler ekleyin
- **Translations:** Çeviriler yapın
- **Tutorials:** Eğitimler oluşturun

### 3. Testing

- **Unit Tests:** Test coverage artırın
- **Integration Tests:** Entegrasyon testleri
- **E2E Tests:** End-to-end testler

### 4. Community

- **Issue Triage:** Issue'ları kategorize edin
- **Support:** Sorulara cevap verin
- **Reviews:** PR'ları review edin

---

## Proje Yapısı

```
stackvo/
├── cli/                    # CLI komutları
│   ├── stackvo.sh       # Ana CLI
│   ├── commands/          # Alt komutlar
│   └── lib/               # Kütüphaneler
│       └── generators/    # Generator modülleri
├── core/                  # Core dosyalar
│   ├── compose/           # Docker Compose templates
│   ├── traefik/           # Traefik konfigürasyonu
│   └── templates/         # Servis templates
├── projects/              # Kullanıcı projeleri
├── .ui/                   # Web UI
│   ├── index.html         # Ana sayfa
│   └── api/               # API endpoints
├── docs/                  # Dokümantasyon
└── scripts/               # Utility scripts
```

---

## Testing

### Local Testing

```bash
# Generator test
./cli/stackvo.sh generate

# Servisleri başlat
./cli/stackvo.sh up

# Logları kontrol et
./cli/stackvo.sh logs

# Temizle
./cli/stackvo.sh down
```

### CI/CD

GitHub Actions otomatik çalışır:
- Syntax kontrolü
- Docker build
- Integration tests

---

## Code Style

### Bash

```bash
# ✅ Doğru
function my_function() {
    local var="value"
    echo "$var"
}

# ❌ Yanlış
function myFunction {
    var=value
    echo $var
}
```

### Python

```python
# ✅ Doğru
def my_function(param: str) -> str:
    """Docstring"""
    return param.upper()

# ❌ Yanlış
def myFunction(param):
    return param.upper()
```

---

## Bug Reports

Issue açarken:

**Template:**
```markdown
## Bug Açıklaması
[Açıklama]

## Adımlar
1. [Adım 1]
2. [Adım 2]

## Beklenen Davranış
[Beklenen]

## Gerçek Davranış
[Gerçek]

## Environment
- OS: Ubuntu 22.04
- Docker: 24.0.7
- Stackvo: 1.0.0

## Loglar
```
[Loglar]
```
```

---

## Feature Requests

Yeni özellik önerirken:

**Template:**
```markdown
## Özellik Açıklaması
[Açıklama]

## Motivasyon
[Neden gerekli?]

## Önerilen Çözüm
[Nasıl implement edilmeli?]

## Alternatifler
[Başka çözümler?]
```

---

## Recognition

Contributors:
- README.md'de listelenir
- GitHub contributors sayfasında görünür
- Release notes'ta mention edilir

---

## İletişim

Sorularınız için:
- **GitHub Discussions:** [Tartışmalara katıl](https://github.com/stackvo/stackvo/discussions)
- **Issues:** [Soru sor](https://github.com/stackvo/stackvo/issues/new)

---

## License

Katkılarınız [MIT License](https://github.com/stackvo/stackvo/blob/main/LICENSE) altında yayınlanır.
