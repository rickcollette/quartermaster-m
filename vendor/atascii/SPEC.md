# ATASCII Rust Crate Specification

**Status:** Draft reference specification 0.2
**Crate:** `atascii`
**Scope:** Atari 400/800/XL/XE ATASCII, ASCII interoperability, screen/internal codes, editor streams, and terminal behavior.

## 1. Purpose

The crate SHALL provide a lossless, panic-free model for every byte from `0x00` through `0xFF`. It SHALL NOT pretend ATASCII is merely a Unicode encoding. It SHALL keep these domains distinct:

1. Raw ATASCII interchange bytes.
2. Atari E: screen-editor controls.
3. Quoted graphical renderings of bytes that normally act as controls.
4. The 128 ROM glyph identities and inverse-video attribute.
5. Atari internal/screen codes.
6. Keyboard scan/key codes, which are explicitly outside the initial encoding API.
7. ASCII and Unicode translations, which are policy-driven and potentially lossy.

## 2. Normative terminology

`MUST`, `MUST NOT`, `SHOULD`, and `MAY` are normative. “Byte” means an unsigned 8-bit value. “Base code” means bits 0–6. “Inverse” means bit 7 is set in a text mode where bit 7 selects inverse video.

## 3. Required correctness rules

- All 256 values MUST round-trip through `AtasciiByte`.
- Bit 7 MUST be retained independently of the base glyph identity.
- `0x9B` MUST represent ATASCII EOL in editor-stream mode.
- The parser MUST recognize controls at `1B–1F`, `7D–7F`, `9B–9F`, and `FD–FF`.
- ESC MUST quote the following editor-control byte as a printable glyph, except `0x9B`, which remains EOL.
- Raw mode MUST never interpret bytes.
- Glyph mode MUST render all bytes as glyph requests and never perform editor actions.
- Unicode conversion MUST be documented as an approximation. Exact visual fidelity requires original 8x8 bitmap glyph data.
- ASCII conversion MUST translate LF/CR/EOL under an explicit newline policy; it MUST NOT silently treat ASCII CR (`0x0D`) as ATASCII EOL.

## 4. Public data model

### 4.1 `AtasciiByte`

Transparent `u8` newtype. It exposes base-code and inverse-bit operations and cannot be invalid.

### 4.2 `Glyph`

Contains `GlyphId(0..127)`, `inverse`, and `Charset::{Standard,International}`. A glyph is a display request, not a Unicode scalar.

### 4.3 `Control`

Represents the sixteen Atari editor operations:

| Byte | Meaning |
|---:|---|
| `1B` | Escape |
| `1C` | Cursor up |
| `1D` | Cursor down |
| `1E` | Cursor left |
| `1F` | Cursor right |
| `7D` | Clear screen |
| `7E` | Delete/backspace |
| `7F` | Tab |
| `9B` | End of line |
| `9C` | Delete line |
| `9D` | Insert line |
| `9E` | Clear tab stop |
| `9F` | Set tab stop |
| `FD` | Buzzer |
| `FE` | Delete character |
| `FF` | Insert character |

### 4.4 `Token`

`Glyph`, `Control`, or `Raw`. This is the boundary between byte parsing and terminal/editor behavior.

## 5. Parser modes

- `ScreenEditor`: stateful E: semantics and ESC quoting.
- `TextFile`: recognizes native `0x9B` EOL but does not execute other editor controls.
- `RawGlyphs`: every byte becomes a glyph. Intended for charts, fonts, asset editors, and binary dumps.
- `RawBytes`: every byte is preserved without interpretation.

The parser MUST support incremental network input, including an ESC at the end of one packet and its quoted byte in the next packet.


## 5.1 Shared use by editor and terminal projects

The crate SHALL remain application-framework independent so the same version can be included by both an ATASCII Editor Project and an ATASCII Terminal Project.

The editor integration SHALL use raw or file-domain APIs and SHALL NOT inherit a terminal connection profile implicitly:

```text
file bytes -> Parser(TextFile or RawGlyphs) -> editor document -> renderer
```

The terminal integration SHALL use transport normalization and screen-editor parsing:

```text
TCP/serial bytes -> Telnet/SSH layer -> TerminalDecoder -> Token stream -> screen/event model
```

Neither application may maintain a private replacement ATASCII table. Shared byte, control, screen-code, charset, glyph, and translation behavior belongs in this crate. Application-specific UI, storage, Telnet, SSH, serial, and rendering code remains outside it.

## 6. Character sets

The crate MUST support both standard and XL/XE international ROM sets. Version 0.1 exposes the selector and standard approximations; the production 1.0 milestone MUST include audited 128-entry tables for both sets and optionally licensed/redistributable 1024-byte ROM-compatible bitmap assets.

A Unicode table is metadata only. It MUST permit one-to-many aliases and “no exact Unicode equivalent.” The canonical glyph identity remains the 7-bit Atari code.

## 7. Screen/internal code conversion

ATASCII and Atari screen/internal codes MUST be separate newtypes. Conversion SHALL preserve inverse bit 7. For the base code:

```text
ATASCII 00–1F -> screen 40–5F
ATASCII 20–5F -> screen 00–3F
ATASCII 60–7F -> screen 60–7F
```

The reverse mapping SHALL be exact.

## 8. Terminal screen model

The included reference `Screen` models a rectangular character-cell display with cursor, tab stops, line insertion/deletion, character insertion/deletion, clear, wrap, and scroll. Applications MAY replace it with their own renderer while reusing the parser.

