# SSS

En sık gelen sorular, bir kez cevaplanmış. Sizinki burada yoksa sormanın yeri [Discussions](https://github.com/stackvo/stackvo/discussions).

## Gerçekten ücretsiz mi?

Evet. Ücretsiz, MIT lisanslı, her platformda her özellik. Ücretli katman,
hesap ve telemetri yok; hiçbir şey birinin arkasına kilitlenmeyecek.
[Sponsorluk](../insiders/sponsoring.md) bakımı sürdüren şeydir; herkesin
almadığı hiçbir şey satın almaz.

## Docker şart mı?

Evet. Docker Desktop, Docker Engine ya da API uyumlu bir çalışma zamanı —
Podman, Colima, OrbStack. Motorun adı yalnızca bir etikettir; hiçbir şey
hangisinin cevap verdiğine göre dallanmaz.

## Yalnızca Laravel için mi?

Hayır. Laravel, Symfony ve WordPress şablon alır; proje, çalışma zamanı olan
herhangi bir klasördür — PHP 5.6'dan 8.5'e, Node, Python, Go, Ruby, Rust — ve
bunları karıştıran bir monorepo tek projedir.

## Neden kod imzalı değil?

Uygulama yalnızca GitHub Releases üzerinden dağıtılır. Apple Developer üyeliği
ve Authenticode sertifikası bir kimliğe bağlı, yinelenen maliyetlerdir;
zincirden çıkarılan son dış bağımlılık olarak bilerek atlandı. Karşılığında
her sürüm `SHA256SUMS` yayımlar ve güncelleyici minisign imzasını doğrular.

## Mevcut StackVo (Bash / web arayüzü) kurulumum ne olacak?

Çalışmaya devam eder. İkisi de aynı `stackvo.json` ve `.env` dosyalarını okur;
birinde oluşturulan proje diğerinde çalışır. Bu uyumluluk gelenekle değil,
depoya işlenmiş bir sözleşme ve doğrulayıcıyla korunur.

## Aynı servisin iki sürümünü aynı anda çalıştırabilir miyim?

Evet. Servisler örnek olarak kurulur; MySQL 8.0 ve 8.4 yan yana çalışır, her
proje istediğine bağlanır.

## Windows'ta durum ne?

Saf mantık — sürücü harfinden bind mount'a dönüşüm, adlandırılmış boru algılama,
`DOCKER_HOST` şema ayıklama — her platformda test edilir ve Windows CI
matrisindedir. Derleyicinin cevaplayamadığı kısım hâlâ doğrulanmadı: UAC
üzerinden hosts yazımı, gerçek Docker Desktop'a karşı adlandırılmış boru ve
tarayıcıda alan adı çözümü.

## Verilerim nereye gider?

Hiçbir yere. Bkz. [Makineden ne çıkar](privacy.md).

## Sunucuda ya da CI'da kullanabilir miyim?

Tasarım hedefi değil. CLI başsız kullanımı teknik olarak mümkün kılar, ama
uçtan uca test edilmediği için desteklenir denmiyor.

## Yerel bir araca göre Docker'ın bedeli ne?

Docker ve imajlarını içeren bir ilk kurulum, projenin ilk açılışında bir imaj
kurulumu ve Docker VM'in boştaki belleği. Karşılığında her projenin ortamı bir
konteynerdir: çalışan şey bir Dockerfile'ın söylediğidir. Tam tablo
[Karşılaştırma](comparison.md) sayfasında.
