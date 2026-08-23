# Performans katmanı

Ağır dizinleri host dosya sisteminden çıkarıp Docker biriminde tutar. macOS ve Windows'ta Docker'ı yavaş hissettiren yer burasıdır.

## Neden bir liste, tek bir anahtar değil

Kazanç hangi dizini taşıdığınıza bağlıdır. `examples/perf_layer_bench.rs` ile, **bu sürümün derlendiği makinede** ölçüldü — sizinki farklı çıkacaktır, ve Linux'ta geçilecek bir dosya sistemi sınırı zaten yok:

| Taşınan | Framework açılışı | İstek yazmaları |
| --- | --- | --- |
| Hiçbiri (bind mount) | 1,47 s | 1,14 s |
| `vendor` | 0,39 s (3,8 kat) | değişmedi |
| `vendor` + `storage/framework` | 0,40 s | 0,41 s (2,8 kat) |

`vendor` açılışı hızlandırır, yazmalara hiç dokunmaz. Yazmaları hızlandıran `storage/framework`'tür. Karttaki her satır kendi sayısını bu yüzden taşıyor.

`bootstrap/cache` ve `node_modules` sunuluyor ama ölçülmedi; o yüzden satırları yukarıdaki sayılardan birini ödünç almak yerine bunu söylüyor.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Dizin anahtarı | O dizini birime taşır ya da host'a geri alır. |
| Host'a aktar | Birimdeki içeriğin bir anlık görüntüsünü host'a kopyalar. |
| Birimi sil | Birimi ve içindekileri kaldırır. |

Değişiklikler konteynere uygulanana kadar etkili olmaz; kart bunu söyler.

## Bilinmesi gerekenler

- Kendi kodunuz her zaman editörünüzün gördüğü yerde kalır. Yalnızca konteyner içindeki araçların yazdığı dizinler taşınır.
- Taşınan bir dizini editörünüz artık göremez. İçine bakmanız gerekirse **Host'a aktar** ile anlık görüntü alın. O bir kopyadır; konteyner birime yazmayı sürdürür.
- Projede henüz olmayan bir dizin de listelenebilir. Konteyner içindeki araçlar onu oluşturur.
