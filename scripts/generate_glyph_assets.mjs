import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outputDir = resolve(root, "src", "generated");
const svgPath = resolve(outputDir, "atari-glyphs.svg");
const cssPath = resolve(outputDir, "atari-glyphs.css");
const glyphSize = 8;
const sheetColumns = 16;
const sheetRows = 16;
const atariBlue = "#1f4cad";
const atariWhite = "#f4f6ff";
const standardScreenFontHex = [
  "0000000000000000001818181800180000666666000000000066ff6666ff6600183e603c067c180000666c1830664600",
  "1c361c386f663b000018181800000000000e1c18181c0e00007038181838700000663cff3c6600000018187e18180000",
  "00000000001818300000007e00000000000000000018180000060c1830604000003c666e76663c000018381818187e00",
  "003c660c18307e00007e0c180c663c00000c1c3c6c7e0c00007e607c06663c00003c607c66663c00007e060c18303000",
  "003c663c66663c00003c663e060c380000001818001818000000181800181830060c1830180c060000007e00007e0000",
  "6030180c18306000003c660c18001800003c666e6e603e0000183c66667e6600007c667c66667c00003c666060663c00",
  "00786c66666c7800007e607c60607e00007e607c60606000003e60606e663e000066667e66666600007e181818187e00",
  "0006060606663c0000666c78786c66000060606060607e000063777f6b6363000066767e7e6e6600003c666666663c00",
  "007c66667c606000003c6666666c3600007c66667c6c6600003c603c06063c00007e1818181818000066666666667e00",
  "00666666663c18000063636b7f7763000066663c3c6666000066663c18181800007e0c1830607e00001e181818181e00",
  "00406030180c0600007818181818780000081c3663000000000000000000ff0000367f7f3e1c08001818181f1f181818",
  "0303030303030303181818f8f8000000181818f8f8181818000000f8f818181803070e1c3870e0c0c0e070381c0e0703",
  "0103070f1f3f7fff000000000f0f0f0f80c0e0f0f8fcfeff0f0f0f0f00000000f0f0f0f000000000ffff000000000000",
  "000000000000ffff00000000f0f0f0f0001c1c7777081c000000001f1f181818000000ffff000000181818ffff181818",
  "00003c7e7e7e3c0000000000ffffffffc0c0c0c0c0c0c0c0000000ffff181818181818ffff000000f0f0f0f0f0f0f0f0",
  "1818181f1f000000786078607e181e0000183c7e18181800001818187e3c18000018307e3018000000180c7e0c180000",
  "00183c7e7e3c180000003c063e663e000060607c66667c0000003c6060603c000006063e66663e0000003c667e603c00",
  "000e183e1818180000003e66663e067c0060607c666666000018003818183c00000600060606063c0060606c786c6600",
  "0038181818183c000000667f7f6b630000007c666666660000003c6666663c0000007c66667c606000003e66663e0606",
  "00007c666060600000003e603c067c0000187e1818180e000000666666663e0000006666663c18000000636b7f3e3600",
  "0000663c183c660000006666663e0c7800007e0c18307e0000183c7e7e183c001818181818181818007e787c6e660600",
  "081838783818080010181c1e1c181000",
].join("");
const standardScreenFont = Uint8Array.from(
  standardScreenFontHex.match(/../g).map(byte => Number.parseInt(byte, 16)),
);

if (standardScreenFont.length !== 128 * glyphSize) {
  throw new Error(`Expected 1024 Atari glyph bytes, got ${standardScreenFont.length}`);
}

function screenCodeForAtascii(byte) {
  const base = byte & 0x7f;
  if (base < 0x20) return base + 0x40;
  if (base < 0x60) return base - 0x20;
  return base;
}

function patternFor(byte) {
  const screenCode = screenCodeForAtascii(byte);
  const offset = screenCode * glyphSize;
  return Array.from(standardScreenFont.slice(offset, offset + glyphSize), row =>
    Array.from({ length: glyphSize }, (_, index) => (row & (0x80 >> index) ? "#" : ".")).join(""),
  );
}

function isOn(value) {
  return value !== "." && value !== "0";
}

function drawGlyph(rects, pattern, cellColumn, cellRow, inverse) {
  const originX = cellColumn * glyphSize;
  const originY = cellRow * glyphSize;
  const background = inverse ? atariWhite : atariBlue;
  const foreground = inverse ? atariBlue : atariWhite;
  rects.push(`<rect x="${originX}" y="${originY}" width="${glyphSize}" height="${glyphSize}" fill="${background}"/>`);
  pattern.forEach((row, rowIndex) => {
    for (let columnIndex = 0; columnIndex < glyphSize; columnIndex += 1) {
      if (isOn(row[columnIndex])) {
        rects.push(`<rect x="${originX + columnIndex}" y="${originY + rowIndex}" width="1" height="1" fill="${foreground}"/>`);
      }
    }
  });
}

function percent(index) {
  return `${((index / (sheetColumns - 1)) * 100).toFixed(6).replace(/\.?0+$/, "")}%`;
}

function glyphClass(byte) {
  return `glyph-${byte.toString(16).padStart(2, "0")}`;
}

function generateSvg() {
  const rects = [];
  for (let byte = 0; byte <= 0x7f; byte += 1) {
    const column = byte % sheetColumns;
    const row = Math.floor(byte / sheetColumns);
    const pattern = patternFor(byte);
    drawGlyph(rects, pattern, column, row, false);
    drawGlyph(rects, pattern, column, row + 8, true);
  }
  return [
    '<?xml version="1.0" encoding="UTF-8"?>',
    `<svg xmlns="http://www.w3.org/2000/svg" width="${sheetColumns * glyphSize}" height="${sheetRows * glyphSize}" viewBox="0 0 ${sheetColumns * glyphSize} ${sheetRows * glyphSize}" shape-rendering="crispEdges">`,
    ...rects,
    "</svg>",
    "",
  ].join("\n");
}

function generateCss() {
  const rules = [
    "/* Generated by scripts/generate_glyph_assets.mjs. Do not edit directly. */",
    `.cell{background-image:url("./atari-glyphs.svg");background-size:${sheetColumns * 100}% ${sheetRows * 100}%;background-repeat:no-repeat;background-position:0 0}`,
  ];
  for (let byte = 0; byte <= 0x7f; byte += 1) {
    const column = byte % sheetColumns;
    const row = Math.floor(byte / sheetColumns);
    const normalPosition = `${percent(column)} ${percent(row)}`;
    const inversePosition = `${percent(column)} ${percent(row + 8)}`;
    rules.push(`.${glyphClass(byte)}{background-position:${normalPosition}}`);
    rules.push(`.${glyphClass(byte)}.inverse{background-position:${inversePosition}}`);
  }
  return `${rules.join("\n")}\n`;
}

mkdirSync(outputDir, { recursive: true });
writeFileSync(svgPath, generateSvg(), "utf8");
writeFileSync(cssPath, generateCss(), "utf8");
console.log(`Generated Atari glyph sprite: ${svgPath}`);
console.log(`Generated Atari glyph CSS: ${cssPath}`);
