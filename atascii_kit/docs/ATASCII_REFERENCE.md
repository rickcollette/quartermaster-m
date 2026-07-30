# ATASCII Reference

ATASCII is the Atari 8-bit character-interchange encoding. It preserves much of
ASCII, replaces many control positions with graphics, and normally uses bytes
128–255 as inverse-video forms of bytes 0–127.

The ATASCII end-of-line byte is decimal 155 (`$9B`), not ASCII CR (`$0D`) or
LF (`$0A`). This is critical for files, modems, Telnet terminals, and BBSes.

## Three code spaces

- **ATASCII** — CIO, `PRINT`, `INPUT`, `CHR$`, `ASC`, files, modems.
- **Internal/screen code** — bytes stored in text screen memory.
- **Keyboard code** — keyboard subsystem values before ATASCII conversion.

## Graphics 0 conversion

For the low seven bits:

```text
ATASCII 0..31   -> screen code +64
ATASCII 32..95  -> screen code -32
ATASCII 96..127 -> unchanged
```

Preserve bit 7 for inverse characters. The complete 256-byte mapping is in
`data/atascii_screen_code_map.csv`.

## Inverse video

```text
inverse = normal | $80
normal  = inverse & $7F
```

In Graphics 0, ANTIC renders the high-bit form as an inverse glyph. Graphics
modes 1 and 2 instead use the upper two bits for four color selections.

## Control bytes

| Dec | Hex | Function |
|---:|---:|---|
| 27 | 1B | Escape |
| 28 | 1C | Cursor up |
| 29 | 1D | Cursor down |
| 30 | 1E | Cursor left |
| 31 | 1F | Cursor right |
| 125 | 7D | Clear screen |
| 126 | 7E | Backspace/delete |
| 127 | 7F | Tab |
| 155 | 9B | End of line |
| 156 | 9C | Delete line |
| 157 | 9D | Insert line |
| 158 | 9E | Clear tab stop |
| 159 | 9F | Set tab stop |
| 253 | FD | Buzzer |
| 254 | FE | Delete character |
| 255 | FF | Insert character |

Many control bytes also have visible glyphs. In the screen editor, Escape before
the control keystroke generally inserts the visible form rather than executing
the control.

## Custom character sets

Each standard glyph is eight bytes. Copying a character set into RAM, editing
the glyph bytes, and changing `CHBASE` (shadow location 756) enables custom fonts,
tiles, icons, and inexpensive animation.

XL/XE systems also include an international character set with accented Latin
glyphs replacing several graphics characters.
