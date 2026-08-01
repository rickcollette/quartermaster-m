use atascii::{AtasciiByte, Charset, Control, DecodeDomain, Glyph, Parser, Token};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentMode {
    Atascii,
    Ascii,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CellDto {
    pub byte: u8,
    pub inverse: bool,
    pub display: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedDocument {
    pub path: Option<String>,
    pub mode: DocumentMode,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<CellDto>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDocumentRequest {
    pub path: String,
    pub mode: DocumentMode,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<CellDto>,
    pub trim_trailing_spaces: bool,
}

fn blank_cell() -> CellDto {
    CellDto {
        byte: b' ',
        inverse: false,
        display: " ".into(),
    }
}

fn glyph_cell(glyph: Glyph) -> CellDto {
    let base = glyph.id.0;
    CellDto {
        byte: base,
        inverse: glyph.inverse,
        display: glyph.unicode_approx().to_string(),
    }
}

pub(crate) fn decode_document_bytes(
    bytes: Vec<u8>,
    path: Option<String>,
    mode: DocumentMode,
    width: usize,
    height: usize,
) -> Result<LoadedDocument, String> {
    if width == 0 || height == 0 || width > 256 || height > 4096 {
        return Err("invalid document dimensions".into());
    }
    let mut cells = vec![blank_cell(); width * height];
    let mut warnings = Vec::new();
    let mut row = 0usize;
    let mut col = 0usize;
    let mut wrapped_at_width = false;

    match mode {
        DocumentMode::Atascii => {
            let mut parser = Parser::new(DecodeDomain::TextFile, Charset::Standard);
            for raw in bytes {
                match parser.feed(AtasciiByte(raw)) {
                    Some(Token::Control(Control::EndOfLine)) => {
                        if !wrapped_at_width {
                            row += 1;
                        }
                        col = 0;
                        wrapped_at_width = false;
                    }
                    Some(Token::Glyph(glyph)) => {
                        wrapped_at_width = false;
                        if row < height && col < width {
                            cells[row * width + col] = glyph_cell(glyph);
                        }
                        col += 1;
                        if col >= width {
                            row += 1;
                            col = 0;
                            wrapped_at_width = true;
                        }
                    }
                    Some(Token::Control(_)) | Some(Token::Raw(_)) | None => {}
                }
                if row >= height {
                    break;
                }
            }
        }
        DocumentMode::Ascii => {
            let text = String::from_utf8_lossy(&bytes)
                .replace("\r\n", "\n")
                .replace('\r', "\n");
            for ch in text.chars() {
                if ch == '\n' {
                    if !wrapped_at_width {
                        row += 1;
                    }
                    col = 0;
                    wrapped_at_width = false;
                    continue;
                }
                wrapped_at_width = false;
                let byte = if ch.is_ascii() {
                    ch as u8
                } else {
                    warnings.push(format!(
                        "replaced unsupported Unicode character {ch:?} with '?'"
                    ));
                    b'?'
                };
                if row < height && col < width {
                    cells[row * width + col] = CellDto {
                        byte,
                        inverse: false,
                        display: (byte as char).to_string(),
                    };
                }
                col += 1;
                if col >= width {
                    row += 1;
                    col = 0;
                    wrapped_at_width = true;
                }
                if row >= height {
                    break;
                }
            }
        }
    }

    Ok(LoadedDocument {
        path,
        mode,
        width,
        height,
        cells,
        warnings,
    })
}

pub(crate) fn encode_document_bytes(request: &SaveDocumentRequest) -> Result<Vec<u8>, String> {
    if request.width == 0 || request.cells.len() < request.width * request.height {
        return Err("document cell buffer is incomplete".into());
    }
    let mut output = Vec::new();
    for row in 0..request.height {
        let start = row * request.width;
        let mut end = start + request.width;
        if request.trim_trailing_spaces {
            while end > start
                && request.cells[end - 1].byte == b' '
                && !request.cells[end - 1].inverse
            {
                end -= 1;
            }
        }
        for cell in &request.cells[start..end] {
            let base = cell.byte & 0x7f;
            match request.mode {
                DocumentMode::Atascii => output.push(base | if cell.inverse { 0x80 } else { 0 }),
                DocumentMode::Ascii => output.push(if base.is_ascii() { base } else { b'?' }),
            }
        }
        if row + 1 < request.height {
            match request.mode {
                DocumentMode::Atascii => output.push(0x9b),
                DocumentMode::Ascii => output.extend_from_slice(b"\r\n"),
            }
        }
    }
    Ok(output)
}

pub(crate) fn ascii_text_to_atascii(bytes: &[u8]) -> Vec<u8> {
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                if bytes.get(index + 1) == Some(&b'\n') {
                    index += 1;
                }
                output.push(0x9b);
            }
            b'\n' => output.push(0x9b),
            b'\t' => output.push(0x7f),
            byte @ 0x20..=0x7e => output.push(byte),
            _ => output.push(b'?'),
        }
        index += 1;
    }
    output
}

