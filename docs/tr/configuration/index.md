---
title: Konfigürasyon
description: Stackvo konfigürasyon kılavuzu - Tüm platformlar için adım adım konfigürasyon
---

# Konfigürasyon

Stackvo'un konfigürasyon sistemi esnek ve katmanlı bir yapıya sahiptir. Bu bölüm, global sistem ayarlarından proje bazlı özelleştirmelere, özel webserver konfigürasyonlarından runtime ayarlarına kadar her seviyede nasıl tam kontrol sağlayabileceğinizi detaylı olarak açıklamaktadır. Üç farklı konfigürasyon seviyesi ile maksimum esneklik ve özelleştirme imkanı sunulmaktadır.

---

## Konfigürasyon Seviyeleri

Stackvo, 3 farklı seviyede konfigürasyon sunar. Her seviye farklı bir amaç için tasarlanmıştır ve birlikte çalışarak maksimum esneklik sağlar:

<div class="grid cards" markdown>

-   :material-cog:{ .lg .middle } __Global__

    ---

    `.env` dosyası üzerinden yönetilir ve tüm sistemi etkiler

    [:octicons-arrow-right-24: Global Konfigürasyon](global.md)

-   :material-file-cog:{ .lg .middle } __Proje__

    ---

    `stackvo.json` dosyası ile proje özelinde ayarlar tanımlanır

    [:octicons-arrow-right-24: Proje Konfigürasyonu](project.md)

-   :material-file-edit:{ .lg .middle } __Özel__

    ---

    `.stackvo/` dizininde özel webserver ve runtime ayarları

    [:octicons-arrow-right-24: Özel Konfigürasyon](custom.md)
</div>
