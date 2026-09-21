# Çalışma alanı ve dosyalar

StackVo'nun yazdığı her şey bilinen bir yerdedir. Bu sayfa o listedir.

## Çalışma alanı

Tek klasör, varsayılan `~/.stackvo`. **Durum klasörün kendisidir**: veritabanı
yok.

```text
~/.stackvo/
├── .env                    yığının ayarları — yalnızca bir ayar değişince yazılır
├── generated/              compose dosyaları, Dockerfile'lar, yönlendirme — üretilir, düzenlenmez
├── certs/                  sertifika ve otoritesi
├── commands.json           makine geneli komutlar, manifest'tekiyle aynı biçim
└── projects/
    └── shop/               proje başına bir klasör
        ├── stackvo.json    manifest
        ├── .stackvo/       site.json ve uygulamanın istek üzerine yazdıkları
        └── Dockerfile      üretilir
```

`generated/` her an silinebilir, gerektiğinde yeniden üretilir. Hiç `.env`
olmayan bir çalışma alanı yapılandırılmamış değil, normal durumdur:
varsayılanlar uygulamanın içindedir, dosya bir şeyi değiştirince ortaya çıkar.

## Gerisi nerede

| | macOS | Windows | Linux |
| --- | --- | --- | --- |
| Tercihler, yardım önbelleği, CLI bağlantıları | `~/Library/Application Support/StackVo/` | `%APPDATA%\StackVo\` | `~/.config/stackvo/` |
| Uygulama günlüğü, günlük döner | `~/Library/Logs/StackVo/` | `%LOCALAPPDATA%\StackVo\logs\` | `~/.local/state/stackvo/logs/` |
| Yığın durumu | `~/.stackvo/` | `~/.stackvo/` | `~/.stackvo/` |

`~/.stackvo` sizindir, taşıyabilir ve silebilirsiniz; uygulamanın açılmak için
ihtiyaç duyduğu hiçbir şey orada tutulmaz. CLI bağlantıları bilerek tercihlerin
yanında: yığını sıfırlayınca kaybolan bir PATH girdisi hiçbir yeri göstermez.

## Uygulamanın kendiliğinden yazdıkları

| Dosya | Ne için |
| --- | --- |
| `preferences.json` | Uygulama ayarları. |
| `stats-history.json` | Konteyner başına CPU ve bellek okumaları; yeniden başlatınca grafik boş kalmasın diye. |
| `audit.jsonl` | Yetkili ya da geri alınamaz her iş için bir satır: hosts yazımı, sertifika güveni, proje silme, `.env` anahtarı değişimi, veritabanı geri yükleme. **Ayarlar → Denetim kaydı** okur. |
| `crash-<zaman>-<pid>.txt` | Çökme raporu, bir kez gösterilir. |
| `help-cache/` | Depodan çekilen yardım belgeleri; **?** panelleri çevrimdışı çalışsın diye. |

## Projenize yazdıkları, yalnızca siz isteyince

| Dosya | Düğme |
| --- | --- |
| `.stackvo/site.json` | Proje ayarları |
| `.stackvo/context.json` | Konteynerde çalışan bir asistanın bilmesi gerekenler |
| `.devcontainer/` | Devcontainer → Dosyaları yaz |
| `stackvo.preset.json` | **Ayarlar → Çalışma alanı → Bu yığını dışa aktar** |
| `CLAUDE.md`, `AGENTS.md`, `.cursor/rules/…` | **Ayarlar → Yapay zekâ asistanları** — yalnızca StackVo işaretleri arasındaki blok |

## Ortam değişkenleri

| Değişken | Ne yapar |
| --- | --- |
| `STACKVO_ROOT` | Çalışma alanını taşır. |
| `STACKVO_LOG` | Log seviyesi, örn. `stackvo_desktop=debug`. |
| `STACKVO_POLICY_FILE` | Başka bir politika dosyasını gösterir; root olmadan denemek için. |
| `DOCKER_HOST` | Her zamanki gibi; gerektiğinde şema ayıklanır. |

## Yönetilen makineler

Bir yönetici ayarları belirleyip kilitleyen bir politika dosyası dağıtabilir:

| | Yol |
| --- | --- |
| macOS | `/Library/Managed Preferences/com.stackvo.desktop.json` |
| Windows | `%ProgramData%\StackVo\policy.json` |
| Linux | `/etc/stackvo/policy.json` |

```json
{
  "schemaVersion": 1,
  "settings": { "DEFAULT_TLD_SUFFIX": "corp.test", "SERVER_TYPE": "nginx" },
  "locked": ["DEFAULT_TLD_SUFFIX"],
  "registryPrefix": "registry.corp.example/proxy"
}
```

`settings` hem gömülü varsayılanı hem çalışma alanının `.env`'sini ezer;
`locked` o anahtarlara Ayarlar'dan yazmayı reddeder; `registryPrefix` üretilen
her imaj adının önüne konur. Bu bir güvenlik sınırı değildir ve uygulama bunu
söyler: iş birliği yapan bir uygulamaya kuruluşun niyetini bildirir. **Ayarlar →
Uyumluluk** bunun bu makinede gerçekten tutup tutmadığını ölçer.

## Parolaları `.env` dışına

**Ayarlar → Kimlik bilgileri nerede tutuluyor** bir parolayı ya da token'ı
makinenin anahtar deposuna taşır ve geride bir referans bırakır:

```sh
SERVICE_MYSQL_ROOT_PASSWORD=keychain:SERVICE_MYSQL_ROOT_PASSWORD@a1b2c3d4
```

Bu, değeri yedeklenen ve destek konularına yapıştırılan dosyadan çıkarır.
Diskten çıkarmaz: gerçek değer yine üretilen compose dosyasına yazılır, çünkü
Compose onu oradan okur.
