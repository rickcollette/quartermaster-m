use crate::error::{BroadsideError, Result};
use crate::types::{
    ImageType, MediaType, ATR_HEADER_SIZE, MAX_SECTORS, SECTOR_DOUBLE, SECTOR_SINGLE,
};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub trait DiskImage {
    fn media_type(&self) -> MediaType;
    fn path(&self) -> &Path;
    fn sector_size(&self) -> usize;
    fn total_sectors(&self) -> u32;
    fn image_type(&self) -> ImageType;
    fn read_sector(&mut self, sector: u32) -> Result<Vec<u8>>;
    fn write_sector(&mut self, sector: u32, data: &[u8]) -> Result<()>;
    fn set_image_type(&mut self, image_type: ImageType) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
}

fn validate_geometry(sectors: u32, sector_size: usize) -> Result<()> {
    if sectors == 0 || sectors > MAX_SECTORS {
        return Err(BroadsideError::InvalidArgument(format!(
            "sector count must be 1..={MAX_SECTORS}"
        )));
    }
    if sector_size != SECTOR_SINGLE && sector_size != SECTOR_DOUBLE {
        return Err(BroadsideError::InvalidArgument(
            "sector size must be 128 or 256".into(),
        ));
    }
    Ok(())
}

pub struct AtrImage {
    path: PathBuf,
    file: File,
    total_sectors: u32,
    sector_size: usize,
    image_type: ImageType,
}

impl AtrImage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new().read(true).write(true).open(&path)?;
        let size = file.metadata()?.len();
        let mut h = [0u8; ATR_HEADER_SIZE];
        file.read_exact(&mut h)?;
        if h[0] != 0x96 || h[1] != 0x02 {
            return Err(BroadsideError::InvalidImage(
                "missing ATR 0x0296 signature".into(),
            ));
        }
        let sector_size = u16::from_le_bytes([h[4], h[5]]) as usize;
        if sector_size != SECTOR_SINGLE && sector_size != SECTOR_DOUBLE {
            return Err(BroadsideError::InvalidImage(format!(
                "unsupported ATR sector size {sector_size}"
            )));
        }
        let paragraphs = (h[2] as u32) | ((h[3] as u32) << 8) | ((h[6] as u32) << 16);
        let declared_bytes = paragraphs as u64 * 16;
        let standard_bytes = size.saturating_sub(ATR_HEADER_SIZE as u64);
        let image_type = if declared_bytes == standard_bytes {
            ImageType::Standard
        } else {
            ImageType::Sio2Ide
        };
        let data_bytes = standard_bytes;
        let total_sectors = if sector_size == SECTOR_DOUBLE {
            if data_bytes < (3 * SECTOR_SINGLE) as u64 {
                return Err(BroadsideError::InvalidImage("ATR is too short".into()));
            }
            3 + ((data_bytes - (3 * SECTOR_SINGLE) as u64) / SECTOR_DOUBLE as u64) as u32
        } else {
            (data_bytes / SECTOR_SINGLE as u64) as u32
        };
        validate_geometry(total_sectors, sector_size)?;
        Ok(Self {
            path,
            file,
            total_sectors,
            sector_size,
            image_type,
        })
    }

    pub fn create(path: impl AsRef<Path>, sectors: u32, sector_size: usize) -> Result<Self> {
        validate_geometry(sectors, sector_size)?;
        let path = path.as_ref().to_path_buf();
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)?;
        let data_bytes = 3 * SECTOR_SINGLE + (sectors.saturating_sub(3) as usize * sector_size);
        let paragraphs = (data_bytes / 16) as u32;
        let mut h = [0u8; ATR_HEADER_SIZE];
        h[0] = 0x96;
        h[1] = 0x02;
        h[2] = paragraphs as u8;
        h[3] = (paragraphs >> 8) as u8;
        h[6] = (paragraphs >> 16) as u8;
        h[4..6].copy_from_slice(&(sector_size as u16).to_le_bytes());
        file.write_all(&h)?;
        file.set_len((ATR_HEADER_SIZE + data_bytes) as u64)?;
        file.flush()?;
        Ok(Self {
            path,
            file,
            total_sectors: sectors,
            sector_size,
            image_type: ImageType::Standard,
        })
    }

    fn offset_and_len(&self, sector: u32) -> Result<(u64, usize)> {
        if sector == 0 || sector > self.total_sectors {
            return Err(BroadsideError::InvalidArgument(format!(
                "sector {sector} outside image"
            )));
        }
        if self.sector_size == SECTOR_DOUBLE && sector > 3 {
            Ok((
                (ATR_HEADER_SIZE + 3 * SECTOR_SINGLE + (sector as usize - 4) * SECTOR_DOUBLE)
                    as u64,
                SECTOR_DOUBLE,
            ))
        } else {
            Ok((
                (ATR_HEADER_SIZE + (sector as usize - 1) * SECTOR_SINGLE) as u64,
                SECTOR_SINGLE,
            ))
        }
    }

    fn write_header(&mut self) -> Result<()> {
        let data_bytes =
            3 * SECTOR_SINGLE + (self.total_sectors.saturating_sub(3) as usize * self.sector_size);
        let mut h = [0u8; ATR_HEADER_SIZE];
        h[0] = 0x96;
        h[1] = 0x02;
        h[4..6].copy_from_slice(&(self.sector_size as u16).to_le_bytes());
        match self.image_type {
            ImageType::Standard => {
                let p = (data_bytes / 16) as u32;
                h[2] = p as u8;
                h[3] = (p >> 8) as u8;
                h[6] = (p >> 16) as u8;
            }
            ImageType::Sio2Ide => {
                let n = if self.sector_size == SECTOR_SINGLE {
                    self.total_sectors >> 1
                } else {
                    self.total_sectors
                };
                h[2] = ((n & 0x0f) << 4) as u8;
                h[3] = ((n & 0x0ff0) >> 4) as u8;
            }
        }
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&h)?;
        Ok(())
    }
}

