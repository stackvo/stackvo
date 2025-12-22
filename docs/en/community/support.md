# Support

Stackvo için destek alma yolları. Bu sayfa, GitHub Discussions, GitHub Issues ve dokümantasyon gibi destek kanallarını, iyi soru sorma rehberini, bug report ve feature request şablonlarını, yanıt sürelerini ve iletişim bilgilerini detaylı olarak açıklamaktadır. Community bazlı destek sisteminin nasıl çalıştığını ve en hızlı şekilde yardım almanın yollarını içerir.

## 🆘 Destek Kanalları

### 1. GitHub Discussions (Önerilen)

**En iyi seçenek:** Soru sormak, fikir paylaşmak, tartışmak için.

[💬 Discussions'a git →](https://github.com/stackvo/stackvo/discussions)

**Kategoriler:**
- 💡 **Ideas** - Özellik önerileri
- 🙏 **Q&A** - Sorular ve cevaplar
- 📣 **Announcements** - Duyurular
- 💬 **General** - Genel tartışmalar

### 2. GitHub Issues

**Bug reports ve feature requests için.**

[🐛 Issue aç →](https://github.com/stackvo/stackvo/issues/new)

**Ne zaman kullanılır:**
- Bug bulduğunuzda
- Yeni özellik önerdiğinizde
- Dokümantasyon hatası gördüğünüzde

### 3. Documentation

**Önce dokümantasyona bakın:**

- [Getting Started](../started/index.md)
- [Installation](../installation/index.md)
- [Configuration](../configuration/index.md)
- [Guides](../guides/index.md)
- [FAQ](faq.md)
- [Troubleshooting](troubleshooting.md)

---

## 📝 Soru Sorma Rehberi

### İyi Soru Nasıl Sorulur?

#### ✅ İyi Örnek

```markdown
## Sorun: MySQL container başlamıyor

**Environment:**
- OS: Ubuntu 22.04
- Docker: 24.0.7
- Stackvo: 1.0.0

**Adımlar:**
1. ./cli/stackvo.sh generate
2. ./cli/stackvo.sh up

**Hata:**
```
Error: MySQL container exited with code 1
```

**Loglar:**
```
docker logs stackvo-mysql
[ERROR] InnoDB: Cannot allocate memory
```

**Denediklerim:**
- Docker restart
- ./cli/stackvo.sh down && ./cli/stackvo.sh up
```

#### ❌ Kötü Örnek

```
MySQL çalışmıyor yardım edin
```

### Soru Şablonu

```markdown
## Sorun Başlığı

**Environment:**
- OS: [Ubuntu/macOS/Windows]
- Docker: [version]
- Stackvo: [version]

**Sorun Açıklaması:**
[Detaylı açıklama]

**Adımlar:**
1. [Adım 1]
2. [Adım 2]

**Beklenen Davranış:**
[Ne olmasını bekliyordunuz?]

**Gerçek Davranış:**
[Ne oldu?]

**Hata Mesajı:**
```
[Hata mesajı]
```

**Loglar:**
```
[İlgili loglar]
```

**Denediklerim:**
- [Deneme 1]
- [Deneme 2]
```

---

## 🐛 Bug Report Rehberi

### Bug Nasıl Raporlanır?

1. **Önce arayın:** Aynı bug daha önce raporlanmış mı?
2. **Reproduce edin:** Bug'ı tekrar oluşturabilir misiniz?
3. **Minimal örnek:** En basit haliyle gösterin
4. **Environment:** Sistem bilgilerini ekleyin
5. **Loglar:** İlgili logları paylaşın

### Bug Report Şablonu

```markdown
## Bug Açıklaması

[Kısa ve net açıklama]

## Reproduce Adımları

1. [Adım 1]
2. [Adım 2]
3. [Adım 3]

## Beklenen Davranış

[Ne olmalıydı?]

## Gerçek Davranış

[Ne oldu?]

## Screenshots

[Varsa ekran görüntüleri]

## Environment

- **OS:** Ubuntu 22.04
- **Docker:** 24.0.7
- **Docker Compose:** 2.23.0
- **Stackvo:** 1.0.0
- **Browser:** Chrome 120 (Web UI için)

## Loglar

```bash
# stackvo doctor
[Çıktı]

# Container logs
docker logs stackvo-mysql
[Loglar]

# Generator log
cat core/generator.log
[Loglar]
```

## Ek Bilgiler

[Diğer ilgili bilgiler]
```

---

## 💡 Feature Request Rehberi

### Özellik Nasıl Önerilir?

1. **Arayın:** Benzer öneri var mı?
2. **Use case:** Neden gerekli?
3. **Çözüm:** Nasıl implement edilmeli?
4. **Alternatifler:** Başka çözümler?

### Feature Request Şablonu

```markdown
## Özellik Açıklaması

[Özelliği kısaca açıklayın]

## Motivasyon

[Neden bu özellik gerekli?]

## Use Case

[Hangi senaryolarda kullanılacak?]

**Örnek:**
```
[Kod örneği]
```

## Önerilen Çözüm

[Nasıl implement edilmeli?]

## Alternatifler

[Başka çözüm yolları?]

## Ek Bilgiler

[Diğer ilgili bilgiler]
```

---

## 🤝 Katkıda Bulunma

Stackvo'a katkıda bulunmak ister misiniz?

[Contributing Guide →](contributing.md)

**Katkı Alanları:**
- 💻 Kod
- 📝 Dokümantasyon
- 🧪 Testing
- 🌍 Çeviri
- 🎨 Design
- 📢 Community

---

## 📊 Destek İstatistikleri

<div class="grid cards" markdown>

-   **🐛 Open Issues**
    
    GitHub Issues
    
    [Issues →](https://github.com/stackvo/stackvo/issues)

-   **💬 Discussions**
    
    Aktif tartışmalar
    
    [Discussions →](https://github.com/stackvo/stackvo/discussions)

-   **👥 Contributors**
    
    Topluluk desteği
    
    [Contributors →](index.md#contributors)

-   **📖 Documentation**
    
    Kapsamlı rehberler
    
    [Docs →](../index.md)

</div>

---

## ⏱️ Yanıt Süreleri

**GitHub Issues:**
- İlk yanıt: 24-48 saat
- Çözüm: Karmaşıklığa bağlı

**GitHub Discussions:**
- Community desteği: Değişken
- Maintainer desteği: 1-3 gün

**Not:** Stackvo açık kaynak bir projedir. Yanıt süreleri garanti değildir.

---

## 🌟 Premium Support

Şu anda premium support sunulmamaktadır. Tüm destek community bazlıdır.

---

## 📞 İletişim

### GitHub

- **Repository:** [stackvo/stackvo](https://github.com/stackvo/stackvo)
- **Issues:** [Bug reports](https://github.com/stackvo/stackvo/issues)
- **Discussions:** [Q&A](https://github.com/stackvo/stackvo/discussions)
- **Pull Requests:** [Contributions](https://github.com/stackvo/stackvo/pulls)

### Email

- **General:** stackvo@example.com
- **Security:** security@stackvo.example.com

### Social Media

- **Twitter:** [@stackvo](https://twitter.com/stackvo)
- **LinkedIn:** [Stackvo](https://linkedin.com/company/stackvo)

---

## 🔒 Security Issues

Güvenlik açığı bulduysanız:

1. **Public issue açmayın**
2. **Email gönderin:** security@stackvo.example.com
3. **Detay verin:** Açık, impact, reproduce
4. **Bekleyin:** 48 saat içinde yanıt

---

## 📚 Ek Kaynaklar

- [Documentation](../index.md)
- [FAQ](faq.md)
- [Troubleshooting](troubleshooting.md)
- [Contributing](contributing.md)
- [Changelog](../changelog.md)

---
