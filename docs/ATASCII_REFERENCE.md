# ATASCII complete reference

[Documentation index](README.md) · [Keyboard and mouse](KEYBOARD_AND_MOUSE.md) · [File formats](FILE_FORMATS.md)

This reference documents every ATASCII byte from `$00` through `$FF`, its base glyph, inverse state, Atari screen-code mapping, input convention, and context-sensitive control meaning. The 256-row appendix is generated from the project's canonical machine-readable map by `npm run docs:atascii`.

## The byte model

```text
base glyph = ATASCII byte & $7F
inverse    = (ATASCII byte & $80) != 0
```

In a display-cell context, `$80–$FF` are inverse forms of `$00–$7F`. In an interpreted ATASCII stream, several high bytes are editor controls rather than printable cells. In particular, QuarterMaster/M text files use `$9B` as end-of-line.

## ATASCII versus screen codes

ATASCII is the keyboard/text-stream ordering. Atari display memory uses screen codes:

| ATASCII base range | Screen-code conversion |
|---|---|
| `$00–$1F` | Add `$40` |
| `$20–$5F` | Subtract `$20` |
| `$60–$7F` | Unchanged |

Preserve bit 7 to preserve inverse video. The full result for every byte appears below.

## Text-file behavior in QuarterMaster/M

- `$9B` starts the next row.
- Glyph tokens populate cells.
- Width overflow wraps to the next row.
- Other interpreted editor-control commands are ignored during ordinary text-file loading rather than executed against the document.
- Saving inserts `$9B` between rows.
- Trailing ordinary spaces are trimmed; inverse spaces remain.

This makes loading deterministic and prevents embedded screen-editor commands from unexpectedly clearing or restructuring a document.

## Control bytes

| Decimal | Hex | Interpreted function | Typical Atari key | Notes |
|---:|---:|---|---|---|
| 27 | `$1B` | Escape | ESC | Escape/control prefix |
| 28 | `$1C` | Cursor up | CTRL-- | Moves cursor up |
| 29 | `$1D` | Cursor down | CTRL-= | Moves cursor down |
| 30 | `$1E` | Cursor left | CTRL-+ | Moves cursor left |
| 31 | `$1F` | Cursor right | CTRL-* | Moves cursor right |
| 125 | `$7D` | Clear screen | CTRL-< | Clears screen |
| 126 | `$7E` | Backspace/delete | BACK SPACE | Deletes previous character |
| 127 | `$7F` | Tab | TAB | Advances to tab stop |
| 155 | `$9B` | End of line | RETURN | ATASCII newline |
| 156 | `$9C` | Delete line | SHIFT-DELETE | Deletes line |
| 157 | `$9D` | Insert line | SHIFT-INSERT | Inserts line |
| 158 | `$9E` | Clear tab stop | CTRL-TAB | Clears tab stop |
| 159 | `$9F` | Set tab stop | SHIFT-TAB | Sets tab stop |
| 253 | `$FD` | Buzzer | CTRL-2 | Sounds buzzer |
| 254 | `$FE` | Delete character | CTRL-DELETE | Deletes at cursor |
| 255 | `$FF` | Insert character | CTRL-INSERT | Inserts at cursor |

### Quoting and context

Atari software can quote control codes so that the glyph associated with the same low seven bits appears rather than the command executing. A saved screen, display-memory dump, terminal protocol, and plain ATASCII text file can therefore assign different semantics to the same numeric byte. QuarterMaster/M's document loader deliberately uses its TextFile decoding domain.

## Base glyph families

### `$00–$1F`: graphics and cursor symbols

| Hex | Name | Hex | Name | Hex | Name | Hex | Name |
|---:|---|---:|---|---:|---|---:|---|
| `$00` | Heart | `$08` | Lower-right triangle | `$10` | Club | `$18` | Diamond |
| `$01` | Tee right | `$09` | Lower-right block | `$11` | Upper-left corner | `$19` | Four-way junction |
| `$02` | Right half bar | `$0A` | Lower-left triangle | `$12` | Horizontal bar | `$1A` | Filled circle |
| `$03` | Lower-right corner | `$0B` | Upper-right block | `$13` | Tee down | `$1B` | Escape |
| `$04` | Tee left | `$0C` | Upper-left block | `$14` | Tee up | `$1C` | Cursor up |
| `$05` | Upper-right corner | `$0D` | Top bar | `$15` | Left half bar | `$1D` | Cursor down |
| `$06` | Diagonal slash | `$0E` | Bottom bar | `$16` | Lower-left corner | `$1E` | Cursor left |
| `$07` | Diagonal backslash | `$0F` | Lower-left block | `$17` | Vertical bar | `$1F` | Cursor right |

### `$20–$5F`: uppercase ASCII-compatible range

This range includes space, punctuation, digits, uppercase letters, and ASCII punctuation through underscore. Its screen codes are `$00–$3F`.

### `$60–$7F`: Atari/lowercase range

