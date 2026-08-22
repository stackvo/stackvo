# Ağ ve TLS

Servislerin paylaştığı Docker ağı ve HTTPS ile sunulup sunulmadıkları.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Docker ağı | Tüm servislerin katıldığı ağın adı. |
| HTTPS ile sun | Alan adı soneki için yerel sertifika üretir ve bağlar. |
| HTTP'yi HTTPS'e yönlendir | Düz istekler sitenin kendisi yerine bir yönlendirmeyle yanıtlanır. |
| Varsayılana dön | Ağ adını başlangıç değerine döndürür. |

## Bilinmesi gerekenler

- Ağ adını değiştirmek, sonraki başlatmada konteynerleri yeniden kurar.
- HTTPS'i kapatmak yalnızca sertifikayı devre dışı bırakmaz: HTTPS giriş noktası da üretilmez, ama bütün yönlendirmeler onu hedeflemeye devam eder. Yeniden açılana kadar hiçbir proje çözülmez.
- Yönlendirme yalnızca HTTPS açıkken anlamlıdır. Kapalı bir şemaya yönlendirmek hiçbir yere çıkmaz, o yüzden anahtar pasif kalır.
