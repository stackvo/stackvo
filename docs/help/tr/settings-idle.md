# Boştaki projeleri askıya al

Kimsenin istemediği projeleri durdurur.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Boşta dakika | Bu süre boyunca istek almayan projeler durdurulur. `0` özelliği kapatır. |
| N tanesini şimdi askıya al | Eşiği geçmiş projeleri hemen durdurur. |

## Nasıl ölçülür

Proxy'nin erişim günlüğünden ölçülür. Tek dürüst sinyal budur: php-fpm hizmet verirken de uyurken de CPU kullanmaz, o yüzden CPU'ya bakmak yanıltır.

Günlüğün hiç anmadığı bir proje asla askıya alınmaz.

## Bilinmesi gerekenler

- Askıya alınan proje yalnızca durdurulmuştur. Listeden, tepside ya da ⌘K ile yeniden başlatın.
- İstek üzerine uyandırma yoktur. Durmuş bir projeye gelen istek onu başlatmaz.
