# Troubleshooting and support

[Documentation index](README.md) · [User guide](USER_GUIDE.md) · [ATR guide](ATR_GUIDE.md) · [Building](BUILDING_AND_RELEASES.md)

## Official help channel

Report bugs, request features, and submit documentation corrections at:

https://github.com/rickcollette/quartermaster-m/issues

The same URL appears under **Help → Get Help / Report Issue** and can be copied from the in-app Help Center.

## Before reporting a problem

Record:

1. QuarterMaster/M version from the upper-right title strip.
2. Windows edition/version and CPU architecture.
3. Exact actions starting from application launch.
4. Expected result and actual result.
5. Full error text; do not paraphrase it.
6. Active location (`LOCAL`, `D1:`–`D4:` and directory).
7. ATASCII/ASCII mode and 40/XEP-80 width.
8. ATR filesystem and geometry, when relevant.
9. Whether the problem also occurs with a fresh small file/image.
10. A minimal shareable file or redacted screenshot, if possible.

Never post private data, credentials, proprietary disk images, or copyrighted material you cannot redistribute.

## Startup

### A command window appears

Release builds should use the Windows GUI subsystem. The Rust executable contains:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

A debug executable can still be console-oriented. Confirm you launched the release `quartermaster-m.exe` from `src-tauri\target\release` or `packages\win64`, not a debug artifact or a wrapper script.

### Splash appears but main window never opens

The main window is revealed only after the frontend calls the native `app_ready` command.

Try:

1. wait for any security scan to finish;
2. confirm Microsoft Edge WebView2 Runtime is installed;
3. launch the current EXE from a writable/ordinary user folder;
4. run `npm run build` in a source checkout to detect frontend errors;
5. temporarily move/rename the application's `virtual-drive.json` state file if a stale mount path is suspected;
6. file an issue with version and Windows details.

### Blank window or WebView2 error

QuarterMaster/M is a Tauri application and requires Microsoft Edge WebView2 Runtime. Production installers are configured to embed the x64 offline runtime installer. A standalone development EXE expects the machine to already have WebView2.

Install/update WebView2 from Microsoft or use a production package when available.

### Application icon is wrong

Windows may cache executable icons. Verify the current EXE was rebuilt after changes under `src-tauri\icons`, then copy it to a new filename/location or rebuild the Windows icon cache before assuming the binary still contains old resources.

## Editor

### Double-click/right-click Open from ATR does nothing

Current behavior should load the file and show an animated Loading… overlay.

Check:

- the item is a file, not directory;
- the correct D1:–D4: drive is selected/mounted;
- the operation overlay/error is not behind another window;
- the chosen mode is appropriate for the content;
- Refresh Directory succeeds;
- raw extraction of the same file succeeds.

If raw extraction succeeds but editor open fails, attach the smallest shareable file and exact error to an issue.

### Inverse glyph is white on white

The expected inverse appearance is blue glyph on a white cell. Rebuild the frontend so the latest generated glyph CSS and style sheet are embedded:

```powershell
npm run build
```

If only a particular glyph fails, report its ATASCII byte from the status bar.

### Typing produces the wrong control action

