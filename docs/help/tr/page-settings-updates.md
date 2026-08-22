# Güncellemeler

Yeni sürümün var olup olmadığını denetler ve kurar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Güncellemeleri denetle | Yayın sunucusuna sorar. |
| Kur ve yeniden başlat | Güncellemeyi indirir, kurar ve uygulamayı yeniden başlatır. |

## Bilinmesi gerekenler

- Güncellemeler imzalıdır. Uygulama, indirdiği paketi içine gömülü açık anahtarla doğrular; doğrulanmayan bir paket kurulmaz.
- Yapının içinde açık anahtar yoksa güncelleme denetimi kapalıdır ve kart bunu söyler.
- Kurulum uygulamayı kapatır. Çalışan konteynerleriniz etkilenmez.
