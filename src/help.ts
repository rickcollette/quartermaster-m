import { atariGlyphClass } from "./atariGlyphs";

export const SUPPORT_URL = "https://github.com/rickcollette/quartermaster-m/issues";

type HelpSection = {
  id: string;
  title: string;
  summary: string;
  body: string;
};

const helpSections: HelpSection[] = [
  {
    id: "start",
    title: "Start Here",
    summary: "A five-minute tour from launch to a saved Atari document.",
    body: `
      <h3>QuarterMaster/M in five steps</h3>
      <ol>
        <li><strong>Choose a location.</strong> Click the Local Folder root or a mounted D1:–D4: drive or directory in Explorer. The gold outline and toolbar location label show the active destination.</li>
        <li><strong>Create or open.</strong> Use New, Open, or Save above the editor. These commands always follow the active location. Double-clicking a file in either Explorer tree also opens it.</li>
        <li><strong>Edit.</strong> Type, use the cursor keys, or drag across glyphs to make a rectangular selection. Right-click inside the editor for Cut, Copy, Paste, Select Glyph, Select All, and Inverse Selected Glyphs.</li>
        <li><strong>Choose the representation.</strong> ATASCII writes Atari bytes and <code>$9B</code> line endings. ASCII writes Windows-compatible text with CRLF line endings.</li>
        <li><strong>Save.</strong> Press <kbd>Ctrl+Shift+S</kbd>. A file opened from an ATR folder defaults back to that folder; a local file defaults to its local directory.</li>
      </ol>
      <div class="help-callout"><strong>First thing to remember:</strong> Explorer selection controls where New, Open, and Save operate. Selecting a file targets its containing directory.</div>
      <h3>Screen geometry</h3>
      <p>40 COL is the standard Atari display width. XEP-80 provides an 80-column editing surface. Both modes contain 357 document rows and use a 24-row scrolling viewport.</p>
      <h3>Unsaved changes</h3>
      <p>An asterisk in the status bar marks a modified document. QuarterMaster/M asks before an action discards unsaved work.</p>
    `,
  },
  {
    id: "interface",
    title: "Interface Tour",
    summary: "Menus, toolbar, Explorer, editor, status, splash, and activity feedback.",
    body: `
      <h3>Menus</h3>
      <dl>
        <dt>File</dt><dd>New, Open, Save, Save As, and Export ASCII for the current document.</dd>
        <dt>Edit</dt><dd>Find and Find and Replace across all 357 rows.</dd>
        <dt>View</dt><dd>Switch between a 40-column Atari surface and an 80-column XEP-80 surface.</dd>
        <dt>ATR</dt><dd>Open/Mount, Refresh Directory, Open File From ATR, Create ATR, and Unmount ATR.</dd>
        <dt>BASIC</dt><dd>Open, tokenize, or export Atari BASIC programs on the host or inside the active ATR.</dd>
        <dt>Help</dt><dd>Open this Help Center directly at the guide, keyboard reference, ATASCII map, or support page. Check for Updates compares the installed version with the published release and offers portable EXE or MSI installation. License displays the complete bundled GPL license, and About identifies the application, version, author, and handle.</dd>
      </dl>
      <h3>Toolbar</h3>
      <p>40 COL and XEP-80 are on the far left. New, Open, and Save sit directly above the editor. The location label identifies their current target. The ATASCII/ASCII selector controls document encoding. Inverse changes the inverse state of newly typed glyphs; Insert toggles insert/overwrite behavior; Clear resets the full document after confirmation.</p>
      <h3>Explorer</h3>
      <p>The top tree is the local Windows folder. The lower tree contains four independent ATR drive slots, D1: through D4:. Click a root, directory, or file to make its location active. The selected item can also be managed with the action buttons or its right-click menu.</p>
      <h3>Status bar and activity</h3>
      <p>The status bar reports the document name, dirty state, drive, row and column, selection dimensions, encoding mode, inverse state, insert/overwrite state, current byte, total geometry, and visible-row count. Longer operations display an animated Loading…, Saving…, Mounting…, Importing…, Exporting…, or equivalent activity panel. The splash screen appears while the native application initializes.</p>
    `,
  },
  {
    id: "editor",
    title: "Editor & Selection",
    summary: "Typing, navigation, rectangular selections, clipboard behavior, and inverse video.",
    body: `
      <h3>Typing and navigation</h3>
      <p>Click a glyph cell to place the caret. Type ordinary characters or use Atari control-glyph shortcuts. Arrow, Home, End, Page Up, Page Down, Enter, and Tab navigate the 357-row surface. Insert shifts subsequent glyphs to the right within the current row; overwrite replaces the current glyph.</p>
      <h3>Mouse editing</h3>
      <ul>
        <li><strong>Click:</strong> place the caret and clear the previous selection.</li>
        <li><strong>Drag:</strong> select a rectangular glyph area.</li>
        <li><strong>Shift-click:</strong> extend the selection rectangle from its anchor.</li>
        <li><strong>Right-click:</strong> open the editor menu. If the click is outside the selection, the clicked glyph becomes the current selection first.</li>
      </ul>
      <h3>Clipboard</h3>
      <p>Cut, Copy, and Paste work with both the application clipboard and the Windows clipboard. Copies made inside QuarterMaster/M retain the rectangle, Atari byte values, and inverse flags for a faithful in-app paste. The Windows clipboard receives readable text; external text pasted into the editor is converted to supported Atari glyphs.</p>
      <h3>Selections and inverse</h3>
      <p>Shift plus an arrow key extends a selection. <kbd>Ctrl+Shift+A</kbd> selects the full surface. Backspace, Delete, or typing clears/replaces the selected rectangle. Choose <strong>Inverse Selected Glyphs</strong> to toggle bit 7 for every selected glyph. This is distinct from the toolbar Inverse button, which changes only newly typed glyphs.</p>
      <h3>Find and replace</h3>
      <p><kbd>Ctrl+Shift+F</kbd> opens Find and <kbd>Ctrl+Shift+H</kbd> opens Find and Replace. Search can match case and wrap through all 357 rows. Matches never cross a 40/80-column row boundary and inverse video is ignored while comparing base glyph bytes. Replace shifts the remainder of that fixed row and clips/pads at its edge; Replace All processes non-overlapping matches throughout the document.</p>
      <h3>Row deletion</h3>
      <p><kbd>Ctrl+Delete</kbd> clears every cell to the right of the cursor on the current row while preserving the glyph under the cursor. <kbd>Ctrl+Shift+Delete</kbd> deletes the entire current row, pulls every later row upward, and blanks the final document row. Both commands are also available by right-clicking a glyph; Delete Line acts on the right-clicked row.</p>
      <div class="help-callout"><strong>Width changes:</strong> 40 → 80 preserves the first 40 columns and adds blank columns. 80 → 40 keeps columns 1–40 and discards columns 41–80.</div>
    `,
  },
  {
    id: "files",
    title: "Files & Locations",
    summary: "Location-aware New/Open/Save, local folders, modes, and exports.",
    body: `
      <h3>Location-aware commands</h3>
      <p>New, Open, and Save act on the highlighted location. A local directory opens a Windows file dialog rooted there. A mounted ATR directory opens an in-app Atari filename dialog rooted there. Selecting a file uses its parent directory. Opening a file also makes its location active.</p>
      <h3>New</h3>
      <p>New asks for a destination immediately, then creates a blank 357-row document marked as modified. The suggested name is <code>UNTITLED.ATA</code> in ATASCII mode or <code>UNTITLED.TXT</code> in ASCII mode.</p>
      <h3>Open</h3>
      <p>Open decodes the chosen bytes using the current ATASCII/ASCII mode and the current 40/80-column width. In ATASCII mode, <code>$9B</code> starts a new row, printable/glyph bytes populate cells, and non-display control commands are ignored. In ASCII mode, CRLF, CR, and LF are normalized to rows; unsupported Unicode is replaced with <code>?</code> and reported.</p>
      <h3>Save and Save As</h3>
      <p>Trailing non-inverse spaces are removed from each row. ATASCII output preserves inverse bit 7 and inserts <code>$9B</code> between rows. ASCII output removes inverse state, replaces unsupported bytes with <code>?</code>, and inserts CRLF between rows. Saving to an existing ATR file uses an overwrite operation.</p>
      <h3>Export ASCII</h3>
      <p>File → Export ASCII writes the current editor as Windows-readable text without changing the document's native location. For files in an ATR, right-click <strong>Export ASCII</strong> to convert ATASCII line endings and inverse text; tokenized <code>.BAS</code> files are detokenized when possible. <strong>Extract Raw</strong> makes an unchanged byte-for-byte host copy instead.</p>
    `,
  },
  {
    id: "atr",
    title: "ATR Explorer",
    summary: "Four drives, mounting, directory browsing, file actions, drag/drop, and safety.",
    body: `
      <h3>Mount and browse</h3>
      <p>Use ATR → Open/Mount or the Explorer Mount button, choose an <code>.atr</code> image, and assign it to D1:–D4:. The directory appears immediately in Explorer. Expand directories, single-click to select, double-click a file to load it into the editor, or right-click for file operations.</p>
      <h3>File and directory actions</h3>
      <dl>
        <dt>Open</dt><dd>Load a file into the editor using the current mode and width.</dd>
        <dt>Add File</dt><dd>Import a host file into the selected drive/directory.</dd>
        <dt>Export ASCII</dt><dd>Create readable host text, detokenizing BASIC where possible.</dd>
        <dt>Extract Raw</dt><dd>Copy the exact native bytes to the host.</dd>
        <dt>New Folder</dt><dd>Create a SpartaDOS directory. DOS 2 filesystems are flat.</dd>
        <dt>Rename / Delete</dt><dd>Change or remove the selected file/directory after validation and confirmation.</dd>
        <dt>Refresh / Unmount</dt><dd>Reload the on-disk directory or close that drive slot.</dd>
      </dl>
      <h3>Drag and drop</h3>
      <ul>
        <li><strong>Host → ATR:</strong> drop a file on a drive or directory. A <code>.BAS</code> host listing is tokenized as Atari BASIC; other text is converted to ATASCII, including <code>$9B</code> line endings.</li>
        <li><strong>ATR → ATR:</strong> drag a file to another mounted drive/directory. Native bytes are copied unchanged.</li>
        <li><strong>ATR → host:</strong> use right-click Export ASCII or Extract Raw. Native outbound shell dragging is intentionally represented by these explicit actions.</li>
      </ul>
      <div class="help-callout"><strong>Write safety:</strong> do not edit the same ATR image in another program while it is mounted. Refresh after any external change, and keep backups of irreplaceable disk images.</div>
    `,
  },
  {
    id: "create-atr",
    title: "Create an ATR",
    summary: "Filesystem choices, all presets, custom geometry, labels, and density details.",
    body: `
      <h3>One configuration window</h3>
      <p>ATR → Create ATR opens a single GUI. Choose a target drive, filesystem, geometry, and—when using SpartaDOS—a volume label. Select <strong>Choose File & Create</strong> only after the validation message is clear.</p>
      <table class="help-data-table">
        <thead><tr><th>Preset</th><th>Sectors</th><th>Bytes/sector</th><th>Notes</th></tr></thead>
        <tbody>
          <tr><td>90K</td><td>720</td><td>128</td><td>Single density</td></tr>
          <tr><td>130K</td><td>1,040</td><td>128</td><td>Enhanced density</td></tr>
          <tr><td>180K</td><td>720</td><td>256</td><td>Double density</td></tr>
          <tr><td>360K</td><td>1,440</td><td>256</td><td>Double-sided DD; SpartaDOS required</td></tr>
          <tr><td>16M</td><td>65,535</td><td>256</td><td>Large partition; SpartaDOS required</td></tr>
          <tr><td>Custom</td><td>16–65,535</td><td>128 or 256</td><td>Limits depend on filesystem</td></tr>
        </tbody>
      </table>
      <p>DOS 2 custom images accept 368–1,040 sectors. SpartaDOS custom images accept 16–65,535 sectors. Double-density ATR images retain 128-byte sectors for boot sectors 1–3 and use the selected 256-byte size afterward.</p>
      <h3>Volume labels</h3>
      <p>SpartaDOS labels contain 1–8 letters, numbers, spaces, underscores, or hyphens and must begin with a letter or number. Labels are uppercased. DOS 2 does not use this field.</p>
    `,
  },
  {
    id: "basic",
    title: "Atari BASIC",
    summary: "Native tokenization, detokenization, host/ATR commands, and practical limits.",
    body: `
      <h3>Native conversion</h3>
      <p>QuarterMaster/M contains a native Rust Atari BASIC tokenizer/detokenizer; it does not launch a console utility or require an external converter. A tokenized <code>.BAS</code> is a binary Atari BASIC program, while a listing is editable text with line numbers.</p>
      <h3>BASIC menu commands</h3>
      <dl>
        <dt>Open Tokenized BASIC From Disk / ATR</dt><dd>Decode the binary program and load its listing into the editor in ASCII mode.</dd>
        <dt>Save Tokenized BASIC To Disk / ATR</dt><dd>Parse the current text listing and write a binary Atari BASIC program.</dd>
        <dt>Save Detokenized Listing To Disk / ATR</dt><dd>Write the current listing as either ASCII or ATASCII text without tokenizing it.</dd>
      </dl>
      <h3>Writing listings</h3>
      <p>Give each program line a numeric line number and valid Atari BASIC syntax. The tokenizer recognizes the standard statement and expression token tables implemented by QuarterMaster/M. Errors such as a missing line number, unsupported expression character, unknown variable, malformed number, or line that exceeds Atari's encoded length are reported without writing a partial destination.</p>
      <div class="help-callout"><strong>Do not identify format by extension alone:</strong> Atari convention commonly uses <code>.BAS</code> for tokenized binaries, but a host <code>.BAS</code> dropped into an ATR is treated as a text listing to tokenize. Use Extract Raw when you need unchanged bytes.</div>
    `,
  },
  {
    id: "shortcuts",
    title: "Keyboard & Mouse",
    summary: "The complete application shortcut reference.",
    body: `
      <h3>Application and clipboard</h3>
      <table class="help-data-table"><tbody>
        <tr><th>Ctrl+Shift+N</th><td>New at the active location</td><th>Ctrl+Shift+O</th><td>Open at the active location</td></tr>
        <tr><th>Ctrl+Shift+S</th><td>Save at the active location</td><th>Ctrl+Shift+A</th><td>Select all glyphs</td></tr>
        <tr><th>Ctrl+Shift+X</th><td>Cut selection</td><th>Ctrl+Shift+C</th><td>Copy selection</td></tr>
        <tr><th>Ctrl+Shift+V</th><td>Paste</td><th>F2</th><td>Toggle inverse typing</td></tr>
        <tr><th>Ctrl+Shift+F</th><td>Find</td><th>Ctrl+Shift+H</th><td>Find and Replace</td></tr>
      </tbody></table>
      <h3>Navigation and editing</h3>
      <table class="help-data-table"><tbody>
        <tr><th>Arrow keys</th><td>Move one glyph</td><th>Shift+Arrow</th><td>Extend rectangular selection</td></tr>
        <tr><th>Home / End</th><td>First / last column of row</td><th>Page Up / Down</th><td>Move 24 rows</td></tr>
        <tr><th>Enter</th><td>First column of next row</td><th>Tab</th><td>Next 8-column tab stop</td></tr>
        <tr><th>Backspace</th><td>Delete before caret / selection</td><th>Delete</th><td>Delete at caret / selection</td></tr>
        <tr><th>Ctrl+Delete</th><td>Clear cells after the cursor on this row</td><th>Ctrl+Shift+Delete</th><td>Delete row and pull later rows up</td></tr>
        <tr><th>Insert</th><td>Toggle insert/overwrite</td><th>Escape</th><td>Clear selection or leave inverse typing</td></tr>
      </tbody></table>
      <h3>ATASCII control-glyph entry</h3>
      <p>Every plain <kbd>Ctrl+letter</kbd> combination enters its ATASCII code <code>$01</code>–<code>$1A</code> (for example Ctrl+A enters <code>$01</code> and Ctrl+Q enters <code>$11</code>). QuarterMaster/M application commands therefore require <kbd>Ctrl+Shift+letter</kbd>. Use the ATASCII Map to identify every resulting glyph. Mouse: click places the caret; drag selects; Shift-click extends; right-click opens editor actions.</p>
    `,
  },
  {
    id: "atascii",
    title: "ATASCII Map",
    summary: "Every normal and inverse byte, glyph, screen code, and special control meaning.",
    body: `
      <h3>All 256 byte values</h3>
      <p>Each row below covers one base byte and its bit-7 inverse partner, so the table represents all <strong>256 ATASCII values</strong>. Screen codes are shown separately because Atari display memory does not use ATASCII byte order.</p>
      <div class="help-callout"><strong>Context matters:</strong> bytes <code>$9B–$9F</code> and <code>$FD–$FF</code> are screen-editor controls in an interpreted stream. In a display-cell context, bit 7 selects the inverse form of the corresponding base glyph. QuarterMaster/M uses <code>$9B</code> as end-of-line when loading/saving ATASCII text.</div>
      <div class="atascii-tools">
        <label>Find <input type="search" data-atascii-search placeholder="hex, decimal, name, key…"></label>
        <label>Group <select data-atascii-group><option value="all">All</option><option value="graphics">Graphics</option><option value="text">Text</option><option value="controls">Controls</option></select></label>
        <span data-atascii-count></span>
      </div>
      <div class="atascii-table-wrap">
        <table class="help-data-table atascii-table">
          <thead><tr><th>Glyph</th><th>ATASCII</th><th>Inverse</th><th>Screen</th><th>Name / input / special meaning</th></tr></thead>
          <tbody data-atascii-body></tbody>
        </table>
      </div>
      <h3>Conversion rules</h3>
      <p>For a base ATASCII code: <code>$00–$1F → +$40</code>, <code>$20–$5F → −$20</code>, and <code>$60–$7F → unchanged</code>. Preserve bit 7 to retain inverse video. ATASCII end-of-line is <code>$9B</code>.</p>
    `,
  },
  {
    id: "formats",
    title: "Formats & Conversion",
    summary: "ATASCII, ASCII, ATR, tokenized BASIC, and what each conversion preserves.",
    body: `
      <h3>Document encodings</h3>
      <table class="help-data-table">
        <thead><tr><th>Format</th><th>Line ending</th><th>Inverse</th><th>Best use</th></tr></thead>
        <tbody>
          <tr><td>ATASCII text</td><td><code>$9B</code></td><td>Bit 7 preserved</td><td>Atari software and disk images</td></tr>
          <tr><td>ASCII text</td><td>CRLF</td><td>Removed</td><td>Windows editors, source control, sharing</td></tr>
          <tr><td>Tokenized BASIC</td><td>Binary records</td><td>Program-dependent</td><td>LOAD/RUN in Atari BASIC</td></tr>
          <tr><td>ATR</td><td>Sector image</td><td>N/A</td><td>Complete emulated floppy/partition</td></tr>
        </tbody>
      </table>
      <h3>Conversion matrix</h3>
      <ul>
        <li>ASCII → ATASCII: CRLF/CR/LF become <code>$9B</code>, tab becomes <code>$7F</code>, printable ASCII is retained, unsupported bytes become <code>?</code>.</li>
        <li>ATASCII → ASCII: <code>$9B</code> becomes CRLF, <code>$7F</code> becomes tab, inverse is removed, and graphical/control bytes without an ASCII representation are omitted.</li>
        <li>Editor → ATASCII: every cell's base code and inverse flag are written; trailing ordinary spaces are trimmed per row.</li>
        <li>ATR → ATR: raw file bytes remain unchanged.</li>
      </ul>
    `,
  },
  {
    id: "support",
    title: "Troubleshooting & Help",
    summary: "Recovery checks, useful issue details, and the official support URL.",
    body: `
      <h3>Quick checks</h3>
      <ol>
        <li>Confirm the gold active-location outline points to the expected local/ATR directory.</li>
        <li>Confirm the ATASCII/ASCII selector and 40/80-column width match the file.</li>
        <li>For an ATR problem, select the correct D1:–D4: drive and choose Refresh Directory.</li>
        <li>Use Extract Raw before conversion when diagnosing an unknown file.</li>
        <li>Keep the error text: tokenizer, filesystem, and validation messages are designed to identify the failing input.</li>
      </ol>
      <h3>Report a problem or request a feature</h3>
      <p>Use the QuarterMaster/M GitHub issue tracker:</p>
      <p class="support-url"><a href="${SUPPORT_URL}" target="_blank" rel="noopener noreferrer">${SUPPORT_URL}</a></p>
      <button type="button" class="help-copy-button" data-copy-support>Copy support URL</button>
      <h3>Check for updates</h3>
      <p>Choose <strong>Help → Check for Updates</strong>. QuarterMaster/M compares its installed semantic version with the published release manifest. On Windows, <strong>Download Portable EXE</strong> saves the versioned executable beside the copy currently running, and <strong>Download &amp; Install MSI</strong> downloads the package to a temporary update folder and launches Windows Installer. On macOS, <strong>Download macOS DMG</strong> downloads the universal disk image to the update folder and opens it.</p>
      <p class="help-callout">The portable Windows download requires write access to the current application folder and expects WebView2 to be installed. The MSI is the recommended Windows choice for a normal installation and bare-machine prerequisites. macOS builds are distributed as universal DMGs.</p>
      <h3>Include this information</h3>
      <ul>
        <li>QuarterMaster/M version from the upper-right title bar.</li>
        <li>Windows version and whether WebView2 is installed.</li>
        <li>Exact steps, expected result, actual result, and full error text.</li>
        <li>Document mode, 40/80-column setting, filesystem type, disk geometry, and drive number.</li>
        <li>A minimal sample file or ATR image if it can be shared legally and safely.</li>
        <li>A screenshot with private paths or data redacted.</li>
      </ul>
    `,
  },
];

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

