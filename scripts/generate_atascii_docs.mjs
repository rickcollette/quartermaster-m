import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const mapPath = resolve(root, "atascii_kit/data/atascii_screen_code_map.csv");
const controlPath = resolve(root, "atascii_kit/data/atascii_control_codes.csv");
const outputPath = resolve(root, "docs/ATASCII_REFERENCE.md");

function parseCsv(path) {
  const [header, ...rows] = readFileSync(path, "utf8").trim().split(/\r?\n/);
  const keys = header.split(",");
  return rows.map(row => Object.fromEntries(row.split(",").map((value, index) => [keys[index], value])));
}

const map = parseCsv(mapPath);
const controls = new Map(parseCsv(controlPath).map(row => [Number(row.decimal), row]));

const graphicNames = [
  "Heart", "Tee right", "Right half bar", "Lower-right corner",
  "Tee left", "Upper-right corner", "Diagonal slash", "Diagonal backslash",
  "Lower-right triangle", "Lower-right block", "Lower-left triangle", "Upper-right block",
  "Upper-left block", "Top bar", "Bottom bar", "Lower-left block",
  "Club", "Upper-left corner", "Horizontal bar", "Tee down",
  "Tee up", "Left half bar", "Lower-left corner", "Vertical bar",
  "Diamond", "Four-way junction", "Filled circle", "Escape",
  "Cursor up", "Cursor down", "Cursor left", "Cursor right",
];

function hex(value) {
  return `$${Number(value).toString(16).padStart(2, "0").toUpperCase()}`;
}

function baseName(base) {
  if (base < 0x20) return graphicNames[base];
  if (base === 0x20) return "Space";
  if (base >= 0x21 && base <= 0x5f) return `ASCII \`${String.fromCharCode(base).replace("|", "\\|")}\``;
  if (base === 0x60) return "Atari diamond";
  if (base >= 0x61 && base <= 0x7a) return `Lowercase \`${String.fromCharCode(base)}\``;
  if (base === 0x7b) return "Spade";
  if (base === 0x7c) return "Vertical bar";
  if (base === 0x7d) return "Clear-screen glyph";
  if (base === 0x7e) return "Backspace/delete glyph";
  return "Tab glyph";
}

function baseInput(base) {
  if (base === 0) return "Ctrl+@ (mapping; current editor letter handler does not emit it)";
  if (base >= 1 && base <= 0x1a) {
    const letter = String.fromCharCode(0x40 + base);
    return `Ctrl+${letter}`;
  }
  if (base === 0x20) return "Space";
  if (base >= 0x21 && base <= 0x7a && base !== 0x60) return "Type the character";
  if (base === 0x7d) return "Ctrl+< in an Atari screen-editor stream";
  if (base === 0x7e) return "Backspace in an Atari screen-editor stream";
  if (base === 0x7f) return "Tab in an Atari screen-editor stream";
  return "Load/paste an ATASCII glyph or use an Atari-aware source";
}

function byteMeaning(byte, base, inverse) {
  const control = controls.get(byte);
  const display = `${inverse ? "Inverse " : ""}${baseName(base)}`;
  if (!control) return display;
  return `${control.function} control when interpreted; ${display} in quoted/display-cell context`;
}

function escapeCell(text) {
  return String(text).replaceAll("|", "\\|").replaceAll("\n", " ");
}

const rows = map.map(row => {
  const byte = Number(row.atascii_dec);
  const base = Number(row.base_code);
  const inverse = row.inverse === "1";
  const control = controls.get(byte);
  const input = control?.typical_key || (inverse ? `Inverse form of ${baseInput(base)}` : baseInput(base));
  return `| ${byte} | \`${hex(byte)}\` | \`${hex(base)}\` | ${inverse ? "Yes" : "No"} | ${row.screen_dec} | \`${hex(row.screen_dec)}\` | ${escapeCell(byteMeaning(byte, base, inverse))} | ${escapeCell(input)} |`;
}).join("\n");