- `$60`: Atari diamond (not the ASCII grave-accent glyph on the Atari screen).
- `$61–$7A`: lowercase letters.
- `$7B`: spade.
- `$7C`: vertical bar.
- `$7D`: clear-screen glyph/control in the appropriate context.
- `$7E`: backspace/delete glyph/control.
- `$7F`: tab glyph/control.

## Complete 256-byte map

The **Meaning** column gives both interpreted-control and display-cell interpretations where they differ. Decimal screen codes come directly from the canonical round-trip table.

| ATASCII decimal | ATASCII hex | Base | Inverse | Screen decimal | Screen hex | Meaning | Input / typical key |
|---:|---:|---:|:---:|---:|---:|---|---|
| 0 | `$00` | `$00` | No | 64 | `$40` | Heart | Ctrl+@ (mapping; current editor letter handler does not emit it) |
| 1 | `$01` | `$01` | No | 65 | `$41` | Tee right | Ctrl+A |
| 2 | `$02` | `$02` | No | 66 | `$42` | Right half bar | Ctrl+B |
| 3 | `$03` | `$03` | No | 67 | `$43` | Lower-right corner | Ctrl+C |
| 4 | `$04` | `$04` | No | 68 | `$44` | Tee left | Ctrl+D |
| 5 | `$05` | `$05` | No | 69 | `$45` | Upper-right corner | Ctrl+E |
| 6 | `$06` | `$06` | No | 70 | `$46` | Diagonal slash | Ctrl+F |
| 7 | `$07` | `$07` | No | 71 | `$47` | Diagonal backslash | Ctrl+G |
| 8 | `$08` | `$08` | No | 72 | `$48` | Lower-right triangle | Ctrl+H |
| 9 | `$09` | `$09` | No | 73 | `$49` | Lower-right block | Ctrl+I |
| 10 | `$0A` | `$0A` | No | 74 | `$4A` | Lower-left triangle | Ctrl+J |
| 11 | `$0B` | `$0B` | No | 75 | `$4B` | Upper-right block | Ctrl+K |
| 12 | `$0C` | `$0C` | No | 76 | `$4C` | Upper-left block | Ctrl+L |
| 13 | `$0D` | `$0D` | No | 77 | `$4D` | Top bar | Ctrl+M |
| 14 | `$0E` | `$0E` | No | 78 | `$4E` | Bottom bar | Ctrl+N |
| 15 | `$0F` | `$0F` | No | 79 | `$4F` | Lower-left block | Ctrl+O |
| 16 | `$10` | `$10` | No | 80 | `$50` | Club | Ctrl+P |
| 17 | `$11` | `$11` | No | 81 | `$51` | Upper-left corner | Ctrl+Q |
| 18 | `$12` | `$12` | No | 82 | `$52` | Horizontal bar | Ctrl+R |
| 19 | `$13` | `$13` | No | 83 | `$53` | Tee down | Ctrl+S |
| 20 | `$14` | `$14` | No | 84 | `$54` | Tee up | Ctrl+T |
| 21 | `$15` | `$15` | No | 85 | `$55` | Left half bar | Ctrl+U |
| 22 | `$16` | `$16` | No | 86 | `$56` | Lower-left corner | Ctrl+V |
| 23 | `$17` | `$17` | No | 87 | `$57` | Vertical bar | Ctrl+W |
| 24 | `$18` | `$18` | No | 88 | `$58` | Diamond | Ctrl+X |
| 25 | `$19` | `$19` | No | 89 | `$59` | Four-way junction | Ctrl+Y |
| 26 | `$1A` | `$1A` | No | 90 | `$5A` | Filled circle | Ctrl+Z |
| 27 | `$1B` | `$1B` | No | 91 | `$5B` | Escape control when interpreted; Escape in quoted/display-cell context | ESC |
| 28 | `$1C` | `$1C` | No | 92 | `$5C` | Cursor up control when interpreted; Cursor up in quoted/display-cell context | CTRL-- |
| 29 | `$1D` | `$1D` | No | 93 | `$5D` | Cursor down control when interpreted; Cursor down in quoted/display-cell context | CTRL-= |
| 30 | `$1E` | `$1E` | No | 94 | `$5E` | Cursor left control when interpreted; Cursor left in quoted/display-cell context | CTRL-+ |
| 31 | `$1F` | `$1F` | No | 95 | `$5F` | Cursor right control when interpreted; Cursor right in quoted/display-cell context | CTRL-* |
| 32 | `$20` | `$20` | No | 0 | `$00` | Space | Space |
| 33 | `$21` | `$21` | No | 1 | `$01` | ASCII `!` | Type the character |
| 34 | `$22` | `$22` | No | 2 | `$02` | ASCII `"` | Type the character |
| 35 | `$23` | `$23` | No | 3 | `$03` | ASCII `#` | Type the character |
| 36 | `$24` | `$24` | No | 4 | `$04` | ASCII `$` | Type the character |
| 37 | `$25` | `$25` | No | 5 | `$05` | ASCII `%` | Type the character |
| 38 | `$26` | `$26` | No | 6 | `$06` | ASCII `&` | Type the character |
| 39 | `$27` | `$27` | No | 7 | `$07` | ASCII `'` | Type the character |
| 40 | `$28` | `$28` | No | 8 | `$08` | ASCII `(` | Type the character |
| 41 | `$29` | `$29` | No | 9 | `$09` | ASCII `)` | Type the character |
| 42 | `$2A` | `$2A` | No | 10 | `$0A` | ASCII `*` | Type the character |
| 43 | `$2B` | `$2B` | No | 11 | `$0B` | ASCII `+` | Type the character |
| 44 | `$2C` | `$2C` | No | 12 | `$0C` | ASCII `,` | Type the character |
| 45 | `$2D` | `$2D` | No | 13 | `$0D` | ASCII `-` | Type the character |
| 46 | `$2E` | `$2E` | No | 14 | `$0E` | ASCII `.` | Type the character |
| 47 | `$2F` | `$2F` | No | 15 | `$0F` | ASCII `/` | Type the character |
| 48 | `$30` | `$30` | No | 16 | `$10` | ASCII `0` | Type the character |
| 49 | `$31` | `$31` | No | 17 | `$11` | ASCII `1` | Type the character |
| 50 | `$32` | `$32` | No | 18 | `$12` | ASCII `2` | Type the character |
| 51 | `$33` | `$33` | No | 19 | `$13` | ASCII `3` | Type the character |
| 52 | `$34` | `$34` | No | 20 | `$14` | ASCII `4` | Type the character |
| 53 | `$35` | `$35` | No | 21 | `$15` | ASCII `5` | Type the character |
| 54 | `$36` | `$36` | No | 22 | `$16` | ASCII `6` | Type the character |
| 55 | `$37` | `$37` | No | 23 | `$17` | ASCII `7` | Type the character |
| 56 | `$38` | `$38` | No | 24 | `$18` | ASCII `8` | Type the character |
| 57 | `$39` | `$39` | No | 25 | `$19` | ASCII `9` | Type the character |
| 58 | `$3A` | `$3A` | No | 26 | `$1A` | ASCII `:` | Type the character |
| 59 | `$3B` | `$3B` | No | 27 | `$1B` | ASCII `;` | Type the character |
| 60 | `$3C` | `$3C` | No | 28 | `$1C` | ASCII `<` | Type the character |
| 61 | `$3D` | `$3D` | No | 29 | `$1D` | ASCII `=` | Type the character |
| 62 | `$3E` | `$3E` | No | 30 | `$1E` | ASCII `>` | Type the character |
| 63 | `$3F` | `$3F` | No | 31 | `$1F` | ASCII `?` | Type the character |
| 64 | `$40` | `$40` | No | 32 | `$20` | ASCII `@` | Type the character |
| 65 | `$41` | `$41` | No | 33 | `$21` | ASCII `A` | Type the character |
| 66 | `$42` | `$42` | No | 34 | `$22` | ASCII `B` | Type the character |
| 67 | `$43` | `$43` | No | 35 | `$23` | ASCII `C` | Type the character |
| 68 | `$44` | `$44` | No | 36 | `$24` | ASCII `D` | Type the character |
| 69 | `$45` | `$45` | No | 37 | `$25` | ASCII `E` | Type the character |
| 70 | `$46` | `$46` | No | 38 | `$26` | ASCII `F` | Type the character |
| 71 | `$47` | `$47` | No | 39 | `$27` | ASCII `G` | Type the character |
| 72 | `$48` | `$48` | No | 40 | `$28` | ASCII `H` | Type the character |
| 73 | `$49` | `$49` | No | 41 | `$29` | ASCII `I` | Type the character |
| 74 | `$4A` | `$4A` | No | 42 | `$2A` | ASCII `J` | Type the character |
| 75 | `$4B` | `$4B` | No | 43 | `$2B` | ASCII `K` | Type the character |
| 76 | `$4C` | `$4C` | No | 44 | `$2C` | ASCII `L` | Type the character |
| 77 | `$4D` | `$4D` | No | 45 | `$2D` | ASCII `M` | Type the character |
| 78 | `$4E` | `$4E` | No | 46 | `$2E` | ASCII `N` | Type the character |
| 79 | `$4F` | `$4F` | No | 47 | `$2F` | ASCII `O` | Type the character |
| 80 | `$50` | `$50` | No | 48 | `$30` | ASCII `P` | Type the character |
| 81 | `$51` | `$51` | No | 49 | `$31` | ASCII `Q` | Type the character |
| 82 | `$52` | `$52` | No | 50 | `$32` | ASCII `R` | Type the character |
| 83 | `$53` | `$53` | No | 51 | `$33` | ASCII `S` | Type the character |
| 84 | `$54` | `$54` | No | 52 | `$34` | ASCII `T` | Type the character |
| 85 | `$55` | `$55` | No | 53 | `$35` | ASCII `U` | Type the character |
| 86 | `$56` | `$56` | No | 54 | `$36` | ASCII `V` | Type the character |
| 87 | `$57` | `$57` | No | 55 | `$37` | ASCII `W` | Type the character |
| 88 | `$58` | `$58` | No | 56 | `$38` | ASCII `X` | Type the character |
| 89 | `$59` | `$59` | No | 57 | `$39` | ASCII `Y` | Type the character |
| 90 | `$5A` | `$5A` | No | 58 | `$3A` | ASCII `Z` | Type the character |
| 91 | `$5B` | `$5B` | No | 59 | `$3B` | ASCII `[` | Type the character |
| 92 | `$5C` | `$5C` | No | 60 | `$3C` | ASCII `\` | Type the character |
| 93 | `$5D` | `$5D` | No | 61 | `$3D` | ASCII `]` | Type the character |
| 94 | `$5E` | `$5E` | No | 62 | `$3E` | ASCII `^` | Type the character |
| 95 | `$5F` | `$5F` | No | 63 | `$3F` | ASCII `_` | Type the character |
| 96 | `$60` | `$60` | No | 96 | `$60` | Atari diamond | Load/paste an ATASCII glyph or use an Atari-aware source |
| 97 | `$61` | `$61` | No | 97 | `$61` | Lowercase `a` | Type the character |
| 98 | `$62` | `$62` | No | 98 | `$62` | Lowercase `b` | Type the character |
| 99 | `$63` | `$63` | No | 99 | `$63` | Lowercase `c` | Type the character |
| 100 | `$64` | `$64` | No | 100 | `$64` | Lowercase `d` | Type the character |
| 101 | `$65` | `$65` | No | 101 | `$65` | Lowercase `e` | Type the character |
| 102 | `$66` | `$66` | No | 102 | `$66` | Lowercase `f` | Type the character |
| 103 | `$67` | `$67` | No | 103 | `$67` | Lowercase `g` | Type the character |
| 104 | `$68` | `$68` | No | 104 | `$68` | Lowercase `h` | Type the character |
| 105 | `$69` | `$69` | No | 105 | `$69` | Lowercase `i` | Type the character |
| 106 | `$6A` | `$6A` | No | 106 | `$6A` | Lowercase `j` | Type the character |
| 107 | `$6B` | `$6B` | No | 107 | `$6B` | Lowercase `k` | Type the character |
| 108 | `$6C` | `$6C` | No | 108 | `$6C` | Lowercase `l` | Type the character |
| 109 | `$6D` | `$6D` | No | 109 | `$6D` | Lowercase `m` | Type the character |
| 110 | `$6E` | `$6E` | No | 110 | `$6E` | Lowercase `n` | Type the character |
| 111 | `$6F` | `$6F` | No | 111 | `$6F` | Lowercase `o` | Type the character |
| 112 | `$70` | `$70` | No | 112 | `$70` | Lowercase `p` | Type the character |
| 113 | `$71` | `$71` | No | 113 | `$71` | Lowercase `q` | Type the character |
| 114 | `$72` | `$72` | No | 114 | `$72` | Lowercase `r` | Type the character |
| 115 | `$73` | `$73` | No | 115 | `$73` | Lowercase `s` | Type the character |
| 116 | `$74` | `$74` | No | 116 | `$74` | Lowercase `t` | Type the character |
| 117 | `$75` | `$75` | No | 117 | `$75` | Lowercase `u` | Type the character |
| 118 | `$76` | `$76` | No | 118 | `$76` | Lowercase `v` | Type the character |
| 119 | `$77` | `$77` | No | 119 | `$77` | Lowercase `w` | Type the character |
| 120 | `$78` | `$78` | No | 120 | `$78` | Lowercase `x` | Type the character |
| 121 | `$79` | `$79` | No | 121 | `$79` | Lowercase `y` | Type the character |
| 122 | `$7A` | `$7A` | No | 122 | `$7A` | Lowercase `z` | Type the character |
| 123 | `$7B` | `$7B` | No | 123 | `$7B` | Spade | Load/paste an ATASCII glyph or use an Atari-aware source |
| 124 | `$7C` | `$7C` | No | 124 | `$7C` | Vertical bar | Load/paste an ATASCII glyph or use an Atari-aware source |
| 125 | `$7D` | `$7D` | No | 125 | `$7D` | Clear screen control when interpreted; Clear-screen glyph in quoted/display-cell context | CTRL-< |
| 126 | `$7E` | `$7E` | No | 126 | `$7E` | Backspace/delete control when interpreted; Backspace/delete glyph in quoted/display-cell context | BACK SPACE |
| 127 | `$7F` | `$7F` | No | 127 | `$7F` | Tab control when interpreted; Tab glyph in quoted/display-cell context | TAB |
| 128 | `$80` | `$00` | Yes | 192 | `$C0` | Inverse Heart | Inverse form of Ctrl+@ (mapping; current editor letter handler does not emit it) |
| 129 | `$81` | `$01` | Yes | 193 | `$C1` | Inverse Tee right | Inverse form of Ctrl+A |
| 130 | `$82` | `$02` | Yes | 194 | `$C2` | Inverse Right half bar | Inverse form of Ctrl+B |
| 131 | `$83` | `$03` | Yes | 195 | `$C3` | Inverse Lower-right corner | Inverse form of Ctrl+C |
| 132 | `$84` | `$04` | Yes | 196 | `$C4` | Inverse Tee left | Inverse form of Ctrl+D |
| 133 | `$85` | `$05` | Yes | 197 | `$C5` | Inverse Upper-right corner | Inverse form of Ctrl+E |
| 134 | `$86` | `$06` | Yes | 198 | `$C6` | Inverse Diagonal slash | Inverse form of Ctrl+F |
| 135 | `$87` | `$07` | Yes | 199 | `$C7` | Inverse Diagonal backslash | Inverse form of Ctrl+G |
| 136 | `$88` | `$08` | Yes | 200 | `$C8` | Inverse Lower-right triangle | Inverse form of Ctrl+H |
| 137 | `$89` | `$09` | Yes | 201 | `$C9` | Inverse Lower-right block | Inverse form of Ctrl+I |
| 138 | `$8A` | `$0A` | Yes | 202 | `$CA` | Inverse Lower-left triangle | Inverse form of Ctrl+J |
| 139 | `$8B` | `$0B` | Yes | 203 | `$CB` | Inverse Upper-right block | Inverse form of Ctrl+K |
| 140 | `$8C` | `$0C` | Yes | 204 | `$CC` | Inverse Upper-left block | Inverse form of Ctrl+L |
| 141 | `$8D` | `$0D` | Yes | 205 | `$CD` | Inverse Top bar | Inverse form of Ctrl+M |
| 142 | `$8E` | `$0E` | Yes | 206 | `$CE` | Inverse Bottom bar | Inverse form of Ctrl+N |
| 143 | `$8F` | `$0F` | Yes | 207 | `$CF` | Inverse Lower-left block | Inverse form of Ctrl+O |
| 144 | `$90` | `$10` | Yes | 208 | `$D0` | Inverse Club | Inverse form of Ctrl+P |
| 145 | `$91` | `$11` | Yes | 209 | `$D1` | Inverse Upper-left corner | Inverse form of Ctrl+Q |
| 146 | `$92` | `$12` | Yes | 210 | `$D2` | Inverse Horizontal bar | Inverse form of Ctrl+R |
| 147 | `$93` | `$13` | Yes | 211 | `$D3` | Inverse Tee down | Inverse form of Ctrl+S |
| 148 | `$94` | `$14` | Yes | 212 | `$D4` | Inverse Tee up | Inverse form of Ctrl+T |
| 149 | `$95` | `$15` | Yes | 213 | `$D5` | Inverse Left half bar | Inverse form of Ctrl+U |
| 150 | `$96` | `$16` | Yes | 214 | `$D6` | Inverse Lower-left corner | Inverse form of Ctrl+V |
| 151 | `$97` | `$17` | Yes | 215 | `$D7` | Inverse Vertical bar | Inverse form of Ctrl+W |
| 152 | `$98` | `$18` | Yes | 216 | `$D8` | Inverse Diamond | Inverse form of Ctrl+X |
| 153 | `$99` | `$19` | Yes | 217 | `$D9` | Inverse Four-way junction | Inverse form of Ctrl+Y |
| 154 | `$9A` | `$1A` | Yes | 218 | `$DA` | Inverse Filled circle | Inverse form of Ctrl+Z |
| 155 | `$9B` | `$1B` | Yes | 219 | `$DB` | End of line control when interpreted; Inverse Escape in quoted/display-cell context | RETURN |
| 156 | `$9C` | `$1C` | Yes | 220 | `$DC` | Delete line control when interpreted; Inverse Cursor up in quoted/display-cell context | SHIFT-DELETE |
| 157 | `$9D` | `$1D` | Yes | 221 | `$DD` | Insert line control when interpreted; Inverse Cursor down in quoted/display-cell context | SHIFT-INSERT |
| 158 | `$9E` | `$1E` | Yes | 222 | `$DE` | Clear tab stop control when interpreted; Inverse Cursor left in quoted/display-cell context | CTRL-TAB |
| 159 | `$9F` | `$1F` | Yes | 223 | `$DF` | Set tab stop control when interpreted; Inverse Cursor right in quoted/display-cell context | SHIFT-TAB |
| 160 | `$A0` | `$20` | Yes | 128 | `$80` | Inverse Space | Inverse form of Space |
| 161 | `$A1` | `$21` | Yes | 129 | `$81` | Inverse ASCII `!` | Inverse form of Type the character |
| 162 | `$A2` | `$22` | Yes | 130 | `$82` | Inverse ASCII `"` | Inverse form of Type the character |
| 163 | `$A3` | `$23` | Yes | 131 | `$83` | Inverse ASCII `#` | Inverse form of Type the character |
| 164 | `$A4` | `$24` | Yes | 132 | `$84` | Inverse ASCII `$` | Inverse form of Type the character |
| 165 | `$A5` | `$25` | Yes | 133 | `$85` | Inverse ASCII `%` | Inverse form of Type the character |
| 166 | `$A6` | `$26` | Yes | 134 | `$86` | Inverse ASCII `&` | Inverse form of Type the character |
| 167 | `$A7` | `$27` | Yes | 135 | `$87` | Inverse ASCII `'` | Inverse form of Type the character |
| 168 | `$A8` | `$28` | Yes | 136 | `$88` | Inverse ASCII `(` | Inverse form of Type the character |
| 169 | `$A9` | `$29` | Yes | 137 | `$89` | Inverse ASCII `)` | Inverse form of Type the character |
| 170 | `$AA` | `$2A` | Yes | 138 | `$8A` | Inverse ASCII `*` | Inverse form of Type the character |
| 171 | `$AB` | `$2B` | Yes | 139 | `$8B` | Inverse ASCII `+` | Inverse form of Type the character |
| 172 | `$AC` | `$2C` | Yes | 140 | `$8C` | Inverse ASCII `,` | Inverse form of Type the character |
| 173 | `$AD` | `$2D` | Yes | 141 | `$8D` | Inverse ASCII `-` | Inverse form of Type the character |
| 174 | `$AE` | `$2E` | Yes | 142 | `$8E` | Inverse ASCII `.` | Inverse form of Type the character |
| 175 | `$AF` | `$2F` | Yes | 143 | `$8F` | Inverse ASCII `/` | Inverse form of Type the character |
| 176 | `$B0` | `$30` | Yes | 144 | `$90` | Inverse ASCII `0` | Inverse form of Type the character |
| 177 | `$B1` | `$31` | Yes | 145 | `$91` | Inverse ASCII `1` | Inverse form of Type the character |
| 178 | `$B2` | `$32` | Yes | 146 | `$92` | Inverse ASCII `2` | Inverse form of Type the character |
| 179 | `$B3` | `$33` | Yes | 147 | `$93` | Inverse ASCII `3` | Inverse form of Type the character |
| 180 | `$B4` | `$34` | Yes | 148 | `$94` | Inverse ASCII `4` | Inverse form of Type the character |
| 181 | `$B5` | `$35` | Yes | 149 | `$95` | Inverse ASCII `5` | Inverse form of Type the character |
| 182 | `$B6` | `$36` | Yes | 150 | `$96` | Inverse ASCII `6` | Inverse form of Type the character |
| 183 | `$B7` | `$37` | Yes | 151 | `$97` | Inverse ASCII `7` | Inverse form of Type the character |
| 184 | `$B8` | `$38` | Yes | 152 | `$98` | Inverse ASCII `8` | Inverse form of Type the character |
| 185 | `$B9` | `$39` | Yes | 153 | `$99` | Inverse ASCII `9` | Inverse form of Type the character |
| 186 | `$BA` | `$3A` | Yes | 154 | `$9A` | Inverse ASCII `:` | Inverse form of Type the character |
| 187 | `$BB` | `$3B` | Yes | 155 | `$9B` | Inverse ASCII `;` | Inverse form of Type the character |
| 188 | `$BC` | `$3C` | Yes | 156 | `$9C` | Inverse ASCII `<` | Inverse form of Type the character |
| 189 | `$BD` | `$3D` | Yes | 157 | `$9D` | Inverse ASCII `=` | Inverse form of Type the character |
| 190 | `$BE` | `$3E` | Yes | 158 | `$9E` | Inverse ASCII `>` | Inverse form of Type the character |
| 191 | `$BF` | `$3F` | Yes | 159 | `$9F` | Inverse ASCII `?` | Inverse form of Type the character |
| 192 | `$C0` | `$40` | Yes | 160 | `$A0` | Inverse ASCII `@` | Inverse form of Type the character |
| 193 | `$C1` | `$41` | Yes | 161 | `$A1` | Inverse ASCII `A` | Inverse form of Type the character |
| 194 | `$C2` | `$42` | Yes | 162 | `$A2` | Inverse ASCII `B` | Inverse form of Type the character |
| 195 | `$C3` | `$43` | Yes | 163 | `$A3` | Inverse ASCII `C` | Inverse form of Type the character |
| 196 | `$C4` | `$44` | Yes | 164 | `$A4` | Inverse ASCII `D` | Inverse form of Type the character |
| 197 | `$C5` | `$45` | Yes | 165 | `$A5` | Inverse ASCII `E` | Inverse form of Type the character |
| 198 | `$C6` | `$46` | Yes | 166 | `$A6` | Inverse ASCII `F` | Inverse form of Type the character |
| 199 | `$C7` | `$47` | Yes | 167 | `$A7` | Inverse ASCII `G` | Inverse form of Type the character |
| 200 | `$C8` | `$48` | Yes | 168 | `$A8` | Inverse ASCII `H` | Inverse form of Type the character |
| 201 | `$C9` | `$49` | Yes | 169 | `$A9` | Inverse ASCII `I` | Inverse form of Type the character |
| 202 | `$CA` | `$4A` | Yes | 170 | `$AA` | Inverse ASCII `J` | Inverse form of Type the character |
| 203 | `$CB` | `$4B` | Yes | 171 | `$AB` | Inverse ASCII `K` | Inverse form of Type the character |
| 204 | `$CC` | `$4C` | Yes | 172 | `$AC` | Inverse ASCII `L` | Inverse form of Type the character |
| 205 | `$CD` | `$4D` | Yes | 173 | `$AD` | Inverse ASCII `M` | Inverse form of Type the character |
| 206 | `$CE` | `$4E` | Yes | 174 | `$AE` | Inverse ASCII `N` | Inverse form of Type the character |
| 207 | `$CF` | `$4F` | Yes | 175 | `$AF` | Inverse ASCII `O` | Inverse form of Type the character |
| 208 | `$D0` | `$50` | Yes | 176 | `$B0` | Inverse ASCII `P` | Inverse form of Type the character |
| 209 | `$D1` | `$51` | Yes | 177 | `$B1` | Inverse ASCII `Q` | Inverse form of Type the character |
| 210 | `$D2` | `$52` | Yes | 178 | `$B2` | Inverse ASCII `R` | Inverse form of Type the character |
| 211 | `$D3` | `$53` | Yes | 179 | `$B3` | Inverse ASCII `S` | Inverse form of Type the character |
| 212 | `$D4` | `$54` | Yes | 180 | `$B4` | Inverse ASCII `T` | Inverse form of Type the character |
| 213 | `$D5` | `$55` | Yes | 181 | `$B5` | Inverse ASCII `U` | Inverse form of Type the character |
| 214 | `$D6` | `$56` | Yes | 182 | `$B6` | Inverse ASCII `V` | Inverse form of Type the character |
| 215 | `$D7` | `$57` | Yes | 183 | `$B7` | Inverse ASCII `W` | Inverse form of Type the character |
| 216 | `$D8` | `$58` | Yes | 184 | `$B8` | Inverse ASCII `X` | Inverse form of Type the character |
| 217 | `$D9` | `$59` | Yes | 185 | `$B9` | Inverse ASCII `Y` | Inverse form of Type the character |
| 218 | `$DA` | `$5A` | Yes | 186 | `$BA` | Inverse ASCII `Z` | Inverse form of Type the character |
| 219 | `$DB` | `$5B` | Yes | 187 | `$BB` | Inverse ASCII `[` | Inverse form of Type the character |
| 220 | `$DC` | `$5C` | Yes | 188 | `$BC` | Inverse ASCII `\` | Inverse form of Type the character |
| 221 | `$DD` | `$5D` | Yes | 189 | `$BD` | Inverse ASCII `]` | Inverse form of Type the character |
| 222 | `$DE` | `$5E` | Yes | 190 | `$BE` | Inverse ASCII `^` | Inverse form of Type the character |
| 223 | `$DF` | `$5F` | Yes | 191 | `$BF` | Inverse ASCII `_` | Inverse form of Type the character |
| 224 | `$E0` | `$60` | Yes | 224 | `$E0` | Inverse Atari diamond | Inverse form of Load/paste an ATASCII glyph or use an Atari-aware source |
| 225 | `$E1` | `$61` | Yes | 225 | `$E1` | Inverse Lowercase `a` | Inverse form of Type the character |
| 226 | `$E2` | `$62` | Yes | 226 | `$E2` | Inverse Lowercase `b` | Inverse form of Type the character |
| 227 | `$E3` | `$63` | Yes | 227 | `$E3` | Inverse Lowercase `c` | Inverse form of Type the character |
| 228 | `$E4` | `$64` | Yes | 228 | `$E4` | Inverse Lowercase `d` | Inverse form of Type the character |
| 229 | `$E5` | `$65` | Yes | 229 | `$E5` | Inverse Lowercase `e` | Inverse form of Type the character |
| 230 | `$E6` | `$66` | Yes | 230 | `$E6` | Inverse Lowercase `f` | Inverse form of Type the character |
| 231 | `$E7` | `$67` | Yes | 231 | `$E7` | Inverse Lowercase `g` | Inverse form of Type the character |
| 232 | `$E8` | `$68` | Yes | 232 | `$E8` | Inverse Lowercase `h` | Inverse form of Type the character |
| 233 | `$E9` | `$69` | Yes | 233 | `$E9` | Inverse Lowercase `i` | Inverse form of Type the character |
| 234 | `$EA` | `$6A` | Yes | 234 | `$EA` | Inverse Lowercase `j` | Inverse form of Type the character |
| 235 | `$EB` | `$6B` | Yes | 235 | `$EB` | Inverse Lowercase `k` | Inverse form of Type the character |
| 236 | `$EC` | `$6C` | Yes | 236 | `$EC` | Inverse Lowercase `l` | Inverse form of Type the character |
| 237 | `$ED` | `$6D` | Yes | 237 | `$ED` | Inverse Lowercase `m` | Inverse form of Type the character |
| 238 | `$EE` | `$6E` | Yes | 238 | `$EE` | Inverse Lowercase `n` | Inverse form of Type the character |
| 239 | `$EF` | `$6F` | Yes | 239 | `$EF` | Inverse Lowercase `o` | Inverse form of Type the character |
| 240 | `$F0` | `$70` | Yes | 240 | `$F0` | Inverse Lowercase `p` | Inverse form of Type the character |
| 241 | `$F1` | `$71` | Yes | 241 | `$F1` | Inverse Lowercase `q` | Inverse form of Type the character |
| 242 | `$F2` | `$72` | Yes | 242 | `$F2` | Inverse Lowercase `r` | Inverse form of Type the character |
| 243 | `$F3` | `$73` | Yes | 243 | `$F3` | Inverse Lowercase `s` | Inverse form of Type the character |
| 244 | `$F4` | `$74` | Yes | 244 | `$F4` | Inverse Lowercase `t` | Inverse form of Type the character |
| 245 | `$F5` | `$75` | Yes | 245 | `$F5` | Inverse Lowercase `u` | Inverse form of Type the character |
| 246 | `$F6` | `$76` | Yes | 246 | `$F6` | Inverse Lowercase `v` | Inverse form of Type the character |
| 247 | `$F7` | `$77` | Yes | 247 | `$F7` | Inverse Lowercase `w` | Inverse form of Type the character |
| 248 | `$F8` | `$78` | Yes | 248 | `$F8` | Inverse Lowercase `x` | Inverse form of Type the character |
| 249 | `$F9` | `$79` | Yes | 249 | `$F9` | Inverse Lowercase `y` | Inverse form of Type the character |
| 250 | `$FA` | `$7A` | Yes | 250 | `$FA` | Inverse Lowercase `z` | Inverse form of Type the character |
| 251 | `$FB` | `$7B` | Yes | 251 | `$FB` | Inverse Spade | Inverse form of Load/paste an ATASCII glyph or use an Atari-aware source |
| 252 | `$FC` | `$7C` | Yes | 252 | `$FC` | Inverse Vertical bar | Inverse form of Load/paste an ATASCII glyph or use an Atari-aware source |
| 253 | `$FD` | `$7D` | Yes | 253 | `$FD` | Buzzer control when interpreted; Inverse Clear-screen glyph in quoted/display-cell context | CTRL-2 |
| 254 | `$FE` | `$7E` | Yes | 254 | `$FE` | Delete character control when interpreted; Inverse Backspace/delete glyph in quoted/display-cell context | CTRL-DELETE |
| 255 | `$FF` | `$7F` | Yes | 255 | `$FF` | Insert character control when interpreted; Inverse Tab glyph in quoted/display-cell context | CTRL-INSERT |

## Machine-readable data

- [Complete ATASCII ↔ screen-code CSV](../atascii_kit/data/atascii_screen_code_map.csv)
- [Control-code CSV](../atascii_kit/data/atascii_control_codes.csv)
- [ATASCII kit reference](../atascii_kit/docs/ATASCII_REFERENCE.md)
- [Source attribution](../atascii_kit/SOURCES_AND_ATTRIBUTION.md)
- [Printable ASCII/ATASCII chart PDF](../atascii_kit/charts/ascii_atascii_table.pdf)

## Reference sources

The repository's provenance file records the historical/public references used to assemble the kit. Useful human-readable references include:

- [Mapping the Atari, Appendix 10: ATASCII/ANTIC character set](https://www.atariarchives.org/mapping/appendix10.php)
- [AtariWiki ATASCII table](https://atariwiki.org/wiki/Wiki.jsp?page=Atari+ATASCII+Table)
- [Atari Home Computer Technical Reference Notes](https://ftp3.us.freebsd.org/pub/misc/bitsavers/pdf/atari/400_800/CO16555_Atari_Home_Computer_Technical_Reference_Notes_1982.pdf)

QuarterMaster/M's executable behavior is defined by the vendored `atascii` crate and the canonical CSV committed with this project. Historical references are supporting documentation.

## Regenerating this page

```powershell
npm run docs:atascii
```

The generator reads both canonical CSV files and rewrites this reference. Review the Markdown diff whenever canonical mapping data changes.
