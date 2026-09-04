# Güncellemeler

Yeni sürümün var olup olmadığını denetler ve kurar.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Güncellemeleri denetle | Yayın sunucusuna sorar. |
| Kur ve yeniden başlat | Güncellemeyi indirir, kurar ve uygulamayı yeniden başlatır. |
| Beta sürümleri de al | Denetimin sunduklarına ön sürümleri ekler. Kararlı sürümler yine gelir. |

## Bilinmesi gerekenler

- Güncellemeler imzalıdır. Uygulama, indirdiği paketi içine gömülü açık anahtarla doğrular; doğrulanmayan bir paket kurulmaz.
- Yapının içinde açık anahtar yoksa güncelleme denetimi kapalıdır ve kart bunu söyler.
- Kurulum uygulamayı kapatır. Çalışan konteynerleriniz etkilenmez.
- Beta, "kararlı artı ön sürümler" demektir; ayrı bir akış değildir. Beta
  seçili bir kurulum her kararlı sürümü yine alır; kötü çıkan bir beta, bir
  sonraki sürümle (kararlı ya da beta) geride kalır. Kararlı bir kuruluma
  hiçbir zaman ön sürüm sunulmaz.
- Beta anahtarı StackVo bir sonraki açılışında geçerli olur: güncelleyici
  nereye bakacağını açılışta bir kez okur. O ana kadar beta seçili kurulum
  kararlı kanalı denetlemeye devam eder; bu her iki yönde de güvenlidir.
- Henüz hiç beta yayımlanmadıysa beta seçili kurulum yalnızca kararlı
  güncellemeleri alır. Anahtar, güncellemelerin gelmesini hiçbir durumda
  durduramaz.
