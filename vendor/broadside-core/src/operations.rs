use std::fs;
use std::path::{Path, PathBuf};

use crate::dos2::{dump_hex, Dos2};
use crate::error::{BroadsideError, Result};
use crate::image::{convert_image, create_image, open_image, DiskImage};
use crate::sparta::Sparta;
use crate::types::{FsType, ImageType, MediaType};

#[derive(Debug, Clone)]
pub enum Command {
    Create,
    Format,
    List,
    Info,
    Dump(u32),
    Extract,
    Add,
    Delete,
    Convert(MediaType),
    SetImageType(ImageType),
    SetName(String),
    Mkdir(String),
    Rmdir(String),
}

#[derive(Debug, Clone)]
pub struct CopySpec {
    pub source: PathBuf,
    pub destination: String,
}

#[derive(Debug, Clone)]
pub struct OperationOptions {
    pub command: Command,
    pub image: PathBuf,
    pub second_image: Option<PathBuf>,
    pub media: Option<MediaType>,
    pub fs: FsType,
    pub sectors: u32,
    pub sector_size: usize,
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,
    pub name: Option<String>,
    pub mask: String,
    pub list_format: Option<String>,
    /// Optional volume label used while creating a SpartaDOS filesystem.
    pub volume_label: Option<String>,
    pub force: bool,
    /// Host files copied byte-for-byte into a newly created/formatted image.
    pub copy_files: Vec<CopySpec>,
}

/// Execute a complete disk-image operation.
///
/// The returned strings are presentation-neutral status or listing lines. GUI,
/// TUI, CLI, and service applications can display or transform them as needed.
pub fn execute(options: &OperationOptions) -> Result<Vec<String>> {
    match options.command.clone() {
        Command::Create => create(options),
        Command::Convert(media) => convert(options, media),
        _ => with_open_image(options),
    }
}

fn create(o: &OperationOptions) -> Result<Vec<String>> {
    if o.image.exists() && !o.force {
        return Err(BroadsideError::AlreadyExists(format!(
            "{} (use force to overwrite)",
            o.image.display()
        )));
    }
    let media = o.media.unwrap_or(MediaType::Atr);
    let mut image = create_image(&o.image, media, o.sectors, o.sector_size)?;
    match o.fs {
        FsType::Dos2 => Dos2::new(image.as_mut()).format()?,
        FsType::Sparta2 => Sparta::format(image.as_mut(), o.volume_label.as_deref())?,
    }
    let mut out = vec![format!(
        "Created {}: {} sectors x {} bytes, {}, {}",
        o.image.display(),
        o.sectors,
        o.sector_size,
        media,
        o.fs
    )];
    for spec in &o.copy_files {
        let data = fs::read(&spec.source)?;
        match o.fs {
            FsType::Dos2 => Dos2::new(image.as_mut()).insert(&spec.destination, &data)?,
            FsType::Sparta2 => Sparta::new(image.as_mut())?.insert(&spec.destination, &data)?,
        }
        out.push(format!(
            "Copied {} as {} ({} bytes, byte-exact)",
            spec.source.display(),
            spec.destination,
            data.len()
        ));
    }
    image.flush()?;
    Ok(out)
}

fn convert(o: &OperationOptions, media: MediaType) -> Result<Vec<String>> {
    let dest = o.second_image.as_ref().ok_or_else(|| {
        BroadsideError::InvalidArgument("conversion requires a destination image".into())
    })?;
    if dest.exists() && !o.force {
        return Err(BroadsideError::AlreadyExists(format!(
            "{} (use force to overwrite)",
            dest.display()
        )));
    }
    let image = open_image(&o.image, o.media)?;
    convert_image(image, dest, media)?;
    Ok(vec![format!(
        "Converted {} -> {} ({media})",
        o.image.display(),
        dest.display()
    )])
}

fn with_open_image(o: &OperationOptions) -> Result<Vec<String>> {
    let mut image = open_image(&o.image, o.media)?;
    match o.command.clone() {
        Command::Info => info(image.as_mut(), o),
        Command::Dump(sector) => {
            let data = image.read_sector(sector)?;
            Ok(dump_hex(&data).lines().map(str::to_owned).collect())
        }
        Command::SetImageType(image_type) => {
            image.set_image_type(image_type)?;
            image.flush()?;
            Ok(vec!["Image type updated".into()])
        }
        Command::Format => match o.fs {
            FsType::Dos2 => {
                Dos2::new(image.as_mut()).format()?;
                Ok(vec![format!(
                    "Formatted {} as Atari DOS 2",
                    o.image.display()
                )])
            }
            FsType::Sparta2 => {
                Sparta::format(image.as_mut(), o.volume_label.as_deref())?;
                Ok(vec![format!(
                    "Formatted {} as SpartaDOS 2.x / SDX",
                    o.image.display()
                )])
            }
        },
        Command::List => list(image.as_mut(), o),
        Command::Extract => extract(image.as_mut(), o),
        Command::Add => add(image.as_mut(), o),
        Command::Delete => delete(image.as_mut(), o),
        Command::SetName(name) => {
            require_sparta(o.fs, "volume labels")?;
            let mut sparta = Sparta::new(image.as_mut())?;
            sparta.set_label(&name)?;
            Ok(vec![format!("Volume label set to {name}")])
        }
        Command::Mkdir(path) => {
            require_sparta(o.fs, "directories")?;
            let mut sparta = Sparta::new(image.as_mut())?;
            sparta.mkdir(&path)?;
            Ok(vec![format!("Created directory {path}")])
        }
        Command::Rmdir(path) => {
            require_sparta(o.fs, "directories")?;
            let mut sparta = Sparta::new(image.as_mut())?;
            sparta.delete(&path, true)?;
            Ok(vec![format!("Removed directory {path}")])
        }
        Command::Create | Command::Convert(_) => unreachable!(),
    }
}

