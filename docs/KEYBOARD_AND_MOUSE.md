# Keyboard and mouse reference

[Documentation index](README.md) · [User guide](USER_GUIDE.md) · [ATASCII reference](ATASCII_REFERENCE.md)

## Application shortcuts

| Shortcut | Action | Location-sensitive |
|---|---|---|
| Ctrl+Shift+N | New document | Yes |
| Ctrl+Shift+O | Open document | Yes |
| Ctrl+Shift+S | Save document | Yes |
| Ctrl+Shift+A | Select all 40×357 or 80×357 cells | No |
| Ctrl+Shift+X | Cut selected rectangle | No |
| Ctrl+Shift+C | Copy selected rectangle | No |
| Ctrl+Shift+V | Paste | No |
| Ctrl+Shift+F | Find | No |
| Ctrl+Shift+H | Find and Replace | No |
| F2 | Toggle inverse state for newly typed glyphs | No |
| Ctrl+Delete | Clear every cell after the cursor on the current row | No |
| Ctrl+Shift+Delete | Delete the current row and pull later rows upward | No |
| Insert | Toggle insert/overwrite | No |
| Escape | Clear selection; otherwise leave inverse typing | No |

Plain Ctrl+letter combinations always enter low ATASCII control glyphs. Application commands that use a character therefore require Ctrl+Shift+letter.

## Caret navigation

| Key | Without Shift | With Shift |
|---|---|---|
| Left / Right | Previous / next cell | Extend rectangle one column toward the new focus |
| Up / Down | Same column in previous / next row | Extend rectangle one row toward the new focus |
| Home | First cell of current row | Moves to row start; selection extension is not assigned |
| End | Last cell of current row | Moves to row end; selection extension is not assigned |
| Page Up | 24 rows upward | Moves 24 rows; selection extension is not assigned |
| Page Down | 24 rows downward | Moves 24 rows; selection extension is not assigned |
| Enter | Column 1 of next row | Same as Enter |
| Tab | Next 8-column tab stop | Same as Tab |

Movement is clamped to the first and last document cells. The viewport scrolls to keep the caret visible.

## Deletion and insertion

| Key | No selection | Rectangular selection active |
|---|---|---|
| Backspace | Move left, delete, shift rest of that row left | Clear every selected cell |
| Delete | Delete current cell, shift rest of row left | Clear every selected cell |
| Insert | Toggle row-local insert/overwrite | Toggle row-local insert/overwrite |
| Printable character | Replace/insert at caret | Clear selection, then type |

### Delete after cursor

Press **Ctrl+Delete** to clear every cell to the right of the cursor through the last column of the current row. The glyph under the cursor is preserved. Rows below are not changed.

The same command is available as **Delete After Cursor** in the editor right-click menu and begins at the right-clicked glyph.

### Delete line

Press **Ctrl+Shift+Delete** to delete the entire 40/80-column row containing the cursor. Every later document row moves upward by one row, the final row becomes blank, and the cursor remains at the same row/column where possible.

Right-click any glyph and choose **Delete Line** to delete that glyph's row. Any active rectangular selection is cleared before the row operation.

## Find and replace

Press **Ctrl+Shift+F** for Find or **Ctrl+Shift+H** for Find and Replace. The non-modal panel keeps the matching editor cells visible.

- **Find Next** searches from the caret or after the current match.
- **Match case** distinguishes uppercase and lowercase glyph bytes.
- **Wrap** continues at row 1 after reaching row 357.
- Search compares seven-bit base glyphs, so normal and inverse text match each other.
- A match cannot cross a 40/80-column row boundary.
- Search and replacement fields accept ATASCII/ASCII characters, not multiline text.
- **Replace** changes the selected match and continues to the next.
- **Replace All** replaces every non-overlapping match in all 357 rows.

A replacement removes the matched cells, inserts the replacement using the current inverse-typing state, shifts the remaining cells of that row, and then clips or pads to the fixed row width.

Insert mode never moves content between rows. The last cell on the current row is discarded when a new glyph is inserted into a full row.

## Mouse operations in the editor

| Gesture | Result |
|---|---|
| Left-click | Place caret and clear the previous selection |
| Left-button drag | Select the rectangular bounds between start and end cells |
| Shift-left-click | Extend the rectangle from the existing anchor |
| Right-click in selection | Open editor menu for that selection |
| Right-click outside selection | Select clicked glyph, then open editor menu |
| Mouse wheel | Scroll vertically through the 357-row document |

