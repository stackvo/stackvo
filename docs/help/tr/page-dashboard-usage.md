# Bugün neye mal oldu

UTC gece yarısından beri kullanılan CPU ve tutulan bellek — konteyner başına bir satır.

Bu sayfadaki her şey *şu an*: ne çalışıyor, makine bu saniyede ne kadar yüklü. Bu kart ise, konteyner tabanlı bir kurulum hakkında insanların gerçekten sorduğu soruyu cevaplayan tek karttır: bir öğleden sonrada bana neye mal oldu?

## İki sayı

| Sütun | Nedir |
| --- | --- |
| CPU | Tek bir çekirdeğin saniyesi, dakika olarak gösterilir. `time`'ın raporladığı şeyle aynı: burada on dakika, bir çekirdeğin on dakikası — ya da iki çekirdeğin beş dakikası. |
| Tutulan bellek | Gigabayt-saat. Bellek harcanmaz, **işgal edilir**; bu yüzden birimin içinde zaman vardır: bir saat boyunca tutulan bir gigabayt, ya da yarım saat tutulan iki gigabayt. |

## Sayılar nereden geliyor

Uygulama, yazıldığından beri CPU ve belleği dakikada bir okuyor — proje sayfasındaki sparkline için — ve okumaları iki saat sonra atıyordu. Yeni bir ölçüm yapılmıyor; aynı okumalar atılmak yerine toplanıyor.

İki okuma arasındaki süre **ölçülüyor**, zamanlayıcının ayarlandığı altmış saniye olduğu varsayılmıyor. Beş dakikadan uzun bir boşluk hiçbir şey katmıyor — cuma kapatılıp pazartesi açılan bir dizüstü, hafta sonu için cuma günkü hızından faturalandırılmamalı. Okuma yine sayılıyor ve saat yine ilerliyor; yalnızca zaman reddediliyor. Yani bir toplam, uykudan sonra birkaç dakika eksik olabilir; üç gün uzun olamaz.

Otuz gün saklanıyor. Daha eskisi özetlenmiyor, siliniyor — çünkü bir özetin özeti, kimsenin kontrol edemeyeceği bir sayıdır.

## Ortak servisler bölüştürülmez

`shop` ve `blog` aynı MySQL'i kullanır. Onun belleğini ikisi arasında bölmek uydurma bir sayı olurdu, ve kontrol edemediğiniz bir sayı, üzerine karar veremeyeceğiniz bir sayıdır — bu yüzden bir servisin kendi satırı vardır ve ne olduğunu söyler. Yığının kendi konteynerleri, router ve posta yakalayıcı da aynı sebeple listelenir: onları dışarıda bırakmak, Docker'ın bu makinede maliyetini olduğundan az gösterirdi.

## Bütçeler

Bütçeyi yalnız bir proje taşıyabilir, aynı sebeple: ortak bir servis, herhangi bir projenin aşabileceği bir şey değildir. Bütçe proje başına verilir ve bir **makinenin** kararıdır — `stackvo.json`'da değil, bu uygulamanın tercihlerinde durur; çünkü aynı depo bir meslektaşınızın dizüstünde farklı bir alana sahiptir, ve git'e commit edilmiş bir eşik, birinizin diğeriyle pull request üzerinden tartışması olurdu.

Bir proje bütçesini aştığında size **o gün bir kez** söylenir. Örnekleyici dakikada bir çalışır, ve saat ikide bütçeyi aşmış bir proje ikibuçukta hâlâ aşmıştır; okuma başına bir bildirim akşama dört yüz bildirim eder, ve bir saat içinde kapattığınız özellik, size yarın haber verecek olandır. Ertesi günün ilk aşımı yeniden bir bildirimdir.

Sıfır bütçe, bütçe yok demektir. Temizlenmiş bir alan sıfır olarak gelir, ve kutuyu temizlediğiniz anda çalan bir uyarıyı kimse açık bırakmaz.
