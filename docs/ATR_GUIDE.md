# ATR disk-image and Explorer guide

[Documentation index](README.md) · [User guide](USER_GUIDE.md) · [File formats](FILE_FORMATS.md) · [Troubleshooting](TROUBLESHOOTING.md)

## Mental model

QuarterMaster/M treats ATR images as four virtual Atari drives:

```text
Local Windows folder
├── ordinary host files
└── ordinary host directories

D1: mounted ATR
D2: mounted ATR
D3: mounted ATR
D4: mounted ATR
```

Each drive has its own mounted path, filesystem, directory tree, and active state. A persisted virtual-drive state file allows previously selected local folders and mount paths to be restored at startup when still available.

An ATR is a sector image, not a normal Windows folder. Explorer translates filesystem directory operations into sector-level DOS 2 or SpartaDOS updates.

## Supported filesystems

### Atari DOS 2

- Flat namespace; no subdirectories.
- Suitable for standard single/enhanced/double-density floppy images within DOS 2 geometry limits.
- Filenames use Atari 8.3 form.

### SpartaDOS 2

- Hierarchical directories.
- Supports the 360K, 16M, and broader custom geometries used by QuarterMaster/M.
- Supports an optional validated volume label.

Filesystem detection is attempted when mounting an existing ATR. If neither SpartaDOS 2 nor Atari DOS 2 structures can be recognized, mounting reports both detection failures.

## Drives and active location

Click D1:, D2:, D3:, or D4: to select a drive. The drive's root becomes the active location. Click a directory to target that directory, or a file to target its parent.

The active location affects:

- New;
- Open;
- Save/Save As;
- Add File;
- New Folder;
- drop destinations.

Opening or creating an ATR assigns it to the selected/free drive and makes that drive active.

## Mount and browse

1. Select the intended D1:–D4: root.
2. Choose **ATR → Open/Mount** or Explorer **Mount**.
3. Select an `.atr` file.
4. QuarterMaster/M detects its supported filesystem and reads the directory.
5. Expand/collapse directory rows with their disclosure marker.
6. Double-click a file to open it, or right-click for all available actions.

If the same image was changed outside QuarterMaster/M, choose **ATR → Refresh Directory** before doing more work.

## Open a file

Available methods:

- double-click a file row;
- right-click → Open;
- select a file, then use the tree Open button;
- select its directory and choose ATR → Open File From ATR;
- highlight its directory and click toolbar Open.

The editor loads using the current ATASCII/ASCII and 40/80-column selections. Binary content that is not a text document should be raw-extracted rather than opened.

## File and directory operations

### Add File

Imports a host file into the selected drive/directory.

For explicit button/menu addition, the destination name is requested. For drag/drop, the host filename becomes the suggested destination. Names are normalized/validated against Atari filename rules.

### New Folder

Creates a directory in SpartaDOS. DOS 2 is flat, so directory creation is not meaningful there.

### Rename

Renames the selected file or directory after validating the new name and checking filesystem constraints.

### Delete

Deletes the selected entry after confirmation. Directory deletion is subject to filesystem rules; back up important ATRs before destructive changes.

### Export ASCII

Creates Windows-readable text:

- ATASCII `$9B` becomes CRLF.
- ATASCII tab `$7F` becomes tab.
- inverse bit 7 is removed from printable text.
- non-ASCII graphics/control data is omitted.
- tokenized `.BAS` is detokenized when its binary structure is recognized.

This is a conversion, not a byte-preserving backup.

### Extract Raw

Copies bytes unchanged. Use it for:

- executables and machine-language binaries;
- graphics/font data;
- tokenized BASIC that must remain tokenized;
- unknown formats;
- archival copies;
- round-trip verification.

### Refresh

Rereads the filesystem directory and metadata from the ATR path. Refresh does not convert or edit files.

### Unmount

Closes that virtual-drive association. It does not delete the ATR. The document already displayed in the editor remains visible, but a future save follows the currently active location.

## Drag and drop

### Host → ATR

Drop a host file on a mounted drive root or directory.

