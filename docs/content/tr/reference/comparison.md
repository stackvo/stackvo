# Benzer araçlarla karşılaştırma

Aşağıdaki tablo, aynı kategorideki araçların **yaklaşım farklarını** özetler;
"daha iyi/kötü" değil, "hangisi neyi seçmiş" tablosudur.

| | **StackVo** | Herd | ServBay | Laragon | DDEV | Laradock | Devilbox | FlyEnv |
|---|---|---|---|---|---|---|---|---|
| Yaklaşım | Docker + masaüstü | Yerel ikili | Yerel ikili | Yerel ikili | Docker + CLI | Docker + compose | Docker + compose | Yerel ikili |
| Arayüz | Masaüstü + CLI + TUI + MCP | Masaüstü | Masaüstü | Masaüstü | CLI | Yok | Web intranet | Masaüstü |
| Platform | mac · Win · Linux | mac · Win | mac · Win | Win | mac · Win · Linux | hepsi | hepsi | mac · Win · Linux |
| Proje izolasyonu | Konteyner | Site | Site | Site | Konteyner | Paylaşılan yığın | Paylaşılan yığın | Site |
| Otomatik HTTPS | Var (mkcert) | Var | Var | Var | Var | elle | Var | Var |
| Dal başına ortam + **ayrı DB** | Var | Yok | Yok | Yok | kısmî | Yok | Yok | Yok |
| İstek düzeyinde "neden yavaş" | Var (profil+sorgu+dump) | kısmî (Pro) | Yok | Yok | Yok | Yok | Yok | Yok |
| İsteği yeniden gönderme | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| Uygulama içi mail kutusu | Var | Var (Pro) | Var (Pro) | Var | web arayüzü | web arayüzü | web arayüzü | Var |
| Adlandırılmış DB anlık görüntüsü | Var (+ zamanlanmış) | Yok | zamanlanmış | zamanlanmış | Var | Yok | Yok | Yok |
| Monorepo tek proje | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| MCP / AI entegrasyonu | Var — 38 araç, yetki sınırlı | Yok | Var | Yok | Yok | Yok | Yok | Var |
| Üretim imajı kurma | Var | Yok | Yok | Yok | Yok | Var | Yok | Yok |
| Devcontainer dışa aktarma | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| İçe aktarma kaynağı | **7** | 1 | birkaç | Yok | birkaç | Yok | Yok | birkaç |
| Kaynak maliyetini ölçme | Var | Yok | Yok | Yok | Yok | Yok | Yok | Yok |
| Yönetici politikası (MDM) | Var | Yok | takım planı | Yok | Yok | Yok | Yok | Yok |
| Taşınabilir kurulum | Yok (mimari gereği) | Yok | Yok | Var | Yok | Yok | Yok | Var |
| Codespaces / Gitpod içinde | Yok | Yok | Yok | Yok | Var | Var | Var | Yok |
| Fiyat | Ücretsiz, MIT | Ücretsiz + Pro $99/yıl | Ücretsiz + Pro | Ücretsiz | Ücretsiz, Apache-2 | Ücretsiz, MIT | Ücretsiz, MIT | Ücretsiz + Pro $10 |

*Tablo Eylül 2026 itibarıyla, projelerin kendi belgelerine dayanır. Bir satır
hatalıysa lütfen [issue açın](https://github.com/stackvo/stackvo/issues/new/choose),
düzeltilir.*

## Dürüst olmak gerekirse: Docker'ın bedeli

| | StackVo | Host'a PHP kuran bir araç |
|---|---|---|
| İlk kurulum | uygulama (~27 MB) **+ Docker ve imajlar (GB)** | tek yükleyici, ~100 MB |
| Projenin ilk açılışı | imaj kurulumu — dakikalar | saniyeler |
| **PHP sürümü değiştirme** | manifest'i değiştir, imajı yeniden kur | anında |
| Boştaki bellek | Docker VM + Traefik + açık servisler | yalnızca dil çalışma zamanı |

Karşılığında aldığınız şey, hiçbirinin veremediği şeydir: her projenin ortamı
bir konteynerdir; makinenizde çalışan şey `brew` geçmişinizin değil, bir
Dockerfile'ın söylediği şeydir.

**Bilinçli iki sınır:**

- **Taşınabilir kurulum yok ve olamaz.** İmajlar ve volume'lar Docker'ın kendi
  deposunda yaşar; `STACKVO_ROOT` cevabın yarısıdır — çalışma alanınız sizinle
  gelir, motor gelmez.
- **Codespaces/Gitpod içinde çalışmaz.** Bu bir masaüstü uygulamasıdır. Bunun
  yerine **devcontainer dışa aktarır** — burada kurulan proje bir bulut
  ortamında açılabilir.
