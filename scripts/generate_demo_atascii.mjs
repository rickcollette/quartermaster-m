import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(fileURLToPath(new URL("..", import.meta.url)));
const examples = resolve(root, "docs/examples");
const sources = [
  resolve(examples, "quartermaster-command-deck.screen.json"),
  resolve(examples, "quartermaster-screen-composer.screen.json"),
];

const graphics = {
  top: [0x11, 0x12, 0x05],
  separator: [0x01, 0x12, 0x04],
  bottom: [0x16, 0x12, 0x03],
};

function compile(sourcePath) {
  const spec = JSON.parse(readFileSync(sourcePath, "utf8"));
  if (spec.width !== 40) throw new Error(`${sourcePath}: screenshot demos must be 40 columns`);
  if (!Array.isArray(spec.rows) || spec.rows.length !== 24) {
    throw new Error(`${sourcePath}: expected exactly 24 rows, found ${spec.rows?.length ?? 0}`);
  }
  if (!/^[A-Za-z0-9._-]+\.ata$/i.test(spec.output)) {
    throw new Error(`${sourcePath}: invalid output filename ${spec.output}`);
  }

  const bytes = [];
  const preview = [];
  spec.rows.forEach((row, index) => {
    let values;
    let previewRow;
    if (row.kind) {
      const definition = graphics[row.kind];
      if (!definition) throw new Error(`${sourcePath}: row ${index + 1} has unknown kind ${row.kind}`);
      const [left, fill, right] = definition;
      values = [left, ...Array(38).fill(fill), right];
      const previewFill = row.kind === "separator" ? "-" : "-";
      previewRow = `${row.kind === "separator" ? "+" : "+"}${previewFill.repeat(38)}+`;
    } else {
      const text = String(row.text ?? "");
      if ([...text].some(character => character.codePointAt(0) > 0x7f)) {
        throw new Error(`${sourcePath}: row ${index + 1} contains non-ASCII source text`);
      }
      if (text.length > 38) {
        throw new Error(`${sourcePath}: row ${index + 1} has ${text.length} characters; maximum is 38`);
      }
      const inner = text.padEnd(38, " ");
      values = [0x17, ...Buffer.from(inner, "ascii"), 0x17];
      previewRow = `|${inner}|`;
      if (row.inverse) values = values.map(value => value | 0x80);
    }
    if (values.length !== 40) throw new Error(`${sourcePath}: row ${index + 1} did not compile to 40 bytes`);
    bytes.push(...values);
    if (index + 1 < spec.rows.length) bytes.push(0x9b);
    preview.push(previewRow);
  });

  const outputPath = resolve(examples, spec.output);
  if (dirname(outputPath) !== examples) throw new Error(`${sourcePath}: output escapes examples directory`);
  writeFileSync(outputPath, Buffer.from(bytes));
  writeFileSync(outputPath.replace(/\.ata$/i, ".txt"), `${preview.join("\r\n")}\r\n`, "ascii");
  console.log(`Generated ${basename(outputPath)}: ${bytes.length} bytes, 40×24`);
}

mkdirSync(examples, { recursive: true });
sources.forEach(compile);