fn list(image: &mut dyn DiskImage, o: &OperationOptions) -> Result<Vec<String>> {
    match o.fs {
        FsType::Dos2 => {
            let mut fs = Dos2::new(image);
            let mut out = Vec::new();
            for entry in fs.list(&o.mask)? {
                out.push(format!(
                    "{:<12} {:>8} bytes {:>4} sectors {}",
                    entry.name,
                    entry.size_bytes,
                    entry.size_sectors,
                    if entry.locked() { "LOCKED" } else { "" }
                ));
            }
            Ok(out)
        }
        FsType::Sparta2 => {
            let (path, mask) = split_list_target(&o.mask);
            let sector_size = image.sector_size();
            let mut fs = Sparta::new(image)?;
            let rows = fs.list(&path, &mask)?;
            let mut out = Vec::new();
            for entry in &rows {
                out.push(format!(
                    "{} {:<12} {:>4} {:>8} {:02}-{:02}-{:04} {:02}:{:02}:{:02}",
                    if entry.is_dir() { "--D" } else { "---" },
                    entry.name,
                    entry.sectors(sector_size),
                    entry.size_bytes,
                    entry.day,
                    entry.month,
                    2000 + entry.year as u16,
                    entry.hour,
                    entry.minute,
                    entry.second
                ));
            }
            out.push(format!("Total: {} files.", rows.len()));
            Ok(out)
        }
    }
}

fn extract(image: &mut dyn DiskImage, o: &OperationOptions) -> Result<Vec<String>> {
    let src = o
        .input
        .as_ref()
        .ok_or_else(|| BroadsideError::InvalidArgument("extract requires an image path".into()))?;
    let dst = o.output.as_ref().ok_or_else(|| {
        BroadsideError::InvalidArgument("extract requires a host destination".into())
    })?;
    let data = match o.fs {
        FsType::Dos2 => Dos2::new(image).extract(&src.to_string_lossy())?,
        FsType::Sparta2 => Sparta::new(image)?.extract(&src.to_string_lossy())?,
    };
    fs::write(dst, data)?;
    Ok(vec![format!(
        "Extracted {} -> {}",
        src.display(),
        dst.display()
    )])
}

fn add(image: &mut dyn DiskImage, o: &OperationOptions) -> Result<Vec<String>> {
    let src = o
        .input
        .as_ref()
        .ok_or_else(|| BroadsideError::InvalidArgument("add requires a host source".into()))?;
    let dst = o.output.as_ref().ok_or_else(|| {
        BroadsideError::InvalidArgument("add requires an image destination".into())
    })?;
    let data = fs::read(src)?;
    match o.fs {
        FsType::Dos2 => Dos2::new(image).insert(&dst.to_string_lossy(), &data)?,
        FsType::Sparta2 => Sparta::new(image)?.insert(&dst.to_string_lossy(), &data)?,
    }
    Ok(vec![format!(
        "Added {} as {}",
        src.display(),
        dst.display()
    )])
}

fn delete(image: &mut dyn DiskImage, o: &OperationOptions) -> Result<Vec<String>> {
    let name = o
        .name
        .as_deref()
        .ok_or_else(|| BroadsideError::InvalidArgument("delete requires an image path".into()))?;
    match o.fs {
        FsType::Dos2 => Dos2::new(image).delete(name, o.force)?,
        FsType::Sparta2 => Sparta::new(image)?.delete(name, false)?,
    }
    Ok(vec![format!("Deleted {name}")])
}

fn info(image: &mut dyn DiskImage, o: &OperationOptions) -> Result<Vec<String>> {
    let mut out = vec![
        format!("Image: {}", Path::new(image.path()).display()),
        format!("Media: {}", image.media_type()),
        format!("Sectors: {}", image.total_sectors()),
        format!("Sector size: {}", image.sector_size()),
        format!("Image type: {:?}", image.image_type()),
        format!("Filesystem selection: {}", o.fs),
    ];
    match o.fs {
        FsType::Dos2 => {
            let mut dos = Dos2::new(image);
            dos.check()?;
            out.push(format!("Free sectors: {}", dos.free_sectors()?));
        }
        FsType::Sparta2 => {
            let mut sparta = Sparta::new(image)?;
            out.push(format!("Volume label: {}", sparta.label()?));
            out.push(format!("Free sectors: {}", sparta.free_sectors()?));
        }
    }
    Ok(out)
}

fn split_list_target(target: &str) -> (String, String) {
    let normalized = target.replace('\\', ">").replace('/', ">");
    if let Some(index) = normalized.rfind('>') {
        (
            normalized[..index].to_string(),
            normalized[index + 1..].to_string(),
        )
    } else {
        (String::new(), normalized)
    }
}

fn require_sparta(fs: FsType, feature: &str) -> Result<()> {
    if fs == FsType::Sparta2 {
        Ok(())
    } else {
        Err(BroadsideError::Unsupported(format!(
            "{feature} require SpartaDOS"
        )))
    }
}
