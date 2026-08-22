# Hosts dosyası

Bu çalışma alanındaki her alan adı isimle çözülür, yani her biri `/etc/hosts` dosyasında bir satır ister.

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Tümünü düzelt | Eksik satırları ekler, gereksizleri kaldırır. Parolanızı sorar. |

## Kartın söyledikleri

| Durum | Anlamı |
| --- | --- |
| Hepsi çözülüyor | Yapılacak bir şey yok. |
| Eksik | Bu adresler tarayıcıda açılmaz. |
| Elle eklenmiş | Satırı StackVo yazmamış. Dokunulmaz. |
| Artık gerekmeyen | StackVo'nun yazdığı ama bu çalışma alanının artık kullanmadığı satırlar. Aynı buton kaldırır. |

## Bilinmesi gerekenler

- StackVo yalnızca kendi blok işaretleri arasındaki satırları değiştirir. Dosyanın geri kalanına dokunmaz.
- Joker adlar hosts dosyasına yazılamaz. Bir joker alt alan adı gerekiyorsa Yerel DNS kartını kullanın.
- Değişiklik yönetici hakkı ister. Onaylamadan hiçbir şey yazılmaz.
