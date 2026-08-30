# Dış uygulamalar

Terminal, editör ve tarayıcı hangi uygulamada açılsın.

## Kontroller

| Kontrol                     | Ne yapar                                                               |
| --------------------------- | ---------------------------------------------------------------------- |
| Terminal                    | "Terminalde aç" butonlarının kullanacağı uygulama.                     |
| Editör                      | "Editörde aç" butonlarının kullanacağı uygulama.                       |
| Tarayıcı                    | Adreslerin açılacağı tarayıcı.                                         |
| Veritabanı istemcisi komutu | Bir servisteki "istemcide aç" menüsünün "Other…" satırı bunu kullanır. |

## Bilinmesi gerekenler

- Listede yalnızca bu makinede kurulu olanlar görünür. Kurulu olmayan bir uygulama pasif gösterilir.
- Her liste **Other…** ile bitiyor; listede olmayan bir uygulama için. Seçildiğinde onu başlatan komutun kutusu açılır ve çalışan tek şey o kutudur — tespit varsayılan kalır, yerine geçilmez.
- Komut bir başlatıcı ve bayraklarıdır, fazlası değil. Açılacak şey — klasör, adres, bağlantı dizgesi — son argüman olarak eklenir.
- **Burası bir kabuk değildir.** `$HOME`, `&&`, boru ve yönlendirme düz metindir. Boşluk içeren yolu tırnak içine alın; ters bölü ters bölüdür, yani `"C:\Program Files\Sublime Text\subl.exe"` yazıldığı gibi çalışır.
- Terminalin çalıştıracağı komut için kendi bayrağı gerekir, çünkü her terminal onu başka türlü alır. Bayrağı da kutuya yazın: `alacritty -e sh -c`, `wezterm start --`, `wt.exe cmd.exe /K`.
- **Other…** sizin yerinize seçilmez. Yazdığınız uygulama yoksa ya da kutu boşsa, butonlar tespit edilmiş bir uygulamaya döner — seçili bir uygulama kaldırıldığında yaptığının aynısı.
- Bu ayarlar yalnızca uygulamanın açtığı şeyleri etkiler; işletim sisteminizin varsayılanlarını değiştirmez.
