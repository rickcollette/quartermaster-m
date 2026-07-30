use broadside_core::{
    dos2::Dos2,
    execute,
    image::{open_image, DiskImage},
    sparta::Sparta,
    Command, FsType, MediaType, OperationOptions,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

use crate::basic_native::{detokenize_bytes, tokenize_listing};
use crate::document::{
    ascii_text_to_atascii, atascii_text_to_ascii, decode_document_bytes, encode_document_bytes,
    DocumentMode, LoadedDocument, SaveDocumentRequest,
};

const DRIVE_COUNT: usize = 4;
const LOCAL_TREE_DEPTH: usize = 2;
const LOCAL_TREE_LIMIT: usize = 250;

#[derive(Debug, Clone)]
pub(crate) struct MountedAtr {
    pub(crate) path: PathBuf,
    pub(crate) fs: FsType,
}

#[derive(Debug, Clone)]
pub(crate) struct FileManagerState {
    pub(crate) local_folder: Option<PathBuf>,
    pub(crate) active_drive: usize,
    pub(crate) drives: Vec<Option<MountedAtr>>,
}

impl Default for FileManagerState {
    fn default() -> Self {
        Self {
            local_folder: None,
            active_drive: 0,
            drives: vec![None; DRIVE_COUNT],
        }
    }
}

#[derive(Default)]
pub struct AtrState(pub Mutex<FileManagerState>);

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedMountedAtr {
    path: String,
    filesystem: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PersistedFileManagerState {
    local_folder: Option<String>,
    active_drive: usize,
    drives: Vec<Option<PersistedMountedAtr>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtrCreateRequest {
    pub path: String,
    pub filesystem: String,
    pub drive: Option<u8>,
    pub sectors: u32,
    pub sector_size: usize,
    pub volume_label: Option<String>,
    pub force: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtrStatus {
    pub mounted: bool,
    pub active_drive: Option<String>,
    pub path: Option<String>,
    pub filesystem: Option<String>,
    pub entries: Vec<String>,
    pub info: Vec<String>,
    pub tree: Vec<AtrTreeEntry>,
    pub drives: Vec<AtrDriveStatus>,
    pub local_folder: Option<String>,
    pub local_tree: Vec<LocalTreeEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AtrDriveStatus {
    pub drive: String,
    pub mounted: bool,
    pub active: bool,
    pub path: Option<String>,
    pub filesystem: Option<String>,
    pub entries: Vec<String>,
    pub info: Vec<String>,
    pub tree: Vec<AtrTreeEntry>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtrTreeEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub children: Vec<AtrTreeEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTreeEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size_bytes: u64,
    pub children: Vec<LocalTreeEntry>,
}

fn parse_fs(value: &str) -> Result<FsType, String> {
    match value.to_ascii_lowercase().as_str() {
        "dos2" | "atari" => Ok(FsType::Dos2),
        "sparta" | "sparta2" | "sdx" => Ok(FsType::Sparta2),
        _ => Err(format!("unsupported ATR filesystem: {value}")),
    }
}

fn validate_dos2(image: &mut dyn DiskImage) -> broadside_core::Result<()> {
    if image.total_sectors() < 368 {
        return Err(broadside_core::BroadsideError::InvalidImage(
            "image is too small for Atari DOS 2".into(),
        ));
    }
    let vtoc = image.read_sector(360)?;
    if vtoc.len() < 100 || vtoc[0] != 2 {
        return Err(broadside_core::BroadsideError::InvalidImage(
            "not an Atari DOS 2 VTOC".into(),
        ));
    }
    let declared = u16::from_le_bytes([vtoc[1], vtoc[2]]) as u32;
    let free = u16::from_le_bytes([vtoc[3], vtoc[4]]) as u32;
    if declared == 0 || declared > image.total_sectors() || free > declared {
        return Err(broadside_core::BroadsideError::InvalidImage(
            "invalid Atari DOS 2 VTOC sector counts".into(),
        ));
    }
    let mut dos = Dos2::new(image);
    dos.check()?;
    dos.list("*.*")?;
    Ok(())
}

fn detect_fs(path: &PathBuf) -> Result<FsType, String> {
    let mut image = open_image(path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
    match Sparta::new(image.as_mut()).and_then(|mut sparta| sparta.free_sectors().map(|_| ())) {
        Ok(()) => return Ok(FsType::Sparta2),
        Err(sparta_error) => {
            drop(image);
            let mut image = open_image(path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
            validate_dos2(image.as_mut()).map_err(|dos_error| {
                format!(
                    "could not detect ATR filesystem as SpartaDOS 2 or Atari DOS 2: {sparta_error}; {dos_error}"
                )
            })?;
        }
    }
    Ok(FsType::Dos2)
}

fn persisted_mount_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create app data folder: {e}"))?;
    Ok(dir.join("virtual-drive.json"))
}

fn drive_label(index: usize) -> String {
    format!("D{}:", index + 1)
}

fn drive_index(drive: Option<u8>, state: &FileManagerState) -> Result<usize, String> {
    match drive {
        Some(value @ 1..=4) => Ok(value as usize - 1),
        Some(value) => Err(format!("drive must be D1: through D4:, got D{value}:")),
        None => Ok(state.active_drive.min(DRIVE_COUNT - 1)),
    }
}

fn choose_mount_index(drive: Option<u8>, state: &FileManagerState) -> Result<usize, String> {
    if drive.is_some() {
        return drive_index(drive, state);
    }
    Ok(state
        .drives
        .iter()
        .position(Option::is_none)
        .unwrap_or_else(|| state.active_drive.min(DRIVE_COUNT - 1)))
}

fn persist_state(app: &tauri::AppHandle, state: &FileManagerState) -> Result<(), String> {
    let path = persisted_mount_path(app)?;
    if state.local_folder.is_some() || state.drives.iter().any(Option::is_some) {
        let persisted = PersistedFileManagerState {
            local_folder: state
                .local_folder
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            active_drive: state.active_drive.min(DRIVE_COUNT - 1),
            drives: state
                .drives
                .iter()
                .map(|mounted| {
                    mounted.as_ref().map(|mounted| PersistedMountedAtr {
                        path: mounted.path.to_string_lossy().into_owned(),
                        filesystem: match mounted.fs {
                            FsType::Dos2 => "dos2".into(),
                            FsType::Sparta2 => "sparta2".into(),
                        },
                    })
                })
                .collect(),
        };
        let data = serde_json::to_string_pretty(&persisted)
            .map_err(|e| format!("cannot serialize explorer state: {e}"))?;
        fs::write(&path, data).map_err(|e| format!("cannot save explorer state: {e}"))?;
    } else if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("cannot clear explorer state: {e}"))?;
    }
    Ok(())
}

fn restore_state(app: &tauri::AppHandle) -> Result<FileManagerState, String> {
    let path = persisted_mount_path(app)?;
    if !path.exists() {
        return Ok(FileManagerState::default());
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("cannot read explorer state: {e}"))?;
    let persisted: PersistedFileManagerState =
        serde_json::from_str(&data).map_err(|e| format!("cannot parse explorer state: {e}"))?;
    let mut state = FileManagerState {
        local_folder: persisted.local_folder.map(PathBuf::from),
        active_drive: persisted.active_drive.min(DRIVE_COUNT - 1),
        drives: vec![None; DRIVE_COUNT],
    };
    for (index, mounted) in persisted.drives.into_iter().take(DRIVE_COUNT).enumerate() {
        if let Some(mounted) = mounted {
            let restored = MountedAtr {
                path: PathBuf::from(mounted.path),
                fs: parse_fs(&mounted.filesystem)?,
            };
            drive_status_for(index, state.active_drive, restored.clone())?;
            state.drives[index] = Some(restored);
        }
    }
    Ok(state)
}

fn tree_for(mounted: &MountedAtr) -> Result<Vec<AtrTreeEntry>, String> {
    let mut image = open_image(&mounted.path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
    match mounted.fs {
        FsType::Dos2 => {
            let mut dos = Dos2::new(image.as_mut());
            let mut entries = dos
                .list("*.*")
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|entry| AtrTreeEntry {
                    path: entry.name.clone(),
                    name: entry.name,
                    is_directory: false,
                    size_bytes: entry.size_bytes as u64,
                    children: vec![],
                })
                .collect::<Vec<_>>();
            sort_tree_entries(&mut entries);
            Ok(entries)
        }
        FsType::Sparta2 => {
            let mut sparta = Sparta::new(image.as_mut()).map_err(|e| e.to_string())?;
            sparta_tree(&mut sparta, "", 0)
        }
    }
}

fn sparta_tree(
    sparta: &mut Sparta<'_>,
    path: &str,
    depth: usize,
) -> Result<Vec<AtrTreeEntry>, String> {
    if depth > 32 {
        return Err("SpartaDOS directory tree is too deep".into());
    }
    let rows = sparta.list(path, "*").map_err(|e| e.to_string())?;
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let child_path = if path.is_empty() {
            row.name.clone()
        } else {
            format!("{path}>{}", row.name)
        };
        let is_directory = row.is_dir();
        let children = if is_directory {
            sparta_tree(sparta, &child_path, depth + 1)?
        } else {
            vec![]
        };
        entries.push(AtrTreeEntry {
            name: row.name,
            path: child_path,
            is_directory,
            size_bytes: row.size_bytes as u64,
            children,
        });
    }
    sort_tree_entries(&mut entries);
    Ok(entries)
}

fn sort_tree_entries(entries: &mut [AtrTreeEntry]) {
    entries.sort_by(|left, right| {
        right
            .is_directory
            .cmp(&left.is_directory)
            .then_with(|| left.name.cmp(&right.name))
    });
}

pub(crate) fn options(command: Command, mounted: &MountedAtr) -> OperationOptions {
    OperationOptions {
        command,
        image: mounted.path.clone(),
        second_image: None,
        media: Some(MediaType::Atr),
        fs: mounted.fs,
        sectors: 720,
        sector_size: 128,
        input: None,
        output: None,
        name: None,
        mask: if mounted.fs == FsType::Sparta2 {
            ">*".into()
        } else {
            "*.*".into()
        },
        list_format: None,
        volume_label: None,
        force: false,
        copy_files: vec![],
    }
}

pub(crate) fn extract_file_bytes(
    mounted: &MountedAtr,
    image_name: &str,
) -> Result<Vec<u8>, String> {
    let mut image = open_image(&mounted.path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
    match mounted.fs {
        FsType::Dos2 => Dos2::new(image.as_mut())
            .extract(image_name)
            .map_err(|e| e.to_string()),
        FsType::Sparta2 => Sparta::new(image.as_mut())
            .map_err(|e| e.to_string())?
            .extract(image_name)
            .map_err(|e| e.to_string()),
    }
}

pub(crate) fn add_file_bytes(
    mounted: &MountedAtr,
    image_name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let mut image = open_image(&mounted.path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
    match mounted.fs {
        FsType::Dos2 => Dos2::new(image.as_mut())
            .insert(image_name, bytes)
            .map_err(|e| e.to_string()),
        FsType::Sparta2 => Sparta::new(image.as_mut())
            .map_err(|e| e.to_string())?
            .insert(image_name, bytes)
            .map_err(|e| e.to_string()),
    }
}

fn replace_file_bytes(
    mounted: &MountedAtr,
    image_name: &str,
    bytes: &[u8],
    overwrite: bool,
) -> Result<(), String> {
    let mut image = open_image(&mounted.path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
    match mounted.fs {
        FsType::Dos2 => {
            let mut dos = Dos2::new(image.as_mut());
            let exists = dos
                .list("*.*")
                .map_err(|e| e.to_string())?
                .iter()
                .any(|entry| entry.name.eq_ignore_ascii_case(image_name));
            if !exists {
                return dos.insert(image_name, bytes).map_err(|e| e.to_string());
            }
            if !overwrite {
                return Err(format!("{image_name} already exists"));
            }
            let original = dos.extract(image_name).map_err(|e| e.to_string())?;
            dos.delete(image_name, false).map_err(|e| e.to_string())?;
            if let Err(error) = dos.insert(image_name, bytes) {
                return match dos.insert(image_name, &original) {
                    Ok(()) => Err(error.to_string()),
                    Err(restore_error) => Err(format!(
                        "{error}; restoring the original file also failed: {restore_error}"
                    )),
                };
            }
            Ok(())
        }
        FsType::Sparta2 => {
            let split = image_name.rfind(['>', '/', '\\']);
            let (parent, name) = split
                .map(|index| (&image_name[..index], &image_name[index + 1..]))
                .unwrap_or(("", image_name));
            let mut sparta = Sparta::new(image.as_mut()).map_err(|e| e.to_string())?;
            let existing = sparta
                .list(parent, "*")
                .map_err(|e| e.to_string())?
                .into_iter()
                .find(|entry| entry.name.eq_ignore_ascii_case(name));
            let Some(existing) = existing else {
                return sparta.insert(image_name, bytes).map_err(|e| e.to_string());
            };
            if existing.is_dir() {
                return Err(format!("{image_name} is a directory"));
            }
            if !overwrite {
                return Err(format!("{image_name} already exists"));
            }
            let original = sparta.extract(image_name).map_err(|e| e.to_string())?;
            sparta
                .delete(image_name, false)
                .map_err(|e| e.to_string())?;
            if let Err(error) = sparta.insert(image_name, bytes) {
                return match sparta.insert(image_name, &original) {
                    Ok(()) => Err(error.to_string()),
                    Err(restore_error) => Err(format!(
                        "{error}; restoring the original file also failed: {restore_error}"
                    )),
                };
            }
            Ok(())
        }
    }
}

fn atari_file_name(path: &Path) -> Result<String, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} has no usable filename", path.display()))?;
    let file_path = Path::new(file_name);
    let clean = |value: &str, limit: usize| {
        value
            .chars()
            .filter_map(|character| {
                let upper = character.to_ascii_uppercase();
                (upper.is_ascii_alphanumeric() || upper == '_').then_some(upper)
            })
            .take(limit)
            .collect::<String>()
    };
    let stem = clean(
        file_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("FILE"),
        8,
    );
    let stem = if stem.is_empty() { "FILE".into() } else { stem };
    let extension = clean(
        file_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or(""),
        3,
    );
    Ok(if extension.is_empty() {
        stem
    } else {
        format!("{stem}.{extension}")
    })
}

fn atr_destination_path(
    mounted: &MountedAtr,
    destination_directory: &str,
    name: &str,
) -> Result<String, String> {
    let directory = destination_directory
        .trim()
        .trim_matches(|character| matches!(character, '>' | '/' | '\\'));
    if mounted.fs == FsType::Dos2 {
        if !directory.is_empty() {
            return Err("Atari DOS 2 disks do not support subdirectories".into());
        }
        return Ok(name.to_ascii_uppercase());
    }
    Ok(if directory.is_empty() {
        name.to_ascii_uppercase()
    } else {
        format!("{directory}>{}", name.to_ascii_uppercase())
    })
}

fn normalized_ascii_listing(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn ascii_listing_for_host(text: &str) -> Vec<u8> {
    let mut output = Vec::with_capacity(text.len() + text.len() / 40);
    for character in normalized_ascii_listing(text).chars() {
        if character == '\n' {
            output.extend_from_slice(b"\r\n");
        } else if character.is_ascii() {
            output.push(character as u8);
        } else {
            output.push(b'?');
        }
    }
    output
}

fn host_file_to_atari_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let is_basic = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bas"));
    if is_basic {
        let listing =
            String::from_utf8_lossy(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(&bytes));
        tokenize_listing(&normalized_ascii_listing(&listing))
    } else {
        Ok(ascii_text_to_atascii(&bytes))
    }
}

fn atr_file_to_host_ascii(image_name: &str, bytes: &[u8]) -> Vec<u8> {
    let is_basic = Path::new(image_name)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bas"));
    if is_basic {
        if let Ok(listing) = detokenize_bytes(bytes) {
            return ascii_listing_for_host(&listing);
        }
    }
    atascii_text_to_ascii(bytes)
}

fn copy_file_between_atrs(
    source: &MountedAtr,
    source_image_name: &str,
    destination: &MountedAtr,
    destination_directory: &str,
) -> Result<String, String> {
    let bytes = extract_file_bytes(source, source_image_name)?;
    let name = source_image_name
        .split(['>', '/', '\\'])
        .rfind(|part| !part.is_empty())
        .ok_or_else(|| "source ATR filename is empty".to_string())?;
    let destination_path = atr_destination_path(destination, destination_directory, name)?;
    if source.path == destination.path && source_image_name.eq_ignore_ascii_case(&destination_path)
    {
        return Err("the file is already in that ATR directory".into());
    }
    add_file_bytes(destination, &destination_path, &bytes)?;
    Ok(destination_path)
}

pub(crate) fn current(state: &AtrState) -> Result<MountedAtr, String> {
    current_for(state, None)
}

pub(crate) fn current_for(state: &AtrState, drive: Option<u8>) -> Result<MountedAtr, String> {
    let state = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    let index = drive_index(drive, &state)?;
    state.drives[index]
        .clone()
        .ok_or_else(|| format!("{} has no ATR image mounted", drive_label(index)))
}

pub(crate) fn status(state: &AtrState) -> Result<AtrStatus, String> {
    let state = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?
        .clone();
    status_for_state(&state)
}

fn status_for_state(state: &FileManagerState) -> Result<AtrStatus, String> {
    let drives = state
        .drives
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, mounted)| match mounted {
            Some(mounted) => drive_status_for(index, state.active_drive, mounted),
            None => Ok(empty_drive_status(index, state.active_drive)),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let active = drives
        .get(state.active_drive.min(DRIVE_COUNT - 1))
        .or_else(|| drives.iter().find(|drive| drive.mounted));
    let local_tree = state
        .local_folder
        .as_ref()
        .map(|path| local_tree_for(path, 0).unwrap_or_default())
        .unwrap_or_default();
    Ok(AtrStatus {
        mounted: active.is_some_and(|drive| drive.mounted),
        active_drive: active.map(|drive| drive.drive.clone()),
        path: active.and_then(|drive| drive.path.clone()),
        filesystem: active.and_then(|drive| drive.filesystem.clone()),
        entries: active
            .map(|drive| drive.entries.clone())
            .unwrap_or_default(),
        info: active.map(|drive| drive.info.clone()).unwrap_or_default(),
        tree: active.map(|drive| drive.tree.clone()).unwrap_or_default(),
        drives,
        local_folder: state
            .local_folder
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        local_tree,
    })
}

fn empty_drive_status(index: usize, active_drive: usize) -> AtrDriveStatus {
    AtrDriveStatus {
        drive: drive_label(index),
        mounted: false,
        active: index == active_drive,
        path: None,
        filesystem: None,
        entries: vec![],
        info: vec![],
        tree: vec![],
    }
}

fn drive_status_for(
    index: usize,
    active_drive: usize,
    mounted: MountedAtr,
) -> Result<AtrDriveStatus, String> {
    let entries = execute(&options(Command::List, &mounted)).map_err(|e| e.to_string())?;
    let info = execute(&options(Command::Info, &mounted)).map_err(|e| e.to_string())?;
    let tree = tree_for(&mounted)?;
    Ok(AtrDriveStatus {
        drive: drive_label(index),
        mounted: true,
        active: index == active_drive,
        path: Some(mounted.path.to_string_lossy().into_owned()),
        filesystem: Some(mounted.fs.to_string()),
        entries,
        info,
        tree,
    })
}

fn local_tree_for(path: &Path, depth: usize) -> Result<Vec<LocalTreeEntry>, String> {
    let rows = fs::read_dir(path)
        .map_err(|e| format!("cannot read local folder {}: {e}", path.display()))?;
    let mut entries = Vec::new();
    for row in rows.take(LOCAL_TREE_LIMIT) {
        let row = row.map_err(|e| format!("cannot read local folder entry: {e}"))?;
        let metadata = row
            .metadata()
            .map_err(|e| format!("cannot read metadata for {}: {e}", row.path().display()))?;
        let is_directory = metadata.is_dir();
        let children = if is_directory && depth < LOCAL_TREE_DEPTH {
            local_tree_for(&row.path(), depth + 1).unwrap_or_default()
        } else {
            vec![]
        };
        entries.push(LocalTreeEntry {
            name: row.file_name().to_string_lossy().into_owned(),
            path: row.path().to_string_lossy().into_owned(),
            is_directory,
            size_bytes: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
            children,
        });
    }
    entries.sort_by(|left, right| {
        right
            .is_directory
            .cmp(&left.is_directory)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    Ok(entries)
}

#[tauri::command]
pub fn atr_create(
    request: AtrCreateRequest,
    state: tauri::State<'_, AtrState>,
    app: tauri::AppHandle,
) -> Result<AtrStatus, String> {
    let fs = parse_fs(&request.filesystem)?;
    let image_path = PathBuf::from(&request.path);
    let op = OperationOptions {
        command: Command::Create,
        image: image_path.clone(),
        second_image: None,
        media: Some(MediaType::Atr),
        fs,
        sectors: request.sectors,
        sector_size: request.sector_size,
        input: None,
        output: None,
        name: None,
        mask: "*.*".into(),
        list_format: None,
        volume_label: request.volume_label,
        force: request.force,
        copy_files: vec![],
    };
    execute(&op).map_err(|e| e.to_string())?;
    let mounted = MountedAtr {
        path: image_path,
        fs,
    };
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    let index = choose_mount_index(request.drive, &guard)?;
    guard.active_drive = index;
    guard.drives[index] = Some(mounted);
    persist_state(&app, &guard)?;
    status_for_state(&guard)
}

#[tauri::command]
pub fn atr_mount(
    path: String,
    filesystem: Option<String>,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
    app: tauri::AppHandle,
) -> Result<AtrStatus, String> {
    let path = PathBuf::from(path);
    let fs = match filesystem
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => parse_fs(value)?,
        None => detect_fs(&path)?,
    };
    let mounted = MountedAtr { path, fs };
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    let index = choose_mount_index(drive, &guard)?;
    guard.active_drive = index;
    guard.drives[index] = Some(mounted);
    persist_state(&app, &guard)?;
    status_for_state(&guard)
}

#[tauri::command]
pub fn atr_select_drive(
    drive: u8,
    state: tauri::State<'_, AtrState>,
    app: tauri::AppHandle,
) -> Result<AtrStatus, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    guard.active_drive = drive_index(Some(drive), &guard)?;
    persist_state(&app, &guard)?;
    status_for_state(&guard)
}

#[tauri::command]
pub fn atr_status(
    state: tauri::State<'_, AtrState>,
    app: tauri::AppHandle,
) -> Result<AtrStatus, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    if guard.local_folder.is_none() && guard.drives.iter().all(Option::is_none) {
        *guard = restore_state(&app)?;
    }
    status_for_state(&guard)
}

#[tauri::command]
pub fn atr_close(
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
    app: tauri::AppHandle,
) -> Result<AtrStatus, String> {
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    let index = drive_index(drive, &guard)?;
    guard.drives[index] = None;
    if guard.active_drive == index {
        guard.active_drive = guard
            .drives
            .iter()
            .position(Option::is_some)
            .unwrap_or(index)
            .min(DRIVE_COUNT - 1);
    }
    persist_state(&app, &guard)?;
    status_for_state(&guard)
}

#[tauri::command]
pub fn local_open_folder(
    path: String,
    state: tauri::State<'_, AtrState>,
    app: tauri::AppHandle,
) -> Result<AtrStatus, String> {
    let folder = PathBuf::from(path);
    if !folder.is_dir() {
        return Err(format!("{} is not a folder", folder.display()));
    }
    let mut guard = state
        .0
        .lock()
        .map_err(|_| "ATR state lock failed".to_string())?;
    guard.local_folder = Some(folder);
    persist_state(&app, &guard)?;
    status_for_state(&guard)
}

#[tauri::command]
pub fn atr_write_document(
    image_name: String,
    request: SaveDocumentRequest,
    drive: Option<u8>,
    overwrite: Option<bool>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let bytes = encode_document_bytes(&request)?;
    replace_file_bytes(&mounted, &image_name, &bytes, overwrite.unwrap_or(false))?;
    status(&state)
}

#[tauri::command]
pub fn atr_add_host_file(
    host_path: String,
    image_name: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let mut op = options(Command::Add, &mounted);
    op.input = Some(PathBuf::from(host_path));
    op.output = Some(PathBuf::from(image_name));
    execute(&op).map_err(|e| e.to_string())?;
    status(&state)
}

#[tauri::command]
pub fn atr_import_host_files(
    host_paths: Vec<String>,
    destination_directory: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    if host_paths.is_empty() {
        return Err("no host files were dropped".into());
    }
    let mounted = current_for(&state, drive)?;
    for host_path in host_paths {
        let path = PathBuf::from(&host_path);
        if !path.is_file() {
            return Err(format!(
                "only files can be imported into an ATR: {}",
                path.display()
            ));
        }
        let name = atari_file_name(&path)?;
        let image_name = atr_destination_path(&mounted, &destination_directory, &name)?;
        let bytes = host_file_to_atari_bytes(&path)?;
        add_file_bytes(&mounted, &image_name, &bytes)?;
    }
    status(&state)
}

#[tauri::command]
pub fn atr_copy_file(
    source_image_name: String,
    source_drive: Option<u8>,
    destination_directory: String,
    destination_drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let source = current_for(&state, source_drive)?;
    let destination = current_for(&state, destination_drive)?;
    copy_file_between_atrs(
        &source,
        &source_image_name,
        &destination,
        &destination_directory,
    )?;
    status(&state)
}

#[tauri::command]
pub fn atr_extract_file(
    image_name: String,
    host_path: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let mut op = options(Command::Extract, &mounted);
    op.input = Some(PathBuf::from(image_name));
    op.output = Some(PathBuf::from(host_path));
    execute(&op).map_err(|e| e.to_string())?;
    status(&state)
}

#[tauri::command]
pub fn atr_export_ascii(
    image_name: String,
    host_path: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let bytes = extract_file_bytes(&mounted, &image_name)?;
    let output = atr_file_to_host_ascii(&image_name, &bytes);
    let destination = PathBuf::from(&host_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create destination folder: {error}"))?;
    }
    fs::write(&destination, output)
        .map_err(|error| format!("cannot export {}: {error}", destination.display()))?;
    status(&state)
}

#[tauri::command]
pub fn atr_delete_file(
    image_name: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let mut op = options(Command::Delete, &mounted);
    op.name = Some(image_name);
    execute(&op).map_err(|e| e.to_string())?;
    status(&state)
}

#[tauri::command]
pub fn atr_delete_entry(
    image_name: String,
    is_directory: bool,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let mut op = if is_directory {
        if mounted.fs != FsType::Sparta2 {
            return Err("directories require a SpartaDOS ATR".into());
        }
        options(Command::Rmdir(image_name), &mounted)
    } else {
        let mut op = options(Command::Delete, &mounted);
        op.name = Some(image_name);
        op
    };
    op.force = is_directory;
    execute(&op).map_err(|e| e.to_string())?;
    status(&state)
}

#[tauri::command]
pub fn atr_rename_entry(
    image_name: String,
    new_name: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    let mut image = open_image(&mounted.path, Some(MediaType::Atr)).map_err(|e| e.to_string())?;
    match mounted.fs {
        FsType::Dos2 => Dos2::new(image.as_mut())
            .rename(&image_name, &new_name)
            .map_err(|e| e.to_string())?,
        FsType::Sparta2 => Sparta::new(image.as_mut())
            .map_err(|e| e.to_string())?
            .rename(&image_name, &new_name)
            .map_err(|e| e.to_string())?,
    }
    status(&state)
}

#[tauri::command]
pub fn atr_open_document(
    image_name: String,
    mode: DocumentMode,
    width: usize,
    height: usize,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<LoadedDocument, String> {
    let mounted = current_for(&state, drive)?;
    let bytes = extract_file_bytes(&mounted, &image_name)?;
    decode_document_bytes(bytes, None, mode, width, height)
}

#[tauri::command]
pub fn atr_mkdir(
    path: String,
    drive: Option<u8>,
    state: tauri::State<'_, AtrState>,
) -> Result<AtrStatus, String> {
    let mounted = current_for(&state, drive)?;
    if mounted.fs != FsType::Sparta2 {
        return Err("directories require a SpartaDOS ATR".into());
    }
    execute(&options(Command::Mkdir(path), &mounted)).map_err(|e| e.to_string())?;
    status(&state)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_atr(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "quartermaster-{name}-{}-{suffix}.atr",
            std::process::id()
        ))
    }

    fn create_atr(path: &Path, fs: FsType) {
        create_atr_with_geometry(path, fs, 720, 128);
    }

    fn create_atr_with_geometry(path: &Path, fs: FsType, sectors: u32, sector_size: usize) {
        execute(&OperationOptions {
            command: Command::Create,
            image: path.to_path_buf(),
            second_image: None,
            media: Some(MediaType::Atr),
            fs,
            sectors,
            sector_size,
            input: None,
            output: None,
            name: None,
            mask: "*.*".into(),
            list_format: None,
            volume_label: Some("QMASTER".into()),
            force: true,
            copy_files: vec![],
        })
        .unwrap();
    }

    #[test]
    fn detects_dos2_atr() {
        let path = temp_atr("dos2");
        create_atr(&path, FsType::Dos2);
        assert_eq!(detect_fs(&path).unwrap(), FsType::Dos2);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn detects_sparta_atr() {
        let path = temp_atr("sparta");
        create_atr(&path, FsType::Sparta2);
        assert_eq!(detect_fs(&path).unwrap(), FsType::Sparta2);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn creates_360k_and_16m_sparta_atrs() {
        for (name, sectors) in [("sparta-360k", 1_440), ("sparta-16m", 65_535)] {
            let path = temp_atr(name);
            create_atr_with_geometry(&path, FsType::Sparta2, sectors, 256);

            let mut image = open_image(&path, Some(MediaType::Atr)).unwrap();
            assert_eq!(image.total_sectors(), sectors);
            assert_eq!(image.sector_size(), 256);
            assert!(Sparta::new(image.as_mut()).is_ok());
            drop(image);

            let expected_data_bytes = 3 * 128 + (sectors as u64 - 3) * 256;
            assert_eq!(
                std::fs::metadata(&path).unwrap().len(),
                16 + expected_data_bytes
            );
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn sparta_tree_includes_extensionless_subdirectories() {
        let path = temp_atr("sparta-tree");
        create_atr(&path, FsType::Sparta2);
        let mounted = MountedAtr {
            path: path.clone(),
            fs: FsType::Sparta2,
        };
        execute(&options(Command::Mkdir("GAMES".into()), &mounted)).unwrap();
        execute(&options(Command::Mkdir("GAMES>LEVELS".into()), &mounted)).unwrap();

        let tree = tree_for(&mounted).unwrap();
        let games = tree
            .iter()
            .find(|entry| entry.name == "GAMES")
            .expect("GAMES directory should be visible");
        assert!(games.is_directory);
        assert!(
            games
                .children
                .iter()
                .any(|entry| entry.name == "LEVELS" && entry.is_directory),
            "nested extensionless directory should be visible"
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn renames_dos2_file() {
        let path = temp_atr("dos2-rename");
        let input = path.with_extension("txt");
        create_atr(&path, FsType::Dos2);
        std::fs::write(&input, b"HELLO").unwrap();
        let mounted = MountedAtr {
            path: path.clone(),
            fs: FsType::Dos2,
        };
        let mut add = options(Command::Add, &mounted);
        add.input = Some(input.clone());
        add.output = Some(PathBuf::from("BEFORE.TXT"));
        execute(&add).unwrap();

        let mut image = open_image(&path, Some(MediaType::Atr)).unwrap();
        Dos2::new(image.as_mut())
            .rename("BEFORE.TXT", "AFTER.TXT")
            .unwrap();
        drop(image);
        let tree = tree_for(&mounted).unwrap();
        assert!(tree.iter().any(|entry| entry.name == "AFTER.TXT"));
        assert!(!tree.iter().any(|entry| entry.name == "BEFORE.TXT"));

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn renames_sparta_directory_with_children() {
        let path = temp_atr("sparta-rename");
        create_atr(&path, FsType::Sparta2);
        let mounted = MountedAtr {
            path: path.clone(),
            fs: FsType::Sparta2,
        };
        execute(&options(Command::Mkdir("GAMES".into()), &mounted)).unwrap();
        execute(&options(Command::Mkdir("GAMES>LEVELS".into()), &mounted)).unwrap();

        let mut image = open_image(&path, Some(MediaType::Atr)).unwrap();
        Sparta::new(image.as_mut())
            .unwrap()
            .rename("GAMES", "DEMOS")
            .unwrap();
        drop(image);
        let tree = tree_for(&mounted).unwrap();
        let demos = tree
            .iter()
            .find(|entry| entry.name == "DEMOS")
            .expect("renamed directory should be visible");
        assert!(demos.is_directory);
        assert!(demos.children.iter().any(|entry| entry.name == "LEVELS"));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn document_save_can_replace_existing_dos2_and_sparta_files() {
        for (name, fs, image_name) in [
            ("dos2-replace", FsType::Dos2, "NOTES.ATA"),
            ("sparta-replace", FsType::Sparta2, "DOCS>NOTES.ATA"),
        ] {
            let path = temp_atr(name);
            create_atr(&path, fs);
            let mounted = MountedAtr {
                path: path.clone(),
                fs,
            };
            if fs == FsType::Sparta2 {
                execute(&options(Command::Mkdir("DOCS".into()), &mounted)).unwrap();
            }
            add_file_bytes(&mounted, image_name, b"OLD\x9b").unwrap();

            assert!(replace_file_bytes(&mounted, image_name, b"NEW\x9b", false).is_err());
            assert_eq!(
                extract_file_bytes(&mounted, image_name).unwrap(),
                b"OLD\x9b"
            );

            replace_file_bytes(&mounted, image_name, b"NEW\x9b", true).unwrap();
            assert_eq!(
                extract_file_bytes(&mounted, image_name).unwrap(),
                b"NEW\x9b"
            );

            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn opens_nested_sparta_document_without_host_temp_name() {
        let path = temp_atr("sparta-open-nested");
        let input = path.with_extension("txt");
        create_atr(&path, FsType::Sparta2);
        let mounted = MountedAtr {
            path: path.clone(),
            fs: FsType::Sparta2,
        };
        execute(&options(Command::Mkdir("DOCS".into()), &mounted)).unwrap();
        std::fs::write(&input, b"HELLO\x9bWORLD").unwrap();
        let mut add = options(Command::Add, &mounted);
        add.input = Some(input.clone());
        add.output = Some(PathBuf::from("DOCS>README.TXT"));
        execute(&add).unwrap();

        let bytes = extract_file_bytes(&mounted, "DOCS>README.TXT").unwrap();
        assert_eq!(bytes, b"HELLO\x9bWORLD");
        let document = decode_document_bytes(bytes, None, DocumentMode::Atascii, 40, 200).unwrap();
        assert_eq!(document.cells[0].byte, b'H');
        assert_eq!(document.cells[40].byte, b'W');

        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn drag_conversion_imports_text_and_tokenized_basic() {
        let text_path = temp_atr("dragged-long-filename").with_extension("txt");
        let basic_path = temp_atr("program").with_extension("bas");
        std::fs::write(&text_path, b"HELLO\r\nWORLD").unwrap();
        std::fs::write(&basic_path, b"10 PRINT \"HELLO\"\r\n20 END\r\n").unwrap();

        assert_eq!(
            host_file_to_atari_bytes(&text_path).unwrap(),
            b"HELLO\x9bWORLD"
        );
        let tokenized = host_file_to_atari_bytes(&basic_path).unwrap();
        let listing = detokenize_bytes(&tokenized).unwrap();
        assert!(listing.contains("10 PRINT \"HELLO\""));
        assert!(listing.contains("20 END"));
        assert!(atari_file_name(&text_path).unwrap().ends_with(".TXT"));

        let _ = std::fs::remove_file(text_path);
        let _ = std::fs::remove_file(basic_path);
    }

    #[test]
    fn ascii_export_detokenizes_basic_and_translates_atascii() {
        assert_eq!(
            atr_file_to_host_ascii("README.TXT", b"HELLO\x9bWORLD"),
            b"HELLO\r\nWORLD"
        );
        let tokenized = tokenize_listing("10 PRINT \"HELLO\"\n20 END\n").unwrap();
        let listing = atr_file_to_host_ascii("PROGRAM.BAS", &tokenized);
        let listing = String::from_utf8(listing).unwrap();
        assert!(listing.contains("10 PRINT \"HELLO\"\r\n"));
        assert!(listing.contains("20 END\r\n"));
    }

    #[test]
    fn atr_to_atr_drag_preserves_original_bytes() {
        let source_path = temp_atr("drag-source");
        let destination_path = temp_atr("drag-destination");
        create_atr(&source_path, FsType::Dos2);
        create_atr(&destination_path, FsType::Sparta2);
        let source = MountedAtr {
            path: source_path.clone(),
            fs: FsType::Dos2,
        };
        let destination = MountedAtr {
            path: destination_path.clone(),
            fs: FsType::Sparta2,
        };
        let original = vec![0x00, 0xff, 0x9b, b'A', 0x10, 0x80];
        add_file_bytes(&source, "BINARY.DAT", &original).unwrap();

        let copied = copy_file_between_atrs(&source, "BINARY.DAT", &destination, "").unwrap();
        assert_eq!(copied, "BINARY.DAT");
        assert_eq!(
            extract_file_bytes(&destination, "BINARY.DAT").unwrap(),
            original
        );

        let _ = std::fs::remove_file(source_path);
        let _ = std::fs::remove_file(destination_path);
    }
}
