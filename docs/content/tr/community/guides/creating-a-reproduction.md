# Yeniden üretim hazırlama

Yeniden üretim, hatayı hâlâ gösteren en küçük kurulumdur: temiz bir çalışma
alanı, tek proje, önemi olan en az ayar ve oraya götüren adımlar. "Yeniden
üretilemedi" ile düzeltme arasındaki farktır; çünkü StackVo'da ters giden
şeylerin çoğu belirli bir makinedeki belirli bir manifesttir ve ikisi de
olmayan bir bildirim geliştiriciyi ikisini de tahmin etmeye bırakır.

## Rehber

### Ortam

StackVo her şeyi tek bir çalışma alanında tutar (varsayılan `~/.stackvo`) ve
`STACKVO_ROOT` değişkeni onu taşır. İkinci bir çalışma alanının maliyeti
yoktur ve gerçek projelerinizi resmin dışında tutar. Değişkeni ayarlayın ve
uygulamayı o kabuktan başlatın:

=== "macOS ve Linux"

    ```bash
    export STACKVO_ROOT=~/stackvo-repro
    ```

=== "Windows"

    ```powershell
    $env:STACKVO_ROOT = "$HOME\stackvo-repro"
    ```

Uygulama boş klasörü ilk çalıştırma sayar (tek soru, sertifika, yığın) ve
her şeyi oraya yazar. Sonrasında klasörü silin; geriye hiçbir şey kalmaz.
(**Ayarlar → Çalışma alanı** zaten var olan çalışma alanları arasında geçiş
yapar; geri dönmek içindir, yenisini oluşturmak için değil.)

!!! note "Aynı anda tek yığın"

    Compose projesi her makinede `stackvo` adını taşır; dolayısıyla geçici
    çalışma alanı ile gerçek olanınız aynı anda ayakta olamaz. Önce
    sizinkini indirin (`stackvo down` ya da tepsi), sonra geri kaldırın.

### En küçük yeniden üretim

1. **Önce güncelleyin.** Yalnızca son sürüm desteklenir;
   **Ayarlar → Güncellemeler** kurar. Sonrasında kaybolan hata zaten
   düzeltilmişti.

2. **Şablondan tek proje.** **Yeni proje**, en yakın şablon, varsayılan
   ayarlar. Hata şimdiden görünüyorsa burada durun; yeniden üretim budur.

3. **Hata görünene kadar her seferinde tek ayar ekleyin:** PHP sürümü, bir
   servis, bir alan adı, bir hook, `.env` satırı. En son eklediğiniz şey,
   bildirimde adlandırılacak şeydir.

4. **Önemi olmayanı çıkarın.** Ayarları ve servisleri teker teker kaldırın,
   her birinden sonra hatanın hâlâ orada olduğunu doğrulayın. Geriye kalan
   yeniden üretimdir; gerisi gürültüydü.

Sonuç genellikle bir düzine satırlık bir `stackvo.json` olur. Olduğu gibi
bildirime yapıştırın: manifest budur ve manifest, geliştiricinin ihtiyaç
duyduğu şeyin çoğudur.

### Adımlar

Numaralı, temiz çalışma alanından başlayıp yanlış olan şeyde biten:

```text
1. STACKVO_ROOT=~/stackvo-repro, ilk çalıştırma, varsayılanlar
2. Yeni proje, Laravel şablonu, ad "repro"
3. Servisler → Redis'i aç
4. Projeyi yeniden başlat
5. https://repro.loc adresini aç — uygulama bekleniyordu, 502 geldi
```

"Bazen" bir adım değildir. Yalnızca bazen oluyorsa ne sıklıkta olduğunu ve
olmadığı seferlerde neyin farklı olduğunu söyleyin.

### Neler eklenmeli

| Ekleyin | Nereden gelir |
| --- | --- |
| Manifest | Proje klasöründeki `stackvo.json` |
| Değiştirilen `.env` satırları | Çalışma alanının `.env` dosyası; **Ayarlar → Çalışma dizini → Klasörü aç** gösterir. Yalnızca değiştirdiğiniz satırlar, sır olmayan değerler |
| Tanı paketi | **Ayarlar → Uygulama günlüğü → Tanılama paketi kaydet**; log, ön kontroller, Doktor raporu ve çökme raporları tek arşivde, sırlar yazılırken maskelenir |
| Doktor'ın sözü | `stackvo doctor` ya da **Ayarlar → Doktor**; işaretlediği satırları yapıştırın |
| Sürüm ve platform | **Ayarlar → Güncellemeler**; işletim sistemi; Docker çalışma zamanı ve sürümü |

Ekran görüntüsü ne gördüğünüzü, paket nedenini gösterir. İkisi de varsa
ikisini de ekleyin; seçmek zorundaysanız paketi.

### Neler dışarıda kalmalı

- Gerçek projeleriniz. İkinci çalışma alanı bunun için var.
- İçinde parola ya da token olan her şey. Paket onları maskeler; yapıştırılan
  bir `.env` maskelemez.
- İkinci bir hata. Kendi issue'sunu ve kendi yeniden üretimini alır.

## Sonra

Manifest, adımlar ve paketle
[hata bildirimini](https://github.com/stackvo/stackvo/issues/new/choose)
açın ve geçici çalışma alanını silin. Formun gerisi
[Hata bildirme](../contributing/reporting-a-bug.md) sayfasında.
