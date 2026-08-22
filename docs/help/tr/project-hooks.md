# Bu proje başlarken ve dururken

`stackvo.json` içinde tanımlı komutlar. Proje başlarken, dururken ya da yeniden derlenirken çalışırlar.

## Neden onay isteniyor

Bir hook'u repoyu yazan kişi yazar ve Başlat'a bastığınızda çalışır.

- **Konteynerde** çalışan adımlar onay istemez. O konteyner zaten bu deponun kodunu çalıştırıyor.
- **Bu makinede** çalışan adımlar onay ister. Sizin makinenizde, sizin haklarınızla çalışırlar.

Komutlar tam hâliyle, satır satır yazılır. Yanlarında nerede çalışacakları belirtilir. Okumadan onaylamak için bir özet sunulmaz.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Bu komutları onayla | Ekranda gördüğünüz komut listesini onaylar. |
| Onayı geri çek | Onayı kaldırır. Makinede çalışan adımlar bir daha çalışmaz. |

## Bilinmesi gerekenler

- Onay tam olarak o komutlara kaydedilir. Manifest değişirse yeniden sorulur. Yani onay, listeye verilmiş bir makbuzdur; projeye verilmiş bir güven oyu değil.
- Bir yönetici hook'ları kapatmış olabilir. O durumda kart bunu söyler; makinede çalışan adımlar kapalıyken konteynerdekiler etkilenmez.