const highControlNames = new Map<number, string>([
  [0x1b, "Inverse Escape glyph; $9B is END OF LINE / RETURN in text"],
  [0x1c, "Inverse cursor-up glyph; $9C deletes a logical line"],
  [0x1d, "Inverse cursor-down glyph; $9D inserts a logical line"],
  [0x1e, "Inverse cursor-left glyph; $9E clears a tab stop"],
  [0x1f, "Inverse cursor-right glyph; $9F sets a tab stop"],
  [0x7d, "Inverse clear-screen glyph; $FD sounds the buzzer"],
  [0x7e, "Inverse backspace glyph; $FE deletes a character"],
  [0x7f, "Inverse tab glyph; $FF inserts a character"],
]);

function baseName(base: number): string {
  if (base < 0x20) {
    const key = base === 0 ? "Ctrl+@" : base <= 0x1a ? `Ctrl+${String.fromCharCode(0x40 + base)}` : "";
    return `${graphicNames[base]}${key ? ` · ${key}` : ""}`;
  }
  if (base === 0x20) return "Space";
  if (base >= 0x21 && base <= 0x5f) return `ASCII “${String.fromCharCode(base)}”`;
  if (base === 0x60) return "Atari diamond";
  if (base >= 0x61 && base <= 0x7a) return `Lowercase “${String.fromCharCode(base)}”`;
  if (base === 0x7b) return "Spade";
  if (base === 0x7c) return "Vertical bar";
  if (base === 0x7d) return "Clear screen · Ctrl+<";
  if (base === 0x7e) return "Backspace / delete · Backspace";
  return "Tab · Tab";
}

