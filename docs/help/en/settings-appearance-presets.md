# Presets

Name a look, save it, come back to it in one click — and move it somewhere else.

## Controls

| Control | What it does |
| --- | --- |
| Preset name | The name to save under. |
| Save | Stores all the current appearance settings under that name. |
| Clicking a preset | Applies that look. |
| Delete | Removes the preset. |
| Copy as settings | Puts the current look on the clipboard as JSON, for another copy of this app. |
| Copy as a Vuetify theme | Puts it on the clipboard as a `createVuetify` call carrying both themes, for a project that is not this app. |
| A look, as JSON | Paste settings copied from elsewhere, then Import. |

## Worth knowing

- A preset holds the theme, colour, palette, typeface, scale, density and corner radius together.
- Presets live on this machine and do not travel with the workspace. Copying a look as settings is how one gets to another machine.
- An import is checked before it is applied. A field this build does not recognise, or a value outside what the controls offer, is skipped and named in the message — the rest of the look still comes across.
- A paste that is not a look at all is refused outright rather than partly applied, so a mistyped paste cannot reset the look you were adding to.
- The Vuetify theme snippet is generated, not stored. It carries the light and dark palettes exactly as this app renders them, including the secondary colour derived from your accent.
