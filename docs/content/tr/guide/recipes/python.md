# Python

`https://api.loc` adresinde bir Django, Flask ya da FastAPI projesi;
paketler konteynerde kurulu, konuşacağı bir PostgreSQL var.

## 1. Oluşturun

**Projeler → +**, sonra şablonlar altından **Django**, **Flask** ya da
**FastAPI**; ya da **Python** çalışma zamanı ve bir sürümle (2.7'den
3.14'e) **Boş proje**. Adı `api` olsun.

Boş projede form komutları sorar:

| Alan | Tipik değer |
| --- | --- |
| Kurulum komutu | `pip install -r requirements.txt` |
| Derleme komutu | boş ya da `python manage.py collectstatic --noinput` |
| Başlatma komutu | `gunicorn app:app --bind 0.0.0.0:8000`, `uvicorn main:app --host 0.0.0.0 --port 8000` ya da `python manage.py runserver 0.0.0.0:8000` |

Proxy konteynerin portuna bakar; `127.0.0.1`'e değil `0.0.0.0`'a bağlanın,
yoksa proxy ulaşamaz.

## 2. Veritabanı

**Katalog → Yayında olanlar** PostgreSQL kurar; **Proje → Bu projenin ihtiyaç
duyduğu servisler** projeye bildirir ve sunucu adını, portu ve parolayı
gösterir. Bunları projenin ayarlarına ya da `.env` dosyasına koyun, sonra
projenin klasöründen migrate edin:

```bash
stackvo python manage.py migrate
```

## 3. Paketler

`stackvo python`, `stackvo exec pip` ve `stackvo shell` konteynerin içinde,
konteynerin Python'uyla çalışır. `requirements.txt` dosyasına eklenen paket
bir sonraki yeniden derlemede kurulur; hızlı bir deneme için `stackvo shell`
içinden kurun, kalıcı olunca dosyaya ekleyin.

## 4. Kod üzerinde çalışmak

Ana makinede bir dosyayı düzenlemek, yeniden derleyene kadar konteynerde
hiçbir şeyi değiştirmez; imaj derleme anında alınmış bir kopya taşır.
Düzenle-yenile döngüsü için çerçevenin kendi yeniden yükleyicisini
(`runserver`, `flask --debug`, `uvicorn --reload`) başlatma komutu yapın ve
kaynağı **Proje → Dev sunucusu** açıkken bağlayın; Node projelerinin
kullandığı anahtarın aynısı.

## Her gün

```bash
stackvo python -V
stackvo exec pip list
stackvo python manage.py createsuperuser
stackvo shell
```
