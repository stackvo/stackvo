# Kimlik bilgileri nerede tutuluyor

Veritabanı şifreleri, token'lar ve sunucu kimlikleri `.env` yerine bu makinenin anahtar deposunda durabilir.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Taşı | Değeri Keychain, Credential Manager ya da Secret Service içine kaydeder ve `.env`'de bir referans bırakır. |
| Geri al | Değeri anahtar deposundan `.env` dosyasına geri yazar. |

## Ne kazandırır, ne kazandırmaz

Taşımak, değeri yedeklenen, senkronlanan ve destek konularına yapıştırılan dosyadan çıkarır.

Değer hâlâ `generated/docker-compose.dynamic.yml` içine yazılır; Compose onu oradan okur. Yani bu işlem şifreyi `.env`'den çıkarır, diskten çıkarmaz.

## Kimsenin taşımadıklarını taramak

Yukarıdaki liste, `.env` içinde bir kimlik bilgisi olduğunu **bildikten sonra** izlenen yöndür. **Kimlik bilgisi tara** ise diğeridir: kimsenin taşımadıklarını bulur — ve daha kötüsü, var olma sebebi olan hâli: git'in takip ettiği bir dosyada duranları.

**Değerin** şekline bakar, yalnız anahtarın adına değil. Maskeleme adları sonekle eşler (`PASSWORD`, `TOKEN`, `KEY`); bu maskeleme için doğrudur ve burada yetmez: `MY_FAVOURITE_THING` adlı bir değişken de pekâlâ bir AWS anahtarı tutabilir. Bu yüzden her kural, sahibinin yayımladığı bir şekildir — `AKIA…`, `ghp_…`, `xoxb-…`, `sk_live_…`, bir PEM özel anahtar başlığı — ve ad kuralı ikinci, bağımsız bir ağ olarak korunur.

"Uzun ve rastgele görünen dize" kuralı **bilerek yoktur**. Öyle bir kural küçültülmüş JavaScript'te, bir kilit dosyasındaki özette ve base64 bir görselde ateşlenir; insanların görmezden gelmeyi öğrendiği bir tarayıcı, hiç tarayıcı olmamasından kötüdür — bir kaçırma bir bulguya, bir yanlış pozitif ise özelliğin tamamına mal olur.

| Neye bakar | Neden |
| --- | --- |
| `.env` | Bu makinedeki değerler. Anahtar deposuna taşınmış bir anahtar bulgu değildir — uygulamanın istediği şeyi yapmışsınızdır. |
| Git'in **takip ettiği** dosyalar | Takip edilen şey, tam olarak makineden çıkan şeydir. `node_modules` ve derleme çıktısı okunmaz, çünkü kimse onları push etmez. |

### Bir bulgu, değerin yerine ne taşır

Sırrı alıntılayan bir rapor, onun ikinci bir kopyasıdır — hem de insanların fotoğrafını çekip sohbet penceresine yapıştırdığı bir ekranda. Ama tek başına "asla yazma", elinizde üzerine bir şey yapamayacağınız bir satır bırakır: *AWS erişim anahtarı* diyen iki satır, bunun iki yerdeki tek anahtar mı yoksa iki anahtar mı olduğunu söylemez. Bu yüzden her bulgu, bu alandaki her tarayıcının taşıdığını taşır:

| | Nedir | Ne işe yarar |
| --- | --- | --- |
| Önizleme | `AKIA…MPLE` — ilk ve son dört karakter | Parola yöneticinizdeki dördün arasından **hangi** anahtar olduğunu tanımak |
| Parmak izi | Değerin sha256'sının on iki hex karakteri | Tek parmak izi taşıyan iki satır, **iki yerdeki tek sırdır** — bir anahtar döndürmekle iki anahtar döndürmek arasındaki fark |

On altı karakterden kısa bir değer bütünüyle maskelenir: kısa bir parolanın her iki ucundan dörder karakter, parolanın kendisidir.

### Hiç commit'lenmiş mi?

**Yol üzerinden** sorulur — `git log --all -- <yol>` — asla değer üzerinden. `git log -S<sır>` o sırrı bir komut satırına koyardı; makinedeki her süreç onu `ps` ile okuyabilir. Yol cevabı ayrıca daha güçlüdür: commit'lenip sonra silinmiş bir dosya, içindeki her şeyle birlikte hâlâ geçmiştedir — değer araması ise birinin yarısını döndürdüğü anda onu kaçırır.

`.env` hakkında bu yüzden iki ayrı cevap var: **şu anda takip ediliyor** ve **bir noktada commit'lenmiş**. İkincisi, insanların en çok yanıldığı yer. Dosyayı bugün takipten çıkarmak onu geçmişten çıkarmaz.

### `.env`'i git'ten çıkarmak

Bir proje seçilmişse ve `.env`'i takip ediliyorsa, kart onarımı sunar — ve standardın yaptığını standardın sırasıyla yapar:

1. **`git rm --cached`** — takipten çıkarır, dosyayı diskte bırakır. `git rm` değil: o, yığınınızın üzerinde çalıştığı yapılandırmayı silerdi.
2. **`.gitignore`** — tahmin edilmez, `git check-ignore` ile **sorulur**; çünkü `.gitignore`, `.git/info/exclude` ve genel bir dışlama dosyası birlikte karar verir. Satır yalnız cevap hayırsa eklenir.
3. **`.env.example`** — baştan beri takip edilmesi gereken dosya: aynı anahtarlar, değersiz, yorumlarınız ve gruplamanız korunarak. Yalnız yoksa yazılır; sizinkinin üzerine üretilmiş bir dosya yazmak, sonraki kişi için yazdıklarınızı çöpe atmak olurdu.

Yapmadığı ve bunu **söylediği** iki şey:

- Kaldırma **hazırlanmıştır** (staged), commit'lenmemiştir. Siz commit'leyip push edene kadar bu makineden hiçbir şey çıkmamıştır.
- Geçmişi yeniden yazamaz. Dosya bir kez commit'lendiyse içindeki her değer hâlâ depodadır — **onları döndürün.** İnsanların atladığı adım budur, ve açığı gerçekten kapatan tek adım odur.

Tarama sınırlıdır — 2.000 dosya, her biri yarım megabayta kadar — ve kaç tanesini atladığını söyler. Dört yüz dosyanın üzerinden geçip hiçbir şey söylemeyen bir tarama, temiz bir depo gibi okunurdu.

## Bilinmesi gerekenler

- `stackvo.sh` komut satırı aracı anahtar deposunu okuyamaz. Bu çalışma alanında onu da kullanıyorsanız kimlik bilgilerini `.env`'de bırakın.
- Bu makinede uygulamanın ulaşabildiği bir anahtar deposu yoksa hiçbir şey taşınamaz; kart bunu söyler.
- Anahtar deposunu işaret eden bir kimlik bilgisi çözülemiyorsa dosya üretimi engellenir. Anahtar zincirinizi açın ya da değeri geri alın.
