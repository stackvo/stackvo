# Bu yığını dışa aktar

Hangi servislerin etkin olduğunu ve sürümlerini küçük bir JSON dosyasına yazar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Önayar adı | Dosyaya yazılacak ad. |
| Dosyaya kaydet | JSON dosyasını yazar. |

Kart, kaydetmeden önce dosyanın içeriğini gösterir.

## Bilinmesi gerekenler

- Dosyada parola yoktur. Biçimde onları koyacak bir yer yoktur.
- Sürüm kontrolüne eklemek güvenlidir. Amacı da budur: bir ekibin aynı yığını kullanması.
- Dosya yalnızca "hangi servisler ve hangi sürümler" bilgisini taşır. Portlarınızı ve verilerinizi taşımaz.
