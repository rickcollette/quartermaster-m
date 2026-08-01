# QuarterMaster/M complete user guide

[Documentation index](README.md) · [ATASCII reference](ATASCII_REFERENCE.md) · [ATR guide](ATR_GUIDE.md) · [Troubleshooting](TROUBLESHOOTING.md)

## Purpose

QuarterMaster/M is an Atari 8-bit content workstation. It edits ATASCII and ASCII documents, displays authentic Atari glyphs and inverse video, manages files inside ATR images, creates new DOS 2 or SpartaDOS disk images, and converts Atari BASIC programs between tokenized and editable forms.

The application is designed around one rule:

> **New, Open, and Save operate on the active location highlighted in Explorer.**

## First session

1. Start `quartermaster-m.exe`. The splash screen is visible while the native window and saved drive state initialize.
2. Choose a storage area:
   - click **Open Folder** to browse host files; or
   - choose **ATR → Open/Mount** to mount an ATR image in D1:, D2:, D3:, or D4:.
3. Click the desired Explorer root or directory. A gold outline and the toolbar's location label confirm the active location.
4. Use **New**, **Open**, or double-click a file.
5. Edit. Drag across cells for a rectangular selection; right-click in the editor for editing actions.
6. Choose **ATASCII** or **ASCII** in the toolbar.
7. Press **Ctrl+Shift+S**. The save UI opens at the active directory.

## Window anatomy

### Title strip

The upper-left identifies QuarterMaster/M as the ATASCII Editor. The upper-right shows the exact application version. Include this version in support requests.

### Menu bar

#### File

| Command | Behavior |
|---|---|
| New | Chooses a new file at the active location, then creates a blank modified document |
| Open… | Opens from the active local/ATR directory |
| Save | Saves through a location-aware dialog, defaulting to the current name/directory |
| Save As… | Uses the same location-aware destination flow, allowing another name |
| Export ASCII… | Writes a Windows-readable version of the current editor without changing its native location |

#### View

- **40 Column Atari** changes the editor to 40 columns per row.
- **80 Column XEP-80** changes the editor to 80 columns per row.

#### Edit

- **Find…** opens the row-bounded search panel.
- **Find and Replace…** adds replacement controls and Replace All.

Use Ctrl+Shift+F and Ctrl+Shift+H. Plain Ctrl+F and Ctrl+H remain ATASCII glyph inputs.

Width conversion copies only columns that fit. Expanding 40 → 80 preserves the first 40 and adds blank columns. Reducing 80 → 40 discards columns 41–80, so save an 80-column copy first when those columns matter.

#### ATR

The ATR menu intentionally contains only:

```text
ATR
----
Open/Mount
Refresh Directory
Open File from ATR
Create ATR
Unmount ATR
```

Detailed file management remains in Explorer and its context menus.

#### BASIC

The BASIC menu converts between tokenized Atari BASIC and editable listings on either local storage or the active ATR. See [Atari BASIC Guide](BASIC_GUIDE.md).

#### Help

- **Help Center** opens the manual at Start Here.
- **Keyboard & Mouse** opens the complete shortcut page.
- **ATASCII Map** opens a searchable representation of all 256 bytes.
- **Get Help / Report Issue** provides troubleshooting steps and the official issue URL.
- **Check for Updates** compares the installed semantic version with the repository's published `current-version` manifest for the current platform. Windows can download a portable EXE beside the running application or download and launch the setup installer. macOS can download and open the universal DMG. Linux shows the published package names for the current Linux release.
- **License** displays the complete `LICENSE` file bundled with QuarterMaster/M.
- **About** displays the application name, exact version, copyright, author, and handle.

Press Escape or click Close to dismiss the Help Center. It can be resized with the main window; the topic navigation and article scroll independently.

### Toolbar

#### Far-left view controls

- **40 COL:** standard 40-column Atari geometry.
- **XEP-80:** 80-column editing geometry.

The active button is visually reversed.

#### Editor controls

