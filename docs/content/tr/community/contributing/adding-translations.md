# Çeviri ekleme

StackVo aynı anda üç yerde İngilizce ve Türkçe konuşur. Bir çeviri üçüne de
dokunabilir ya da yalnızca iyileştirdiğiniz tekine; ikisi de makbuldür.

| Ne | Nerede | Kim denetler |
| --- | --- | --- |
| Uygulama arayüzü | `src/i18n/locales/en.js`, `tr.js` | `tests/i18n.spec.js`: her dilde aynı anahtarlar, kullanılan her anahtar tanımlı |
| Yardım kartları, karttaki **?** | `docs/help/en/`, `docs/help/tr/` | `tests/help-topics.spec.js`: her konu iki dilde |
| Bu site | `docs/content/en/`, `docs/content/tr/` | strict derleme: her sayfa iki yapılandırmada listeli |

## Bir çeviriyi iyileştirmek

En yararlı tür ve en hızlısı.

1. **Arayüz metni.** Dizeyi `src/i18n/locales/tr.js` (ya da `en.js`) içinde
   bulun; anahtarlar görünüme göre gruplanmıştır, ayarlar sayfasının dizeleri
   `settings` bloğundadır. Değeri değiştirin, anahtarı asla.
   `npm run test:js` iki dosyanın hâlâ uyuştuğunu doğrular.
2. **Bir yardım kartı.** O kartın [Uygulama içi yardım](../../help/index.md)
   altındaki sayfasındaki kalem dosyayı açar. İlk satırdaki `# başlığı`
   koruyun; uygulama onunla başlamayan bir belgeyi reddeder.
3. **Bir site sayfası.** Sayfanın üstündeki kalem. İki yapılandırma aynı
   sayfaları listeler, sayfa yerini korur.

İki dilde de olduğu gibi kalan kelimeler: *StackVo*, *Docker*, *snapshot*,
*worktree*, *tunnel*, ürün adları ve komut adları. Doğal okunan bir cümle,
kelimesi kelimesine olandan iyidir.

## Yeni bir dil eklemek

Henüz üçüncü bir dil yok ve eklemek bir dosya bırakmak değil, kod
değişikliğidir; parçalar birlikte planlanabilsin diye önce bir
[issue](https://github.com/stackvo/stackvo/issues/new/choose) ile söyleyin.
Parçalar:

1. **Arayüz.** `en.js`'deki her anahtarı taşıyan yeni bir
   `src/i18n/locales/<kod>.js`; `src/i18n/index.js` içinde Vuetify'ın o dil
   için yerel ayarıyla birlikte kaydedilir ve uygulamanın kabul ettiği kodlara
   eklenir. `tests/i18n.spec.js` eksik bir anahtarı reddeder.
2. **Yardım kartları.** Konu başına bir belgeyle `docs/help/<kod>/`; yüzden
   fazla. Hepsi yazılana kadar uygulama o konu için İngilizce belgeyi gösterir:
   `en` yedektir, yani yarı çevrilmiş bir dil ilk pull request'ten itibaren
   kullanılabilir.
3. **Bu site.** `mkdocs.tr.yml`'nin yaptığı gibi `mkdocs.yml`'den miras alan
   bir `docs/mkdocs.<kod>.yml`, bir `docs/content/<kod>/` ağacı, değiştiricinin
   sunması için `extra.alternate` içinde bir satır ve yayın iş akışında bir
   derleme satırı.

Arayüzle başlayın. İnsanın ilk gördüğü odur; gerisi yazılana kadar yedek
taşır.

## Üslup

Kısa cümleler. Okurun bir derdi var ve düzyazı için burada değil. Bir
düğmenin ne yaptığını ve ne yazdığını söyleyin. Yardım kartlarının
[yazım kılavuzu](https://github.com/stackvo/stackvo/blob/main/docs/help/README.md)
standarttır.
