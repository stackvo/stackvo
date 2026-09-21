# Kurulum

Platform başına bir yükleyici;
[GitHub'daki son sürümden](https://github.com/stackvo/stackvo/releases/latest)
indirilir. [Ana sayfa](../index.md) makinenize uygun dosyayı seçer.

| Platform | Dosya | Mimari |
| --- | --- | --- |
| macOS 10.15+ | `.dmg` | Apple Silicon, Intel |
| Windows 10+ | `-setup.exe` | x64, ARM64 |
| Linux | `.deb`, `.rpm`, `.AppImage` | x86_64, aarch64 |

<figure markdown>
![Ayarlar sayfasında Hakkında bölümü](../screenshots/web/settings-about.webp){ loading=lazy }
<figcaption>Ayarlar: kurulu sürüm ve güncelleme denetimi.</figcaption>
</figure>

## Kurun

=== "macOS"

    1. `.dmg` dosyasını açın ve **StackVo**'yu **Applications** klasörüne
       sürükleyin.
    2. Açın. İlk seferde macOS *"StackVo is damaged and can't be opened"*
       diyebilir. Bu mesaj dosyayla değil karantina özniteliğiyle ilgilidir —
       aşağıya bakın.

    Uygulamaya sağ tıklayıp **Aç**'ı, sonra yine **Aç**'ı seçin; ya da terminalden:

    ```sh
    xattr -dr com.apple.quarantine /Applications/StackVo.app
    ```

=== "Windows"

    1. `-setup.exe` dosyasını çalıştırın.
    2. SmartScreen *"Windows protected your PC"* der. **More info**, sonra
       **Run anyway** tıklayın.
    3. Yükleyiciyi bitirin ve StackVo'yu Başlat menüsünden açın.

=== "Linux"

    Dağıtımınızın kullandığı biçimi seçin:

    ```sh
    # Debian, Ubuntu
    sudo dpkg -i StackVo_<sürüm>_amd64.deb

    # Fedora, RHEL, openSUSE
    sudo rpm -i StackVo-<sürüm>-1.x86_64.rpm

    # Herhangi bir dağıtım
    chmod +x StackVo_<sürüm>_amd64.AppImage
    ./StackVo_<sürüm>_amd64.AppImage
    ```

    ARM makinede `amd64` / `x86_64` yerine `arm64` / `aarch64` kullanın.

## İşletim sistemi neden uyarıyor

Sürümler kod imzalı değil; bu bir eksiklik değil bilinçli bir karar. Uygulama
yalnızca GitHub Releases üzerinden dağıtılır; Apple Developer üyeliği ve
Authenticode sertifikası bir kimliğe bağlı, yinelenen maliyetlerdir.
Karşılığında:

- her sürüm dosyalarının yanında `SHA256SUMS.txt` yayımlar, indirdiğinizi
  doğrulayabilirsiniz;
- uygulamanın kendi güncelleyicisi bir şey kurmadan önce güncelleme
  bildiriminin **minisign** imzasını doğrular.

## Çalıştığını anlayın

StackVo'yu açın. Panelde bir **Docker motoru** kartı görünür: *çalışıyor* ve
bulduğu platform. Docker çalışmıyorsa kart söyler ve başlatmak için bir düğme
sunar. Kontrolün tamamı bu — yazacak bir şey yok.

İsteğe bağlı: `stackvo` komut satırı uygulamanın içinde gelir. **Ayarlar →
Araçlar** tek düğmeyle PATH'e ekler; sonra terminalde:

```sh
stackvo status
```

## Güncellemeler

**Ayarlar → Güncellemeler** yeni sürüm olup olmadığını denetler ve kurar.
Güncellemeler imzalıdır; doğrulamadan geçmeyen paket kurulmaz. Kurulum
uygulamayı kapatır; çalışan konteynerleriniz etkilenmez.

**Beta sürümleri de al** seçeneği ön sürümleri de denetime ekler. Beta kurulum
her kararlı sürümü almaya devam eder; kararlı kuruluma hiçbir zaman beta
sunulmaz. Anahtar bir sonraki açılışta geçerli olur.

## StackVo'yu kaldırmak

StackVo'nun yazdığı her şey bilinen yerlerdedir; kaldırmak kısa bir listedir:

| Ne | Nerede | Nasıl |
| --- | --- | --- |
| Uygulama | Applications, Program Files ya da paket | Silin, ya da `apt remove` / `rpm -e` |
| Çalışma alanı | Varsayılan `~/.stackvo` (**Ayarlar → Çalışma alanı** yolu gösterir) | Klasörü silin. `projects/` içindeki proje kodunuz da gider; isterseniz önce taşıyın. |
| Hosts satırları | Hosts dosyanız; **Ayarlar → Alan adı ve ağ** satırları listeler | Satırları kaldırın |
| Komut satırı | Eklediyseniz bir PATH satırı | `stackvo path-remove` |
| Sertifika otoritesi | Güvendiyseniz güven deponuz | `mkcert -uninstall` |
| Docker imajları ve volume'lar | Docker'ın kendi deposu | `docker system prune` hiçbir konteynerin kullanmadıklarını kaldırır |

Sonraki adım: [İlk çalıştırma](first-run.md).
