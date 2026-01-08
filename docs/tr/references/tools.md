# Araçlar Referansı

Stackvo'da mevcut tüm yönetim ve admin araçları için kapsamlı referans. Bu web tabanlı araçlar, veritabanları, cache sistemleri, message queue'lar ve PHP performansını yönetmek için grafiksel arayüzler sağlar. Tüm araçlar Stackvo Web UI üzerinden erişilebilir ve tools container içinde çalışır.

---

## Araçlar Kategorileri

- Veritabanı Yönetim Araçları (4)
- Cache Yönetim Araçları (1)
- Message Queue Yönetim Araçları (1)
- Performans İzleme Araçları (1)

---

## Veritabanı Yönetim Araçları

### Adminer

**Versiyon:** 4.8.1  
**URL:** `https://adminer.stackvo.loc`  
**Environment Variable:** `TOOLS_ADMINER_ENABLE`

**Açıklama:**  
Adminer, PHP ile yazılmış tam özellikli bir veritabanı yönetim aracıdır. phpMyAdmin'e hafif bir alternatiftir ve tek bir arayüzde birden fazla veritabanı sistemini destekler.

**Desteklenen Veritabanları:**
- MySQL
- MariaDB
- PostgreSQL
- SQLite
- MongoDB
- Oracle
- MS SQL
- Elasticsearch

**Temel Özellikler:**
- Evrensel veritabanı arayüzü
- Hafif yapı (tek PHP dosyası)
- Birden fazla veritabanı sistemi desteği
- Çeşitli formatlarda veri dışa/içe aktarma (SQL, CSV, XML)
- Özel SQL sorguları çalıştırma
- Tablo, view, trigger ve stored procedure yönetimi
- Kullanıcı ve yetki yönetimi
- Veritabanı şeması görselleştirme

**Bağlantı Örnekleri:**

MySQL/MariaDB:
```
System: MySQL
Server: stackvo-mysql
Username: stackvo
Password: stackvo
Database: stackvo
```

PostgreSQL:
```
System: PostgreSQL
Server: stackvo-postgres
Username: stackvo
Password: root
Database: stackvo
```

MongoDB:
```
System: MongoDB
Server: stackvo-mongo
Username: root
Password: root
Database: stackvo
```

**Konfigürasyon:**
```bash
TOOLS_ADMINER_ENABLE=true
TOOLS_ADMINER_VERSION=4.8.1
TOOLS_ADMINER_URL=adminer
```

---

### PhpMyAdmin

**Versiyon:** 5.2.1  
**URL:** `https://phpmyadmin.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPMYADMIN_ENABLE`

**Açıklama:**  
PhpMyAdmin, MySQL ve MariaDB veritabanları için en popüler web tabanlı yönetim aracıdır. Gelişmiş özelliklerle kapsamlı bir veritabanı yönetim arayüzü sağlar.

**Desteklenen Veritabanları:**
- MySQL
- MariaDB

**Temel Özellikler:**
- MySQL/MariaDB için sezgisel web arayüzü
- Veritabanları, tablolar, alanlar ve indeksleri görüntüleme, oluşturma ve değiştirme
- SQL ifadeleri ve toplu sorguları çalıştırma
- Veri içe/dışa aktarma (SQL, CSV, XML, PDF, Excel, vb.)
- Kullanıcı ve yetki yönetimi
- Sunucu konfigürasyonu ve durum izleme
- Görsel sorgu oluşturucu
- Veritabanı arama ve değiştirme
- Sık kullanılan sorguları yer imlerine ekleme
- Birden fazla sunucu yönetimi

**Bağlantı:**
```
Server: stackvo-mysql (veya stackvo-mariadb)
Username: stackvo
Password: stackvo
```

**Root Erişimi:**
```
Username: root
Password: root
```

**Konfigürasyon:**
```bash
TOOLS_PHPMYADMIN_ENABLE=true
TOOLS_PHPMYADMIN_VERSION=5.2.1
TOOLS_PHPMYADMIN_URL=phpmyadmin
```

---

### PhpPgAdmin

**Versiyon:** 7.13.0  
**URL:** `https://phppgadmin.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPPGADMIN_ENABLE`

**Açıklama:**  
PhpPgAdmin, PostgreSQL veritabanları için web tabanlı bir yönetim aracıdır. PostgreSQL sunucuları, veritabanları ve nesneleri yönetmek için kullanıcı dostu bir arayüz sağlar.

**Desteklenen Veritabanları:**
- PostgreSQL

**Temel Özellikler:**
- Eksiksiz PostgreSQL veritabanı yönetimi
- Veritabanları, şemalar, tablolar ve view'ları görüntüleme ve değiştirme
- Söz dizimi vurgulama ile SQL sorguları çalıştırma
- Veri içe/dışa aktarma
- Kullanıcı, grup ve yetki yönetimi
- Fonksiyon, trigger ve sequence oluşturma ve yönetme
- Görsel şema tarayıcı
- Gelişmiş arama özellikleri
- PostgreSQL'e özgü özellikler için destek (array, JSON, vb.)

