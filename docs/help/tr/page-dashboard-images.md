# İmajlar

Bu makinedeki Docker imajlarının sayısı ve toplam boyutu.

## Bilinmesi gerekenler

- Sayı bu makinedeki tüm imajları kapsar; yalnızca StackVo'nun derlediklerini değil.
- Bir projeyi her yeniden derlediğinizde yeni bir imaj katmanı oluşur. Eski katmanlar birikir; disk doluyorsa `docker image prune` ile temizleyebilirsiniz.