Plain Ctrl+letter combinations insert low ATASCII graphic bytes. Application character shortcuts use Ctrl+Shift+letter, such as Ctrl+Shift+S for Save and Ctrl+Shift+F for Find. See [Keyboard and Mouse](KEYBOARD_AND_MOUSE.md#entering-atascii-control-graphics).

### Selection is rectangular instead of line-shaped

This is intentional. Atari screens are fixed cell grids, so drag selection represents a rectangle suitable for moving panels, menus, and graphic regions.

### Copy/paste loses graphics in another application

Windows clipboard text is a readable interoperability representation. Exact Atari bytes, inverse flags, and rectangle shape are retained only by the internal QuarterMaster/M clipboard. Save as ATASCII or use raw operations for lossless transfer.

### Rows 41–80 disappeared

Changing XEP-80 to 40 COL keeps only the first 40 columns. Reopen an unsaved source or restore the last saved 80-column copy.

### File contains many empty lines

The editor has 357 rows and serialization places a line separator between each row, though trailing spaces within each row are trimmed. If a consuming tool does not want trailing blank rows, remove them in the consuming workflow or use an export/post-processing step. Preserve the native file until the intended Atari application has been tested.

### Non-ASCII characters become `?`

ASCII mode and host-text import support the ASCII byte range, not arbitrary Unicode transliteration. Replace the character with a supported glyph or construct the desired Atari graphic using ATASCII mode.

## Explorer and active locations

### Save opens in the wrong place

Look for the gold active-location outline and the toolbar location label. New/Open/Save follow that location, which can differ from the currently displayed document.

Select the desired local/ATR directory and retry.

### Mounted drive disappeared after restart

QuarterMaster/M persists mount paths, not embedded copies. A drive cannot be restored if the ATR was moved, renamed, deleted, inaccessible, or no longer recognized. Mount it again at its new path.

### Local tree does not reflect an external change

Reopen the local folder. ATR changes have a dedicated Refresh Directory command; local host-tree refresh is performed by reopening/operations that update its state.

## ATR operations

### ATR will not mount

Possible causes:

- not an ATR or damaged header;
- unsupported filesystem;
- filesystem metadata corruption;
- access denied/path unavailable;
- file in use;
- image geometry not understood by DOS 2/SpartaDOS parser.

Preserve the image unchanged. Try an archival copy and include filesystem/geometry details in the report.

### “D#: has no ATR image mounted”

The command targeted a drive slot without an image. Click the intended mounted drive and retry.

### New Folder fails

Directories require SpartaDOS. DOS 2 is flat. Also validate the name and ensure the directory/file allocation structures have capacity.

### Filename rejected

Document dialog validation uses:

```text
[A-Z0-9_]{1,8} optionally followed by .[A-Z0-9_]{1,3}
```

Use an 8.3 uppercase name such as `SCREEN.ATA` or `PROGRAM.BAS`.

### Disk full / directory full

Extract or delete unneeded files, use a larger image, or create another ATR. Total sector capacity is not equal to usable file capacity because filesystem metadata, allocation tables, directories, and reserved sectors consume space.

### External tool and QuarterMaster/M disagree

Do not write the same ATR concurrently. Close/unmount it in all tools, work from a backup, and compare a raw extraction. Include the image geometry and the other tool/version in an issue.

### Import changed my file

Host-to-ATR text import is a conversion. `.BAS` is tokenized; other text gets ATASCII line endings. Use ATR-to-ATR copy or raw-specific operations for byte preservation.

## Atari BASIC

### Text listing opened as garbage

You likely used normal Open on a tokenized binary or used the BASIC detokenizer on plain text. See [Atari BASIC Guide](BASIC_GUIDE.md#the-two-basic-representations).

### Tokenizer reports syntax/number/variable error

Reduce the listing to the smallest failing line. Confirm:

- line number exists;
- statement/function spelling matches Atari BASIC;
- punctuation/operator is supported ASCII;
- quotes are balanced;
- variable suffix `$`/`(` is consistent;
- numeric literal fits the converter.

### Tokenized program behaves differently on Atari

Preserve both listing and binary. Report:

- minimal source;
- produced binary;
- emulator/real hardware and BASIC revision;
- expected/actual output;
- QuarterMaster/M version.

## Build problems

### `npm`/`tauri` not found

Install Node.js/npm, run `npm install`, and ensure the local `node_modules\.bin` tools are available through npm scripts.

### Rust linker/toolchain failure

Install the stable MSVC Rust toolchain and Tauri's Windows prerequisites, including Visual Studio C++ Build Tools and an appropriate Windows SDK.

### Frontend builds but EXE is stale

`npm run build` only creates `dist`. Rebuild Tauri:

```powershell
npm run tauri -- build --no-bundle
Copy-Item src-tauri\target\release\quartermaster-m.exe packages\win64\quartermaster-m.exe -Force
```

### Production packaging takes too long during development

Do not run MSI/NSIS bundling for ordinary development. Use `--no-bundle`. The longer production audit is reserved for releases.

## Issue template

Copy and fill this into a GitHub issue:

```markdown
### Version/environment
- QuarterMaster/M:
- Windows:
- EXE source: packages/win64, target/release, installer, other
- WebView2:

### Document/disk context
- Active location:
- Mode: ATASCII or ASCII
- Width: 40 or 80
- ATR filesystem/geometry:

### Steps
1.
2.
3.

### Expected

### Actual

### Exact error

### Attachments
- Minimal file/ATR:
- Redacted screenshot:
```