| Host input | ATR result |
|---|---|
| `.BAS` text listing | Native tokenized Atari BASIC program |
| Other text | ATASCII text (`$9B` lines, `$7F` tabs) |
| Unsupported bytes during text import | Replaced with `?` |

The `.BAS` drag rule assumes the host file is a readable listing. To import an already-tokenized `.BAS` unchanged, use an explicit raw-oriented path/workflow rather than the text-listing drag convention.

### ATR → ATR

Drag a file from one mounted drive to another drive/directory. The source bytes are copied unchanged. This is the preferred path for preserving tokenized programs, inverse ATASCII, binaries, and any format QuarterMaster/M does not interpret.

### ATR → Windows

Native outbound Windows shell drag is not used. Right-click and choose:

- **Export ASCII** for translated text; or
- **Extract Raw** for exact bytes.

The explicit choice prevents silent conversion surprises.

## Create an ATR image

Choose **ATR → Create ATR**. One configuration window contains all choices.

### Drive

Choose D1:, D2:, D3:, or D4:. Creating a disk mounts the new image in that slot.

### Filesystem

- **DOS2:** flat Atari DOS 2 filesystem.
- **Sparta:** SpartaDOS filesystem with directories/volume label.

360K and 16M force SpartaDOS.

### Geometry presets

| Name | Total sectors | Data sector size | Approximate data capacity | Structure |
|---|---:|---:|---:|---|
| 90K | 720 | 128 | 92,160 bytes | Single density |
| 130K | 1,040 | 128 | 133,120 bytes | Enhanced density |
| 180K | 720 | 256 | 183,936 bytes* | Double density |
| 360K | 1,440 | 256 | 368,256 bytes* | DD, two-sided physical analogue |
| 16M | 65,535 | 256 | 16,776,192 bytes* | Large-capacity partition |

\* Sectors 1–3 are 128-byte boot sectors in the standard mixed-size double-density ATR layout, so simple `sectors × 256` arithmetic is 384 bytes larger than actual sector data. The ATR also includes a 16-byte header, and filesystem metadata reduces usable file space.

The 360K preset represents 1,440 sectors: 18 sectors × 40 tracks × 2 sides in its floppy analogue. The 16M preset uses the maximum 65,535-sector count intended for this format.

### Custom geometry

| Filesystem | Sector range | Sector sizes |
|---|---:|---|
| DOS 2 | 368–1,040 | 128 or 256 |
| SpartaDOS | 16–65,535 | 128 or 256 |

QuarterMaster/M validates the selected range before enabling creation.

### SpartaDOS volume label

- 1–8 characters.
- Letters, numbers, spaces, `_`, and `-`.
- First character must be a letter or number.
- Stored uppercase.

The label control is disabled for DOS 2.

### Destination

After configuration, **Choose File & Create** opens a host save dialog for the `.atr` image. The image is formatted, written, mounted in the selected drive, and shown in Explorer.

## Filenames

The document-save dialog accepts conventional Atari 8.3 names:

```text
NAME
NAME.EXT
PROGRAM.BAS
SCREEN.ATA
```

The current frontend validation uses uppercase letters, digits, and underscore: 1–8 characters before an optional dot and 1–3 character extension. Directory paths are maintained separately by the SpartaDOS tree.

## Concurrency and safety

- Do not mount the same writable ATR in more than one application.
- Do not mount one host ATR path in multiple QuarterMaster/M drives for concurrent edits.
- Wait for the animated operation overlay to finish.
- Refresh after external writes.
- Keep immutable originals and work on copies when editing historic software.
- Raw-extract valuable files before rename/delete/format experiments.

## When an operation fails

QuarterMaster/M leaves the UI available and displays the backend error. Capture the exact text. Check:

1. correct drive and directory;
2. valid Atari filename;
3. free space and directory capacity;
4. filesystem support for the operation;
5. host path permissions;
6. whether another program has the ATR open;
7. whether the image is damaged.

Continue with [Troubleshooting](TROUBLESHOOTING.md).