**Bağlantı:**
```
Server: stackvo-postgres
Username: stackvo
Password: root
Database: stackvo
```

**Konfigürasyon:**
```bash
TOOLS_PHPPGADMIN_ENABLE=true
TOOLS_PHPPGADMIN_VERSION=7.13.0
TOOLS_PHPPGADMIN_URL=phppgadmin
```

---

### PhpMongo

**Versiyon:** 1.3.3  
**URL:** `https://phpmongo.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPMONGO_ENABLE`

**Açıklama:**  
PhpMongo, MongoDB veritabanlarını, koleksiyonları ve dokümanları yönetmek için sezgisel bir arayüz sağlayan web tabanlı bir MongoDB yönetim aracıdır.

**Desteklenen Veritabanları:**
- MongoDB

**Temel Özellikler:**
- MongoDB veritabanı ve koleksiyon yönetimi
- Doküman CRUD işlemleri (Create, Read, Update, Delete)
- JSON doküman görüntüleyici ve düzenleyici
- MongoDB sorguları çalıştırma
- Koleksiyonları içe/dışa aktarma
- İndeks yönetimi
- Kullanıcı ve rol yönetimi
- Veritabanı istatistikleri ve izleme
- GridFS dosya yönetimi

**Bağlantı:**
```
Server: stackvo-mongo
Port: 27017
Username: root
Password: root
Database: stackvo
Authentication Database: admin
```

**Konfigürasyon:**
```bash
TOOLS_PHPMONGO_ENABLE=true
TOOLS_PHPMONGO_VERSION=1.3.3
TOOLS_PHPMONGO_URL=phpmongo
```

---

## Cache Yönetim Araçları

### PhpMemcachedAdmin

**Versiyon:** 1.3.0  
**URL:** `https://phpmemcachedadmin.stackvo.loc`  
**Environment Variable:** `TOOLS_PHPMEMCACHEDADMIN_ENABLE`

**Açıklama:**  
PhpMemcachedAdmin, Memcached sunucuları için web tabanlı bir yönetim aracıdır. Cache altyapınız için gerçek zamanlı izleme ve yönetim yetenekleri sağlar.

**Desteklenen Sistemler:**
- Memcached

**Temel Özellikler:**
- Gerçek zamanlı Memcached sunucu izleme
- Cache istatistiklerini görüntüleme (hit rate, bellek kullanımı, bağlantılar)
- Önbelleğe alınmış öğeleri ve değerlerini görüntüleme
- Tekil cache öğelerini silme veya tüm cache'i temizleme
- Birden fazla sunucu desteği
- Cache performansı için görsel grafikler
- Bellek kullanımı görselleştirme
- Bağlantı izleme

**Bağlantı:**
```
Server: stackvo-memcached
Port: 11211
```

**Konfigürasyon:**
```bash
TOOLS_PHPMEMCACHEDADMIN_ENABLE=true
TOOLS_PHPMEMCACHEDADMIN_VERSION=1.3.0
TOOLS_PHPMEMCACHEDADMIN_URL=phpmemcachedadmin
```

---

## Message Queue Yönetim Araçları

### Kafbat (Kafka UI)

**Versiyon:** 1.4.2  
**URL:** `https://kafbat.stackvo.loc`  
**Environment Variable:** `TOOLS_KAFBAT_ENABLE`

**Açıklama:**  
Kafbat (eski adıyla Kafka UI), Apache Kafka kümelerini yönetmek ve izlemek için modern bir web arayüzüdür. Topic'ler, mesajlar, consumer grupları ve küme konfigürasyonu ile çalışmak için kapsamlı araçlar sağlar.

**Desteklenen Sistemler:**
- Apache Kafka
- Kafka Connect
- Schema Registry

**Temel Özellikler:**
- Kafka küme izleme ve yönetimi
- Topic oluşturma, konfigürasyon ve silme
- Topic'lerdeki mesajları görüntüleme ve arama
- Topic'lere mesaj gönderme
- Consumer grup izleme ve yönetimi
- Partition ve replica yönetimi
- Kafka Connect connector yönetimi
- Schema Registry entegrasyonu
- ACL (Access Control List) yönetimi
- Gerçek zamanlı metrikler ve istatistikler

**Bağlantı:**
```
Kafka Broker: stackvo-kafka:9092
```

**Konfigürasyon:**
```bash
TOOLS_KAFBAT_ENABLE=true
TOOLS_KAFBAT_VERSION=1.4.2
TOOLS_KAFBAT_URL=kafbat
```

**Not:** Kafbat, Kafka servisinin aktif olmasını gerektirir:
```bash
SERVICE_KAFKA_ENABLE=true
```

---

## Performans İzleme Araçları

### OpCache GUI

**Versiyon:** 3.6.0  
**URL:** `https://opcache.stackvo.loc`  
**Environment Variable:** `TOOLS_OPCACHE_ENABLE`

**Açıklama:**  
OpCache GUI, PHP OPcache'i izlemek ve yönetmek için web tabanlı bir arayüzdür. Önbelleğe alınmış scriptler, bellek kullanımı ve cache performansı hakkında detaylı istatistikler sağlar.

