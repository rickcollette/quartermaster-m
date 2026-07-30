import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import pngjs from "pngjs";

const { PNG } = pngjs;

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const sourcePath = path.join(root, "quartermaster-splash.png");
const outputPath = path.join(root, "public", "quartermaster-splash.png");
const version = fs.readFileSync(path.join(root, "VERSION"), "utf8").trim();
const label = `v${version}`;

const glyphs = {
  "0": ["111", "101", "101", "101", "101", "101", "111"],
  "1": ["010", "110", "010", "010", "010", "010", "111"],
  "2": ["111", "001", "001", "111", "100", "100", "111"],
  "3": ["111", "001", "001", "111", "001", "001", "111"],
  "4": ["101", "101", "101", "111", "001", "001", "001"],
  "5": ["111", "100", "100", "111", "001", "001", "111"],
  "6": ["111", "100", "100", "111", "101", "101", "111"],
  "7": ["111", "001", "001", "010", "010", "100", "100"],
  "8": ["111", "101", "101", "111", "101", "101", "111"],
  "9": ["111", "101", "101", "111", "001", "001", "111"],
  ".": ["0", "0", "0", "0", "0", "0", "1"],
  "v": ["00000", "00000", "10001", "10001", "01010", "01010", "00100"],
};

function textWidth(text, scale) {
  return [...text].reduce((sum, char, index) => {
    const glyph = glyphs[char];
    if (!glyph) throw new Error(`No splash glyph for ${char}`);
    return sum + glyph[0].length * scale + (index === text.length - 1 ? 0 : scale * 2);
  }, 0);
}

function blendPixel(image, x, y, color) {
  if (x < 0 || y < 0 || x >= image.width || y >= image.height) return;
  const offset = (image.width * y + x) << 2;
  const alpha = color[3] / 255;
  const inverse = 1 - alpha;
  image.data[offset] = Math.round(color[0] * alpha + image.data[offset] * inverse);
  image.data[offset + 1] = Math.round(color[1] * alpha + image.data[offset + 1] * inverse);
  image.data[offset + 2] = Math.round(color[2] * alpha + image.data[offset + 2] * inverse);
  image.data[offset + 3] = 255;
}

function fillRect(image, x, y, width, height, color) {
  for (let py = y; py < y + height; py += 1) {
    for (let px = x; px < x + width; px += 1) {
      blendPixel(image, px, py, color);
    }
  }
}

function drawText(image, text, x, y, scale, color) {
  let cursor = x;
  for (const char of text) {
    const glyph = glyphs[char];
    for (let row = 0; row < glyph.length; row += 1) {
      for (let col = 0; col < glyph[row].length; col += 1) {
        if (glyph[row][col] === "1") {
          fillRect(image, cursor + col * scale, y + row * scale, scale, scale, color);
        }
      }
    }
    cursor += glyph[0].length * scale + scale * 2;
  }
}

const image = PNG.sync.read(fs.readFileSync(sourcePath));
const scale = Math.max(5, Math.round(image.width / 210));
const width = textWidth(label, scale);
const x = Math.round(image.width - width - image.width * 0.035);
const y = Math.round(image.height * 0.045);

drawText(image, label, x + scale, y + scale, scale, [18, 38, 82, 180]);
drawText(image, label, x, y, scale, [4, 9, 18, 255]);

fs.mkdirSync(path.dirname(outputPath), { recursive: true });
fs.writeFileSync(outputPath, PNG.sync.write(image));
console.log(`Generated splash asset: ${outputPath} (${label})`);