- **New / Open / Save:** target the active Explorer location.
- **Location label:** identifies `LOCAL: path` or `D1: path` (and similarly D2:–D4:).
- **ATASCII / ASCII:** chooses how files are loaded and saved.
- **Inverse:** toggles inverse state for newly typed glyphs.
- **Insert:** toggles row-local insertion versus overwrite.
- **Clear:** clears the entire document after confirmation.

### Explorer

Explorer has two independently scrollable data areas in one pane.

#### Local Folder

- **Open Folder** chooses a Windows directory and builds a tree.
- **Open File** invokes the same location-aware open behavior after making Local active.
- Click the local root or a directory to make it active.
- Click a file to select it; its parent directory becomes the location for New/Open/Save.
- Double-click a file to load it.
- Drag a host file to a mounted ATR drive/directory to import it.

#### ATR Mounts

Four roots, D1:–D4:, are always shown.

- **Mount** chooses and mounts an ATR in the selected drive slot.
- **Unmount** closes the selected drive slot.
- **Refresh** rereads the selected mounted image.
- **Add** imports a host file.
- **New Dir** creates a SpartaDOS directory.

The bottom row of tree actions operates on the current selection: Open, Add, Extract, New Dir, Delete, and Unmount. A description below the buttons reports the selected item.

### Editor viewport

The document is 357 rows high. The viewport shows approximately 24 rows and scrolls through the rest. Each cell stores:

- a seven-bit base glyph code;
- an inverse flag corresponding to bit 7; and
- a display approximation used by the UI/backend transfer structure.

40-column cells are wider than XEP-80 cells, but the file encoding principles are the same.

### Status bar

The status bar reports:

- document filename or default untitled name;
- `*` when modified;
- ATR drive when the document came from an ATR;
- one-based row and column;
- rectangular selection width × height, when present;
- ATASCII or ASCII mode;
- inverse (`INV`) or normal typing state;
- insert (`INS`) or overwrite (`OVR`) state;
- current byte in hexadecimal;
- document geometry (`40×357` or `80×357`);
- visible-row count (`24 ROW VIEW`).

## Selecting the active location

The active location is independent of the document currently displayed.

- Clicking the Local root or local directory makes Local active.
- Clicking any D1:–D4: root/directory makes that ATR path active.
- Clicking a file targets its parent directory.
- Opening a file makes that file's location active.
- Mounting or creating an ATR makes its assigned drive active.

This allows a document opened from D1: to be saved intentionally into D2: simply by highlighting a D2: directory before Save. If you want it saved beside the original, reselect the original directory.

## Creating a document

1. Select the desired Local or ATR directory.
2. Set 40 COL/XEP-80 and ATASCII/ASCII as desired.
3. Click New or press Ctrl+Shift+N.
4. Choose a filename. ATR names must be 1–8 uppercase letters/numbers/underscore, with an optional 1–3 character extension.
5. The editor becomes blank and modified.
6. Enter content and save.

If another document is modified, QuarterMaster/M asks before discarding it.

## Opening a document

There are four equivalent entry points:

- toolbar Open;
- File → Open;
- double-click in Local Explorer;
- double-click or right-click → Open in ATR Explorer.

The selected mode and width control decoding. If an ATASCII file looks broken:

1. confirm ATASCII mode;
2. confirm 40 versus 80 columns;
3. use the right-click raw extraction option to preserve a diagnostic copy;
4. consult [File Formats](FILE_FORMATS.md).

ATASCII decoding treats `$9B` as a row break. Other recognized non-glyph editor controls are not inserted as cells. ASCII decoding normalizes CRLF, CR, and LF. Non-ASCII Unicode becomes `?` and produces a warning.

## Editing

### Caret navigation

- Click to position.
- Arrow keys move one cell.
- Home/End move to the first/last column.
- Page Up/Page Down move 24 rows.
- Enter moves to column 1 of the next row.
- Tab moves to the next eight-column tab stop.

### Insert and overwrite

Overwrite replaces the cell under the caret. Insert shifts cells to the right only within the current row and drops the last cell of that row.

### Deletion

Backspace moves left and deletes. Delete removes the current cell. Both shift the rest of that row left and add a blank at the right edge. With a selection, either key clears the full selected rectangle.

