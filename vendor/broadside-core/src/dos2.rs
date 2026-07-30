use crate::error::{BroadsideError, Result};
use crate::image::DiskImage;
use crate::types::FileEntry;

const VTOC_SECTOR: u32 = 360;
const DIR_START: u32 = 361;
const DIR_SECTORS: u32 = 8;
const FREE_MASKS: [u8; 8] = [0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x01];

pub struct Dos2<'a> {
    image: &'a mut dyn DiskImage,
}
impl<'a> Dos2<'a> {
    pub fn new(image: &'a mut dyn DiskImage) -> Self {
        Self { image }
    }
    pub fn check(&mut self) -> Result<()> {
        if self.image.total_sectors() < 368 {
            return Err(BroadsideError::Filesystem(
                "image is too small for Atari DOS 2".into(),
            ));
        }
        let vtoc = self.image.read_sector(VTOC_SECTOR)?;
        if vtoc.len() < 100 {
            return Err(BroadsideError::Filesystem("short VTOC sector".into()));
        }
        Ok(())
    }
    pub fn format(&mut self) -> Result<()> {
        if self.image.total_sectors() < 368 || self.image.total_sectors() > 1040 {
            return Err(BroadsideError::Unsupported(
                "DOS 2 formatting supports 368..=1040 sectors".into(),
            ));
        }
        let sector_len = self.image.sector_size();
        let zero = vec![0u8; sector_len];
        for s in 1..=self.image.total_sectors() {
            self.image.write_sector(s, &zero)?;
        }
        let mut vtoc = vec![0u8; sector_len];
        vtoc[0] = 2;
        let max = self.image.total_sectors().min(720);
        for sector in 1..=max {
            Self::set_bitmap_bit(&mut vtoc, sector, true);
        }
        for sector in 1..=3 {
            Self::set_bitmap_bit(&mut vtoc, sector, false);
        }
        Self::set_bitmap_bit(&mut vtoc, VTOC_SECTOR, false);
        for sector in DIR_START..DIR_START + DIR_SECTORS {
            Self::set_bitmap_bit(&mut vtoc, sector, false);
        }
        let free = (max - 12) as u16;
        vtoc[1..3].copy_from_slice(&(max as u16).to_le_bytes());
        vtoc[3..5].copy_from_slice(&free.to_le_bytes());
        self.image.write_sector(VTOC_SECTOR, &vtoc)?;
        for s in DIR_START..DIR_START + DIR_SECTORS {
            self.image.write_sector(s, &zero)?;
        }
        self.image.flush()
    }
    fn set_bitmap_bit(vtoc: &mut [u8], sector: u32, free: bool) {
        let sec = if sector == 720 { 0 } else { sector };
        let idx = (sec / 8) as usize + 10;
        if idx < vtoc.len() {
            let mask = FREE_MASKS[(sec % 8) as usize];
            if free {
                vtoc[idx] |= mask
            } else {
                vtoc[idx] &= !mask
            }
        }
    }
    fn bitmap_is_free(vtoc: &[u8], sector: u32) -> bool {
        let sec = if sector == 720 { 0 } else { sector };
        let idx = (sec / 8) as usize + 10;
        idx < vtoc.len() && vtoc[idx] & FREE_MASKS[(sec % 8) as usize] != 0
    }
    fn update_sector_state(&mut self, sector: u32, free: bool) -> Result<()> {
        let mut v = self.image.read_sector(VTOC_SECTOR)?;
        let mut count = u16::from_le_bytes([v[3], v[4]]);
        let was = Self::bitmap_is_free(&v, sector);
        if was != free {
            if free {
                count = count.saturating_add(1)
            } else {
                count = count.saturating_sub(1)
            };
            Self::set_bitmap_bit(&mut v, sector, free);
            v[3..5].copy_from_slice(&count.to_le_bytes());
            self.image.write_sector(VTOC_SECTOR, &v)?;
        }
        Ok(())
    }
    fn find_free_sector(&mut self) -> Result<u32> {
        let v = self.image.read_sector(VTOC_SECTOR)?;
        for s in 1..=self.image.total_sectors().min(720) {
            if Self::bitmap_is_free(&v, s) {
                return Ok(s);
            }
        }
        Err(BroadsideError::NoSpace)
    }
    pub fn free_sectors(&mut self) -> Result<u16> {
        let v = self.image.read_sector(VTOC_SECTOR)?;
        Ok(u16::from_le_bytes([v[3], v[4]]))
    }
    fn decode_name(raw: &[u8]) -> String {
        let base = String::from_utf8_lossy(&raw[..8]).trim_end().to_string();
        let ext = String::from_utf8_lossy(&raw[8..11]).trim_end().to_string();
        if ext.is_empty() {
            base
        } else {
            format!("{base}.{ext}")
        }
    }
    fn encode_name(name: &str) -> Result<[u8; 11]> {
        let upper = name.to_ascii_uppercase();
        let mut parts = upper.splitn(2, '.');
        let base = parts.next().unwrap_or("");
        let ext = parts.next().unwrap_or("");
        if base.is_empty()
            || base.len() > 8
            || ext.len() > 3
            || !base.bytes().all(Self::valid_name_char)
            || !ext.bytes().all(Self::valid_name_char)
        {
            return Err(BroadsideError::InvalidArgument(format!(
                "'{name}' is not a DOS 2 8.3 filename"
            )));
        }
        let mut out = [b' '; 11];
        out[..base.len()].copy_from_slice(base.as_bytes());
        out[8..8 + ext.len()].copy_from_slice(ext.as_bytes());
        Ok(out)
    }
    fn valid_name_char(c: u8) -> bool {
        c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-' | b'@')
    }
    pub fn list(&mut self, mask: &str) -> Result<Vec<FileEntry>> {
        self.check()?;
        let mut out = Vec::new();
        for ds in 0..DIR_SECTORS {
            let data = self.image.read_sector(DIR_START + ds)?;
            for slot in 0..8usize {
                let o = slot * 16;
                let status = data[o];
                if status == 0 {
                    return Ok(out);
                }
                if status == 0x80 || status & 0x40 == 0 {
                    continue;
                }
                let name = Self::decode_name(&data[o + 5..o + 16]);
                if !wildcard_match(mask, &name) {
                    continue;
                }
                let sectors = u16::from_le_bytes([data[o + 1], data[o + 2]]);
                let first = u16::from_le_bytes([data[o + 3], data[o + 4]]);
                let file_id = (ds * 8 + slot as u32) as u8;
                let size = self.file_size(first, file_id)?;
                out.push(FileEntry {
                    name,
                    size_bytes: size,
                    size_sectors: sectors,
                    first_sector: first,
                    attributes: status,
                    file_id,
                });
            }
        }
        Ok(out)
    }
    fn file_size(&mut self, first: u16, id: u8) -> Result<usize> {
        let mut s = first as u32;
        let mut total = 0usize;
        let mut guard = 0;
        while s != 0 {
            guard += 1;
            if guard > self.image.total_sectors() {
                return Err(BroadsideError::Filesystem("file sector chain loop".into()));
            }
            let d = self.image.read_sector(s)?;
            let n = d.len();
            if n < 3 {
                return Err(BroadsideError::Filesystem("short file sector".into()));
            }
            if d[n - 3] >> 2 != id {
                return Err(BroadsideError::Filesystem(
                    "file ID mismatch in sector chain".into(),
                ));
            }
            let next = (((d[n - 3] & 3) as u16) << 8) | d[n - 2] as u16;
            let used = d[n - 1] as usize;
            if used > n - 3 {
                return Err(BroadsideError::Filesystem("invalid used-byte count".into()));
            }
            total += used;
            s = next as u32;
        }
        Ok(total)
    }
    pub fn extract(&mut self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .list(name)?
            .into_iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| BroadsideError::NotFound(name.into()))?;
        let mut out = Vec::with_capacity(entry.size_bytes);
        let mut s = entry.first_sector as u32;
        while s != 0 {
            let d = self.image.read_sector(s)?;
            let n = d.len();
            if d[n - 3] >> 2 != entry.file_id {
                return Err(BroadsideError::Filesystem("file ID mismatch".into()));
            }
            let next = (((d[n - 3] & 3) as u16) << 8) | d[n - 2] as u16;
            let used = d[n - 1] as usize;
            out.extend_from_slice(&d[..used]);
            s = next as u32;
        }
        Ok(out)
    }
    fn find_free_slot(&mut self) -> Result<(u32, usize, u8)> {
        for ds in 0..DIR_SECTORS {
            let d = self.image.read_sector(DIR_START + ds)?;
            for slot in 0..8usize {
                let status = d[slot * 16];
                if status == 0 || status == 0x80 {
                    return Ok((DIR_START + ds, slot, (ds * 8 + slot as u32) as u8));
                }
            }
        }
        Err(BroadsideError::Filesystem("directory is full".into()))
    }
    pub fn insert(&mut self, name: &str, bytes: &[u8]) -> Result<()> {
        if self
            .list(name)?
            .iter()
            .any(|e| e.name.eq_ignore_ascii_case(name))
        {
            return Err(BroadsideError::AlreadyExists(name.into()));
        }
        let encoded = Self::encode_name(name)?;
        let payload = self.image.sector_size() - 3;
        let needed = (bytes.len() + payload - 1) / payload;
        if needed == 0 {
            return Err(BroadsideError::InvalidArgument(
                "empty files are not supported by DOS 2".into(),
            ));
        }
        if (self.free_sectors()? as usize) < needed {
            return Err(BroadsideError::NoSpace);
        }
        let (dir_sector, slot, id) = self.find_free_slot()?;
        let mut allocated = Vec::with_capacity(needed);
        for _ in 0..needed {
            let s = self.find_free_sector()?;
            self.update_sector_state(s, false)?;
            allocated.push(s);
        }
        for (i, &s) in allocated.iter().enumerate() {
            let start = i * payload;
            let end = (start + payload).min(bytes.len());
            let used = end - start;
            let next = allocated.get(i + 1).copied().unwrap_or(0);
            let mut d = vec![0u8; self.image.sector_size()];
            d[..used].copy_from_slice(&bytes[start..end]);
            let n = d.len();
            d[n - 3] = (id << 2) | (((next >> 8) & 3) as u8);
            d[n - 2] = (next & 0xff) as u8;
            d[n - 1] = used as u8;
            self.image.write_sector(s, &d)?;
        }
        let mut dir = self.image.read_sector(dir_sector)?;
        let o = slot * 16;
        dir[o] = 0x42;
        dir[o + 1..o + 3].copy_from_slice(&(needed as u16).to_le_bytes());
        dir[o + 3..o + 5].copy_from_slice(&(allocated[0] as u16).to_le_bytes());
        dir[o + 5..o + 16].copy_from_slice(&encoded);
        self.image.write_sector(dir_sector, &dir)?;
        self.image.flush()
    }
    pub fn rename(&mut self, name: &str, new_name: &str) -> Result<()> {
        let encoded = Self::encode_name(new_name)?;
        let entries = self.list("*.*")?;
        let entry = entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| BroadsideError::NotFound(name.into()))?;
        if entries
            .iter()
            .any(|e| e.file_id != entry.file_id && e.name.eq_ignore_ascii_case(new_name))
        {
            return Err(BroadsideError::AlreadyExists(new_name.into()));
        }
        let ds = DIR_START + (entry.file_id as u32 / 8);
        let mut data = self.image.read_sector(ds)?;
        let offset = (entry.file_id as usize % 8) * 16 + 5;
        data[offset..offset + 11].copy_from_slice(&encoded);
        self.image.write_sector(ds, &data)?;
        self.image.flush()
    }
    pub fn delete(&mut self, name: &str, clear: bool) -> Result<()> {
        let entry = self
            .list(name)?
            .into_iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| BroadsideError::NotFound(name.into()))?;
        let mut s = entry.first_sector as u32;
        while s != 0 {
            let d = self.image.read_sector(s)?;
            let n = d.len();
            let next = (((d[n - 3] & 3) as u16) << 8) | d[n - 2] as u16;
            self.update_sector_state(s, true)?;
            if clear {
                self.image.write_sector(s, &vec![0u8; n])?;
            }
            s = next as u32;
        }
        let ds = DIR_START + (entry.file_id as u32 / 8);
        let mut d = self.image.read_sector(ds)?;
        d[(entry.file_id as usize % 8) * 16] = 0x80;
        self.image.write_sector(ds, &d)?;
        self.image.flush()
    }
}

