# Architecture

[Documentation index](README.md) · [Building](BUILDING_AND_RELEASES.md) · [Contributing](CONTRIBUTING.md)

## Overview

QuarterMaster/M is a Tauri 2 desktop application:

```text
Windows / WebView2
        │
        ▼
TypeScript + HTML + CSS frontend
        │ Tauri commands / serialized DTOs
        ▼
Rust application backend
   ├── document codec
   ├── ATR/file manager
   ├── native Atari BASIC converter
   ├── vendored ATASCII crate
   └── vendored Broadside filesystem crate
        │
        ▼
Host files and ATR sector images
```

The frontend owns interaction state and rendering. Rust owns file I/O, disk-image mutation, persistent mount state, document serialization, and BASIC binary conversion.

## Frontend

### `src/main.ts`

The `Editor` class coordinates:

- application shell/menu/toolbar rendering;
- 40/80-column, 357-row cell buffer;
- caret, insert/overwrite, inverse typing;
- rectangular selections and internal clipboard;
- Local and ATR Explorer trees;
- four virtual drives and active-location semantics;
- drag/drop;
- Tauri command dispatch;
- location-aware dialogs;
- activity overlay;
- disk creation form;
- BASIC menu workflows.

Long-running Tauri commands pass through an invocation wrapper that maps command names to user-facing activity labels.

### `src/help.ts`

Defines the in-app Help Center:

- topic navigation;
- complete operational manual;
- searchable/filterable 256-byte ATASCII mapping;
- glyph previews using generated Atari assets;
- issue tracker URL/copy action;
- modal keyboard and accessibility behavior.

The ATASCII table creates 128 base/inverse rows, representing all 256 byte values.

### `src/atariGlyphs.ts`

Maps seven-bit values to generated CSS classes/labels and translates every plain Ctrl+letter key event to a low ATASCII graphic code. Application character shortcuts use Ctrl+Shift+letter.

### `src/generated/`

Generated SVG/CSS glyph assets. `scripts/generate_glyph_assets.mjs` uses the project's Atari charset data/source and produces pixel-preserving CSS backgrounds.

### `src/style.css`

Defines:

- Atari-inspired visual theme;
- shell/menu/toolbar/workbench;
- Explorer and drag states;
- fixed-cell editor and inverse/selection/caret states;
- activity and disk-configuration dialogs;
- full Help Center layout and mapping table.

### `src/types.ts`

Shared frontend shapes for documents, cells, ATR status, drives, and trees. Serde's camelCase convention keeps Rust DTOs compatible.

## Native backend

### `src-tauri/src/lib.rs`

Constructs Tauri, installs the dialog plugin, manages `AtrState`, exposes the command set, and coordinates splash-to-main window transition through `app_ready`.

### `src-tauri/src/main.rs`

Thin binary entry point. Release builds use the Windows GUI subsystem to avoid a console window.

### `src-tauri/src/document.rs`

Implements:

- ATASCII and ASCII modes;
- loaded/saved cell DTOs;
- width/height validation;
- parser-driven ATASCII decoding;
- ASCII newline normalization;
- inverse-aware ATASCII encoding;
- CRLF ASCII encoding;
- host ASCII → ATASCII import;
- ATASCII → ASCII export.

Dimensions are backend-bounded at width 1–256 and height 1–4096; the application currently requests 40/80 × 357.

### `src-tauri/src/atr.rs`

Implements:

- four-drive state;
- supported filesystem detection;
- persisted mount/local paths;
- directory-tree DTOs;
- ATR creation/mount/select/close/status;
- local tree;
- read/write/add/import/copy/extract/export;
- delete/rename/mkdir;
- guarded/rollback-aware image mutation.

Mount state is stored in the platform application-data directory as `virtual-drive.json`.

### `src-tauri/src/basic.rs`

Adapts editor documents and ATR files to native BASIC operations, including host and ATR destinations.

### `src-tauri/src/basic_native.rs`

Contains the saved-program tokenizer/detokenizer and an internal CLI entrypoint retained for development use. The GUI calls library functions directly; it does not spawn the CLI.

### `src-tauri/src/version.rs`

Generated Rust version constant.

## Vendored crates

### `vendor/atascii`

The canonical parser and glyph/control model:

- `AtasciiByte`;
- standard charset/glyph IDs;
- inverse handling;
- text-file decode domain;
- control enum;
- ATASCII/screen-code conversion.

Vendoring ensures the UI and Rust document backend share the project's tested semantics.

### `vendor/broadside-core`

Provides ATR image and Atari filesystem operations used by the backend, including DOS 2 and SpartaDOS structures.

## Canonical ATASCII kit

`atascii_kit/` contains:

- complete round-trip CSV;
- control-code CSV;
- charts and historical reference scans;
- source attribution;
- kit-specific technical documentation.

`scripts/generate_atascii_docs.mjs` uses these data files to create the exhaustive Markdown reference.

## Important data flows

### Local file → editor

```text
Windows path
  → load_document
  → bytes
  → ATASCII parser or ASCII decoder
  → LoadedDocument DTO
  → frontend cell grid
```

### Editor → ATR file

```text
frontend cell grid
  → SaveDocumentRequest
  → encode_document_bytes
  → atr_write_document
  → filesystem mutation
  → updated AtrStatus/tree
```

### Host drop → ATR

```text
host path
  ├── .BAS listing → tokenize_listing → binary bytes
  └── other text   → ascii_text_to_atascii
  → add file bytes
  → updated directory
```

### ATR → ATR

```text
source filesystem extract bytes
  → unchanged byte vector
  → destination filesystem add bytes
```

### Tokenized BASIC → editor

```text
binary saved program
  → validate header/tables/records
  → detokenized ASCII listing
  → document ASCII decoder
  → cell grid
```

## Persistence

Only paths and drive selection are persisted. ATR contents remain in their host `.atr` files. The editor document itself is not an autosaved database. Users must save modified documents.

## Error strategy

Rust commands return `Result<T, String>` with contextual operation/path errors. The frontend catches these, ends activity state, and displays an alert. Mutating ATR operations attempt to preserve/restore the original image where the backend operation uses its guarded mutation path.

## Security boundaries

- Native file selection uses the Tauri dialog plugin.
- The frontend has no arbitrary Node.js filesystem access.
- External content is rendered as text in trees/dialogs.
- Help HTML is static project-owned content.
- Unknown binary files should use raw extraction; text conversion is explicit.

## Test areas

Current Rust tests cover key document and BASIC behavior, including:

- `$9B` ATASCII EOL;
- inverse save;
- ASCII/ATASCII drag conversion;
- native BASIC simple round trip;
- numeric decoding;
- variable-name tables;
- statement splitting;
- ATR/backend invariants in their respective modules.

Frontend build type-checking and screenshot smoke checks complement native tests.
