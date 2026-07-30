# Building and release packaging

[Documentation index](README.md) · [Architecture](ARCHITECTURE.md) · [Contributing](CONTRIBUTING.md) · [Troubleshooting](TROUBLESHOOTING.md)

## Supported development environment

The active target is Windows x64.

Install:

- Node.js and npm;
- stable Rust with the MSVC target;
- Microsoft Visual C++ Build Tools and Windows SDK required by Tauri;
- Microsoft Edge WebView2 Runtime;
- Python 3 for version/package helper scripts;
- optional GNU Make if using the Makefile.

## Install dependencies

```powershell
npm install
```

The Rust dependencies are resolved by Cargo during the first Tauri build. The project uses local path dependencies under `vendor/`.

## Hot-reload development

```powershell
npm run tauri dev
```

This:

1. regenerates Atari glyph assets;
2. starts Vite on the configured development URL;
3. starts a debug Tauri application;
4. reloads frontend changes.

Debug behavior is not identical to a release executable, especially the Windows console subsystem attribute.

## Frontend-only build

```powershell
npm run build
```

This runs:

```text
npm run glyphs
tsc
vite build
```

Output goes to `dist/`. It verifies TypeScript and frontend bundling but does not update the native EXE.

## Development EXE: preferred current workflow

During active development, build only the runnable application and disable installer bundling:

```powershell
npm run tauri -- build --no-bundle
```

The native release executable is:

```text
src-tauri\target\release\quartermaster-m.exe
```

Stage the testable copy:

```powershell
New-Item -ItemType Directory -Force packages\win64 | Out-Null
Copy-Item src-tauri\target\release\quartermaster-m.exe packages\win64\quartermaster-m.exe -Force
```

Run:

```powershell
.\packages\win64\quartermaster-m.exe
```

This EXE is fully runnable on a development machine with WebView2 already installed. It is not a self-installing bare-machine distribution.

### Why no command window appears

`src-tauri/src/main.rs` selects the Windows GUI subsystem in non-debug builds:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

Use a release build when testing this behavior.

## Lightweight development checks

Use checks proportional to the change:

```powershell
npm run build
```

For Rust/backend changes:

```powershell
Push-Location src-tauri
cargo test
Pop-Location
```

For mapping documentation:

```powershell
npm run docs:atascii
```

Full installer audits, offline-runtime payload checks, and package hashes are intentionally deferred until a production release.

## Versioning

The canonical format is:

```text
MAJOR.MINOR.PATCH
```

Example:

```text
1.0.29
```

The version synchronizer updates:

- `VERSION`;
- `package.json`;
- `package-lock.json`;
- `src-tauri/Cargo.toml`;
- `src-tauri/tauri.conf.json`;
- `src/version.ts`;
- `src-tauri/src/version.rs`.

Synchronize current value:

```powershell
python scripts\version.py
```

Bump the patch version:

```powershell
python scripts\version.py --bump-patch
```

Set an explicit release version:

```powershell
python scripts\version.py --set 1.1.0
```

Do not bump the version merely for an intermediate local test unless that is the project's chosen release practice.

## Production release workflow

Production packaging is intentionally more extensive than development:

1. choose and synchronize release version;
2. clean old package staging;
3. run frontend/Rust checks and tests appropriate for release;
4. run the Tauri bundle build;
5. confirm MSI and NSIS outputs;
6. include/audit the offline x64 WebView2 runtime;
7. audit adjacent runtime DLL dependencies;
8. verify Windows GUI subsystem;
9. stage installers, portable EXE, icon, dependency report, and hashes;
10. generate `current-version` from the staged versioned EXE and MSI names;
11. push source and the manifest, then publish the matching files as assets on the `vMAJOR.MINOR.PATCH` GitHub release;
12. smoke-test on a representative clean Windows VM.

Build Tauri bundles:

```powershell
npm run tauri build
```

Or use the Make target, which bumps BUILD first:

```text
make build
```

Stage and audit the existing production build:

```powershell
python scripts\package_win64.py
```

The package helper expects the NSIS and MSI bundle intermediates to exist. It is not the development EXE staging command.

## Application updates

**Help → Check for Updates** retrieves:

```text
https://raw.githubusercontent.com/rickcollette/quartermaster-m/refs/heads/main/current-version
```

The two-line manifest uses:

```text
VERSION:exe:PORTABLE_FILENAME.exe
VERSION:msi:INSTALLER_FILENAME.msi
```

Both entries must use the same semantic `MAJOR.MINOR.PATCH` version. The package helper writes this file from the actual staged filenames.

Update assets are downloaded from the matching GitHub release tag:

```text
https://github.com/rickcollette/quartermaster-m/releases/download/vVERSION/FILENAME
```

Therefore every published manifest must have a corresponding `vVERSION` release containing both files. The portable EXE is written beside the currently running executable and is not launched automatically. The MSI is downloaded to the user's temporary QuarterMaster/M update folder and launched through Windows Installer.

The native updater validates semantic versions and filenames, rejects path components, rechecks the manifest before downloading, uses fixed HTTPS hosts, and refuses to overwrite the running executable.

## WebView2 and bare machines

The Tauri bundle configuration uses:

```json
"webviewInstallMode": {
  "type": "offlineInstaller",
  "silent": true
}
```

A production installer should therefore carry the x64 offline WebView2 bootstrap payload and install it silently when missing. The package audit checks that both MSI and NSIS sources reference the payload and that non-system runtime DLL imports are staged.

The standalone development EXE does not itself contain an installer workflow. Test it on a machine that already has WebView2.

## Icons and splash

### Icons

Tauri reads the configured icon set under `src-tauri/icons/`:

- `favicon.ico`;
- `favicon-32x32.png`;
- `android-chrome-192x192.png`;
- `android-chrome-512x512.png`.

Rebuild the native executable after changing icons.

### Splash

The source splash is `quartermaster-splash.png`. The runtime copy is `public/quartermaster-splash.png`, used by `public/splashscreen.html`. Keep the copies synchronized when updating the artwork.

## Generated files

- `src/generated/atari-glyphs.svg`
- `src/generated/atari-glyphs.css`
- `docs/ATASCII_REFERENCE.md`
- `dist/`
- `src-tauri/target/`
- `packages/`

The first two are regenerated by `npm run glyphs`; the ATASCII page is regenerated by `npm run docs:atascii`. Build/staging outputs are ignored by Git.

## Release smoke test

At minimum:

- startup splash appears promptly and closes;
- no command window appears;
- current icon appears;
- Help Center and ATASCII search open;
- Check for Updates correctly reports current/newer/available states;
- local New/Open/Save work;
- a file opens from an ATR by double-click and context Open;
- ATR overwrite/refresh/unmount work;
- 360K and 16M create as SpartaDOS;
- rectangular select/copy/paste/inverse work;
- ASCII export and raw extraction differ as documented;
- BASIC tokenize/detokenize round-trip a small listing;
- activity messages appear during backend work.
