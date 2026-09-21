# Supported stack

Every runtime, web server, database and tool StackVo can put in a container, with the versions it offers. Anything here is one pick in the new-project panel or the catalogue.

## Languages and versions

| Language | Versions | Default |
| --- | --- | --- |
| **PHP** | 5.6 · 7.0–7.4 · 8.0–8.5 | 8.4 |
| **Node.js** | 16 · 18 · 20 · 21 · 22 · 23 | 22 |
| **Python** | 2.7 · 3.5–3.14 | 3.14 |
| **Go** | 1.11–1.23 | 1.23 |
| **Ruby** | 2.4–3.3 | 3.3 |
| **Rust** | 1.70–1.84 | 1.84 |

## Web servers

`nginx` · `apache` · `caddy` · `frankenphp` · **`swoole`** · **`roadrunner`**

The last two are Laravel Octane's two drivers; both *are* the HTTP server, so
Traefik points at 8000 rather than 80.

## Services

| Category | Services |
| --- | --- |
| **Databases** | MySQL · MariaDB · PostgreSQL · MongoDB · Cassandra · ClickHouse · MS SQL Server |
| **Cache** | Redis · Memcached · Valkey · Dragonfly |
| **Queue / messaging** | RabbitMQ · Kafka · Soketi · Beanstalkd |
| **Search** | Elasticsearch · Kibana · Meilisearch · Typesense · Solr |
| **Storage** | MinIO |
| **Monitoring** | Grafana · Prometheus · Graylog |
| **Dev tools** | MailHog · Mailpit · Blackfire |
| **Admin UIs** | phpMyAdmin · Adminer · pgAdmin · Kafbat · mongo-express · phpCacheAdmin |

Services are installed as instances: MySQL 8.0 and 8.4 can run side by side,
and each project connects to the one it asked for.

<figure markdown>
![The catalogue](../screenshots/web/market.webp){ loading=lazy }
<figcaption>The catalogue</figcaption>
</figure>

## PHP extensions

More than 80 extensions are known (`apcu`, `imagick`, `intl`, `redis`,
`swoole`, `mongodb`, `xdebug`, `sqlsrv`…). The default set is one that has
been **verified to build** on the PHP version you picked.
