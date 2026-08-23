# Bu deponun kendisiyle getirdiği konteynerler

`stackvo.json` içinde `sidecars` altında bildirilen ek konteynerler. Projenin kendi compose bloğuna render edilirler ve projeyle birlikte kalkıp inerler.

## Sidecar bir servis değildir

**Servis** bir katalog kimliğidir — bu makinenin paketlerden karşıladığı bir ihtiyaç, isteyen her projenin paylaştığı. **Sidecar** ise deponun yanında getirdiği bir konteynerdir ve yalnız bu projeye aittir.

Fark şuradan önemli: aynı deponun farklı proje adlarıyla iki kopyası, iki ayrı konteyner ve iki ayrı birim alır ve çakışamaz — çünkü buradaki her ad dosyada bildirilmez, projenin adından türetilir.

## Ona ulaşmak

Yalnız bu projenin ağı içinden, satırda gösterilen konteyner adıyla:

```
QDRANT_HOST=stackvo-shop-vectors
QDRANT_PORT=6333
```

Host portu ve host yolu yoktur; bu bir eksiklik değil, kurulumun kendisidir. `stackvo php`'nin projenin konteynerinde koşmasını meşru kılan gerekçe — konteyner zaten bu deponun kodunu çalıştırıyor — dosyanın adlandırdığı **yeni** bir imaj için geçerli değildir; bu yüzden projenin kendi ağının dışına uzanan her şey, onaylanacağı bir kapı kurulana kadar reddedilir.

Pratik sonucu: uygulama onu kullanabilir, sen tarayıcıda açamazsın.

## Bildirmek

```json
"sidecars": {
  "vectors": {
    "image": "qdrant/qdrant:v1.19.0",
    "about": "Öneri sayfası için vektör araması",
    "env": { "QDRANT__SERVICE__API_KEY": "local-only" },
    "volumes": [{ "name": "storage", "path": "/qdrant/storage" }]
  }
}
```

`image` mutlaka etiket taşımalı. Etiketsiz bir imaj, geçen ay çekenin altından kayar — paket sürümünün sabitlenmesiyle aynı gerekçe.

`env` depoya işlenir, yani yapılandırma içindir, sırlar için değil: ekipteki herkesin okuyabildiği dosyadır.

`volumes` Docker adlandırılmış birimleridir. Host bind mount, bu biçimin reddettiği şeydir.

## Ne zaman paket istemek yerine bunu yazmalı

Konteyner, makinenin sunduğu bir şey değil de bu deponun bir ayrıntısıysa. Tek bir projenin indekslediği bir vektör veritabanı, üçüncü taraf bir API'nin taklidi, ekibin kendi baktığı bir worker imajı — hiçbiri herkesin paylaştığı bir kataloga ait değildir.
