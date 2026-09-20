# Sorun giderme

İlk çalıştırmada altı şey ters gider. Her birinin bir düğmesi var.

<figure markdown>
![Ayarlar sayfasında Doktor](../screenshots/web/settings-doctor.webp){ loading=lazy }
<figcaption>Doktor: her bulgu, yanında onarımıyla.</figcaption>
</figure>

## Docker çalışmıyor

Panel söyler ve başlatmayı teklif eder. Docker başladığı hâlde uygulama hâlâ
görmüyorsa **Ayarlar → Doktor → Docker motoru** hangi soket ve bağlamın
kullanıldığını gösterir; birden fazla Docker kuruluysa ilk bakılacak yer
burasıdır.

## Alan adı açılmıyor

Proje satırı adresin çözülmediğini söyler ve **Hosts girdisi ekle** sunar.
Kabul edin; önce fark gösterilir, parolanızı bir kez girersiniz. Tarayıcı
hâlâ ulaşamıyorsa 80 ya da 443 portunu başka bir şeyin tutmadığını kontrol
edin — **Doktor** programı adıyla söyler.

## Tarayıcı sertifika uyarısı veriyor

**Ayarlar → HTTPS sertifikası** üç şey gösterir:

| Kart ne diyor | Ne yapın |
| --- | --- |
| **Yeniden üretilmeli** ya da **Kapsanmıyor** altında bir alan adı | **Sertifikayı yeniden üret**'e basın. |
| **CA güvenilmiyor** | **CA'ya güven (terminalde)**'ye basın, sonra tarayıcıyı kapatıp yeniden açın. |
| Firefox dışında her yerde güvenilir | Firefox'un kendi deposu var; `nss` kurup güven adımını yeniden çalıştırın. |

## Node uygulaması 502 veriyor

Uygulamanız `127.0.0.1` değil `0.0.0.0` adresine bağlanmalı. Traefik yalnızca
localhost'u dinleyen bir sunucuya ulaşamaz.

## İlk kurulum yavaş

Çalışma zamanı imajı ve şablonda yükleyicinin imajı bir kez indirilir. Kurulum
Docker'ın çıktısını akıtır, ilerlediğini görürsünüz. Aynı türden ikinci proje
hızlıdır.

## Bende çalışıyor, onda çalışmıyor

**Ayarlar → Uygulama günlüğü → Tanılama paketi kaydet** günlüğü, kontrolleri,
Doktor raporunu ve ortamı tek arşive yazar. Parolalar ve token'lar günlük
yazılırken maskelenir. Gönderin, ya da onunkini **Başka bir makineyle
karşılaştır** ile açın — dönen şey yalnızca iki makinenin uyuşmadığı
noktalardır: Docker sürümü, bir tarafta açık olan servis, PHP sürümü.

## Hâlâ takıldıysanız

**Ayarlar → Doktor** neyin bozuk olduğunu ve nasıl düzeleceğini
satır satır söyler; çoğu bulgunun yanında bir düğme vardır. Sorununuzu
adlandırmıyorsa
[Tartışmalarda sorun](https://github.com/stackvo/stackvo/discussions) ya da
[sorun açın](https://github.com/stackvo/stackvo/issues/new/choose) — ve paketi
ekleyin. Sürüm numarası tek başına nadiren yeter.
