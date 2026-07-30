use crate::error::{BroadsideError, Result};
use crate::image::DiskImage;

const ENTRY: usize = 23;
const IN_USE: u8 = 0x08;
const DELETED: u8 = 0x10;
const DIRECTORY: u8 = 0x20;
const DEFAULT_LABEL: &str = "UNKNOWN";

#[derive(Debug, Clone)]
pub struct SpartaEntry {
    pub name: String,
    pub attributes: u8,
    pub map_sector: u16,
    pub size_bytes: u32,
    pub day: u8,
    pub month: u8,
    pub year: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}
impl SpartaEntry {
    pub fn is_dir(&self) -> bool {
        self.attributes & DIRECTORY != 0
    }
    pub fn sectors(&self, ss: usize) -> u32 {
        (self.size_bytes + ss as u32 - 1) / ss as u32
    }
}

pub struct Sparta<'a> {
    image: &'a mut dyn DiskImage,
    root_map: u16,
    bitmap_start: u16,
    bitmap_count: u8,
}
impl<'a> Sparta<'a> {
    /// Create a fresh SpartaDOS 2.x filesystem suitable for SpartaDOS X.
    ///
    /// This is the host-side equivalent of SDX `BUILDDIR`: it writes a new
    /// boot/VIB sector, free-sector bitmap, root directory sector map, and an
    /// empty root directory. The operation is destructive.
    pub fn format(image: &mut dyn DiskImage, label: Option<&str>) -> Result<()> {
        let total = image.total_sectors();
        let sector_size = image.sector_size();
        if total < 16 || total > u16::MAX as u32 {
            return Err(BroadsideError::InvalidArgument(
                "SpartaDOS requires between 16 and 65535 logical sectors".into(),
            ));
        }
        if sector_size != 128 && sector_size != 256 {
            return Err(BroadsideError::InvalidArgument(
                "SpartaDOS requires 128-byte or 256-byte logical sectors".into(),
            ));
        }

        // Bitmap bit N describes logical sector N. Bit zero is reserved.
        let bits_per_bitmap_sector = sector_size * 8;
        let bitmap_count =
            ((total as usize + 1) + bits_per_bitmap_sector - 1) / bits_per_bitmap_sector;
        if bitmap_count == 0 || bitmap_count > u8::MAX as usize {
            return Err(BroadsideError::InvalidArgument(
                "filesystem needs too many bitmap sectors".into(),
            ));
        }
        let bitmap_start = 4u16;
        let root_map = bitmap_start + bitmap_count as u16;
        let root_data = root_map + 1;
        if root_data as u32 > total {
            return Err(BroadsideError::InvalidArgument(
                "image is too small for SpartaDOS metadata".into(),
            ));
        }

        // Start with every representable sector free, then reserve metadata.
        let mut bitmaps = vec![vec![0u8; sector_size]; bitmap_count];
        for sector in 1..=total as usize {
            let page = sector / bits_per_bitmap_sector;
            let within = sector % bits_per_bitmap_sector;
            bitmaps[page][within / 8] |= 0x80 >> (within % 8);
        }
        for sector in 0..=root_data as usize {
            let page = sector / bits_per_bitmap_sector;
            let within = sector % bits_per_bitmap_sector;
            bitmaps[page][within / 8] &= !(0x80 >> (within % 8));
        }
        for (index, bitmap) in bitmaps.iter().enumerate() {
            image.write_sector(bitmap_start as u32 + index as u32, bitmap)?;
        }

        // Root directory sector map.
        let mut map = vec![0u8; sector_size];
        map[4..6].copy_from_slice(&root_data.to_le_bytes());
        image.write_sector(root_map as u32, &map)?;

        // Empty root directory contains only its 23-byte header entry.
        let mut directory = vec![0u8; sector_size];
        directory[0] = 0x28; // in-use + directory
        directory[3] = ENTRY as u8;
        directory[6..17].copy_from_slice(b"MAIN_DIR   ");
        image.write_sector(root_data as u32, &directory)?;

        // SpartaDOS volume information / boot sector. This layout mirrors the
        // conventional SpartaDOS 2.x formatter and is recognized by SDX.
        let mut boot = vec![0u8; image.read_sector(1)?.len()];
        boot[0] = 0x28;
        boot[3] = ENTRY as u8;
        boot[6] = b'M';
        boot[7] = 0x80;
        boot[8] = b'I';
        boot[9..11].copy_from_slice(&root_map.to_le_bytes());
        boot[11..13].copy_from_slice(&(total as u16).to_le_bytes());
        let free = total - root_data as u32;
        boot[13..15].copy_from_slice(&(free as u16).to_le_bytes());
        boot[15] = bitmap_count as u8;
        boot[16..18].copy_from_slice(&bitmap_start.to_le_bytes());
        let first_data = root_data.saturating_add(1);
        boot[20..22].copy_from_slice(&first_data.to_le_bytes());
        let hint = first_data.wrapping_add(0x21);
        boot[18..20].copy_from_slice(&hint.to_le_bytes());
        boot[22..30].fill(b' ');
        let volume = label.unwrap_or(DEFAULT_LABEL).trim().to_ascii_uppercase();
        if volume.is_empty() || volume.as_bytes()[0] == b' ' {
            return Err(BroadsideError::InvalidArgument(
                "SpartaDOS volume label may not be empty or begin with a space".into(),
            ));
        }
        for (index, byte) in volume.bytes().take(8).enumerate() {
            boot[22 + index] = byte;
        }
        boot[30] = 1;
        boot[31] = sector_size as u8; // 256 is encoded as zero
        boot[32] = 0x20; // SpartaDOS 2.x filesystem ID
        boot[38] = 0;
        boot[39] = 0xFA;
        boot[40] = 0;
        image.write_sector(1, &boot)?;
        image.flush()
    }
    pub fn new(image: &'a mut dyn DiskImage) -> Result<Self> {
        let b = image.read_sector(1)?;
        if b.len() < 41 || b[32] != 0x20 {
            return Err(BroadsideError::InvalidImage(
                "not a SpartaDOS 2 filesystem".into(),
            ));
        }
        let total = u16::from_le_bytes([b[11], b[12]]) as u32;
        if total != image.total_sectors() {
            return Err(BroadsideError::InvalidImage(format!(
                "SpartaDOS sector count {total} does not match image {}",
                image.total_sectors()
            )));
        }
        Ok(Self {
            root_map: u16::from_le_bytes([b[9], b[10]]),
            bitmap_start: u16::from_le_bytes([b[16], b[17]]),
            bitmap_count: b[15],
            image,
        })
    }
    pub fn label(&mut self) -> Result<String> {
        let b = self.image.read_sector(1)?;
        Ok(String::from_utf8_lossy(&b[22..30]).trim_end().to_string())
    }
    pub fn set_label(&mut self, s: &str) -> Result<()> {
        let mut b = self.image.read_sector(1)?;
        b[22..30].fill(b' ');
        for (i, c) in s.to_ascii_uppercase().bytes().take(8).enumerate() {
            b[22 + i] = c;
        }
        self.image.write_sector(1, &b)?;
        self.image.flush()
    }
    pub fn free_sectors(&mut self) -> Result<u32> {
        let b = self.image.read_sector(1)?;
        Ok(u16::from_le_bytes([b[13], b[14]]) as u32)
    }
    fn map_data_sectors(&mut self, map: u16) -> Result<Vec<u16>> {
        let mut out = Vec::new();
        let mut m = map;
        let mut guard = 0;
        while m != 0 {
            guard += 1;
            if guard > self.image.total_sectors() {
                return Err(BroadsideError::InvalidImage("sector-map loop".into()));
            }
            let b = self.image.read_sector(m as u32)?;
            for p in (4..b.len()).step_by(2) {
                let s = u16::from_le_bytes([b[p], b[p + 1]]);
                if s != 0 {
                    out.push(s)
                }
            }
            m = u16::from_le_bytes([b[0], b[1]]);
        }
        Ok(out)
    }
    fn read_mapped(&mut self, map: u16, size: usize) -> Result<Vec<u8>> {
        let mut out = Vec::with_capacity(size);
        for s in self.map_data_sectors(map)? {
            let b = self.image.read_sector(s as u32)?;
            let n = (size - out.len()).min(b.len());
            out.extend_from_slice(&b[..n]);
            if out.len() == size {
                break;
            }
        }
        if out.len() < size {
            return Err(BroadsideError::InvalidImage(
                "mapped file is shorter than directory entry size".into(),
            ));
        }
        Ok(out)
    }
    fn parse_name(e: &[u8]) -> String {
        let base = String::from_utf8_lossy(&e[6..14]).trim_end().to_string();
        let ext = String::from_utf8_lossy(&e[14..17]).trim_end().to_string();
        if ext.is_empty() {
            base
        } else {
            format!("{base}.{ext}")
        }
    }
    fn entries_in_map(&mut self, map: u16) -> Result<Vec<SpartaEntry>> {
        let sectors = self.map_data_sectors(map)?;
        if sectors.is_empty() {
            return Ok(vec![]);
        }
        let first = self.image.read_sector(sectors[0] as u32)?;
        let size = (first[3] as usize) | ((first[4] as usize) << 8) | ((first[5] as usize) << 16);
        let data = self.read_mapped(map, size)?;
        let mut v = Vec::new();
        for off in (ENTRY..size).step_by(ENTRY) {
            if off + ENTRY > data.len() {
                break;
            }
            let e = &data[off..off + ENTRY];
            if e[0] == 0 {
                break;
            }
            if e[0] & DELETED != 0 || e[0] & IN_USE == 0 {
                continue;
            }
            v.push(SpartaEntry {
                name: Self::parse_name(e),
                attributes: e[0],
                map_sector: u16::from_le_bytes([e[1], e[2]]),
                size_bytes: (e[3] as u32) | ((e[4] as u32) << 8) | ((e[5] as u32) << 16),
                day: e[17],
                month: e[18],
                year: e[19],
                hour: e[20],
                minute: e[21],
                second: e[22],
            });
        }
        Ok(v)
    }
    fn normalize_parts(path: &str) -> Vec<String> {
        path.trim_matches(|c| c == '>' || c == '/' || c == '\\')
            .split(|c| c == '>' || c == '/' || c == '\\')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_uppercase())
            .collect()
    }
    fn find_in(&mut self, map: u16, name: &str) -> Result<Option<SpartaEntry>> {
        let n = name.trim_end_matches('.').to_ascii_uppercase();
        Ok(self
            .entries_in_map(map)?
            .into_iter()
            .find(|e| e.name.trim_end_matches('.').eq_ignore_ascii_case(&n)))
    }
    fn resolve_dir(&mut self, path: &str) -> Result<u16> {
        let mut map = self.root_map;
        for p in Self::normalize_parts(path) {
            let e = self
                .find_in(map, &p)?
                .ok_or_else(|| BroadsideError::NotFound(p.clone()))?;
            if !e.is_dir() {
                return Err(BroadsideError::InvalidArgument(format!(
                    "{p} is not a directory"
                )));
            }
            map = e.map_sector;
        }
        Ok(map)
    }
    fn split_parent(path: &str) -> (String, String) {
        let mut p = Self::normalize_parts(path);
        let name = p.pop().unwrap_or_default();
        (p.join(">"), name)
    }
    pub fn list(&mut self, path: &str, mask: &str) -> Result<Vec<SpartaEntry>> {
        let map = self.resolve_dir(path)?;
        Ok(self
            .entries_in_map(map)?
            .into_iter()
            .filter(|e| wild(&e.name, &mask.to_ascii_uppercase()))
            .collect())
    }
    pub fn extract(&mut self, path: &str) -> Result<Vec<u8>> {
        let (parent, name) = Self::split_parent(path);
        let map = self.resolve_dir(&parent)?;
        let e = self
            .find_in(map, &name)?
            .ok_or_else(|| BroadsideError::NotFound(path.into()))?;
        if e.is_dir() {
            return Err(BroadsideError::InvalidArgument(
                "cannot extract a directory".into(),
            ));
        }
        self.read_mapped(e.map_sector, e.size_bytes as usize)
    }
    fn bitmap_position(&self, sector: u16) -> (u16, usize, u8) {
        let bits = self.image.sector_size() * 8;
        let idx = sector as usize / bits;
        let within = sector as usize % bits;
        (
            self.bitmap_start + idx as u16,
            within / 8,
            0x80u8 >> (within % 8),
        )
    }
    fn set_free(&mut self, sector: u16, free: bool) -> Result<()> {
        if sector < 4 {
            return Err(BroadsideError::InvalidArgument(
                "cannot allocate reserved sector".into(),
            ));
        }
        let (bm, byte, mask) = self.bitmap_position(sector);
        let mut b = self.image.read_sector(bm as u32)?;
        let was = b[byte] & mask != 0;
        if free {
            b[byte] |= mask
        } else {
            b[byte] &= !mask
        }
        self.image.write_sector(bm as u32, &b)?;
        if was != free {
            let mut boot = self.image.read_sector(1)?;
            let mut count = u16::from_le_bytes([boot[13], boot[14]]);
            if free {
                count = count.saturating_add(1)
            } else {
                count = count.saturating_sub(1)
            }
            boot[13..15].copy_from_slice(&count.to_le_bytes());
            self.image.write_sector(1, &boot)?;
        }
        Ok(())
    }
    fn alloc(&mut self) -> Result<u16> {
        for i in 0..self.bitmap_count as u16 {
            let s = self.bitmap_start + i;
            let b = self.image.read_sector(s as u32)?;
            for (byte, &v) in b.iter().enumerate() {
                if v == 0 {
                    continue;
                }
                for bit in 0..8 {
                    if v & (0x80 >> bit) != 0 {
                        let sector =
                            ((i as usize * self.image.sector_size() + byte) * 8 + bit) as u16;
                        if sector >= 4 && sector as u32 <= self.image.total_sectors() {
                            self.set_free(sector, false)?;
                            return Ok(sector);
                        }
                    }
                }
            }
        }
        Err(BroadsideError::Unsupported("disk is full".into()))
    }
    fn write_map(&mut self, data_sectors: &[u16]) -> Result<u16> {
        let cap = (self.image.sector_size() - 4) / 2;
        let maps = (data_sectors.len() + cap - 1) / cap;
        let mut ms = Vec::new();
        for _ in 0..maps.max(1) {
            ms.push(self.alloc()?)
        }
        for i in 0..ms.len() {
            let mut b = vec![0u8; self.image.sector_size()];
            if i + 1 < ms.len() {
                b[0..2].copy_from_slice(&ms[i + 1].to_le_bytes())
            }
            if i > 0 {
                b[2..4].copy_from_slice(&ms[i - 1].to_le_bytes())
            }
            for (j, s) in data_sectors.iter().skip(i * cap).take(cap).enumerate() {
                b[4 + j * 2..6 + j * 2].copy_from_slice(&s.to_le_bytes())
            }
            self.image.write_sector(ms[i] as u32, &b)?;
        }
        Ok(ms[0])
    }
    fn append_entry(&mut self, dir_map: u16, entry: [u8; ENTRY]) -> Result<()> {
        let sectors = self.map_data_sectors(dir_map)?;
        let first = sectors[0];
        let f = self.image.read_sector(first as u32)?;
        let size = (f[3] as usize) | ((f[4] as usize) << 8) | ((f[5] as usize) << 16);
        let mut data = self.read_mapped(dir_map, size)?;
        let mut off = None;
        for o in (ENTRY..size).step_by(ENTRY) {
            if data[o] == 0 || data[o] & DELETED != 0 {
                off = Some(o);
                break;
            }
        }
        if let Some(o) = off {
            data[o..o + ENTRY].copy_from_slice(&entry);
            self.write_existing_mapped(dir_map, &data)?;
            return Ok(());
        }
        data.extend_from_slice(&entry);
        let new_size = data.len();
        let needed = (new_size + self.image.sector_size() - 1) / self.image.sector_size();
        let mut ds = sectors.clone();
        while ds.len() < needed {
            let s = self.alloc()?;
            ds.push(s);
            self.allocate_to_map(dir_map, s)?
        }
        for (i, s) in ds.iter().enumerate() {
            let mut b = vec![0u8; self.image.sector_size()];
            let a = i * b.len();
            let z = (a + b.len()).min(data.len());
            if a < z {
                b[..z - a].copy_from_slice(&data[a..z])
            }
            if i == 0 {
                b[3] = (new_size & 255) as u8;
                b[4] = ((new_size >> 8) & 255) as u8;
                b[5] = ((new_size >> 16) & 255) as u8;
            }
            self.image.write_sector(*s as u32, &b)?
        }
        Ok(())
    }
    fn write_existing_mapped(&mut self, map: u16, data: &[u8]) -> Result<()> {
        for (i, s) in self.map_data_sectors(map)?.iter().enumerate() {
            let mut b = vec![0u8; self.image.sector_size()];
            let a = i * b.len();
            let z = (a + b.len()).min(data.len());
            if a < z {
                b[..z - a].copy_from_slice(&data[a..z])
            }
            self.image.write_sector(*s as u32, &b)?;
            if z == data.len() {
                break;
            }
        }
        Ok(())
    }
    fn allocate_to_map(&mut self, map: u16, new_sector: u16) -> Result<()> {
        let mut m = map;
        loop {
            let mut b = self.image.read_sector(m as u32)?;
            for p in (4..b.len()).step_by(2) {
                if b[p] == 0 && b[p + 1] == 0 {
                    b[p..p + 2].copy_from_slice(&new_sector.to_le_bytes());
                    self.image.write_sector(m as u32, &b)?;
                    return Ok(());
                }
            }
            let next = u16::from_le_bytes([b[0], b[1]]);
            if next == 0 {
                let n = self.alloc()?;
                b[0..2].copy_from_slice(&n.to_le_bytes());
                self.image.write_sector(m as u32, &b)?;
                let mut nb = vec![0u8; self.image.sector_size()];
                nb[2..4].copy_from_slice(&m.to_le_bytes());
                self.image.write_sector(n as u32, &nb)?;
                m = n
            } else {
                m = next
            }
        }
    }
    fn make_entry(name: &str, attr: u8, map: u16, size: u32) -> Result<[u8; ENTRY]> {
        let mut e = [0u8; ENTRY];
        e[0] = attr;
        e[1..3].copy_from_slice(&map.to_le_bytes());
        e[3] = (size & 255) as u8;
        e[4] = ((size >> 8) & 255) as u8;
        e[5] = ((size >> 16) & 255) as u8;
        let (up, ext) = match name.rsplit_once('.') {
            Some((a, b)) => (a, b),
            None => (name, ""),
        };
        if up.is_empty() || up.len() > 8 || ext.len() > 3 {
            return Err(BroadsideError::InvalidArgument(
                "SpartaDOS names must be 1-8 characters plus optional 1-3 character extension"
                    .into(),
            ));
        }
        e[6..17].fill(b' ');
        for (i, c) in up.to_ascii_uppercase().bytes().enumerate() {
            e[6 + i] = c
        }
        for (i, c) in ext.to_ascii_uppercase().bytes().enumerate() {
            e[14 + i] = c
        }
        Ok(e)
    }
    pub fn insert(&mut self, path: &str, data: &[u8]) -> Result<()> {
        let (parent, name) = Self::split_parent(path);
        let dm = self.resolve_dir(&parent)?;
        if self.find_in(dm, &name)?.is_some() {
            return Err(BroadsideError::AlreadyExists(path.into()));
        }
        let mut ds = Vec::new();
        for chunk in data.chunks(self.image.sector_size()) {
            let s = self.alloc()?;
            let mut b = vec![0u8; self.image.sector_size()];
            b[..chunk.len()].copy_from_slice(chunk);
            self.image.write_sector(s as u32, &b)?;
            ds.push(s)
        }
        let map = self.write_map(&ds)?;
        self.append_entry(dm, Self::make_entry(&name, IN_USE, map, data.len() as u32)?)?;
        self.image.flush()
    }
    pub fn mkdir(&mut self, path: &str) -> Result<()> {
        let (parent, name) = Self::split_parent(path);
        let dm = self.resolve_dir(&parent)?;
        if self.find_in(dm, &name)?.is_some() {
            return Err(BroadsideError::AlreadyExists(path.into()));
        }
        let dir_sector = self.alloc()?;
        let map = self.write_map(&[dir_sector])?;
        let mut b = vec![0u8; self.image.sector_size()];
        b[0] = 0xA8;
        b[1..3].copy_from_slice(&dm.to_le_bytes());
        b[3] = ENTRY as u8;
        let header = Self::make_entry(&name, 0xA8, dm, ENTRY as u32)?;
        b[6..17].copy_from_slice(&header[6..17]);
        self.image.write_sector(dir_sector as u32, &b)?;
        self.append_entry(
            dm,
            Self::make_entry(&name, IN_USE | DIRECTORY, map, ENTRY as u32)?,
        )?;
        self.image.flush()
    }
    pub fn rename(&mut self, path: &str, new_name: &str) -> Result<()> {
        if Self::normalize_parts(new_name).len() != 1
            || new_name.bytes().any(|c| matches!(c, b'>' | b'/' | b'\\'))
        {
            return Err(BroadsideError::InvalidArgument(
                "rename requires a filename, not a path".into(),
            ));
        }
        let (parent, name) = Self::split_parent(path);
        let dm = self.resolve_dir(&parent)?;
        let entry = self
            .find_in(dm, &name)?
            .ok_or_else(|| BroadsideError::NotFound(path.into()))?;
        if let Some(existing) = self.find_in(dm, new_name)? {
            if !existing.name.eq_ignore_ascii_case(&entry.name) {
                return Err(BroadsideError::AlreadyExists(new_name.into()));
            }
        }
        let template = Self::make_entry(
            new_name,
            entry.attributes,
            entry.map_sector,
            entry.size_bytes,
        )?;
        let sectors = self.map_data_sectors(dm)?;
        let first = self.image.read_sector(sectors[0] as u32)?;
        let size = (first[3] as usize) | ((first[4] as usize) << 8) | ((first[5] as usize) << 16);
        let mut data = self.read_mapped(dm, size)?;
        let mut renamed = false;
        for off in (ENTRY..size).step_by(ENTRY) {
            if off + ENTRY > data.len() {
                break;
            }
            let current = &data[off..off + ENTRY];
            if current[0] == 0 {
                break;
            }
            if current[0] & IN_USE != 0
                && current[0] & DELETED == 0
                && Self::parse_name(current).eq_ignore_ascii_case(&name)
            {
                data[off + 6..off + 17].copy_from_slice(&template[6..17]);
                renamed = true;
                break;
            }
        }
        if !renamed {
            return Err(BroadsideError::NotFound(path.into()));
        }
        self.write_existing_mapped(dm, &data)?;
        if entry.is_dir() {
            let child_sectors = self.map_data_sectors(entry.map_sector)?;
            let child_sector = *child_sectors.first().ok_or_else(|| {
                BroadsideError::InvalidImage("directory has no data sector".into())
            })?;
            let mut header = self.image.read_sector(child_sector as u32)?;
            header[6..17].copy_from_slice(&template[6..17]);
            self.image.write_sector(child_sector as u32, &header)?;
        }
        self.image.flush()
    }
    fn free_map_chain(&mut self, map: u16) -> Result<()> {
        let mut m = map;
        while m != 0 {
            let b = self.image.read_sector(m as u32)?;
            let next = u16::from_le_bytes([b[0], b[1]]);
            for p in (4..b.len()).step_by(2) {
                let s = u16::from_le_bytes([b[p], b[p + 1]]);
                if s != 0 {
                    self.set_free(s, true)?
                }
            }
            self.set_free(m, true)?;
            m = next
        }
        Ok(())
    }
    fn mark_deleted(&mut self, dm: u16, name: &str, expect_dir: bool) -> Result<SpartaEntry> {
        let sectors = self.map_data_sectors(dm)?;
        let first = self.image.read_sector(sectors[0] as u32)?;
        let size = (first[3] as usize) | ((first[4] as usize) << 8) | ((first[5] as usize) << 16);
        let mut data = self.read_mapped(dm, size)?;
        for off in (ENTRY..size).step_by(ENTRY) {
            if off + ENTRY > data.len() {
                break;
            }
            let e = &data[off..off + ENTRY];
            if e[0] == 0 {
                break;
            }
            if e[0] & IN_USE != 0
                && e[0] & DELETED == 0
                && Self::parse_name(e).eq_ignore_ascii_case(name)
            {
                let ent = SpartaEntry {
                    name: Self::parse_name(e),
                    attributes: e[0],
                    map_sector: u16::from_le_bytes([e[1], e[2]]),
                    size_bytes: (e[3] as u32) | ((e[4] as u32) << 8) | ((e[5] as u32) << 16),
                    day: e[17],
                    month: e[18],
                    year: e[19],
                    hour: e[20],
                    minute: e[21],
                    second: e[22],
                };
                if ent.is_dir() != expect_dir {
                    return Err(BroadsideError::InvalidArgument(
                        "entry type mismatch".into(),
                    ));
                }
                data[off] = (data[off] & !IN_USE) | DELETED;
                self.write_existing_mapped(dm, &data)?;
                return Ok(ent);
            }
        }
        Err(BroadsideError::NotFound(name.into()))
    }
    pub fn delete(&mut self, path: &str, dir: bool) -> Result<()> {
        let (parent, name) = Self::split_parent(path);
        let dm = self.resolve_dir(&parent)?;
        let e = self
            .find_in(dm, &name)?
            .ok_or_else(|| BroadsideError::NotFound(path.into()))?;
        if dir {
            if !e.is_dir() {
                return Err(BroadsideError::InvalidArgument("not a directory".into()));
            }
            if !self.entries_in_map(e.map_sector)?.is_empty() {
                return Err(BroadsideError::Unsupported("directory is not empty".into()));
            }
        } else if e.is_dir() {
            return Err(BroadsideError::InvalidArgument(
                "use -R to remove a directory".into(),
            ));
        }
        let e = self.mark_deleted(dm, &name, dir)?;
        self.free_map_chain(e.map_sector)?;
        self.image.flush()
    }
}
fn wild(name: &str, pat: &str) -> bool {
    fn go(n: &[u8], p: &[u8]) -> bool {
        if p.is_empty() {
            return n.is_empty();
        }
        match p[0] {
            b'*' => (0..=n.len()).any(|i| go(&n[i..], &p[1..])),
            b'?' => !n.is_empty() && go(&n[1..], &p[1..]),
            c => {
                !n.is_empty()
                    && n[0].to_ascii_uppercase() == c.to_ascii_uppercase()
                    && go(&n[1..], &p[1..])
            }
        }
    }
    go(name.as_bytes(), pat.as_bytes())
}