const content = `# ATASCII complete reference

[Documentation index](README.md) · [Keyboard and mouse](KEYBOARD_AND_MOUSE.md) · [File formats](FILE_FORMATS.md)

This reference documents every ATASCII byte from \`$00\` through \`$FF\`, its base glyph, inverse state, Atari screen-code mapping, input convention, and context-sensitive control meaning. The 256-row appendix is generated from the project's canonical machine-readable map by \`npm run docs:atascii\`.

## The byte model

\`\`\`text
base glyph = ATASCII byte & $7F
inverse    = (ATASCII byte & $80) != 0
\`\`\`

In a display-cell context, \`$80–$FF\` are inverse forms of \`$00–$7F\`. In an interpreted ATASCII stream, several high bytes are editor controls rather than printable cells. In particular, QuarterMaster/M text files use \`$9B\` as end-of-line.

## ATASCII versus screen codes

ATASCII is the keyboard/text-stream ordering. Atari display memory uses screen codes:

| ATASCII base range | Screen-code conversion |
|---|---|
| \`$00–$1F\` | Add \`$40\` |
| \`$20–$5F\` | Subtract \`$20\` |
| \`$60–$7F\` | Unchanged |

Preserve bit 7 to preserve inverse video. The full result for every byte appears below.

## Text-file behavior in QuarterMaster/M

- \`$9B\` starts the next row.
- Glyph tokens populate cells.
- Width overflow wraps to the next row.
- Other interpreted editor-control commands are ignored during ordinary text-file loading rather than executed against the document.
- Saving inserts \`$9B\` between rows.
- Trailing ordinary spaces are trimmed; inverse spaces remain.

This makes loading deterministic and prevents embedded screen-editor commands from unexpectedly clearing or restructuring a document.

## Control bytes

| Decimal | Hex | Interpreted function | Typical Atari key | Notes |
|---:|---:|---|---|---|
${[...controls.entries()].map(([decimal, row]) => `| ${decimal} | \`${hex(decimal)}\` | ${row.function} | ${row.typical_key} | ${row.notes} |`).join("\n")}

### Quoting and context

Atari software can quote control codes so that the glyph associated with the same low seven bits appears rather than the command executing. A saved screen, display-memory dump, terminal protocol, and plain ATASCII text file can therefore assign different semantics to the same numeric byte. QuarterMaster/M's document loader deliberately uses its TextFile decoding domain.

## Base glyph families

### \`$00–$1F\`: graphics and cursor symbols

| Hex | Name | Hex | Name | Hex | Name | Hex | Name |
|---:|---|---:|---|---:|---|---:|---|
${Array.from({ length: 8 }, (_, row) => Array.from({ length: 4 }, (_, column) => {
  const value = row + column * 8;
  return `\`${hex(value)}\` | ${graphicNames[value]}`;
}).join(" | ")).map(row => `| ${row} |`).join("\n")}

### \`$20–$5F\`: uppercase ASCII-compatible range

This range includes space, punctuation, digits, uppercase letters, and ASCII punctuation through underscore. Its screen codes are \`$00–$3F\`.

### \`$60–$7F\`: Atari/lowercase range

- \`$60\`: Atari diamond (not the ASCII grave-accent glyph on the Atari screen).
- \`$61–$7A\`: lowercase letters.
- \`$7B\`: spade.
- \`$7C\`: vertical bar.
- \`$7D\`: clear-screen glyph/control in the appropriate context.
- \`$7E\`: backspace/delete glyph/control.
- \`$7F\`: tab glyph/control.

## Complete 256-byte map

The **Meaning** column gives both interpreted-control and display-cell interpretations where they differ. Decimal screen codes come directly from the canonical round-trip table.

| ATASCII decimal | ATASCII hex | Base | Inverse | Screen decimal | Screen hex | Meaning | Input / typical key |
|---:|---:|---:|:---:|---:|---:|---|---|
${rows}

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

QuarterMaster/M's executable behavior is defined by the vendored \`atascii\` crate and the canonical CSV committed with this project. Historical references are supporting documentation.

## Regenerating this page

\`\`\`powershell
npm run docs:atascii
\`\`\`

The generator reads both canonical CSV files and rewrites this reference. Review the Markdown diff whenever canonical mapping data changes.
`;

writeFileSync(outputPath, content, "utf8");
console.log(`Generated ${outputPath} with ${map.length} ATASCII byte rows.`);