pub(crate) fn atascii_text_to_ascii(bytes: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(bytes.len() + bytes.len() / 40);
    for &raw in bytes {
        match raw {
            0x9b => output.extend_from_slice(b"\r\n"),
            0x7f => output.push(b'\t'),
            _ => {
                let byte = raw & 0x7f;
                if (0x20..=0x7e).contains(&byte) {
                    output.push(byte);
                }
            }
        }
    }
    output
}

#[tauri::command]
pub fn load_document(
    path: String,
    mode: DocumentMode,
    width: usize,
    height: usize,
) -> Result<LoadedDocument, String> {
    let bytes = fs::read(&path).map_err(|error| format!("cannot read {path}: {error}"))?;
    decode_document_bytes(bytes, Some(path), mode, width, height)
}

#[tauri::command]
pub fn save_document(request: SaveDocumentRequest) -> Result<(), String> {
    let output = encode_document_bytes(&request)?;
    if let Some(parent) = Path::new(&request.path).parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create destination folder: {error}"))?;
    }
    fs::write(&request.path, output)
        .map_err(|error| format!("cannot save {}: {error}", request.path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atascii_eol_is_9b() {
        let request = SaveDocumentRequest {
            path: std::env::temp_dir()
                .join("qm-test.ata")
                .to_string_lossy()
                .into_owned(),
            mode: DocumentMode::Atascii,
            width: 2,
            height: 2,
            cells: vec![
                CellDto {
                    byte: b'A',
                    inverse: false,
                    display: "A".into(),
                },
                blank_cell(),
                CellDto {
                    byte: b'B',
                    inverse: true,
                    display: "B".into(),
                },
                blank_cell(),
            ],
            trim_trailing_spaces: true,
        };
        assert_eq!(
            encode_document_bytes(&request).unwrap(),
            vec![b'A', 0x9b, b'B' | 0x80]
        );
    }

    #[test]
    fn empty_trimmed_document_saves_zero_bytes() {
        let request = SaveDocumentRequest {
            path: String::new(),
            mode: DocumentMode::Atascii,
            width: 40,
            height: 0,
            cells: vec![],
            trim_trailing_spaces: true,
        };
        assert_eq!(encode_document_bytes(&request).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn drag_text_conversion_normalizes_lines_and_inverse_text() {
        assert_eq!(
            ascii_text_to_atascii(b"\xef\xbb\xbfHELLO\r\nWORLD\n"),
            b"HELLO\x9bWORLD\x9b"
        );
        assert_eq!(
            atascii_text_to_ascii(&[b'H' | 0x80, b'I' | 0x80, 0x9b, b'X']),
            b"HI\r\nX"
        );
    }

    #[test]
    fn exact_width_rows_round_trip_without_inserting_blank_lines() {
        for mode in [DocumentMode::Atascii, DocumentMode::Ascii] {
            let cells = b"BBBBCCCC        "
                .iter()
                .map(|byte| CellDto {
                    byte: *byte,
                    inverse: false,
                    display: (*byte as char).to_string(),
                })
                .collect();
            let request = SaveDocumentRequest {
                path: String::new(),
                mode,
                width: 4,
                height: 4,
                cells,
                trim_trailing_spaces: true,
            };
            let encoded = encode_document_bytes(&request).unwrap();
            let loaded = decode_document_bytes(encoded, None, mode, 4, 4).unwrap();
            let bytes: Vec<u8> = loaded.cells.into_iter().map(|cell| cell.byte).collect();
            assert_eq!(bytes, b"BBBBCCCC        ");
        }
    }
}
