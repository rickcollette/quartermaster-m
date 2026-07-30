# Screenshots

[Documentation index](README.md) · [User guide](USER_GUIDE.md)

## 40-column ATASCII editing

![QuarterMaster/M editing a designed 40-column ATASCII screen](images/quartermaster-editor.png)

This view demonstrates the full application workbench, 40-column Atari geometry, local/ATR Explorer, native glyph rendering, inverse title treatment, toolbar location controls, and status line.

## Rectangular selection and inverse editing

![QuarterMaster/M with a rectangular glyph selection](images/quartermaster-selection.png)

This view demonstrates standard mouse editing inside the editor: a rectangular glyph region is selected and ready for Cut, Copy, Paste, or Inverse Selected Glyphs.

## Refreshing screenshots

The captures are driven by real ATASCII documents under [`examples/`](examples/):

- [`quartermaster-command-deck.ata`](examples/quartermaster-command-deck.ata) → `quartermaster-editor.png`
- [`quartermaster-screen-composer.ata`](examples/quartermaster-screen-composer.ata) → `quartermaster-selection.png`

Open either file in QuarterMaster/M using **ATASCII** and **40 COL**, make the screen look the way you want, and save it back to the same path. Then run:

```powershell
npm run docs:screenshots
```

For text-first editing, change the matching `.screen.json` file and run `npm run docs:demos` before capturing. See the [examples README](examples/README.md). The demo generator overwrites the `.ata` files, so do not run it after a direct in-app edit unless the JSON has also been updated.

Documentation captures should:

1. use the current frontend build;
2. use a sanitized demo screen with exactly 40 columns per designed row;
3. show at least the toolbar, Explorer, viewport, and status bar;
4. avoid personal paths and proprietary disk contents;
5. use a consistent viewport (approximately 1200×900 is suitable);
6. store PNG output under `docs/images/`;
7. retain the filenames used by README unless links are updated together.

The generator enforces exactly 24 rows and produces exactly 40 ATASCII bytes per row. Files edited directly in QuarterMaster/M may use shorter serialized rows; the capture tool pads them to 40 cells while honoring `$9B` line boundaries and inverse bit 7.
