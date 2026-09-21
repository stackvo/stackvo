# Desteklenen yığın

StackVo'nun bir konteynere koyabildiği her çalışma zamanı, web sunucusu, veritabanı ve araç, sunduğu sürümlerle. Buradaki her şey yeni proje panelinde ya da katalogda bir seçimdir.

## Diller ve sürümler

| Dil | Sürümler | Varsayılan |
| --- | --- | --- |
| **PHP** | 5.6 · 7.0–7.4 · 8.0–8.5 | 8.4 |
| **Node.js** | 16 · 18 · 20 · 21 · 22 · 23 | 22 |
| **Python** | 2.7 · 3.5–3.14 | 3.14 |
| **Go** | 1.11–1.23 | 1.23 |
| **Ruby** | 2.4–3.3 | 3.3 |
| **Rust** | 1.70–1.84 | 1.84 |

## Web sunucuları

`nginx` · `apache` · `caddy` · `frankenphp` · **`swoole`** · **`roadrunner`**

Son ikisi Laravel Octane'in iki sürücüsüdür; ikisi de HTTP sunucusunun
kendisidir ve Traefik 80 yerine 8000'e bakar.

## Servisler

| Kategori | Servisler |
| --- | --- |
| **Veritabanı** | MySQL · MariaDB · PostgreSQL · MongoDB · Cassandra · ClickHouse · MS SQL Server |
| **Önbellek** | Redis · Memcached · Valkey · Dragonfly |
| **Kuyruk / mesaj** | RabbitMQ · Kafka · Soketi · Beanstalkd |
| **Arama** | Elasticsearch · Kibana · Meilisearch · Typesense · Solr |
| **Depolama** | MinIO |
| **İzleme** | Grafana · Prometheus · Graylog |
| **Geliştirici** | MailHog · Mailpit · Blackfire |
| **Yönetim arayüzü** | phpMyAdmin · Adminer · pgAdmin · Kafbat · mongo-express · phpCacheAdmin |

Servisler örnek (instance) olarak kurulur: MySQL 8.0 ve 8.4 yan yana çalışabilir
ve her proje istediği örneğe bağlanır.

<figure markdown>
![Katalog](../screenshots/web/market.webp){ loading=lazy }
<figcaption>Katalog</figcaption>
</figure>

## PHP eklentileri

80'den fazla eklenti tanınır (`apcu`, `imagick`, `intl`, `redis`, `swoole`,
`mongodb`, `xdebug`, `sqlsrv`…). Varsayılan set, seçtiğiniz PHP sürümüyle
**kurulabilir olduğu doğrulanmış** bir settir.