**Desteklenen Sistemler:**
- PHP OPcache

**Temel Özellikler:**
- Gerçek zamanlı OPcache istatistikleri
- Bellek kullanımı görselleştirme
- Detaylarıyla önbelleğe alınmış dosya listesi
- Cache hit/miss oranı izleme
- Belirli önbelleğe alınmış dosyaları geçersiz kılma
- Tüm cache'i sıfırlama
- Konfigürasyon özeti
- Performans grafikleri ve çizelgeleri
- Bellek parçalanma analizi

**Kullanım:**
- OPcache istatistiklerini görüntülemek için URL'ye erişin
- Cache verimliliğini ve bellek kullanımını izleyin
- Sık önbelleğe alınan scriptleri belirleyin
- Geliştirme sırasında gerektiğinde cache'i temizleyin

**Konfigürasyon:**
```bash
TOOLS_OPCACHE_ENABLE=true
TOOLS_OPCACHE_VERSION=3.6.0
TOOLS_OPCACHE_URL=opcache
```

**Not:** OPcache istatistikleri, Stackvo'da çalışan tüm PHP proje container'larından toplanır.

---

## Araçlara Erişim

### Web UI Üzerinden

1. Stackvo Web UI'ı açın: `https://stackvo.loc`
2. **Tools** sekmesine gidin
3. İstediğiniz araca tıklayarak yeni sekmede açın

### Doğrudan Erişim

Tüm araçlara doğrudan URL'leri üzerinden erişilebilir:

```
https://adminer.stackvo.loc
https://phpmyadmin.stackvo.loc
https://phppgadmin.stackvo.loc
https://phpmongo.stackvo.loc
https://phpmemcachedadmin.stackvo.loc
https://opcache.stackvo.loc
https://kafbat.stackvo.loc
```

**Önemli:** Bu domainleri `/etc/hosts` dosyanıza eklediğinizden emin olun:

```bash
127.0.0.1  adminer.stackvo.loc
127.0.0.1  phpmyadmin.stackvo.loc
127.0.0.1  phppgadmin.stackvo.loc
127.0.0.1  phpmongo.stackvo.loc
127.0.0.1  phpmemcachedadmin.stackvo.loc
127.0.0.1  opcache.stackvo.loc
127.0.0.1  kafbat.stackvo.loc
```

---

## Araçları Aktif/Pasif Etme

### Bir Aracı Aktif Etme

`.env` dosyasını düzenleyin ve aracın enable flag'ini `true` olarak ayarlayın:

```bash
# Adminer'ı aktif et
TOOLS_ADMINER_ENABLE=true

# PhpMyAdmin'i aktif et
TOOLS_PHPMYADMIN_ENABLE=true
```

### Bir Aracı Pasif Etme

Enable flag'ini `false` olarak ayarlayın:

```bash
# Kafbat'ı pasif et
TOOLS_KAFBAT_ENABLE=false
```

### Değişiklikleri Uygulama

`.env` dosyasını değiştirdikten sonra, konfigürasyonu yeniden oluşturun ve yeniden başlatın:

```bash
./stackvo.sh generate
./stackvo.sh restart
```

---

## Sorun Giderme

### Araca Erişilemiyor

```bash
# Tools container'ının çalışıp çalışmadığını kontrol et
docker ps | grep stackvo-tools

# Container loglarını kontrol et
docker logs stackvo-tools

# Hosts dosyasını doğrula
cat /etc/hosts | grep stackvo.loc

# Tools container'ını yeniden başlat
docker restart stackvo-tools
```

### Bağlantı Hataları

```bash
# Servisin çalışıp çalışmadığını doğrula
docker ps | grep stackvo-mysql

# Network bağlantısını kontrol et
docker exec stackvo-tools ping stackvo-mysql

# .env dosyasındaki servis kimlik bilgilerini doğrula
cat .env | grep SERVICE_MYSQL
```

### Performans Sorunları

```bash
# Container kaynak kullanımını kontrol et
docker stats stackvo-tools

# Detaylı logları görüntüle
docker logs -f stackvo-tools

# Container'ı yeniden başlat
docker restart stackvo-tools
```

---

## Güvenlik Hususları

1. **Production Kullanımı:** Bu araçlar geliştirme ortamları için tasarlanmıştır. Production için şunları düşünün:
   - Araçları devre dışı bırakma veya erişimi kısıtlama
   - Güçlü kimlik doğrulama kullanma
   - IP whitelisting uygulama
   - VPN veya SSH tunneling kullanma

2. **Kimlik Bilgileri:** Varsayılan kimlik bilgileri `.env` dosyasında ayarlanmıştır. Production için bunları değiştirin:
   ```bash
   SERVICE_MYSQL_ROOT_PASSWORD=guclu_sifre_buraya
   SERVICE_POSTGRES_PASSWORD=guclu_sifre_buraya
   ```

3. **SSL/TLS:** `.env` dosyasında `SSL_ENABLE=true` olduğunda tüm araçlara HTTPS üzerinden erişilebilir

---
