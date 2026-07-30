# Contributing to QuarterMaster/M

[Documentation index](README.md) · [Architecture](ARCHITECTURE.md) · [Building](BUILDING_AND_RELEASES.md)

## Start with an issue

For bugs, feature proposals, and documentation corrections:

https://github.com/rickcollette/quartermaster-m/issues

Search existing issues first. For behavioral changes, describe the Atari-era compatibility expectation and the modern desktop interaction expectation separately.

## Development setup

```powershell
npm install
npm run tauri dev
```

Read [Building and Releases](BUILDING_AND_RELEASES.md) for Rust/Tauri prerequisites and the fast development EXE workflow.

## Principles

1. Preserve native Atari bytes when the user chose a raw/native operation.
2. Make conversion explicit and document what is lost.
3. Keep New/Open/Save aligned with the visible active location.
4. Prefer a real GUI over browser prompt chains for multi-field workflows.
5. Keep the ATR menu small; put detailed filesystem operations in Explorer.
6. Report activity during file/disk/BASIC operations.
7. Do not open a console window for release GUI builds.
8. Keep development verification proportional; reserve full package audits for release.
9. Back documentation claims with implemented behavior and canonical mapping data.
10. Treat disk images as valuable mutable binary artifacts—favor guarded writes and useful errors.

## Code map

- Frontend interaction: `src/main.ts`
- In-app documentation: `src/help.ts`
- Theme/layout: `src/style.css`
- Frontend types: `src/types.ts`
- Glyph helpers: `src/atariGlyphs.ts`
- Document codec: `src-tauri/src/document.rs`
- ATR operations: `src-tauri/src/atr.rs`
- BASIC integration: `src-tauri/src/basic.rs`
- BASIC codec: `src-tauri/src/basic_native.rs`
- ATASCII core: `vendor/atascii`
- ATR/filesystems: `vendor/broadside-core`

## Change workflow

1. Reproduce or define the expected behavior.
2. Identify the authoritative layer (frontend state, codec, filesystem, BASIC).
3. Make the smallest cohesive change.
4. Update in-app help when user-visible behavior changes.
5. Update the relevant `docs/*.md`.
6. Run proportional checks.
7. Build/stage a no-bundle EXE for interactive testing.
8. Add/refresh screenshots when the visual UI changes materially.

## Checks

Frontend:

```powershell
npm run build
```

Rust:

```powershell
Push-Location src-tauri
cargo test
Pop-Location
```

Generated ATASCII documentation:

```powershell
npm run docs:atascii
```

Development executable:

```powershell
npm run tauri -- build --no-bundle
Copy-Item src-tauri\target\release\quartermaster-m.exe packages\win64\quartermaster-m.exe -Force
```

Do not require MSI/NSIS creation for ordinary frontend/backend iteration.

## ATASCII changes

The canonical maps are:

- `atascii_kit/data/atascii_screen_code_map.csv`
- `atascii_kit/data/atascii_control_codes.csv`

When changing mapping behavior:

1. verify the historical/source attribution;
2. update canonical data if appropriate;
3. update/test `vendor/atascii`;
4. regenerate documentation;
5. verify glyph rendering in normal and inverse mode;
6. add a focused test for byte/domain semantics.

Remember that display-cell inverse interpretation and screen-editor control interpretation are context-dependent, especially `$9B–$9F` and `$FD–$FF`.

## Documentation standard

User-visible changes are incomplete until:

- in-app Help Center is correct;
- relevant long-form document is correct;
- tables/examples use current UI labels;
- destructive/lossy conversions state what is lost;
- screenshots do not expose private paths/data;
- links use repository-relative destinations where possible;
- the issue tracker URL remains current.

Generated documents should identify their generator and source data.

## Screenshot standard

- Use the actual current UI.
- Show a meaningful ATASCII screen, not a blank editor.
- Capture at a readable Windows-like application size.
- Include at least one normal editing view and one selection/action view.
- Avoid personal paths, disk names, or content unless intentionally sanitized.
- Store optimized PNGs under `docs/images/`.
- Update README thumbnails and `docs/SCREENSHOTS.md`.

## Version changes

Do not edit all version files manually. Use:

```powershell
python scripts\version.py --set MAJOR.MINOR.PATCH
```

or:

```powershell
python scripts\version.py --bump-patch
```

Development-only iterations do not automatically require a version bump.

## Commit hygiene

- Keep generated build outputs out of source control.
- Commit `package-lock.json` and Rust lockfiles used by the application.
- Commit vendored/path dependencies required to build.
- Do not commit `.env`, editor state, logs, package staging, or local ATR experiments unless they are deliberate test fixtures.
- Avoid mixing unrelated refactors with a behavioral fix.

## Reporting security-sensitive issues

The public tracker is appropriate for normal application defects. Do not attach confidential disk images, credentials, private paths, or licensed software. Redact artifacts and describe how maintainers can reproduce with synthetic data.
