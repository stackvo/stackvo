# Worktree'ler

Bir git dalına kendi ortamını verir: kendi klasörü, kendi adresi, kendi veritabanı. İki dal aynı anda çalışır ve checkout'unuza git'in fark edeceği hiçbir şey yazılmaz.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Yeni worktree | Formu açar. |
| Dal | Hangi dalın ortamı olacağı. Zaten başka bir yerde checkout edilmiş bir dal seçilemez. |
| Dalı oluştur | Dal yoksa yenisini açar. |
| Ad | Yeni projenin adı. Boş bırakılırsa daldan türetilir. |
| Veritabanı | Yok, yeni ve boş, ya da bu çalışma alanınınkinin kopyası. |
| Hangi motorda | Kopyanın hangi veritabanı örneğinde duracağı. |
| Ne kadar süre için | Kendi dalınız için boş; bir süre seçmek onu süreli bir kum havuzu yapar. |
| Oluştur | Klasörü, projeyi ve seçilen veritabanını hazırlar. |
| Kaldır | Worktree'yi siler. Dalı ve veritabanını silmek ayrı anahtarlardır, ikisi de varsayılan olarak kapalıdır. |

## Form ne gösterir

Ad, adres ve veritabanı adı formda önceden gösterilir. Bu isimler arka uçtan gelir, yani ekranda gördüğünüz, oluşturulacak olanın aynısıdır.

Bir şey yapılamıyorsa buton pasif kalır ve sebebi ilgili alanın yanında yazar.

## Bu proje bir worktree ise

Kart bunun yerine hangi projenin dalı olduğunu, dalını, adresini ve veritabanını gösterir. Worktree'nin worktree'si oluşturulamaz.

## Bir asistan için kurmak

**Ne kadar süre için** alanını boş bırakırsanız sıradan anlamda bir worktree elde edersiniz: sizin dalınız, siz kaldırana kadar sizin.

Bir süre seçerseniz bir *kum havuzu* elde edersiniz — tek bir iş için kurulmuş, ve kuran kişinin var olduğunu hatırlamayacağı bir ortam. O zaman üç şey birden geçerli olur, ve bu üçü birlikte, dalı bir yapay zekâ asistanına vermeyi makinenizi vermekten farklı bir eyleme çevirir:

1. **Her şeyi kendine ait.** Dizin, ana bilgisayar adı, ortam değişkenleri, ve istediyseniz veritabanının bir kopyası. Yaptığı hiçbir şey, dallandığı projeye ulaşmaz.
2. **Kendi veritabanı girişi** — MySQL ve MariaDB'de, aşağıya bakın. Erişebileceği en kötü şey o kopyadır.
3. **Bunu söyleyen bir kayıt.** Kart, MCP sunucusunun hangi bayraklarla tanıtılacağını gösterir: asistan bu dalı alır, başkasını değil, ve kum havuzunun kalan süresi kadar. O sınırın altında on iki yazma aracı bir projenin sınırlayabildiği dörde iner; yığının tamamını durdurmak onlardan biri değildir.

İş yine de dışarı çıkar: bir kum havuzunun çıktısı **daldır**, ve ortamı kaldırmak dalı silmez. Veritabanı iskeledir.

Zamanlayıcıyla hiçbir şey silinmez, ve hiçbir zaman silinmeyecek — saate bakarak dizin kaldıran bir uygulama, er ya da geç içinde bir sabahın commit'lenmemiş işi olan bir dizini kaldırır. Sürenin yaptığı şey, listenin "zamanı geçti" diyebilmesi; kaldırmak tek tık ve bir karar olarak kalır.

## Dal, ana projenin verisine erişebilir mi

"Kendi veritabanı var" ile "diğerine erişemez" iki ayrı sözdür ve yalnız birincisi bedavadır. Dalın veritabanı da diğer her şeyle aynı motorda durur; ikincisini belirleyen şey, **dala hangi girişin verildiğidir**.

| Kartta yazan | Anlamı |
| --- | --- |
| Veritabanı girişi — kendine ait | Dalın, yalnız kendi şeması üzerinde yetkilendirilmiş bir hesabı var. Dallandığı projeyi okuyamaz, listeleyemez bile. |
| Veritabanı girişi — örnekle ortak | Dal, motorun kendi hesabını kullanır; o örnekteki her veritabanına erişebilir, ana proje dahil. |

Kendine ait giriş **MySQL ve MariaDB**'de düzenleniyor; orada tek şema üzerindeki yetki, uygulamanın sonradan yarattığı tabloları da kapsar. PostgreSQL yapılmadı: yetkilerin, veri kopyalandıktan sonra veritabanının içindeki nesnelere ve sonradan yaratılacaklar için varsayılan yetkilere verilmesi gerekiyor — bunun yarısı, uygulaması kendi tablolarını okuyamayan bir dal üretir. MongoDB ise sınırlanacak bir veritabanı adı yayımlamıyor. Bu ikisinde kart girişin ortak olduğunu söylüyor; sessiz bırakılmış bir eksik değil, söylenmiş bir gerçek.

Bu en çok, dalda çalışan şey siz olmadığınızda önemlidir. Bir dalda "şu düşen testi düzelt" denen bir asistan migration koşabilir, tablo düşürebilir, boşaltabilir; kendine ait bir girişi varsa erişebileceği en kötü şey, kendisine verilmiş kopyadır.

## Bilinmesi gerekenler

- Veritabanı kopyalamak, kaynağın boyutuna göre zaman alır.
- Kaldırmak klasörü siler. Orada commit'lenmemiş bir çalışmanız varsa önce commit'leyin.
- Kaldırma, dalın kendi veritabanı hesabını da düşürür — veritabanının kendisinin silinmesini istemeseniz bile: veriyi tutmak, ona erişebilen bir hesabı tutmak demek değildir.
- Motor hesabı yaratmayı reddederse — genelde `GRANT` yetkisi olmayan bir veritabanı kullanıcısı — worktree yine de oluşturulur ve kart girişin ortak olduğunu söyler. Düzenlenmemiş bir yalıtım hiçbir yerde varmış gibi gösterilmez.
