# Katkı

StackVo ücretsizdir, MIT lisanslıdır ve tek kişi tarafından geliştirilir.
Yeniden üretilebilen her hata bildirimi, netleştirilen her cümle ve testiyle
gelen her pull request projeye geri kazandırılmış zamandır. Bu bölüm, her
birinin ilk seferde yerine oturması için nasıl yapılacağını anlatır.

## Nasıl katkı verebilirsiniz

### Issue açmak

<div class="grid cards" markdown>

- :material-bug-outline: **[Hata bildirme](reporting-a-bug.md)**

    ---

    Bir şey olması gerekenden farklı davranıyor. Önce neye bakmalı, bir
    bildirimin yeniden üretilebilmesi için nelere ihtiyacı var.

- :material-file-document-edit-outline: **[Doküman hatası bildirme](reporting-a-docs-issue.md)**

    ---

    Bu sitedeki bir sayfa, uygulamadaki bir yardım kartı ya da README
    yanlış, belirsiz veya eksik. Düzeltmelerin çoğu bir kalem tıklaması
    uzağında.

- :material-shield-alert-outline: **[Güvenlik açığı bildirme](reporting-a-vulnerability.md)**

    ---

    Özel olarak, asla issue olarak değil. Kapsam ne, 72 saat içinde ne
    bekleyebilirsiniz.

- :material-lightbulb-outline: **[Değişiklik isteme](requesting-a-change.md)**

    ---

    Yapması gerekip yapmadığı bir şey. Düğmeyi değil durumu anlatın; form
    tam olarak bunu sorar.

- :material-forum-outline: **[Soru sorma](https://github.com/stackvo/stackvo/discussions)**

    ---

    "Nasıl yaparım" ve "böyle mi olmalı" soruları Discussions'a aittir;
    cevap bir sonraki kişiye de yarar.

</div>

### Katkı vermek

<div class="grid cards" markdown>

- :material-translate: **[Çeviri ekleme](adding-translations.md)**

    ---

    Uygulama, yardım kartları ve bu site İngilizce ve Türkçe. İkisinden
    birini iyileştirin ya da üçüncüsünü başlatın.

- :material-source-pull: **[Pull request açma](making-a-pull-request.md)**

    ---

    Fork, dal, kapı betiği, taslak PR. Testlerin uyguladığı ev kuralları;
    CI'dan önce siz karşılayın.

</div>

### Rehberler

<div class="grid cards" markdown>

- :material-test-tube: **[Yeniden üretim hazırlama](../guides/creating-a-reproduction.md)**

    ---

    Temiz bir çalışma alanı, tek proje ve hatayı hâlâ gösteren en az ayar:
    "yeniden üretilemedi"yi düzeltmeye çeviren şey.

</div>

## Bir şey açmadan önce

Kısa bir kontrol bir gidiş dönüşü, bazen de issue'nun tamamını kurtarır:

- :material-checkbox-blank-circle-outline: **Son sürümdesiniz.** Yalnızca son sürüm desteklenir; geriye dönük
      düzeltme dalı yoktur. **Ayarlar → Güncellemeler** sürümü gösterir ve
      yenisini kurar.
- :material-checkbox-blank-circle-outline: **Doktor bakmış.** **Ayarlar → Doktor** makinede ters giden çoğu şeyi
      yanında onarımıyla adlandırır.
- :material-checkbox-blank-circle-outline: **Daha önce bildirilmemiş.** Kapalı olanlar dahil
      [issue'ları](https://github.com/stackvo/stackvo/issues) ve
      [tartışmaları](https://github.com/stackvo/stackvo/discussions) arayın.
- :material-checkbox-blank-circle-outline: **Tek bir şey.** Issue başına bir hata, bir istek, bir soru. Ayrı
      şeyler ayrı ayrı değerlendirilebilir.
- :material-checkbox-blank-circle-outline: **Güvenlik sorunu değil.** Öyleyse [buraya](reporting-a-vulnerability.md)
      gider, herkese açık hiçbir yere değil.

Projenin neyi vaat edip edemeyeceği (ücretli katman yok, destek sözleşmesi yok, tek geliştirici) [Sponsor olma](../../insiders/sponsoring.md) sayfasında.

## Haklar ve sorumluluklar

Projenin bir
[davranış kuralları belgesi](https://github.com/stackvo/stackvo/blob/main/CODE_OF_CONDUCT.md)
var. Kısa hâli: insanlara düzgün davranın, işi hak ettiği kadar sert
eleştirin, kişiyi asla.

Issue'ları tek bir geliştirici okur. Bunun anlamı:

- Şablonu atlayan, yeniden üretilemeyen ya da başka birinin kopyası olan bir
  issue cevap yerine bir yönlendirmeyle kapatılabilir. Bu size dair bir
  yargı değil, tek kişinin taşıyabileceği kadarıdır.
- Otomatik bir araçtan çıkan bildirim, bir insan uygulamada doğrulamadıkça
  kapatılır.
- Depoya yazılan her şey herkese açık kalır. Bir yıl sonra bulacak okur için
  yazın.
