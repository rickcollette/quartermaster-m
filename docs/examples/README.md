# Screenshot ATASCII source files

These are the editable sources used by `npm run docs:screenshots`.

## Actual ATASCII files

- `quartermaster-command-deck.ata`
- `quartermaster-screen-composer.ata`

Open either `.ata` file directly in QuarterMaster/M with:

- mode: **ATASCII**
- view: **40 COL**

Edit and save it, then run:

```powershell
npm run docs:screenshots
```

The screenshot tool reads the actual `.ata` bytes. It does not contain a hard-coded copy of the screen text.

## Human-editable layouts

- `quartermaster-command-deck.screen.json`
- `quartermaster-screen-composer.screen.json`

Each JSON file describes exactly 24 rows at 40 columns. Ordinary rows supply up to 38 characters because the generator adds Atari vertical-border glyphs. Supported row kinds:

```json
{ "kind": "top" }
{ "kind": "separator" }
{ "kind": "bottom" }
{ "text": " YOUR TEXT", "inverse": false }
{ "text": " INVERSE ROW", "inverse": true }
```

After editing JSON, compile fresh `.ata` files and ASCII previews:

```powershell
npm run docs:demos
```

This overwrites the generated `.ata` and `.txt` files. If you edited an `.ata` directly in QuarterMaster/M, do not run the generator unless you also transferred the change into its JSON source.

## Generated previews

The `.txt` previews use `+`, `-`, and `|` in place of Atari border graphics and do not preserve inverse video. They exist only for reviewing text and row width in a normal editor. The `.ata` files are authoritative for rendering and screenshots.
