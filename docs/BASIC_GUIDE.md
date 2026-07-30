# Atari BASIC guide

[Documentation index](README.md) · [User guide](USER_GUIDE.md) · [File formats](FILE_FORMATS.md) · [ATR guide](ATR_GUIDE.md)

## The two BASIC representations

Atari BASIC commonly appears in two fundamentally different forms:

### Listing

Human-readable source:

```basic
10 GRAPHICS 0
20 PRINT "QUARTERMASTER/M"
30 FOR I=1 TO 10
40 PRINT I
50 NEXT I
60 END
```

A listing can use:

- ASCII/CRLF for modern Windows tools; or
- ATASCII/`$9B` for Atari `ENTER`/`LIST` workflows.

### Tokenized saved program

A binary structure produced by Atari BASIC's `SAVE` operation. It includes a pointer header, variable-name/value tables, encoded line records, statement/expression tokens, Atari numeric constants, and an immediate-line/end marker. It is not readable text even when named `.BAS`.

QuarterMaster/M converts both forms natively in Rust. No console window or external BASIC parser is launched.

## BASIC menu

### Open Tokenized BASIC From Disk…

1. Choose a host `.BAS` binary.
2. QuarterMaster/M validates the saved-program structure.
3. The binary is detokenized.
4. The editable listing opens in ASCII mode.

The title/status identifies the content as detokenized.

### Save Tokenized BASIC To Disk…

1. Write or open an ASCII listing in the editor.
2. Choose the command.
3. Select a destination, conventionally `PROGRAM.BAS`.
4. QuarterMaster/M parses line numbers/statements/expressions and writes a native saved-program binary.

### Save Detokenized Listing To Disk…

Writes the current listing as text without tokenizing it. Choose:

- ASCII → `.TXT`, `.LST`, or `.BAS` text with CRLF;
- ATASCII → `.LST`/`.BAS` with `$9B`.

### Open Tokenized BASIC From ATR…

Reads a binary file from the active ATR, detokenizes it, and opens the listing. Select the correct D1:–D4: drive first.

### Save Tokenized BASIC To ATR…

Tokenizes the current editor listing and stores the binary program inside the active ATR.

### Save Detokenized Listing To ATR…

Stores editable source text inside the active ATR in the chosen ASCII or ATASCII representation.

## Drag/drop BASIC convention

Dropping a host file whose extension is `.BAS` onto an ATR treats it as a **host text listing** and tokenizes it. This is convenient for modern source files.

An already-tokenized `.BAS` is binary. To preserve it:

- copy it ATR-to-ATR; or
- use a raw byte-preserving import/extraction workflow instead of the host text drag convention.

Do not rely on the extension alone to distinguish text from binary.

## Writing a tokenizable listing

### Line numbers

Every nonblank logical program line needs a decimal line number:

```basic
10 PRINT "VALID"
```

Omitting a line number is an error. Use conventional ascending values so later edits can be inserted.

### Statements

The native token table includes the Atari BASIC statement set represented by the application:

```text
REM DATA INPUT COLOR LIST ENTER LET IF FOR NEXT GOTO GO TO
GOSUB TRAP BYE CONT ? CLOSE CLR DEG DIM END NEW OPEN LOAD SAVE
STATUS NOTE POINT XIO ON POKE PRINT RAD READ RESTORE RETURN RUN
STOP POP GET PUT GRAPHICS PLOT POSITION DOS DRAWTO SETCOLOR
LOCATE SOUND LPRINT CSAVE CLOAD
```

Colon-separated statements are supported. `IF … THEN` is normalized into Atari's statement-record structure.

### Expressions and functions

Supported expression words/functions include:

```text
TO STEP THEN NOT OR AND
STR$ CHR$ USR ASC VAL LEN ADR ATN COS PEEK SIN RND FRE EXP
LOG CLOG SQR SGN ABS INT PADDLE STICK PTRIG STRIG
```

Operators and separators represented by the tokenizer include:

```text
, ; # <= <> >= < > = ^ * + - / ( )
```

### Variables

Numeric, string (`$`), and array forms are collected into a variable-name table. Consistent spelling matters. The tokenizer reports an unknown variable when an expression references a name it could not resolve in the constructed table.

### Numeric literals

Numbers are encoded in Atari BASIC's six-byte BCD floating format. The native converter supports signed decimal values within the precision represented by its ten-digit packed conversion. A malformed literal or excessive significant digits is reported.

### Strings, REM, and DATA

Quoted strings are stored as Atari bytes. `REM` and `DATA` bodies use their special statement treatment rather than normal expression tokenization. Keep quotes balanced and test programs that intentionally embed control bytes.

## Detokenization behavior

The decoder:

1. validates the seven-word pointer header;
2. resolves file offsets from Atari's `$0100` LOMEM basis;
3. parses the variable-name table;
4. walks statement records by encoded lengths;
5. renders known statement/expression tokens;
6. decodes six-byte BCD numbers;
7. stops at the immediate-line marker.

Corrupt pointers, out-of-range records, truncated lines, or too-short files produce explicit errors instead of guessed output. Unknown expression tokens are rendered as hexadecimal markers such as `<$xx>` so the unexpected data remains visible.

## Recommended source-control workflow

1. Keep master programs as ASCII listings (`.bas` or `.lst`) in source control.
2. Edit/test in QuarterMaster/M.
3. Tokenize to a disposable output `.BAS`.
4. Copy the tokenized program into a working ATR.
5. Keep the original ATR immutable where possible.
6. Detokenize and compare when verifying a round trip.

This separates diff-friendly source from emulator-ready binary output.

## Troubleshooting BASIC

| Symptom | Likely cause | Action |
|---|---|---|
| “file is too short for an Atari BASIC header” | Text listing opened as tokenized binary, or truncated file | Open as a normal document or obtain an intact binary |
| Header points outside file | Corrupt/incompatible tokenized program | Raw-extract and preserve; attach minimal sample to an issue |
| Missing line number | Listing syntax | Add a number to each program line |
| Unknown statement/function | Unsupported or misspelled token | Check Atari BASIC spelling and implemented tables |
| Unsupported character in expression | Modern Unicode/unsupported punctuation | Replace with an Atari BASIC operator/ASCII character |
| Unknown variable | Inconsistent name/type or parser could not collect definition | Check spelling, `$`, and `(` suffixes |
| Numeric literal has too many digits | Converter precision limit | Shorten or rewrite the constant |
| ATR save fails after successful tokenization | Disk filename/space/filesystem issue | Check active drive, name, and free capacity |

When reporting a reproducible tokenizer problem, include the smallest listing that fails and the full error message:

https://github.com/rickcollette/quartermaster-m/issues
