# Konteyner

Docker'ın bu projenin konteyneri hakkında bildirdikleri. Buradaki hiçbir şey ayar değildir; her alan motordan okunur.

## Alanlar

| Alan | Anlamı |
| --- | --- |
| Ad | Konteynerin adı, `stackvo-<proje>`. `docker exec` ve `docker logs` bunu ister. |
| Çalışma süresi | Mevcut koşunun süresi. Yeniden başlatmada sıfırlanır. |
| Yeniden başlatma politikası | İçerideki süreç sonlandığında motorun ne yapacağı. |
| DNS kaydı | Alan adının bu makinede çözülüp çözülmediği. Kayıt yoksa tarayıcı projeye ulaşamaz. |
| Durum | Docker'ın ifadesi: çalışıyor, çıktı, yaratıldı. |
| Yaratıldı | Konteynerin ne zaman yapıldığı. Son yeniden derleme zamanıdır, son başlatma zamanı değil. |
| Konteyner kimliği | Kısa hash. |
| İmaj | Konteynerin yapıldığı imaj. |
| Yeniden başlatma sayısı | Motorun kaç kez yeniden başlattığı. |
| İmaj boyutu | İmajın diskte kapladığı yer. |
| Ağ geçidi | Konteynerin yığın ağındaki adresi. |
| Port eşlemeleri | Ana makineye yayımlanan portlar. |

## Kontroller

| Kontrol | Ne yapar |
| --- | --- |
| Kopyala | Ad, kimlik ya da imaj değerini panoya alır. |

## Bilinmesi gerekenler

- Proje derlenene kadar kart boştur. Derlemek için üstteki çubuğu kullanın.
- Çoğu proje hiçbir port yayımlamaz. Yönlendirici onlara ağ üzerinden adıyla ulaşır; bu yüzden port eşlemesi olmayan bir proje de tarayıcıda yanıt verir.
- Yeniden başlatma sayısının kendiliğinden artması bir çökme döngüsüdür. Nedenini Loglar sekmesinde arayın.
