# Profilleyici

Xdebug'in kendi profilleyicisi. Çıktıyı bu uygulamanın okuduğu dosyalara yazar. Ek hesap ya da ek eklenti gerekmez.

## Kipler

| Kip | Ne yapar |
| --- | --- |
| Adım adım hata ayıklama | Her istekte IDE'nize bağlanır. |
| Profilleme | Bir tetikleyici bekler, sonra o isteğin profilini dosyaya yazar. |
| İz | Her fonksiyon girişini ve çıkışını yazar. |

Biri ya da diğeri. İkisini birden açık bırakmak birini bozar, o yüzden kart tek seçim yaptırır.

## Nasıl kayıt alınır

Bir istek talep etmeden hiçbir şey kaydedilmez. İncelemek istediğiniz adrese `?XDEBUG_TRIGGER=1` ekleyin ya da aynı adı çerez olarak tanımlayın. Kart, kullanılacak tetikleyicinin tam adını gösterir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kip seçimi | Yukarıdaki üç kipten birini uygular. |
| Kayıt listesi | Yazılmış profil dosyaları. Birine tıklamak alev grafiğini açar. |
| Sil | Kayıtları temizler. |
| Konteynere uygula | Çalışan konteyner ile seçilen kip uyuşmuyorsa konteyneri yeniden oluşturur. |

## İz kaydının bedeli

İz, profilden çok daha ağırdır. Tek bir istek yüzlerce megabayta çıkabilir. Bir sayfayı kaydedin, sonra kipi geri alın.

Alev grafiği çok uzun bir izin tamamını çizemez. Böyle bir durumda kart, gördüğünüzün isteğin yalnızca başı olduğunu söyler.

## Bilinmesi gerekenler

- Önce Xdebug açık olmalı. Profilleme aynı eklentinin bir kipidir.
- Kip değişikliği konteyner yeniden oluşturulana kadar geçerli olmaz. Kart uyuşmazlığı söyler.
