# Özel rotalar

Bir adı, StackVo'nun başlatmadığı bir şeye yönlendirir. Kendi başlattığınız bir dev sunucusu, başka bir araçtaki servis ya da bir staging adresi için.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Ad | Yönlendirilecek alan adı. |
| Şuraya gider | Hedef adres. |
| Etkin | Rotayı açar ya da kapatır. |
| Rota ekle | Yeni satır açar. |
| Kaldır | Rotayı siler. |
| Kaydet ve uygula | Rotaları yazar ve yönlendiriciye uygular. |

## localhost yazarsanız

`http://localhost:3000` yazın, StackVo düzeltir. Proxy'nin konteynerinin içinde "localhost" proxy'nin kendisidir; düzeltilmeseydi açıklamasız bir 502 alırdınız.

## Bilinmesi gerekenler

- Rota, hedefin çalışıyor olup olmadığını denetlemez. Hedef kapalıysa adres bir hata döner.
- Ad, çalışma alanının soneki altında olmalıdır ki sertifika onu kapsasın.