function screenCode(base: number): number {
  if (base < 0x20) return base + 0x40;
  if (base < 0x60) return base - 0x20;
  return base;
}

function hex(value: number): string {
  return `$${value.toString(16).padStart(2, "0").toUpperCase()}`;
}

function glyph(base: number, inverse: boolean): string {
  return `<span class="help-glyph cell ${atariGlyphClass(base)}${inverse ? " inverse" : ""}" role="img" aria-label="${inverse ? "Inverse " : ""}${baseName(base)}"></span>`;
}

function category(base: number): string {
  if (highControlNames.has(base) || base >= 0x7d) return "controls";
  if (base < 0x20 || base === 0x60 || base >= 0x7b) return "graphics";
  return "text";
}

function mappingRows(): HTMLTableRowElement[] {
  return Array.from({ length: 128 }, (_, base) => {
    const normal = baseName(base);
    const special = highControlNames.get(base);
    const screen = screenCode(base);
    const row = document.createElement("tr");
    row.dataset.category = category(base);
    row.dataset.search = [
      normal, special ?? "", base, base + 128, hex(base), hex(base + 128),
      hex(screen), hex(screen | 0x80),
    ].join(" ").toLowerCase();
    row.innerHTML = `
      <td class="help-glyph-pair">${glyph(base, false)}${glyph(base, true)}</td>
      <td><code>${hex(base)}</code><small>${base}</small></td>
      <td><code>${hex(base | 0x80)}</code><small>${base | 0x80}</small></td>
      <td><code>${hex(screen)}</code> / <code>${hex(screen | 0x80)}</code></td>
      <td>${normal}${special ? `<small class="atascii-special">${special}</small>` : ""}</td>`;
    return row;
  });
}

