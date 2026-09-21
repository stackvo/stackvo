# Makineden ne çıkar

Telemetri yok, raporlama yok. Bu sayfa ağ çağrılarının tam listesidir, iki
bölümde: uygulamanın kendiliğinden yaptığı iki çağrı ve siz bir şeye bastığınız
için olanlar. Tam metin
[PRIVACY.md](https://github.com/stackvo/stackvo/blob/main/PRIVACY.md).

## Uygulamanın kendiliğinden

| Host | Ne zaman | Ne gönderilir |
| --- | --- | --- |
| `github.com` | Ayarlar ekranı açıldığında ve **Güncellemeleri denetle**'ye basınca. Açılışta değil. | Statik bir `latest.json` için tek GET. Kimlik yok, sürüm parametresi yok. |
| `127.0.0.1` | Mail ekranı açıkken | Yalnızca loopback; hiçbir ağ arayüzüne ulaşmaz. |

Listenin tamamı bu.

## Siz istediğiniz için

| Ne yaparsınız | Nereye gider |
| --- | --- |
| Git adresinden proje oluşturmak | Yazdığınız uzak depoya, kendi `git` kimlik bilgilerinizle. |
| Yığını başlatmak | İmajların adını verdiği kayıt defterine — varsayılan Docker Hub. Bunu Docker daemon'unuz yapar. |
| Paylaşım tüneli açmak | Seçtiğiniz sağlayıcıya, yalnızca ona. |
| Servis kataloğunu çekmek | Seçtiğiniz adrese. Düğmeye basana kadar hiçbir şey çekilmez. |
| Servis paketi kurmak | Aynı adrese. İmajın kendisi sonra Docker tarafından çekilir. |
| Yardım paneli açmak | `raw.githubusercontent.com`, o kartın belgesi için. **İstek açtığınız konunun adını** ve dilinizi taşır, başka hiçbir şey. İlk çekişten sonra önbellekte. |
| Örnekleyici profilleyiciyi kurmak | Konteynerin paket aynası ve `github.com`, geçici bir konteynerde. Hiçbir şey yüklenmez. |
| Profil kaydetmek | Kendi projeniz, kendi makinenizde. Host asla yazdığınızdan alınmaz. |
| Host aracı kurmak | `github.com`, `mkcert`'in tek sürüm dosyası için; uygulamaya gömülü özetle doğrulanır. |
| Bağımlılıkları açıklar için denetlemek | `api.osv.dev` — **o projenin bağımlılıklarının adları ve sürümleri**. Gerçek bir ifşa; kendi düğmesinin arkasında, üstünde bunu söyleyen cümleyle. |
| Tanılama paketi göndermek | Nereye gönderirseniz. Parolalar ve token'lar günlük yazılırken maskelenir; arşiv düz metindir, göndermeden önce bakın. |

## Yazdığı ama asla göndermediği

`preferences.json`, `stats-history.json`, uygulama günlüğü, çökme raporları,
`audit.jsonl` — hepsi uygulamanın kendi dizininde, [Çalışma alanı ve
dosyalar](workspace.md) sayfasında listeli. Yeniden oynatma için yakalanan
oturumlar, yakalama penceresi açıkken projenin istek çerezlerini ve form
gövdelerini `generated/debug/<proje>/` altına yazar; makineden asla çıkmaz.

## Bu site

Bu sayfalar GitHub Pages'in sunduğu statik dosyalardır. Çerez koymaz,
analitik yüklemez, üçüncü taraftan yazı tipi çekmez; arama tarayıcınızda
çalışır. Bir sayfanın kendi başına yaptığı tek istek, ana sayfadan son sürümü
adlandırmak için GitHub API'sinedir. Sunucu olarak GitHub'ın ne kaydettiği
GitHub'ın söyleyeceği şeydir: [GitHub gizlilik bildirimi](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement).