pub fn wildcard_match(pattern: &str, value: &str) -> bool {
    let p = pattern.to_ascii_uppercase().into_bytes();
    let v = value.to_ascii_uppercase().into_bytes();
    let (mut pi, mut vi, mut star, mut mark) = (0usize, 0usize, None, 0usize);
    while vi < v.len() {
        if pi < p.len() && (p[pi] == b'?' || p[pi] == v[vi]) {
            pi += 1;
            vi += 1;
        } else if pi < p.len() && p[pi] == b'*' {
            star = Some(pi);
            pi += 1;
            mark = vi;
        } else if let Some(s) = star {
            pi = s + 1;
            mark += 1;
            vi = mark;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == b'*' {
        pi += 1
    }
    pi == p.len()
}

pub fn dump_hex(data: &[u8]) -> String {
    let mut out = String::new();
    for (row, chunk) in data.chunks(16).enumerate() {
        use std::fmt::Write;
        let _ = write!(out, "{:04x}:", row * 16);
        for b in chunk {
            let _ = write!(out, " {b:02x}");
        }
        for _ in chunk.len()..16 {
            out.push_str("   ");
        }
        out.push_str(" | ");
        for &b in chunk {
            out.push(if (0x20..=0x7e).contains(&b) {
                b as char
            } else {
                '.'
            });
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::wildcard_match;
    #[test]
    fn masks() {
        assert!(wildcard_match("*.*", "FOO.BAR"));
        assert!(wildcard_match("F??.*", "FOO.BAR"));
        assert!(!wildcard_match("B*", "FOO.BAR"));
    }
}
