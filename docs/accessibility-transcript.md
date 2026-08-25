# What a screen reader announces

**Generated — do not edit. `npm run a11y:transcript`.**

Every page below is mounted, and its headings and controls are listed in the
order the markup puts them, under the name a screen reader announces. This is
what Y-1 needs a person for, and it is the only part of Y-1 a person is needed
for: the mechanical floor is held by `tests/accessible-names.spec.js` and
`tests/reading-order.spec.js`, which fail the build.

## How to review this

Read down each page and mark any line where:

* **the name does not say what the control does.** "Aç" on its own says nothing
  about what is being opened; "Bu kart ne işe yarar: CPU" does.
* **two lines say the same thing** and a listener could not choose between them.
  Some repetition is fine — a verb per table row, where the row names itself —
  and the rest is not.
* **the order is wrong.** A screen reader reads down this list. If a control
  that decides the meaning of the ones above it appears below them, that is the
  defect this transcript was written to make visible; one of exactly that shape
  was found in the new project drawer.
* **the Turkish is not what a Turkish speaker would say.** The strings were
  written in English and translated. Nothing in this repository compares
  meanings — only keys — so this is the pass no test can stand in for.

A page is mounted with no data behind it, so rows, projects and messages are
absent. What is listed is the frame: what somebody hears before anything loads,
which is also what they hear if nothing ever does.

## Known, and not this application's to fix

A search field's clear button announces as **"temizle" / "Clear"** with nothing
after it. Vuetify builds that name from the field's `label` prop and from
nothing else — not its `aria-label`, not its placeholder — and these fields
carry a placeholder instead of a label on purpose, because a floating label
above a one-line filter costs a row of height on every one of them. So the
choice is a visual one, and it is recorded here rather than changed quietly:
the field itself is named, its clear button is not.

---

# Türkçe

## Dashboard

13 announced, 13 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Panel |
| 2 | button | Bu kart ne işe yarar: Panel |
| 3 | button | Yenile |
| 4 | button | Bu kart ne işe yarar: Sağlık |
| 5 | button | Bu kart ne işe yarar: Projeler |
| 6 | button | Bu kart ne işe yarar: Servisler |
| 7 | button | Bu kart ne işe yarar: İmajlar |
| 8 | button | Bu kart ne işe yarar: İşlemci Yükü |
| 9 | button | Bu kart ne işe yarar: İşlemci Geçmişi |
| 10 | button | Bu kart ne işe yarar: Bellek |
| 11 | button | Bu kart ne işe yarar: Disk |
| 12 | button | Bu kart ne işe yarar: Disk G/Ç |
| 13 | button | Bu kart ne işe yarar: Ağ Trafiği |

## Projects

8 announced, 7 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Projeler |
| 2 | button | Bu kart ne işe yarar: Projeler |
| 3 | button | Yeni proje |
| 4 | button | Yenile |
| 5 | button | Sahiplenilmemiş kod |
| 6 | button | Proje ara... temizle |
| 7 | button | Süzgeçler |
| 8 | button | Yeni proje |

## Logs

10 announced, 10 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Loglar |
| 2 | button | Bu kart ne işe yarar: Loglar |
| 3 | button | Bu kart ne işe yarar: Bütün projeler |
| 4 | button | temizle |
| 5 | button | Düzenli ifade |
| 6 | button | Seviyeye göre filtrele |
| 7 | button | Görünenleri kopyala |
| 8 | button | Çıktıyı takip etme |
| 9 | button | Duraklat |
| 10 | button | Görünümü temizle |

## Dumps

10 announced, 10 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Dump’lar |
| 2 | button | Bu kart ne işe yarar: Dump’lar |
| 3 | button | Bu kart ne işe yarar: Tüm projeler |
| 4 | button | temizle |
| 5 | button | Düzenli ifade |
| 6 | button | Kaynağa göre süz |
| 7 | button | Görünenleri kopyala |
| 8 | button | Duraklat |
| 9 | button | Dump listesini temizle |
| 10 | button | Bu bölüm hakkında |

## Mail

3 announced, 3 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Mail |
| 2 | button | Bu kart ne işe yarar: Mail |
| 3 | button | Bu kart ne işe yarar: Gelen kutusu |

## About

5 announced, 5 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | StackVo |
| 2 | button | Belgeler |
| 3 | button | Kaynak kodu |
| 4 | button | Sorun bildir |
| 5 | button | Üçüncü taraf lisansları |

---

# English

## Dashboard

13 announced, 13 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Dashboard |
| 2 | button | What this card is for: Dashboard |
| 3 | button | Refresh |
| 4 | button | What this card is for: Health |
| 5 | button | What this card is for: Projects |
| 6 | button | What this card is for: Services |
| 7 | button | What this card is for: Images |
| 8 | button | What this card is for: CPU Load |
| 9 | button | What this card is for: CPU Usage History |
| 10 | button | What this card is for: Memory |
| 11 | button | What this card is for: Storage |
| 12 | button | What this card is for: Disk I/O |
| 13 | button | What this card is for: Network Traffic |

## Projects

8 announced, 7 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Projects |
| 2 | button | What this card is for: Projects |
| 3 | button | New project |
| 4 | button | Refresh |
| 5 | button | Unmanaged code |
| 6 | button | Clear Search projects... |
| 7 | button | Filters |
| 8 | button | New project |

## Logs

10 announced, 10 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Logs |
| 2 | button | What this card is for: Logs |
| 3 | button | What this card is for: Every project |
| 4 | button | Clear |
| 5 | button | Regular expression |
| 6 | button | Filter by level |
| 7 | button | Copy what is shown |
| 8 | button | Stop following output |
| 9 | button | Pause |
| 10 | button | Clear the view |

## Dumps

10 announced, 10 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Dumps |
| 2 | button | What this card is for: Dumps |
| 3 | button | What this card is for: All projects |
| 4 | button | Clear |
| 5 | button | Regular expression |
| 6 | button | Filter by source |
| 7 | button | Copy what is shown |
| 8 | button | Pause |
| 9 | button | Clear the dump list |
| 10 | button | About this pane |

## Mail

3 announced, 3 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | Mail |
| 2 | button | What this card is for: Mail |
| 3 | button | What this card is for: Inbox |

## About

5 announced, 5 distinct.

| # | Role | Announced as |
| --- | --- | --- |
| 1 | heading 1 | StackVo |
| 2 | button | Documentation |
| 3 | button | Source code |
| 4 | button | Report an issue |
| 5 | button | Third-party licences |