Mouse selection is intentionally rectangular because an Atari screen is a fixed cell grid, not a flowing modern text paragraph.

## Editor context menu

### Cut

Copies the selection to the internal and Windows clipboards, then replaces the selected rectangle with ordinary spaces.

### Copy

Stores:

- rectangle width and height;
- each cell's seven-bit Atari glyph code;
- each inverse flag;
- a readable plain-text representation for Windows applications.

### Paste

An internal QuarterMaster/M copy is pasted with exact rectangle and inverse metadata. External clipboard text is placed starting at the caret; supported ASCII is retained and unsupported text becomes `?`.

Content is clipped at the editor's right and bottom boundaries.

### Select Glyph

Selects the single cell under the context click.

### Select All

Selects all 14,280 cells in 40-column mode or all 28,560 cells in XEP-80 mode.

### Inverse Selected Glyphs

Toggles inverse on each selected cell. On ATASCII save this toggles bit 7. It does not change the base glyph and does not change the toolbar's inverse-typing state.

### Delete After Cursor / Delete Line

These are row operations described above. The context click establishes which glyph and row they target.

## Entering ATASCII control graphics

Ctrl+letter enters the corresponding low ATASCII code. None of these plain Ctrl+character combinations are application shortcuts:

| Input | Byte | Glyph family |
|---|---:|---|
| Ctrl+@ | `$00` | Heart (not directly generated by the current letter handler) |
| Ctrl+A | `$01` | Tee-right |
| Ctrl+B | `$02` | Right-half bar |
| Ctrl+C | `$03` | Lower-right corner |
| Ctrl+D | `$04` | Tee-left |
| Ctrl+E | `$05` | Upper-right corner |
| Ctrl+F | `$06` | Diagonal slash |
| Ctrl+G | `$07` | Diagonal backslash |
| Ctrl+H | `$08` | Lower-right triangle |
| Ctrl+I | `$09` | Lower-right block |
| Ctrl+J | `$0A` | Lower-left triangle |
| Ctrl+K | `$0B` | Upper-right block |
| Ctrl+L | `$0C` | Upper-left block |
| Ctrl+M | `$0D` | Top bar |
| Ctrl+N | `$0E` | Bottom bar |
| Ctrl+O | `$0F` | Lower-left block |
| Ctrl+P | `$10` | Club |
| Ctrl+Q | `$11` | Upper-left corner |
| Ctrl+R | `$12` | Horizontal bar |
| Ctrl+S | `$13` | Tee-down |
| Ctrl+T | `$14` | Tee-up |
| Ctrl+U | `$15` | Left-half bar |
| Ctrl+V | `$16` | Lower-left corner |
| Ctrl+W | `$17` | Vertical bar |
| Ctrl+X | `$18` | Diamond |
| Ctrl+Y | `$19` | Four-way junction |
| Ctrl+Z | `$1A` | Filled circle |

Reserved low glyphs can still be loaded from ATASCII files, pasted from an internal QuarterMaster/M selection, or produced by another Atari-aware tool.

## Selection examples

### Invert a title bar

1. Drag from its first through last glyph.
2. Right-click inside the gold rectangle.
3. Choose **Inverse Selected Glyphs**.

### Move a rectangular panel

1. Select the panel.
2. Ctrl+Shift+X.
3. Click the destination's upper-left cell.
4. Ctrl+Shift+V.

The rectangle and inverse flags remain intact.

### Copy to a Windows editor

1. Select the desired rows/cells.
2. Ctrl+Shift+C.
3. Paste into the Windows application.

Graphical Atari glyphs may be represented by readable approximations and are not guaranteed to survive a non-Atari round trip. Use ATASCII save or raw ATR operations for fidelity.

## Accessibility and focus notes

- The editor surface exposes a textbox role and receives keyboard focus after commands.
- Menus are operated with mouse/touch-style clicking in the current release.
- The Help Center uses a modal dialog, topic navigation, table headings, search labels, an Escape close action, and glyph `aria-label` descriptions.
- The application uses color plus outlines/labels for active states; selection uses a gold inner border.
