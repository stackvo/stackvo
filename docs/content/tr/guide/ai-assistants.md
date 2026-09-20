# Yapay zekâ asistanları

`stackvo-mcp`, pencerenin sürdüğü çekirdeğin üstünde bir MCP sunucusudur. Bir
asistan *"shop.loc neden yüklenmiyor?"* sorusuna ön kontrol raporundan, hosts
dosyasından, sertifikadan ve konteynerin son yüz log satırından cevap
verebilir — pencere açık olmadan.

<figure markdown>
![Bir projenin yapay zekâ asistanı kartı](../screenshots/web/project-detail-agent.webp){ loading=lazy }
<figcaption>Asistana bu proje hakkında ne söylendiği ve ne yapabileceği.</figcaption>
</figure>

## Kaydetmek

**Ayarlar → Yapay zekâ asistanları** bu makinede bulunan sekiz istemciyi
listeler — Claude Code, Claude Desktop, Cursor, Windsurf, VS Code, Gemini CLI,
Codex, Zed — ve sunucuyu tek tıkla kaydeder. Her istemcinin kendi ayar dosyası
okunur, tek bir `stackvo` girdisi eklenir, dosya içindeki diğer her sunucu
korunarak geri yazılır. Önce bir `.stackvo-backup` kopyası alınır.

## Tasma: varsayılan yalnızca okuma

Erişimi anahtarlarla verirsiniz; panel seçiminizin anlamını cümle olarak geri
yazar — *"bu asistan shop'u yeniden başlatabilir, önümüzdeki yarım saat"*.

| Ayar | Etki |
| --- | --- |
| *(varsayılan)* | Yalnızca okur. 38 aracın 26'sı. |
| **Yazmaya izin ver** | 12 değiştiren aracı ekler — başlat, durdur, yeniden başlat, sertifikayı yeniden üret, snapshot al ve her şeyi durduran `stack_down`. |
| **Projeye bağlı** | Yalnızca o projenin dört yazma aracı sunulur; hiçbir projenin sınırlayamadığı sekiz araç hiç sunulmaz. |
| **Süre sınırı** | Yazma yarısı belirlediğiniz süre sonunda kendiliğinden biter. |
| **Araç araç** | Yalnızca adını verdiğiniz araçlar. |

Yazmaya izin vermeden önce listeyi okuyun: bütün yığını durdurmak ve her
projenin bağımlı olduğu ortak bir servisi durdurmak da içinde.

## Asla yapmadıkları

- **Hiçbir araç parola döndürmez.** Bir test, bu yüzeydeki hiçbir şemada
  `password`, `secret` ya da `token` özelliği olmadığını doğrular.
- **Snapshot geri yüklemek bir araç değildir.** Almak öyledir — bir dosya
  ekler. Canlı satırların üstüne yazmak uygulamanın kendi onayına aittir.
- **Her yazma çağrısı kaydedilir**, redler dâhil, **Ayarlar → Denetim kaydı**
  altında — çoğu girdi işi geri alacak olanı taşır, tek tıkla geri alınabilir.

## Asistana ne zaman kullanacağını söylemek

**Ayarlar → Yapay zekâ kuralları** asistanın zaten okuduğu talimat dosyasına
kısa bir bölüm yazar — `CLAUDE.md`, `AGENTS.md`, `.cursor/rules/`,
`.github/instructions/`, `.windsurf/rules/`, `GEMINI.md`. Yalnızca StackVo'nun
kendi işaretleri arasındaki bölge yazılır; dosyanın gerisi bayt bayt geri
gelir.

## İş için bir sandbox

Asistana makinenizi değil, [süreli bir worktree](branches.md) verin: kendi
dalı, kendi adresi, kendi veritabanı; süresi dolunca gider.
