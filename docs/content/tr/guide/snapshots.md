# Snapshot ve yedekler

Snapshot, bir veritabanının adlı kopyasıdır. Migration'dan önce bir tane
alın; ters giderse adıyla geri yükleyin.

<figure markdown>
![Zamanlanmış işler ve snapshot'lar](../screenshots/web/project-detail-jobs.webp){ loading=lazy }
<figcaption>Zamanlanmış işler: zamanlayıcıdaki adlandırılmış işler, snapshot'lar da aralarında.</figcaption>
</figure>

## Almak

**Proje → Snapshot’lar** → bir ad verin → **Al**. Bir dosya ekler, başka
hiçbir şeyi değiştirmez.

## Geri yüklemek

Aynı panel. Geri yükleme onay ister, çünkü canlı satırların üstüne yazar. Bu
yüzden geri yükleme [yapay zekâ asistanlarına](ai-assistants.md) bilerek
*sunulmaz*: snapshot almak sunulur, geri yüklemek sunulmaz.

## Zamanlama

**Ayarlar → Tercihler → Otomatik yedekler** işi otomatiğe alır: saatlik, günlük ya da haftalık,
son N tanesi tutularak.

Zamanlama **saatten değil, son snapshot'tan** ölçülür. Üç gün kapalı kalan bir
dizüstü açıldığında üç değil bir snapshot borçludur.

- Yalnızca çalışan veritabanları yedeklenir; durmuş servis atlanır.
- Kendi adlandırdığınız snapshot'lar zamanlama tarafından asla silinmez ve
  sınıra sayılmaz.

## Bir örneği kaldırmadan önce

Servis örneğinde **Kaldır** verisini siler. Önce snapshot alın.
