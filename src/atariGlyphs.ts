export function atariGlyphClass(byte: number): string {
  return `glyph-${(byte & 0x7f).toString(16).padStart(2, "0")}`;
}

export function atariGlyphLabel(byte: number): string {
  const base = byte & 0x7f;
  if (base >= 0x01 && base <= 0x1a) {
    return `Ctrl-${String.fromCharCode(base + 0x40)}`;
  }
  if (base === 0x00) return "Ctrl-@";
  if (base >= 0x20 && base <= 0x7e) return String.fromCharCode(base);
  return `$${base.toString(16).padStart(2, "0").toUpperCase()}`;
}

export function controlByteForKey(event: KeyboardEvent): number | null {
  if (!event.ctrlKey || event.metaKey || event.altKey || event.shiftKey) return null;
  const key = event.key.toLowerCase();
  if (!/^[a-z]$/.test(key)) return null;
  return key.charCodeAt(0) - 0x60;
}
