# Servisler ve veritabanları

MySQL, Redis, Elasticsearch ve diğerleri bir **katalogdan** gelir, **örnek**
olarak kurulur ve **proje başına** açılır. Üç iş, üç yer.

<figure markdown>
![Katalog](../screenshots/web/market.webp){ loading=lazy }
<figcaption>Katalog: her biri kendi sürümü ve portuyla örnek olarak kurulan servisler.</figcaption>
</figure>

## 1. Katalogdan kurun

**Katalog → Yayında olanlar** bir kaynağın yayımladıklarını sürümleriyle
listeler. **Kur** paketi diske koyar. Henüz bir şey çalışmaz.

StackVo içinde servis taşımaz; bir kaynak verilene kadar hiçbir şey
kullanılabilir değildir. Katalog imzalıdır — kayıt minisign ile, her paket
sağlama toplamıyla — ve imzasız bir kaynak öyle olduğu söylenerek gösterilir.

## 2. Örnek ekleyin

**Katalog → Servis örnekleri → Ekle** o sürümü bu çalışma alanında başlatır.
Her örneğin kendi verisi ve kendi portu vardır.

Root parolasını belirleme anı şimdidir: imaj onu yalnızca boş bir veri
dizinini ilk kez başlatırken okur. Sonrasında parola ne ise odur.

Bir servisin iki sürümü yan yana çalışır — **MySQL 8.0 ve 8.4** — ve her biri
kendi verisini tutar. **Birincil yap**, örnek adı vermeyen projeler için
varsayılanı belirler.

## 3. Proje için açın

**Proje → Servisler** projenin bildirdiğini ve bu makinenin çalıştırdığını
listeler:

| Durum | Anlamı |
| --- | --- |
| **Burada açık** | Çalışıyor, proje ulaşabilir. |
| **Burada açık değil** | Proje istiyor, bu makine çalıştırmıyor. **Etkinleştir** başlatır. |
| **Önerilen** | Projenin kendi `.env` dosyasından okunan bir tahmin — sözgelimi `DB_CONNECTION=pgsql`. Kendi başına asla yazılmaz; **stackvo.json'a yaz** onu commit'leyeceğiniz bir bildirime çevirir. |

Bağlantı dizesi aynı panelde. Parola tıklayınca görünür.

## Aynı anda iki sürüm

A projesi MySQL 8.0'da, B projesi 8.4'te, ikisi de çalışıyor: servisler tek
bir genel kopya değil örneklerdir. Her proje istediğine bağlanır.

## Kaldırmak

Bir örnekte **Kaldır** verisini de siler. Tutmak istiyorsanız önce
[snapshot](snapshots.md) alın.
