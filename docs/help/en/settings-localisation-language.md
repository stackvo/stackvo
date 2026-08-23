# Language

The language of the interface and the tray menu.

## Controls

| Control | What it does |
| --- | --- |
| Language | Picks the interface language. Applies immediately. |
| Language tag | A tag like `de`, `fr` or `pt-BR` to start a new language pack. |
| Start a translation | Creates a file you can translate for that tag. |
| Remove | Deletes a language pack. |

## Language packs

English and Turkish are built into the app. Other languages are JSON files in the app's configuration directory; the card shows the path of each one.

**Start a translation** writes a file holding every string, with the English text in place, and you replace it line by line. The percentage counts the strings that are no longer the English one — so a brand new pack is at 0% and reaches 100% as you go. Anything you have not reached yet shows in English; a partial translation does not break the interface.

### Saying which way your language reads

Near the top of the file:

```json
"language": { "label": "العربية", "direction": "rtl" }
```

`label` is what the picker calls your language. `direction` is `ltr` or `rtl`, and a pack that says `rtl` lays the whole window out right to left when it is selected — including the dialogs and menus — without touching the switch on the card below. That switch is a preference and still decides for every language that has not stated a direction.

## Worth knowing

- Changing the language relabels the tray menu too.
- The console panels have their own language setting on the card below.
- Words that are the same in your language as in English count as untranslated. The percentage understates slightly, and that is the safe direction.
- A file that does not parse is listed with the error rather than disappearing from the picker.
