# QuarterMaster/M documentation

This is the complete reference set for QuarterMaster/M. The in-app **Help Center** presents the operational material in a searchable desktop format; these Markdown files provide expanded explanations, examples, implementation notes, and links suitable for browsing outside the application.

## Choose a guide

| Document | Use it for |
|---|---|
| [User Guide](USER_GUIDE.md) | Every screen, menu, toolbar control, Explorer action, editor workflow, and status indicator |
| [ATASCII Reference](ATASCII_REFERENCE.md) | All 256 bytes, inverse rules, screen-code conversion, line endings, control codes, and glyph entry |
| [Keyboard and Mouse](KEYBOARD_AND_MOUSE.md) | Complete shortcuts, navigation, rectangular selections, clipboard, and inverse editing |
| [ATR Guide](ATR_GUIDE.md) | D1:–D4:, mounting, browsing, creation, geometry, drag/drop, import/export, and filesystem limits |
| [Atari BASIC Guide](BASIC_GUIDE.md) | Tokenized programs, listings, host/ATR operations, syntax requirements, and error handling |
| [File Formats](FILE_FORMATS.md) | Exact encodings and conversion behavior for ATASCII, ASCII, BASIC, and ATR data |
| [Troubleshooting](TROUBLESHOOTING.md) | Symptom-based diagnosis, recovery, support URL, and issue-report checklist |
| [Building and Releases](BUILDING_AND_RELEASES.md) | Development EXE workflow, production installers, requirements, versioning, and packaging |
| [Architecture](ARCHITECTURE.md) | Frontend/backend boundaries, data flow, persistence, vendored crates, and source map |
| [Contributing](CONTRIBUTING.md) | Contribution workflow, coding expectations, documentation standards, and issue etiquette |
| [Screenshots](SCREENSHOTS.md) | Current application images and instructions for refreshing documentation captures |

Editable screenshot source screens live in [`docs/examples/`](examples/); they include real `.ata` files and human-editable 40×24 JSON layouts.

## Recommended reading paths

### I want to edit an Atari screen

1. [User Guide: First session](USER_GUIDE.md#first-session)
2. [Keyboard and Mouse](KEYBOARD_AND_MOUSE.md)
3. [ATASCII Reference](ATASCII_REFERENCE.md)
4. [File Formats: choosing ATASCII or ASCII](FILE_FORMATS.md#choosing-a-document-mode)

### I want to work inside ATR images

1. [ATR Guide: mental model](ATR_GUIDE.md#mental-model)
2. [ATR Guide: mounting and browsing](ATR_GUIDE.md#mount-and-browse)
3. [ATR Guide: file operations](ATR_GUIDE.md#file-and-directory-operations)
4. [ATR Guide: creating disks](ATR_GUIDE.md#create-an-atr-image)

### I want to move Atari BASIC programs

1. [Atari BASIC Guide](BASIC_GUIDE.md)
2. [File Formats: tokenized BASIC](FILE_FORMATS.md#tokenized-atari-basic)
3. [ATR Guide: drag/drop conversion](ATR_GUIDE.md#drag-and-drop)

### I am building or packaging the application

1. [Building and Releases](BUILDING_AND_RELEASES.md)
2. [Architecture](ARCHITECTURE.md)
3. [Contributing](CONTRIBUTING.md)

## Definitions used throughout the manual

- **Active location:** the Local or D1:–D4: directory selected in Explorer; the destination used by New, Open, and Save.
- **ATASCII:** Atari's 8-bit character/control encoding. QuarterMaster/M uses `$9B` for end-of-line.
- **Base glyph:** an ATASCII glyph with bit 7 clear (`$00–$7F`).
- **Inverse glyph:** the same base glyph with bit 7 set (`$80–$FF`) in display-cell context.
- **Screen code:** the byte value stored in Atari display memory; it is not the same ordering as ATASCII.
- **ATR:** a sector-level image of an Atari disk or partition, beginning with a 16-byte ATR header.
- **Listing:** human-readable Atari BASIC source text with line numbers.
- **Tokenized BASIC:** Atari BASIC's binary saved-program representation.
- **Raw extraction:** byte-for-byte copying without text or BASIC conversion.

## Getting help

Use **Help → Get Help / Report Issue** in the application, or visit:

https://github.com/rickcollette/quartermaster-m/issues

The [Troubleshooting guide](TROUBLESHOOTING.md) explains what diagnostic details to include.
