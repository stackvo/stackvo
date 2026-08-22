# PHP derlemesi

Yeni bir PHP konteynerinin neyle kurulacağı.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Composer sürümü | PHP imajına kurulacak Composer sürümü. `latest` derleme anındaki güncel sürümü izler. |
| Node.js sürümü | PHP konteyneri içindeki varlık derlemeleri için. Node projesi çalışma zamanından ayrıdır. |
| Araçlar | PHP ile birlikte kurulacak ek araçlar. Eklemek için yazın, kaldırmak için çarpıya tıklayın. |
| Sistem paketleri | Konteyner içinde `apt` ile kurulacak paketler. |

## Bilinmesi gerekenler

- Değişiklik bundan sonra üretilen projeleri etkiler. Var olan bir projeye uygulamak için o projeyi yeniden derleyin.
- Sistem paketleri imaj boyutunu ve derleme süresini artırır. Yalnızca gerçekten gerekenleri ekleyin.