Required behavior:

- Default tab stops every eight columns.
- EOL returns to column zero and advances/scrolls.
- Cursor motion clamps at edges.
- Character insertion shifts the current row right.
- Character deletion shifts the current row left.
- Line insertion/deletion affects rows at and below the cursor.
- Buzzer is surfaced as an event in a future event-sink API; it does not alter cells.

## 9. ASCII interoperability

ASCII is 7-bit and does not contain ATASCII graphics or editor commands. Translation MUST therefore expose policy:

- `Strict`: fail on unrepresentable input.
- `Replace`: use `?`.
- `GraphicsApprox`: map recognized Unicode line/block/suit/arrow characters to Atari glyphs.

Recommended newline policies for a future API:

- `AtariEol`: `\n` -> `0x9B`; ignore `\r` in CRLF.
- `Preserve`: no newline rewriting.
- `AsciiCrLf`: `0x9B` -> `\r\n`.
- `UnixLf`: `0x9B` -> `\n`.

No API may call Windows-1252 “extended ASCII.”

## 10. Streaming and BBS use

The crate MUST be safe for Telnet, SSH, serial, WebSocket, and captured-file streams. Transport negotiation belongs outside this crate. Telnet IAC unescaping MUST happen before ATASCII parsing. Baud throttling belongs outside this crate. The parser must never assume packet boundaries equal character or command boundaries.

A BBS terminal should use this pipeline:

```text
transport bytes -> Telnet/SSH protocol layer -> ATASCII Parser -> Token stream
-> Screen/event model -> bitmap or GPU renderer
```

Outbound input should use:

```text
host key event -> Atari key mapping -> ATASCII byte/editor command -> transport layer
```

Keyboard scan codes are not interchangeable with ATASCII and require a separate module.

## 11. Fidelity levels

1. **Lossless bytes:** exact.
2. **Editor semantics:** exact for documented E: operations.
3. **Glyph identity and inverse:** exact.
4. **Unicode text:** approximate and potentially lossy.
5. **Bitmap rendering:** exact only when an authentic/audited character bitmap is supplied.
6. **ANTIC modes 1/2:** require a separate color-bit interpretation because upper bits select color rather than simple inverse.

## 12. Security and robustness

- No unsafe Rust.
- No panics for arbitrary input.
- Bounded screen allocations validated at construction.
- Fuzz parser, translator, and screen operations with arbitrary byte sequences.
- Avoid recursive parsing.
- Translation errors report the offending scalar without truncation.
- Optional `no_std + alloc` support.

## 13. Test requirements

Before 1.0, the suite MUST include:

- Exhaustive 256-byte raw round trips.
- Exhaustive screen-code round trips.
- Every control in escaped and unescaped form.
- ESC split across chunks.
- EOL after ESC.
- Standard and international audited mapping fixtures.
- Inverse forms of all 128 glyphs.
- Golden 40x24 screen captures.
- Real ATASCII animation/BBS fixtures with permission or hashes.
- ASCII CR, LF, CRLF, and mixed newline tests.
- Property/fuzz tests proving no panic.

## 14. Source tree

```text
atascii/
├── Cargo.toml
├── SPEC.md
├── README.md
├── src/
│   ├── lib.rs
│   ├── byte.rs
│   ├── charset.rs
│   ├── control.rs
│   ├── parser.rs
│   ├── screen.rs
│   ├── screen_code.rs
│   └── translate.rs
├── tests/conformance.rs
├── examples/dump.rs
├── docs/
├── benches/
└── fuzz/fuzz_targets/
```

## 15. Planned workspace expansion

For a production ecosystem, split into:

- `atascii-core`: no_std bytes, glyph IDs, tables, parser.
- `atascii-screen`: editor state machine and screen buffer.
- `atascii-codec`: ASCII/Unicode/file codecs.
- `atascii-font`: audited 8x8 bitmap loading and rasterization.
- `atascii-terminal`: key mapping, event sink, terminal profiles.
- `atascii-cli`: convert, inspect, render PNG/text, validate captures.
- `atascii-wasm`: browser bindings.

## 16. Explicit non-goals for core

The core crate does not implement Telnet, SSH, Tauri, GPU rendering, serial ports, ANSI/VT100, X/Y/ZMODEM, or Atari keyboard hardware scanning. Those layers consume or produce this crate’s types.

## 17. Acceptance criteria for 1.0

1. Every byte and inverse glyph round-trips.
2. Both Atari ROM character sets have independently audited tables.
3. Editor parser matches Atari E: behavior on conformance captures.
4. Screen-code conversion is exhaustive.
5. Translation loss is always selected by policy, never hidden.
6. Public API is no_std-capable and contains no unsafe code.
7. At least one terminal and one file converter integrate the crate without private mapping tables.

## 18. Version 0.2 integration acceptance tests

- Native editor file mode treats `0x0D`, `0x0A`, `0x08`, and `0x07` as glyph bytes.
- Terminal CRLF mode collapses a CR/LF pair to one `0x9B`, including when split across input chunks.
- A dangling CR in strict CRLF mode is preserved at end of stream.
- Native Backspace transmits `0x7E`; native Return transmits `0x9B`.
- ASCII-compatible Backspace, DEL, CR, LF, and CRLF are explicit profile choices.
- ESC state survives chunk boundaries.
- Editor and terminal examples compile against the same public crate API.
