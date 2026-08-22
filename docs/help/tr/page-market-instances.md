# Servis örnekleri

Bu çalışma alanının çalıştırdığı sürümler. Her birinin kendi verisi ve kendi portu vardır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Durdur / Başlat | Örneğin konteynerini indirir ya da ayağa kaldırır. |
| Yeniden başlat | Aynı konteyneri durdurup başlatır. |
| Tarayıcıda aç | Yönetim arayüzü olan servislerde adresi açar. |
| Birincil yap | Bu servis türü için varsayılan örneği belirler. Projeler adı verilmemişse birincili kullanır. |
| Ayarlar | Örneğin portunu, kimlik bilgilerini ve diğer değerlerini açar. |
| Detay | Örneğin ayrıntılarını gösterir. |
| Kaldır | Örneği siler. |

## Yeni örnek oluştururken

Kart, paketin kendi varsayılanlarını gösterir. Şimdi değiştirmeye değen kısım kimlik bilgileridir: bir imaj root parolasını yalnızca boş bir veri dizinini ilk kurarken okur, yani ayarlanabileceği tek an budur.

Boş port bulunamazsa kart bunu söyler ve portu kendiniz seçersiniz.

## Bilinmesi gerekenler

- Bir servisin iki sürümü yan yana çalışabilir. Her biri kendi verisini tutar.
- Paketi kaldırılmış bir örnek "Paket yok" olarak görünür. Çalışmaya devam eder ama yeniden kurulamaz.
- Örneği kaldırmak verisini de kaldırır. Saklamak istediğiniz bir veri varsa önce yedek alın.
