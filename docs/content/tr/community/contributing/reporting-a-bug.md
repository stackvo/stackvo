# Hata bildirme

Bir şey olması gerekenden farklı davranıyor. Yeniden üretilebilen bir
bildirim genellikle bir sonraki sürümde düzelir; üretilemeyen ise genellikle
bir soruyla kapanır. Bu sayfa ilk türü yazmak üzerine.

!!! danger "Güvenlik sorunları hata değildir"

    Çalışma alanından kaçmanın, Docker'a sizin vermediğiniz bir girdiyle
    ulaşmanın ya da bir sırrı süreçten dışarı çıkarmanın bir yolu
    [özel olarak](reporting-a-vulnerability.md) bildirilir, asla issue
    olarak değil.

## Bildirmeden önce

### Güncelleyin

Yalnızca son sürüm desteklenir ve geriye dönük düzeltme dalı yoktur.
**Ayarlar → Güncellemeler** hangi sürümde olduğunuzu gösterir ve yenisini
kurar. Hata sonrasında yoksa zaten düzeltilmişti.

### Doktor'a sorun

**Ayarlar → Doktor** makineyi kontrol eder: Docker, soket, portlar, hosts
dosyası, sertifika, disk. İlk çalıştırma sorunlarının çoğu orada, yanında
onarımıyla adlandırılır. Doktor sizinkini adlandırıyorsa bu bir makine
sorunudur ve [Sorun giderme](../../getting-started/troubleshooting.md) bir
issue'dan hızlıdır.

### Arayın

Kapalı olanlar dahil
[issue'lara](https://github.com/stackvo/stackvo/issues?q=is%3Aissue) ve
[tartışmalara](https://github.com/stackvo/stackvo/discussions) bakın.
Bulursanız yenisini açmak yerine sizin durumunuzda yeni olanı oraya ekleyin.

## Bildirimi yazın

Bir [hata bildirimi](https://github.com/stackvo/stackvo/issues/new/choose)
açın. Şablon beş şey sorar ve her birinin bir nedeni var.

### Ne oldu

Ne yaptınız, ne bekliyordunuz, bunun yerine ne aldınız; bu sırayla. Bildirim
başına bir hata: bir issue'daki iki hata yarımşar cevap alır.

İyi bir başlık, birinin arayabileceği bir cümledir:

| | |
| --- | --- |
| :material-check: | *Proje yeniden adlandırıldıktan sonra adlandırılmış snapshot geri yükleme `no such volume` ile başarısız oluyor* |
| :material-close: | *Snapshot’lar bozuk* |

### Sürüm ve platform

**Ayarlar → Güncellemeler** sürümü gösterir. İşletim sistemini ve Docker
çalışma zamanını (Docker Desktop, Colima, OrbStack, Podman, düz motor)
sürümüyle birlikte yazın. Burada ters giden şeylerin çoğu koddan çok makineyle
ilgilidir ve ikisini ayırmanın en hızlı yolu budur.

### Log

**Ayarlar → Uygulama günlüğü → Tanılama paketi kaydet**; logu, ön kontrolleri,
Doktor raporunu ve varsa çökme raporlarını tek arşive yazar. Parolalar ve
token'lar log yazılırken maskelenir ve arşivin içi düz metindir; eklemeden
önce okuyun.

Bir satırını yapıştırmak yerine paketi ekleyin. Hatadan önceki satır çoğu
zaman onu açıklayan satırdır.

### Yeniden üretme adımları

Numaralı ve okurun ulaşabileceği bir durumdan başlayan. "Laravel şablonundan
bir proje oluştur, Redis'i aç, yeniden başlat" bir yeniden üretmedir; "bazen
oluyor" bir ipucu. Belirli bir manifest gerekiyorsa `stackvo.json` dosyasını
yapıştırın.

## Sonra ne olur

Geliştirici her bildirimi okur; çoğu cevap alır ve "çoğu" dürüst kelimedir.
Yeniden üretilemeyen bir bildirim bir soru alır ve cevap gelmezse kapanır.
Beklemek yerine düzeltmeyi tercih ederseniz
[Pull request açma](making-a-pull-request.md) daha kısa yoldur.