impl DiskImage for AtrImage {
    fn media_type(&self) -> MediaType {
        MediaType::Atr
    }
    fn path(&self) -> &Path {
        &self.path
    }
    fn sector_size(&self) -> usize {
        self.sector_size
    }
    fn total_sectors(&self) -> u32 {
        self.total_sectors
    }
    fn image_type(&self) -> ImageType {
        self.image_type
    }
    fn read_sector(&mut self, sector: u32) -> Result<Vec<u8>> {
        let (offset, len) = self.offset_and_len(sector)?;
        let mut data = vec![0u8; len];
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut data)?;
        Ok(data)
    }
    fn write_sector(&mut self, sector: u32, data: &[u8]) -> Result<()> {
        let (offset, len) = self.offset_and_len(sector)?;
        if data.len() < len {
            return Err(BroadsideError::InvalidArgument(format!(
                "sector write needs {len} bytes"
            )));
        }
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data[..len])?;
        Ok(())
    }
    fn set_image_type(&mut self, image_type: ImageType) -> Result<()> {
        self.image_type = image_type;
        self.write_header()
    }
    fn flush(&mut self) -> Result<()> {
        self.file.flush().map_err(Into::into)
    }
}

pub struct RawImage {
    path: PathBuf,
    file: File,
    total_sectors: u32,
    sector_size: usize,
}
impl RawImage {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().read(true).write(true).open(&path)?;
        let size = file.metadata()?.len() as usize;
        let (total_sectors, sector_size) = match size {
            92_160 => (720, 128),
            133_120 => (1040, 128),
            183_936 | 183_926 => (720, 256),
            _ if size >= 384 && (size - 384) % 256 == 0 => (3 + ((size - 384) / 256) as u32, 256),
            _ if size % 128 == 0 => ((size / 128) as u32, 128),
            _ => {
                return Err(BroadsideError::InvalidImage(format!(
                    "cannot infer raw geometry from {size} bytes"
                )))
            }
        };
        validate_geometry(total_sectors, sector_size)?;
        Ok(Self {
            path,
            file,
            total_sectors,
            sector_size,
        })
    }
    pub fn create(path: impl AsRef<Path>, sectors: u32, sector_size: usize) -> Result<Self> {
        validate_geometry(sectors, sector_size)?;
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)?;
        let len = 3 * SECTOR_SINGLE + sectors.saturating_sub(3) as usize * sector_size;
        file.set_len(len as u64)?;
        Ok(Self {
            path,
            file,
            total_sectors: sectors,
            sector_size,
        })
    }
    fn offset_and_len(&self, sector: u32) -> Result<(u64, usize)> {
        if sector == 0 || sector > self.total_sectors {
            return Err(BroadsideError::InvalidArgument(format!(
                "sector {sector} outside image"
            )));
        }
        if self.sector_size == 256 && sector > 3 {
            Ok(((3 * 128 + (sector as usize - 4) * 256) as u64, 256))
        } else {
            Ok((((sector - 1) as usize * 128) as u64, 128))
        }
    }
}
impl DiskImage for RawImage {
    fn media_type(&self) -> MediaType {
        MediaType::Raw
    }
    fn path(&self) -> &Path {
        &self.path
    }
    fn sector_size(&self) -> usize {
        self.sector_size
    }
    fn total_sectors(&self) -> u32 {
        self.total_sectors
    }
    fn image_type(&self) -> ImageType {
        ImageType::Standard
    }
    fn read_sector(&mut self, sector: u32) -> Result<Vec<u8>> {
        let (o, l) = self.offset_and_len(sector)?;
        let mut b = vec![0; l];
        self.file.seek(SeekFrom::Start(o))?;
        self.file.read_exact(&mut b)?;
        Ok(b)
    }
    fn write_sector(&mut self, sector: u32, data: &[u8]) -> Result<()> {
        let (o, l) = self.offset_and_len(sector)?;
        if data.len() < l {
            return Err(BroadsideError::InvalidArgument(format!(
                "sector write needs {l} bytes"
            )));
        }
        self.file.seek(SeekFrom::Start(o))?;
        self.file.write_all(&data[..l])?;
        Ok(())
    }
    fn set_image_type(&mut self, image_type: ImageType) -> Result<()> {
        if image_type == ImageType::Standard {
            Ok(())
        } else {
            Err(BroadsideError::Unsupported(
                "SIO2IDE header encoding only applies to ATR".into(),
            ))
        }
    }
    fn flush(&mut self) -> Result<()> {
        self.file.flush().map_err(Into::into)
    }
}

