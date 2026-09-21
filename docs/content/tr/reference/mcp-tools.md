# MCP araçları

`stackvo-mcp`'nin bir asistana sunduğu 38 araç. 26'sı yalnızca okur; 12'si bir
şeyi değiştirir ve yalnızca **Yazmaya izin ver** ile görünür — anahtarlar için
bkz. [Yapay zekâ asistanları](../guide/ai-assistants.md). Her araç yalnızca
okuma ya da yıkıcı diye işaretlidir; istemci görmediği bir aracı kullanmadan
önce sorabilir.

<figure markdown>
![Ayarlarda yapay zekâ asistanları](../screenshots/web/settings-agents.webp){ loading=lazy }
<figcaption>MCP sunucusunun hangi asistanlara kayıtlı olduğu ve yazmalardaki dizgin.</figcaption>
</figure>

## Okumalar

| Araç | Ne cevaplar |
| --- | --- |
| `stackvo_overview` | Bütün yığının durumu: çalışma alanı, motor, ne açık. |
| `stackvo_doctor` | Tam tanı: her gereksinim, her tutulan port, eksik hosts satırları. |
| `stackvo_system` | Bu makinede ne kaldı: CPU, bellek, takas, diskler, ağ. |
| `stackvo_projects` | Her yönetilen proje: alan adı, çalışma zamanı, kurulu, çalışıyor. |
| `stackvo_project` | Bir proje bütünüyle. |
| `stackvo_container_stats` | Bir konteynerin canlı CPU'su, sınırına göre belleği, ağ ve blok G/Ç. |
| `stackvo_services` | Paylaşılan servisler — veritabanları, önbellekler, arama, kuyruklar — ve sağlıkları. |
| `stackvo_service_instances` | Kurulu her servis sürümü, ayrı denetlenebilir örnek olarak. |
| `stackvo_service_connection` | Bir servise nasıl ulaşılır: şema, host, port, veritabanı, kullanıcı. Parola yok. |
| `stackvo_databases` | Dökülebilen veritabanı servisleri ve veritabanları. |
| `stackvo_snapshots` | Bu çalışma alanının tuttuğu snapshot'lar, en yeniden başlayarak. |
| `stackvo_packages` | Servis kataloğu: kayıt defterinin bildiği her paket. |
| `stackvo_logs` | Bir konteyner logunun son satırları. |
| `stackvo_log_files` | Her projenin yazdığı her log dosyası, en yeniden başlayarak. |
| `stackvo_log_read` | Bir projenin log dosyaları ve bir dosyanın içeriği. |
| `stackvo_certificates` | Sertifika: kapsanan alan adları, eksikler, güven. |
| `stackvo_hosts` | Yığının istediği her alan adı ve hosts dosyasının eşleyip eşlemediği. |
| `stackvo_mail` | Mail yakalayıcının gelen kutusu. |
| `stackvo_mail_message` | Yakalanan bir mesaj bütünüyle. |
| `stackvo_ide_debug` | Kesme noktası neden vurmuyor: port, IDE anahtarı, sunucu adı, kim dinliyor. |
| `stackvo_profiler` | Bir projenin örnekleyici profilleyicisi: kurulu, bağlı, açık. |
| `stackvo_hotspots` | Bir kaydın zamanı nerede geçirdiği. |
| `stackvo_flame` | Bir kayıt alev grafiği olarak. |
| `stackvo_explain_request` | Kaydedilen bir istek neden yavaştı; üç araç birleşik. |
| `stackvo_timeline` | Uygulamanın bildirdiği her şey tek eksende: dump'lar, istekler, işler. |
| `stackvo_query_log` | Bir veritabanına gerçekte ne soruldu. |

## Yazmalar

Yalnızca **Yazmaya izin ver** ile. Proje kapsamında yalnızca ilk dördü
sunulur.

| Araç | Ne yapar | Kapsam |
| --- | --- | --- |
| `stackvo_project_start` | Bir projenin konteynerini başlatır. Tekrarlanabilir. | proje |
| `stackvo_project_stop` | Bir projenin konteynerini durdurur. Tekrarlanabilir. | proje |
| `stackvo_project_restart` | Bir projeyi durdurup başlatır. | proje |
| `stackvo_xdebug_set` | Bir projede adım adım hata ayıklamayı açar ya da kapatır. | proje |
| `stackvo_service_start` | Bir servis örneğini başlatır — `redis`, değil `redis-7-2`. | yığın |
| `stackvo_service_stop` | Bir örneği durdurur. Hiçbir şey silinmez. | yığın |
| `stackvo_service_restart` | Bir örneği yeniden başlatır. | yığın |
| `stackvo_snapshot_take` | Bir veritabanını adlı snapshot'a döker. Dosya ekler, bir şey değiştirmez. | yığın |
| `stackvo_certificates_reissue` | Projelerin alan adları için sertifikayı yeniden üretir. | yığın |
| `stackvo_generate` | Üreticiyi yeniden çalıştırır. | yığın |
| `stackvo_stack_up` | Yığını kaldırır, eksik imajları kurar. | yığın |
| `stackvo_stack_down` | Bütün yığını indirir, projeler dâhil. | yığın |

## Bilerek araç olmayanlar

- **Snapshot geri yüklemek.** Canlı satırların üstüne yazmak uygulamanın kendi
  onayına aittir.
- **Parola döndüren herhangi bir şey.** Bir test, bu yüzeydeki hiçbir şemada
  `password`, `secret` ya da `token` özelliği olmadığını doğrular.
- **Proje kurmak.** Kurulum ilerlemesi uygulamanın olay sistemi üzerinden akar;
  stdio sunucusunun bunu taşıyacak yolu yok.
