---
title: Kurulum
description: Stackvo kurulum kılavuzu - Tüm platformlar için adım adım kurulum
---

# Kurulum

Stackvo'u bilgisayarınıza kurmak oldukça kolaydır ve tüm major işletim sistemlerinde desteklenmektedir. Bu bölüm, Linux, macOS ve Windows platformlarında Docker kurulumundan Stackvo yapılandırmasına kadar tüm adımları detaylı olarak açıklamaktadır. Her işletim sistemi için özel olarak hazırlanmış kılavuzlar, sistem gereksinimlerinden kurulum doğrulamasına kadar her şeyi kapsamaktadır.

---

## İşletim Sistemi Seçimi

Stackvo tüm major işletim sistemlerinde çalışır. İşletim sisteminizi seçin:

<div class="grid cards" markdown>

-   :fontawesome-brands-linux:{ .lg .middle } __Linux__

    ---

    Tüm popüler Linux dağıtımları için geçerli kurulum adımları

    [:octicons-arrow-right-24: Linux Kurulumu](linux.md)

-   :fontawesome-brands-apple:{ .lg .middle } __macOS__

    ---

    Intel ve Apple Silicon (M serisi) işlemciler için uyumlu

    [:octicons-arrow-right-24: macOS Kurulumu](macos.md)

-   :fontawesome-brands-windows:{ .lg .middle } __Windows__

    ---

    WSL2 (Windows Subsystem for Linux) üzerinde çalışır

    [:octicons-arrow-right-24: Windows Kurulumu](windows.md)

</div>

---

!!! tip "Sistem Gereksinimlerini Kontrol Ettiniz mi?"
    Kuruluma başlamadan önce [Sistem Gereksinimleri](../started/requirements.md) sayfasını kontrol edin.

---

## Hızlı Kurulum Yolu

Sisteminizde Docker zaten kuruluysa:

```bash
# 1. Repository'yi klonlayın
git clone https://github.com/stackvo/stackvo.git
cd stackvo

# 2. Konfigürasyon
cp .env.example .env

# 3. Kurulum scriptini çalıştırın
./cli/stackvo.sh install

# 4. Başlatın
./cli/stackvo.sh generate
./cli/stackvo.sh up
```

!!! success "Kurulum Tamamlandı!"
Web UI: [https://stackvo.loc](https://stackvo.loc)

---

## Kurulum Sonrası Ayarlar

### Hosts Dosyası Düzenleme

=== "Linux/macOS"

    ```bash
    sudo nano /etc/hosts
    ```

    Ekleyin:
    ```
    127.0.0.1  stackvo.loc
    127.0.0.1  traefik.stackvo.loc
    ```

=== "Windows"

    Yönetici olarak:
    ```
    notepad C:\Windows\System32\drivers\etc\hosts
    ```

    Ekleyin:
    ```
    127.0.0.1  stackvo.loc
    127.0.0.1  traefik.stackvo.loc
    ```


---

## Kurulum Doğrulama

Kurulumun başarılı olduğunu doğrulayın:

### Servis Kontrolü

```bash
# Tüm servislerin durumu
./cli/stackvo.sh ps

# Logları kontrol et
./cli/stackvo.sh logs
```

### Web UI Kontrolü

Tarayıcınızda açın:

- **Stackvo Dashboard:** https://stackvo.loc/
- **Traefik Dashboard:** http://traefik.stackvo.loc

!!! success "Kurulum Tamamlandı!"
    Artık [Hızlı Başlangıç](../started/quick-start.md) sayfasına geçerek ilk projenizi oluşturabilirsiniz.