pub fn open_image(path: impl AsRef<Path>, media: Option<MediaType>) -> Result<Box<dyn DiskImage>> {
    let path = path.as_ref();
    let media = match media {
        Some(m) => m,
        None => {
            let mut f = File::open(path)?;
            let mut sig = [0u8; 2];
            f.read_exact(&mut sig)?;
            if sig == [0x96, 0x02] {
                MediaType::Atr
            } else {
                MediaType::Raw
            }
        }
    };
    match media {
        MediaType::Atr => Ok(Box::new(AtrImage::open(path)?)),
        MediaType::Raw => Ok(Box::new(RawImage::open(path)?)),
    }
}

pub fn create_image(
    path: impl AsRef<Path>,
    media: MediaType,
    sectors: u32,
    sector_size: usize,
) -> Result<Box<dyn DiskImage>> {
    match media {
        MediaType::Atr => Ok(Box::new(AtrImage::create(path, sectors, sector_size)?)),
        MediaType::Raw => Ok(Box::new(RawImage::create(path, sectors, sector_size)?)),
    }
}

pub fn convert_image(
    mut source: Box<dyn DiskImage>,
    destination: impl AsRef<Path>,
    media: MediaType,
) -> Result<()> {
    let mut target = create_image(
        destination,
        media,
        source.total_sectors(),
        source.sector_size(),
    )?;
    for sector in 1..=source.total_sectors() {
        let data = source.read_sector(sector)?;
        target.write_sector(sector, &data)?;
    }
    target.flush()
}
