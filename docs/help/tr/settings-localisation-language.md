# Dil

Arayüzün ve tepsi menüsünün dili.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Dil | Arayüz dilini seçer. Anında uygulanır. |
| Dil etiketi | Yeni bir dil paketi başlatmak için `de`, `fr` ya da `pt-BR` gibi bir etiket. |
| Çeviri başlat | O etiket için çevirebileceğiniz bir dosya oluşturur. |
| Kaldır | Bir dil paketini siler. |

## Dil paketleri

İngilizce ve Türkçe uygulamanın içine gömülüdür. Diğer diller, uygulamanın yapılandırma klasöründeki JSON dosyalarıdır; kart her birinin yolunu gösterir.

**Çeviri başlat**, bütün dizeleri İngilizce metinleriyle birlikte içeren bir dosya yazar; siz satır satır değiştirirsiniz. Yüzde, artık İngilizce olmayan dizeleri sayar — yani yepyeni bir paket %0'dan başlar ve siz ilerledikçe %100'e gider. Henüz gelmediğiniz her şey İngilizce görünür; eksik çeviri arayüzü bozmaz.

### Dilinizin hangi yöne okunduğunu söylemek

Dosyanın başlarında:

```json
"language": { "label": "العربية", "direction": "rtl" }
```

`label`, seçicide dilinizin adı. `direction` ise `ltr` ya da `rtl`; `rtl` diyen bir paket seçildiğinde pencerenin tamamını — iletişim kutuları ve menüler dahil — sağdan sola dizer, aşağıdaki karttaki anahtara dokunmadan. O anahtar bir tercihtir ve yön belirtmemiş her dil için kararı hâlâ o verir.

## Bilinmesi gerekenler

- Dili değiştirmek tepsi menüsünü de yeniden adlandırır.
- Konsol panellerinin dili ayrıdır; alttaki Konsol dili kartına bakın.
- Dilinizde İngilizcesiyle aynı olan kelimeler çevrilmemiş sayılır. Yüzde biraz eksik gösterir, ve güvenli yön budur.
- Ayrıştırılamayan bir dosya seçiciden kaybolmak yerine hatasıyla listelenir.
