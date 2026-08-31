# Bu makineden ne çıkabilir

Aynı biçimde iki soru, ve daha önce ikisinin de cevabı yoktu: **konteynerlerinizin hangisi internete erişebilir**, ve **her birinin imgesi gerçekte nereden geldi**.

Bu kategoride başka hiçbir araç ikisini de cevaplayamaz, ve sebebi bir eksiklik değil: yerel bir ikilinin konteyneri yoktur, dolayısıyla bir programın trafiğini ötekinden ayıracak ağ ad alanı da yoktur.

## "Dışarı erişebiliyor" bir tahmin değil, bir olgu

Bu, Docker ağının bir özelliğidir ve daemon'a sorulur. `internal: true` ile oluşturulmuş bir ağa geçit (gateway) kurulmaz — her ağı internal olan bir konteyner **dışarı yönlendirilemez**, ve bu, davranıştan çıkarsanmış değil, kanıtlanabilir bir şeydir.

Sütun bilerek asimetriktir:

| Cevap | Ne zaman |
| --- | --- |
| **Evet** | Ağlarından en az birinin geçidi var. Bir çıkış yolu, çıkış yoludur. |
| **Hayır** | Üzerinde olduğu her ağ internal olarak oluşturulmuş. |
| **Anlaşılamıyor** | Daemon'ın tarif etmediği bir ağ var. |

**Her** ağın **bilinen** biçimde internal olması dışında hiçbir şey "Hayır" cevabını hak etmez. Başarısız bir sorguya dayanan bir kapalılık iddiası, böyle bir raporun vermemesi gereken tek yanlış cevaptır; o yüzden okunamayan bir ağ, satırı "Anlaşılamıyor"da bırakır.

## İmge nereden geldi

Her konteyner, oluşturulduğu referansı adlandırır; o referansın kayıt defteri sunucusu da Docker'ın kendi kuralına göre ilk bileşendir. Sunucu adı olmayan bir referans — `mysql:8.0` — `docker.io` olarak gösterilir, çünkü çekildiği yer orasıdır; "hiçbiri" demek, adı en çok anılmaya değer sunucuyu atlamak olurdu.

**Bir yönetici kayıt defteri aynası ayarladıysa**, cevabı olmayan takip sorusu buydu: hangi konteynerler oradan geçmedi. *Aynadan gelmemiş* diye işaretlenen bir satır, ya politika gelmeden önce oluşturulmuştur ya da aynanın dokunmadığı bir referanstan. Aynanın tuttuğu bir makinede özet satırı tam olarak tek bir kayıt defteri listeler.

## Hiçbir şeyin nereye bağlandığını söylemez

Docker bir bağlantı kaydı tutmaz. *"Bu konteyner hangi sunucuyla konuştu"* sorusunu cevaplamak, ya konteynerin ağ ad alanının içinde bir paket yakalaması ya da önünde duran bir vekil sunucu ister; bu uygulama bir raporu doldurmak için ikisini de makinenize kurmaz.

Bu, bir boşluk olarak bırakılmak yerine burada söyleniyor — böylece bu sayfadaki hiçbir şey, konteynerlerinizin gittiği yerlerin listesi gibi okunmasın.

## Bayt sayıları

Bunlar Docker'ın arayüz başına sayaçlarıdır ve her konteynerin başlangıcından beri **tüm** trafiği içerir — makineden çıkanı da, kendi konteynerleriniz arasındaki StackVo ağını da. Dolayısıyla bunları *"bu konteynerden hiç bir şey çıktı mı"* diye okuyun; bu gerçekten işe yarar. İnternet kullanımı diye okumayın; o değiller.

Durmuş bir konteyner sıfır değil, hiçbir şey gösterir: Docker'ın onun için sayacı yoktur, ve bir sıfır, ölçümün yokluğu yerine trafik hakkında bir iddia olurdu.
