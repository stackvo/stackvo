# Gereksinimler

StackVo makinenizde çalışır ve her projeyi bir Docker konteynerine koyar.
Bu yüzden üç şeye ihtiyacı var; üçüncüsü küçük.

| Gereksinim | Ayrıntı |
| --- | --- |
| **Docker** | macOS ve Windows'ta Docker Desktop, Linux'ta Docker Engine. Podman, Colima ve OrbStack da tanınır. |
| **İşletim sistemi** | macOS 10.15 ve üzeri, Windows 10 ve üzeri, x86_64 ya da aarch64 Linux. |
| **Disk** | Uygulama için yaklaşık 27 MB. Docker imajları daha fazla yer tutar; zamanla birkaç gigabayt sayın. |

## Neye ihtiyacınız yok

- **Makinenizde PHP, Node, Python, Go, Ruby ya da Rust gerekmez.** Her
  çalışma zamanı projenin konteynerinde yaşar. Hiç PHP kurulu olmayan bir
  makine PHP projesini sorunsuz çalıştırır.
- **Paket yöneticisi gerekmez.** Homebrew, apt ya da Chocolatey ile hiçbir
  şey kurulmaz.
- **Kalıcı yönetici hakkı gerekmez.** Tek yetkili işlem, `shop.loc` çözülsün
  diye hosts dosyasına bir satır yazmaktır. Uygulama önce tam değişikliği
  gösterir ve bir kez sorar.

## Platforma göre Docker

=== "macOS"

    [Docker Desktop](https://www.docker.com/products/docker-desktop/) ya da
    StackVo'nun tanıdığı alternatiflerden birini kurun: OrbStack, Colima. Bir
    kez başlatın; StackVo soketi kendisi bulur.

=== "Windows"

    [Docker Desktop](https://www.docker.com/products/docker-desktop/) kurup
    başlatın. StackVo Docker'ın adlandırılmış borusu üzerinden bağlanır.

    !!! note "Windows en az test edilmiş platform"
        Mantık her platformda test edilir ve Windows CI matrisindedir; ama
        UAC üzerinden hosts dosyası yazımı, gerçek bir Docker Desktop'a karşı
        boru ve tarayıcıda alan adı çözümü gerçek makinede henüz doğrulanmadı.
        Ters giden bir şey varsa lütfen
        [sorun açın](https://github.com/stackvo/stackvo/issues/new/choose).

=== "Linux"

    Dağıtımınızdan [Docker Engine](https://docs.docker.com/engine/install/)
    ya da Podman kurun. Podman'da rootless soket öncelikli aranır.

!!! tip "Docker çalışmıyorsa"
    Uygulama yine açılır. Panelde söyler ve Docker'ı sizin için başlatmayı
    teklif eder. Motor ayağa kalkana kadar başka bir şey çalışmaz, ama boş bir
    pencereye bakmazsınız.

## Portlar

Projeler Traefik üzerinden sunulur, bu yüzden 80 ve 443 portları boş olmalı.
Bu portlardan birini başka bir şeyin tutması ilk çalıştırmanın en yaygın
sorunudur; **Ayarlar → Doktor** portu ve onu tutan programı adıyla
söyler.

Sonraki adım: [Kurulum](installation.md).
