use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::atr::{add_file_bytes, current, extract_file_bytes, status, AtrState, AtrStatus};
use crate::basic_native::{detokenize_bytes, tokenize_listing};
use crate::document::{
    decode_document_bytes, encode_document_bytes, DocumentMode, LoadedDocument, SaveDocumentRequest,
};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BasicTokenizeRequest {
    pub destination: String,
    pub document: SaveDocumentRequest,
}

fn listing_text(document: &SaveDocumentRequest) -> Result<String> {
    let mut request = document.clone();
    request.mode = DocumentMode::Ascii;
    let bytes = encode_document_bytes(&request)?;
    String::from_utf8(bytes).map_err(|e| format!("BASIC listing is not valid UTF-8/ASCII: {e}"))
}

fn write_host(path: &str, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| format!("cannot create destination folder: {e}"))?;
    }
    fs::write(path, bytes).map_err(|e| format!("cannot write {path}: {e}"))
}

fn extract_atr_bytes(image_name: &str, state: &tauri::State<'_, AtrState>) -> Result<Vec<u8>> {
    let mounted = current(state)?;
    extract_file_bytes(&mounted, image_name)
}

fn add_bytes_to_atr(
    image_name: &str,
    bytes: &[u8],
    state: &tauri::State<'_, AtrState>,
) -> Result<AtrStatus> {
    let mounted = current(state)?;
    add_file_bytes(&mounted, image_name, bytes)?;
    status(state)
}

#[tauri::command]
pub fn basic_detokenize_host(path: String, width: usize, height: usize) -> Result<LoadedDocument> {
    let bytes = fs::read(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let listing = detokenize_bytes(&bytes)?;
    decode_document_bytes(
        listing.into_bytes(),
        Some(path),
        DocumentMode::Ascii,
        width,
        height,
    )
}

#[tauri::command]
pub fn basic_tokenize_host(request: BasicTokenizeRequest) -> Result<()> {
    let text = listing_text(&request.document)?;
    let tokenized = tokenize_listing(&text)?;
    write_host(&request.destination, &tokenized)
}

#[tauri::command]
pub fn basic_save_listing_host(request: BasicTokenizeRequest) -> Result<()> {
    let bytes = encode_document_bytes(&request.document)?;
    write_host(&request.destination, &bytes)
}

#[tauri::command]
pub fn basic_detokenize_atr(
    image_name: String,
    width: usize,
    height: usize,
    state: tauri::State<'_, AtrState>,
) -> Result<LoadedDocument> {
    let bytes = extract_atr_bytes(&image_name, &state)?;
    let listing = detokenize_bytes(&bytes)?;
    decode_document_bytes(
        listing.into_bytes(),
        None,
        DocumentMode::Ascii,
        width,
        height,
    )
}

#[tauri::command]
pub fn basic_tokenize_to_atr(
    image_name: String,
    document: SaveDocumentRequest,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus> {
    let text = listing_text(&document)?;
    let tokenized = tokenize_listing(&text)?;
    add_bytes_to_atr(&image_name, &tokenized, &state)
}

#[tauri::command]
pub fn basic_save_listing_to_atr(
    image_name: String,
    document: SaveDocumentRequest,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus> {
    let bytes = encode_document_bytes(&document)?;
    add_bytes_to_atr(&image_name, &bytes, &state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_round_trip_simple_program() {
        let source = "10 PRINT \"HELLO\"\n20 END\n";
        let binary = tokenize_listing(source).unwrap();
        let listing = detokenize_bytes(&binary).unwrap();
        assert!(listing.contains("10 PRINT \"HELLO\""));
        assert!(listing.contains("20 END"));
    }
}