export function showHelpCenter(initialSection = "start", version = ""): void {
  document.querySelector(".help-backdrop")?.remove();
  const initial = helpSections.some(section => section.id === initialSection) ? initialSection : "start";
  const backdrop = document.createElement("div");
  backdrop.className = "modal-backdrop help-backdrop";
  backdrop.innerHTML = `
    <section class="help-dialog" role="dialog" aria-modal="true" aria-labelledby="helpTitle">
      <header class="help-header">
        <div><h2 id="helpTitle">QuarterMaster/M Help Center</h2><p>Complete application guide${version ? ` · V${version}` : ""}</p></div>
        <button type="button" class="help-close" aria-label="Close Help Center">Close</button>
      </header>
      <div class="help-layout">
        <nav class="help-nav" aria-label="Help topics">
          ${helpSections.map(section => `<button type="button" data-help-section="${section.id}"><strong>${section.title}</strong><small>${section.summary}</small></button>`).join("")}
        </nav>
        <article class="help-article" tabindex="0"></article>
      </div>
    </section>`;
  document.body.appendChild(backdrop);

  const article = backdrop.querySelector<HTMLElement>(".help-article")!;
  const close = (): void => {
    document.removeEventListener("keydown", onKeyDown);
    backdrop.remove();
  };
  const activate = (id: string): void => {
    const section = helpSections.find(candidate => candidate.id === id) ?? helpSections[0];
    backdrop.querySelectorAll<HTMLButtonElement>("[data-help-section]").forEach(button => {
      const active = button.dataset.helpSection === section.id;
      button.classList.toggle("active", active);
      button.setAttribute("aria-current", active ? "page" : "false");
    });
    article.innerHTML = `<div class="help-topic-heading"><p>QUARTERMASTER/M MANUAL</p><h2>${section.title}</h2><p>${section.summary}</p></div>${section.body}`;
    article.scrollTop = 0;
    if (section.id === "atascii") {
      const body = article.querySelector<HTMLTableSectionElement>("[data-atascii-body]")!;
      const rows = mappingRows();
      body.append(...rows);
      const search = article.querySelector<HTMLInputElement>("[data-atascii-search]")!;
      const group = article.querySelector<HTMLSelectElement>("[data-atascii-group]")!;
      const count = article.querySelector<HTMLElement>("[data-atascii-count]")!;
      const filter = (): void => {
        const query = search.value.trim().toLowerCase();
        let visible = 0;
        rows.forEach(row => {
          const matchesGroup = group.value === "all" || row.dataset.category === group.value;
          const matchesQuery = !query || row.dataset.search!.includes(query);
          row.hidden = !(matchesGroup && matchesQuery);
          if (!row.hidden) visible += 2;
        });
        count.textContent = `${visible} of 256 bytes`;
      };
      search.addEventListener("input", filter);
      group.addEventListener("change", filter);
      filter();
    }
    article.querySelector<HTMLButtonElement>("[data-copy-support]")?.addEventListener("click", async event => {
      const button = event.currentTarget as HTMLButtonElement;
      try {
        await navigator.clipboard.writeText(SUPPORT_URL);
        button.textContent = "Copied";
      } catch {
        window.prompt("Copy this support URL:", SUPPORT_URL);
      }
    });
    article.focus();
  };
  const onKeyDown = (event: KeyboardEvent): void => {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  };

  backdrop.querySelector(".help-close")!.addEventListener("click", close);
  backdrop.addEventListener("click", event => {
    if (event.target === backdrop) close();
  });
  backdrop.querySelectorAll<HTMLButtonElement>("[data-help-section]").forEach(button => {
    button.addEventListener("click", () => activate(button.dataset.helpSection!));
  });
  document.addEventListener("keydown", onKeyDown);
  activate(initial);
}
