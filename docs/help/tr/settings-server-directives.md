# Ek yönergeler

Bu sunucu için üretilen her yapılandırmaya eklenecek satırlar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Yönerge metni | Sunucunun kendi sözdiziminde satırlar. Örnek: `client_body_timeout 120s;` |

## Değişken kullanımı

`{{ VAR }}` yazımı `.env` üzerinden yerine konur. Bir sonraki üretimde etkili olur.

## Bilinmesi gerekenler

- Yorumlar ve boş satırlar atılır. Yalnızca not içeren bir dosya hiçbir şeyi değiştirmez.
- Yazdığınız satırlar doğrulanmaz. Geçersiz bir yönerge sunucunun başlamamasına yol açar; sorun yaşarsanız Loglar'a bakın.
- Yalnızca dosya üzerinden yapılandırılan sunucularda çalışır. Yukarıdaki "Nerede geçerli" kartına bakın.