### Mouse selection

- Drag from one cell to another to select the bounding rectangle.
- Shift-click extends from the selection anchor.
- Right-click outside a selection selects the clicked glyph before opening the context menu.

The context menu contains:

- Cut
- Copy
- Paste
- Select Glyph
- Select All
- Inverse Selected Glyphs
- Delete After Cursor
- Delete Line

### Clipboard semantics

The internal clipboard retains rectangular geometry, Atari byte codes, and inverse flags. This gives exact in-app paste behavior. QuarterMaster/M also writes readable text to the Windows clipboard. Pasting external Unicode keeps supported ASCII and substitutes unsupported characters as described by the editor conversion.

### Inverse video

There are two different operations:

- toolbar **Inverse** or F2 controls newly typed cells;
- context-menu **Inverse Selected Glyphs** toggles every selected cell.

An inverse glyph uses the same base code with bit 7 set on ATASCII save. The glyph foreground/background is reversed, avoiding white-on-white rendering.

### Row deletion

- **Ctrl+Delete** or right-click → **Delete After Cursor** clears all cells to the right of the cursor on the current row. It preserves the glyph under the cursor and does not affect later rows.
- **Ctrl+Shift+Delete** or right-click → **Delete Line** removes the entire current/right-clicked row. All subsequent rows move upward and the final row becomes blank.

Both operations clear an active rectangular selection before applying the row edit.

### Find and replace

Use **Edit → Find** or Ctrl+Shift+F. Use **Edit → Find and Replace** or Ctrl+Shift+H for replacement controls.

Search:

- runs across all 357 rows;
- optionally matches case;
- optionally wraps from the end of the document;
- highlights each result as a normal editor selection;
- compares base glyph bytes while ignoring inverse bit 7;
- never allows a match to cross a screen-row boundary.

Replace removes the matched cells, inserts replacement glyphs using the current inverse-typing state, shifts the rest of that fixed row, then clips/pads to 40 or 80 columns. Replace All processes every non-overlapping match in the document. Empty replacement text deletes matches.

## Saving

Save opens at the active location and suggests the current document name.

### Local

A native Windows save dialog is rooted at the selected folder. Parent directories are created when needed.

### ATR

The in-app dialog lists files in the selected directory and validates Atari filenames. Saving can overwrite an existing file. If the displayed file came from an ATR, its drive/path remains the initial default unless you deliberately change the active location.

### Serialized rows

Trailing ordinary spaces are removed per row. Inverse spaces are retained because they are meaningful visible cells. A separator is written between every stored row:

- `$9B` in ATASCII;
- CRLF in ASCII.

## File conversion and export

### File → Export ASCII

Writes the current editor as ASCII/CRLF without moving the editor away from its current file.

### ATR right-click → Export ASCII

Converts ATASCII line endings to CRLF, removes inverse state for printable text, and detokenizes tokenized `.BAS` data when recognized.

### ATR right-click → Extract Raw

Copies native bytes unchanged. Use this for binaries, graphics, executables, already-tokenized BASIC, unknown formats, archival work, or diagnostics.

## Activity and startup feedback

The splash screen appears before the main window and remains until the frontend reports readiness. Backend operations use a modal activity card with an animated ellipsis. Labels include Loading, Saving, Mounting, Refreshing, Importing, Copying, Extracting, Exporting, Tokenizing, Detokenizing, Renaming, Deleting, and Creating Folder/Disk.

While an operation is active, additional editor commands are temporarily blocked to prevent conflicting writes.

## Safe working habits

- Keep backups of original ATR images.
- Do not modify the same mounted ATR in another application.
- Refresh after an external change.
- Use Extract Raw before experimenting with an unknown file.
- Save an 80-column copy before reducing it to 40 columns.
- Use ASCII export for source control; keep ATASCII/raw originals for Atari fidelity.
- Review the active-location label before Save.

## Help and support

Open **Help → Get Help / Report Issue** or visit:

https://github.com/rickcollette/quartermaster-m/issues

See [Troubleshooting](TROUBLESHOOTING.md) for the diagnostic checklist.
