# Insiders

Insider, StackVo'yu `main`'den çalıştıran kişidir: bir sonraki sürümün
kesileceği ağaç, değişiklik günlüğünün *Unreleased* altında listelediği her
düzeltmeyle ve beklemeden. Ayrı bir derleme, sponsor kapısı, açılacak bir
kilit yok; depo herkese açık ve bir insider ile diğerleri arasındaki fark bir
`git clone`.

<div class="grid cards" markdown>

- :material-source-branch: **[Başlarken](getting-started.md)**

    ---

    Bir klondan uygulamanın makinenizde çalışmasına: araçlar, ilk
    derleme, CI'ın zaten soracağı kontroller.

- :material-new-box: **[Yenilikler](whats-new.md)**

    ---

    Her sürüm bir sayfada: uygulamayı kullanan kişi için ne değişti,
    değişiklik günlüğündeki gerekçeler olmadan.

- :material-history: **[Değişiklik günlüğü](changelog.md)**

    ---

    Her değişiklik, nedeniyle. Depodaki dosyanın kendisi; önce
    *Unreleased*.

- :material-update: **[Nasıl güncellenir](upgrade.md)**

    ---

    Uygulamadan, elle, ve bir güncellemenin neye dokunup neye dokunmadığı.

- :material-heart-outline: **[Sponsor olma](sponsoring.md)**

    ---

    Proje neyle yürür, sponsorluk neyi karşılar, neyi satın almaz.

</div>

## `main`'den çalıştırmak ne demek

- **Sürümün önündesiniz.** Değişiklik günlüğünün *Unreleased* altında
  listelediği bir düzeltme, başkasının makinesine gelmeden sizinkinde olur.
  Bir gerileme de öyle; push öncesi kapı betiği ve üç işletim sistemli CI
  bunun için var.
- **Çalışma alanınız aynı.** Geliştirme derlemesi de kurulu kopya gibi
  `~/.stackvo` dizinini okur ve yazar. Girişte ve çıkışta hiçbir şey
  taşınmaz; bir dosyanın yazılış biçimini değiştiren sürüm bunu değişiklik
  günlüğünde söyler.
- **`main`'den gelen bildirim en yararlı türdür.** Sürüm değil commit
  adlandırır ve sürümden önce gelir.
  [Hata bildirimi](../community/contributing/reporting-a-bug.md) formu uyar;
  commit'i yazın.
