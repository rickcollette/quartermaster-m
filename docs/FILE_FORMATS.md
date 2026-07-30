# File formats and conversion behavior

[Documentation index](README.md) · [ATASCII reference](ATASCII_REFERENCE.md) · [ATR guide](ATR_GUIDE.md) · [BASIC guide](BASIC_GUIDE.md)

## Choosing a document mode

The toolbar mode selector changes both loading interpretation and saved output.

| Need | Choose |
|---|---|
| Preserve Atari graphics/inverse and `$9B` lines | ATASCII |
| Read/edit the file in ordinary Windows tools | ASCII |
| Archive unknown content without interpretation | Extract Raw |
| Run a program with Atari BASIC `LOAD` | Tokenized BASIC command |
| Move native content between ATRs | ATR-to-ATR drag |

## ATASCII documents

### Cell model

Each editor cell is represented by:

```text
base code = byte & $7F
inverse   = bit 7
```

On save:

```text
output byte = base code | ($80 when inverse)
```

### Lines

ATASCII end-of-line is `$9B`. QuarterMaster/M emits it between serialized editor rows. Ordinary trailing spaces are trimmed per row; inverse spaces remain because they are visible.

### Loading

The backend uses the vendored ATASCII parser in TextFile/Standard-charset mode:

- EndOfLine advances to the first column of the next row.
- Glyph tokens populate cells.
- Other screen-editor controls and raw/non-display tokens are not inserted.
- Reaching the selected width wraps to the next row.
- Content beyond 357 rows is not loaded.

### Important `$9B` ambiguity

Bitwise, `$9B` is `$1B | $80`, which can describe an inverse Escape glyph in a display-cell context. In an ATASCII text-file stream, `$9B` is the end-of-line control. A program that needs to quote/control-display such a value must define its own transport convention; ordinary QuarterMaster/M text save uses `$9B` as the row separator.

## ASCII documents

### Loading

- UTF-8 BOM is tolerated during host-to-ATASCII import.
- CRLF and lone CR normalize to LF.
- LF advances to the next row.
- ASCII characters populate cells.
- Unsupported Unicode is replaced with `?` and produces a warning.

### Saving

- Base ASCII bytes are written.
- Inverse state is not representable and is removed.
- Unsupported/non-ASCII cell values become `?`.
- Rows are separated by CRLF.
- Ordinary trailing spaces are trimmed.

## Text import into ATR

The host-text conversion is byte-oriented:

| Host input | ATASCII output |
|---|---|
| UTF-8 BOM | Removed |
| CRLF | `$9B` |
| CR | `$9B` |
| LF | `$9B` |
| Tab | `$7F` |
| `$20–$7E` | Preserved |
| Other bytes | `?` |

This converter is intentionally conservative. It does not transliterate arbitrary Unicode into Atari graphics.

## ATASCII export to Windows text

| ATASCII input | ASCII output |
|---|---|
| `$9B` | CRLF |
| `$7F` | Tab |
| Printable base `$20–$7E` | Same byte, with inverse bit removed |
| Other graphics/control bytes | Omitted |

This makes prose/listings readable but is not reversible. Keep the native file when graphics or inverse state matters.

## Tokenized Atari BASIC

A tokenized saved program is not a text encoding. QuarterMaster/M's native representation includes:

- seven 16-bit header/pointer words;
- variable-name table;
- variable-value area;
- statement table;
- line numbers and encoded record lengths;
- statement/expression tokens;
- six-byte Atari BCD constants;
- immediate-line/end marker.

Use BASIC menu operations to convert. Normal ATASCII/ASCII Open is not a substitute for detokenization.

## ATR image structure

An ATR contains:

- a 16-byte image header;
- sector data;
- filesystem allocation/directory metadata;
- user file bytes.

QuarterMaster/M supports DOS 2 and SpartaDOS 2 filesystem structures through the vendored `broadside-core` crate.

### Sector sizes

Single/enhanced-density layouts use 128-byte sectors. Double-density layouts use 128 bytes for boot sectors 1–3 and 256 bytes afterward. The ATR header describes the image's paragraph count and sector size.

### Capacity arithmetic

For a 256-byte-sector ATR with three 128-byte boot sectors:

```text
sector data = 3 × 128 + (sector_count − 3) × 256
ATR file    = 16-byte header + sector data
```

Filesystem structures consume part of sector data, so user-available free space is lower.

## Screen codes versus ATASCII

Atari display memory uses screen codes. Base conversion:

```text
ATASCII $00–$1F -> screen code +$40
ATASCII $20–$5F -> screen code -$20
ATASCII $60–$7F -> unchanged
```

Preserve bit 7 for inverse. See the complete table in [ATASCII Reference](ATASCII_REFERENCE.md).

## Extension conventions

Extensions are hints, not guarantees:

| Extension | Common meaning |
|---|---|
| `.ATA` | ATASCII text/screen |
| `.TXT` | ASCII or sometimes ATASCII text |
| `.LST` | BASIC listing, ASCII or ATASCII |
| `.BAS` | Usually tokenized Atari BASIC on Atari media; often a text listing on a host |
| `.ATR` | ATR sector image |
| `.XEX`, `.COM`, no extension | Often binary/executable; raw extraction recommended |

Use content-aware commands and preserve unknown files raw.

## Conversion preservation matrix

| Operation | Base text | Line endings | Inverse | Graphics | Binary |
|---|---|---|---|---|---|
| ATASCII editor save | Preserved | `$9B` | Preserved | Preserved | Not a general binary editor |
| ASCII editor save | ASCII | CRLF | Lost | `?`/not preserved | No |
| Host text → ATR | Printable ASCII | `$9B` | Not introduced | Unsupported → `?` | No |
| ATR Export ASCII | Printable ASCII | CRLF | Removed | Omitted | No |
| Extract Raw | Exact | Exact | Exact | Exact | Exact |
| ATR → ATR drag | Exact | Exact | Exact | Exact | Exact |
| BASIC tokenize | Parsed source | Binary records | Syntax-dependent | Syntax-dependent | Produces BASIC binary |
| BASIC detokenize | Readable source | Editor rows | Not a byte round trip | Unknowns marked/approximated | Consumes BASIC binary |

## Round-trip recommendations

- For exact preservation: raw extraction or ATR-to-ATR copy.
- For Atari screen editing: ATASCII mode.
- For diffs and collaboration: ASCII export plus native original.
- For BASIC: keep an ASCII master listing and regenerate tokenized output.
- For historic ATRs: edit copies, never the sole original.
