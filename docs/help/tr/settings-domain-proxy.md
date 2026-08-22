# Ters proxy

Traefik. Her projeye ve yönetim arayüzüne onun üzerinden erişilir; TLS'i de o sonlandırır.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Panoyu aç | Traefik'in kendi yönetim panosunu tarayıcıda açar. |

## Kartın gösterdikleri

Yayınlanan portlar: proxy'nin ana makinede dinlediği portlar. Genellikle 80 ve 443.

## Bilinmesi gerekenler

- Projeler kendi portlarını yayımlamaz. Proxy onlara Docker ağı üzerinden adıyla ulaşır.
- 80 ya da 443 portu başka bir şey tarafından tutuluyorsa proxy başlayamaz. Doktor bunu bildirir.
