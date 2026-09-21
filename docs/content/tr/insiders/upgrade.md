# Nasıl güncellenir

Yalnızca son sürüm desteklenir; dolayısıyla güncellemek çoğu hata
bildiriminin ilk cevabıdır. Aynı yere varan iki yol.

## Uygulamadan

**Ayarlar → Güncellemeler** yeni sürüm olup olmadığını kontrol eder ve kurar.
Güncelleme, uygulamaya derlenmiş bir minisign anahtarıyla doğrulanır;
doğrulamayı geçemeyen paket kurulmaz. Kurulum uygulamayı kapatır.
Konteynerleriniz çalışmaya devam eder; onları pencere değil Docker yönetir.

**Beta sürümleri de al** seçeneği, kontrolün sunduklarına ön sürümleri
ekler. Beta kurulumu yine her kararlı sürümü alır; kararlı kuruluma asla beta
önerilmez. Anahtar bir sonraki açılışta etkili olur.

## Elle

Platformunuzun kurulum paketini
[sürümler sayfasından](https://github.com/stackvo/stackvo/releases/latest)
indirin ve var olanın üzerine kurun. Her sürüm dosyalarının yanında
`SHA256SUMS` yayımlar. Platforma göre adımlar ve imzasız derlemede her
işletim sisteminin istediği ek tıklama
[Kurulum](../getting-started/installation.md) sayfasında.

## Güncelleme neye dokunur

| | |
| --- | --- |
| Uygulama | Değiştirilir |
| Çalışma alanınız: projeler, manifestler, `.env` | Dokunulmaz |
| Çalışan konteynerler | Dokunulmaz; onları uygulama değil Docker yönetir |
| Yardım kartları | Sürümün parçası değil. Uygulama onları `main`'den çeker, zaten günceldirler |

## Geri dönmek

[Sürümler sayfasından](https://github.com/stackvo/stackvo/releases) daha eski
bir sürümü şimdikinin üzerine kurun. Önce değişiklik günlüğünde iki sürüm
arasındaki girdileri okuyun: bir dosyanın yazılış biçimini değiştiren sürüm
bunu *Changed* altında söyler.

## Değişiklik günlüğünü okumak

[Değişiklik günlüğü](changelog.md) depodaki tek bir dosyadır ve
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) biçimindedir:
`main`'de olup henüz hiçbir sürümde olmayanlar için bir **Unreleased**
bölümü, sonra her sürüm için *Added*, *Changed*, *Fixed* ve *Removed* ile bir
bölüm. Girdiler neyin yanında nedeni de söyler, bu yüzden uzundur; kısa hâli
[Yenilikler](whats-new.md) sayfasıdır.
