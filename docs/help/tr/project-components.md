# Bu deponun geri kalanı

`api/` Go, `web/` Next.js ve `worker/` Python içeren bir depo burada **tek bir projedir**: tek kayıt, tek başlatma, tek sertifika, tek hostname kümesi.

Bu kategorideki diğer her araç bunu üçe böler. Onların birimi bir *site*'tır — tek dizin, tek çalışma zamanı, tek hostname — dolayısıyla bir monorepo, ilişkili olduklarını sizin hatırlamanız gereken üç kayda dönüşür. Yerel bir ikili başka türlü yapamaz: bir dizinin tek çalışma zamanı vardır, çünkü ona hizmet eden ikilinin bir tanesi vardır.

## Nasıl tanımlanır

```json
{
  "name": "shop",
  "runtime": "php",
  "domain": "shop.loc",
  "components": {
    "api": {
      "runtime": "go",
      "path": "api",
      "domain": "api.shop.loc",
      "build": "go build -o bin/api ./cmd/api",
      "start": "./bin/api",
      "port": 8080
    },
    "worker": {
      "runtime": "python",
      "path": "worker",
      "start": "python worker.py"
    }
  }
}
```

`runtime`, `node` ya da altı dil çalışma zamanından biri. `path`, derlendiği dizin. `start`, konteynerinin çalıştırdığı komut. Geri kalan her şeyin bir varsayılanı var.

**`domain`'i olmayan bir bileşen yanlış yapılandırılmış değildir.** Projenin diğer konteynerlerinden erişilir, dışarıdan hiçbir yerden — bir kuyruk işçisinin istediği tam olarak budur, ve ona bir hostname dayatmak kimsenin istemediği bir URL uydurmak olurdu.

## Üç tanım, üç farklı şey

| `stackvo.json` içinde | Nedir | Paylaşımlı mı? | Burada mı derlenir? | Yönlendirilir mi? |
| --- | --- | --- | --- | --- |
| `services: ["mysql"]` | Bir **ihtiyaç**, katalogdan karşılanır | Makine başına bir tane | Hayır | Hayır |
| `sidecars` | **Başkasının imgesi** | Hayır | Hayır | Hayır |
| `components` | **Bu deponun kendi kodu** | Hayır | Evet | Evet |

## Sizin için ne yapar

- Her bileşenin dizinine bir **Dockerfile** ve bir `.dockerignore` — tek çalışma zamanlı bir projenin kullandığı üreteçlerin aynısından.
- Projenin kendi bloğunda bir **compose servisi**, projenin profilini paylaşarak — böylece `stackvo up shop` hepsini kaldırır, projeyi durdurmak hepsini durdurur.
- Alan adı olan her bileşen için bir **Traefik router'ı**, artı bir `/etc/hosts` satırı ve sertifikada bir ad. Ayrıca çalıştırılacak hiçbir şey yok.

## Neler reddedilir, ve niçin

- **Host portu yok.** Bir bileşene projenin konteynerlerinden ve hostname'i üzerinden tarayıcıdan erişilir. Makinenizde asla port bağlamaz — bir deponun iki kopyasının 8080 için çekişmesini engelleyen şey budur. Bir `ports` anahtarı, manifestte, adıyla ve bu gerekçeyle reddedilir.
- **Yol projenin içinde kalır.** `..`, mutlak yol ve `.`'nın kendisi reddedilir. Bir build context, Docker'ın altındaki **her şeyi** okuduğu şeydir.
- **PHP bir bileşen çalışma zamanı değildir.** Bir PHP parçası bir web sunucusu, bir document root ve bir `php.ini` katmanı ister — üçü de projenin kendi `runtime`'ının zaten ürettiği, ve hiçbiri tek bir projenin içinde birden fazlasına genellenmeyen şeyler. PHP yarısını projenin çalışma zamanı olarak bırakın, diğer dilleri burada tanımlayın.
- **Bir hostname, bir konteyner.** Aynı alan adında iki bileşen, en son okunana sessizce yenilen bir yönlendirme kuralıdır; bu yüzden ikincisi reddedilir ve birincisi kalır.

## Bilinmesi gerekenler

- Konteyner adı `stackvo-<proje>-<id>` — türetilir, asla beyan edilmez; böylece bir deponun iki kopyası tek çakışma değil iki konteyner olur. Ad alanını sidecar'larla paylaşır, dolayısıyla ikisinde de kullanılan bir id manifestte bildirilir.
- Bileşenler birbirleriyle konuşmak için diğer konteynerlerin adlarını kullanır: `localhost` değil, `stackvo-shop-api:8080`.
- Bozuk bir bileşen, ayrışanların yanında bir uyarıdır. Dokuz parçası çalışan bir proje, onuncusu yüzünden açılamaz hâle gelmemeli.
