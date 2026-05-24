#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use lz4::{Decoder as Lz4Decoder, EncoderBuilder as Lz4EncoderBuilder};

use super::nbt::{read_named_tag, write_named_tag, Tag};

pub const SECTOR_BYTES: u32 = 4096;
pub const HEADER_BYTES: u64 = 8192;
pub const CHUNKS_PER_REGION_AXIS: i32 = 32;
pub const OLD_CHUNK_DATA_VERSION_CUTOFF: i32 = 4295;
pub const OLD_CHUNK_REGION_CACHE_SIZE: usize = 1024;
pub const REGION_FILE_STORAGE_CACHE_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionPos {
    pub x: i32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionLocation {
    pub sector_offset: u32,
    pub sector_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionCompression {
    Gzip,
    Deflate,
    None,
    Lz4,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionFile {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionFileStorage {
    dir: PathBuf,
    region_cache: RefCell<BTreeMap<RegionPos, RegionFile>>,
    region_lru: RefCell<VecDeque<RegionPos>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionWrite {
    pub name: String,
    pub tag: Option<Tag>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionIoWorker {
    storage: RegionFileStorage,
    pending_writes: BTreeMap<ChunkPos, PendingRegionWrite>,
    old_chunk_mask_cache: RefCell<BTreeMap<RegionPos, Vec<bool>>>,
    old_chunk_mask_lru: RefCell<VecDeque<RegionPos>>,
    closed: bool,
}

impl ChunkPos {
    pub fn region(self) -> RegionPos {
        RegionPos {
            x: self.x.div_euclid(CHUNKS_PER_REGION_AXIS),
            z: self.z.div_euclid(CHUNKS_PER_REGION_AXIS),
        }
    }

    pub fn local_index(self) -> usize {
        let local_x = self.x.rem_euclid(CHUNKS_PER_REGION_AXIS) as usize;
        let local_z = self.z.rem_euclid(CHUNKS_PER_REGION_AXIS) as usize;
        local_x + local_z * CHUNKS_PER_REGION_AXIS as usize
    }
}

impl RegionPos {
    pub fn file_name(self) -> String {
        format!("r.{}.{}.mca", self.x, self.z)
    }

    pub fn min_chunk_pos(self) -> ChunkPos {
        ChunkPos {
            x: self.x * CHUNKS_PER_REGION_AXIS,
            z: self.z * CHUNKS_PER_REGION_AXIS,
        }
    }

    pub fn max_chunk_pos(self) -> ChunkPos {
        let min = self.min_chunk_pos();
        ChunkPos {
            x: min.x + CHUNKS_PER_REGION_AXIS - 1,
            z: min.z + CHUNKS_PER_REGION_AXIS - 1,
        }
    }
}

impl RegionCompression {
    pub const DEFAULT: Self = Self::Deflate;

    pub fn id(self) -> u8 {
        match self {
            Self::Gzip => 1,
            Self::Deflate => 2,
            Self::None => 3,
            Self::Lz4 => 4,
            Self::Custom => 127,
        }
    }

    pub fn option_name(self) -> Option<&'static str> {
        match self {
            Self::Deflate => Some("deflate"),
            Self::None => Some("none"),
            Self::Lz4 => Some("lz4"),
            Self::Gzip | Self::Custom => None,
        }
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            1 => Self::Gzip,
            2 => Self::Deflate,
            3 => Self::None,
            4 => Self::Lz4,
            127 => Self::Custom,
            _ => return None,
        })
    }

    pub fn from_option_name(option_name: &str) -> Option<Self> {
        match option_name {
            "deflate" => Some(Self::Deflate),
            "none" => Some(Self::None),
            "lz4" => Some(Self::Lz4),
            _ => None,
        }
    }

    pub fn is_valid_id(id: u8) -> bool {
        Self::from_id(id).is_some()
    }
}

impl RegionFile {
    pub fn open(dir: &Path, pos: RegionPos) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let path = dir.join(pos.file_name());
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&path)?;
        if file.metadata()?.len() < HEADER_BYTES {
            file.set_len(HEADER_BYTES)?;
        }
        drop(file);

        let region = Self { path };
        region.sanitize_header_locations()?;
        Ok(region)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn external_chunk_path(&self, chunk: ChunkPos) -> Option<PathBuf> {
        self.path
            .parent()
            .map(|dir| dir.join(format!("c.{}.{}.mcc", chunk.x, chunk.z)))
    }

    fn sanitize_header_locations(&self) -> io::Result<()> {
        let file_len = fs::metadata(&self.path)?.len();
        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
        for index in 0..(CHUNKS_PER_REGION_AXIS * CHUNKS_PER_REGION_AXIS) as usize {
            file.seek(SeekFrom::Start((index * 4) as u64))?;
            let mut bytes = [0u8; 4];
            file.read_exact(&mut bytes)?;
            let sector_offset = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
            let sector_count = bytes[3];
            if sector_offset == 0 {
                continue;
            }
            if sector_offset < 2
                || sector_count == 0
                || sector_offset as u64 * SECTOR_BYTES as u64 > file_len
            {
                file.seek(SeekFrom::Start((index * 4) as u64))?;
                file.write_all(&[0, 0, 0, 0])?;
            }
        }
        Ok(())
    }

    fn allocate_sectors(&self, sectors_needed: usize) -> io::Result<u32> {
        if sectors_needed == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cannot allocate zero region sectors",
            ));
        }

        let file_len = fs::metadata(&self.path)?.len();
        let sector_len = (file_len as usize).div_ceil(SECTOR_BYTES as usize).max(2);
        let mut used = vec![false; sector_len];
        used[0] = true;
        used[1] = true;

        for index in 0..(CHUNKS_PER_REGION_AXIS * CHUNKS_PER_REGION_AXIS) as usize {
            let chunk = ChunkPos {
                x: (index as i32).rem_euclid(CHUNKS_PER_REGION_AXIS),
                z: (index as i32).div_euclid(CHUNKS_PER_REGION_AXIS),
            };
            let Some(location) = self.read_location(chunk)? else {
                continue;
            };
            let start = location.sector_offset as usize;
            let end = start.saturating_add(location.sector_count as usize);
            if end <= used.len() {
                used[start..end].fill(true);
            }
        }

        for start in 2..used.len() {
            let end = start + sectors_needed;
            if end <= used.len() && used[start..end].iter().all(|sector| !*sector) {
                return Ok(start as u32);
            }
        }

        if sector_len + sectors_needed > 0xFF_FF_FF {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "region sector offset exceeds 24-bit header field",
            ));
        }
        Ok(sector_len as u32)
    }

    pub fn read_location(&self, chunk: ChunkPos) -> io::Result<Option<RegionLocation>> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start((chunk.local_index() * 4) as u64))?;
        let mut bytes = [0u8; 4];
        file.read_exact(&mut bytes)?;
        let sector_offset = u32::from_be_bytes([0, bytes[0], bytes[1], bytes[2]]);
        let sector_count = bytes[3];
        if sector_offset == 0 || sector_count == 0 {
            Ok(None)
        } else {
            Ok(Some(RegionLocation {
                sector_offset,
                sector_count,
            }))
        }
    }

    pub fn write_location(&self, chunk: ChunkPos, location: RegionLocation) -> io::Result<()> {
        if location.sector_offset > 0xFF_FF_FF {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "region sector offset exceeds 24-bit header field",
            ));
        }

        let mut file = OpenOptions::new().write(true).open(&self.path)?;
        file.seek(SeekFrom::Start((chunk.local_index() * 4) as u64))?;
        let offset = location.sector_offset.to_be_bytes();
        file.write_all(&[offset[1], offset[2], offset[3], location.sector_count])
    }

    pub fn read_timestamp(&self, chunk: ChunkPos) -> io::Result<u32> {
        let mut file = File::open(&self.path)?;
        let offset = 4096 + (chunk.local_index() * 4) as u64;
        file.seek(SeekFrom::Start(offset))?;
        let mut bytes = [0u8; 4];
        file.read_exact(&mut bytes)?;
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn write_timestamp(&self, chunk: ChunkPos, timestamp: u32) -> io::Result<()> {
        let mut file = OpenOptions::new().write(true).open(&self.path)?;
        let offset = 4096 + (chunk.local_index() * 4) as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&timestamp.to_be_bytes())
    }

    pub fn read_chunk_nbt(&self, chunk: ChunkPos) -> io::Result<Option<(String, Tag)>> {
        let Some(location) = self.read_location(chunk)? else {
            return Ok(None);
        };
        let byte_offset = location.sector_offset as u64 * SECTOR_BYTES as u64;
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(byte_offset))?;
        let mut length_bytes = [0u8; 4];
        if let Err(err) = file.read_exact(&mut length_bytes) {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(err);
        }
        let data_len = u32::from_be_bytes(length_bytes) as usize;
        if data_len == 0 || data_len > location.sector_count as usize * SECTOR_BYTES as usize - 4 {
            return Ok(None);
        }
        let mut data = vec![0u8; data_len];
        if let Err(err) = file.read_exact(&mut data) {
            if err.kind() == io::ErrorKind::UnexpectedEof {
                return Ok(None);
            }
            return Err(err);
        }
        let compression_type = data[0];
        let compressed = &data[1..];
        if compression_type & 0x80 != 0 {
            let Some(external_path) = self.external_chunk_path(chunk) else {
                return Ok(None);
            };
            let external = match fs::read(external_path) {
                Ok(bytes) => bytes,
                Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
                Err(err) => return Err(err),
            };
            return decode_region_payload(compression_type & 0x7F, &external);
        }
        decode_region_payload(compression_type, compressed)
    }

    pub fn does_chunk_exist(&self, chunk: ChunkPos) -> bool {
        self.does_chunk_exist_inner(chunk).unwrap_or(false)
    }

    fn does_chunk_exist_inner(&self, chunk: ChunkPos) -> io::Result<bool> {
        let Some(location) = self.read_location(chunk)? else {
            return Ok(false);
        };
        let byte_offset = location.sector_offset as u64 * SECTOR_BYTES as u64;
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(byte_offset))?;
        let mut header = [0_u8; 5];
        if file.read_exact(&mut header).is_err() {
            return Ok(false);
        }
        let length = u32::from_be_bytes(header[0..4].try_into().unwrap());
        let version_id = header[4];
        if version_id & 0x80 != 0 {
            if !RegionCompression::is_valid_id(version_id & 0x7f) {
                return Ok(false);
            }
            let Some(external_path) = self.external_chunk_path(chunk) else {
                return Ok(false);
            };
            return Ok(external_path.is_file());
        }
        if !RegionCompression::is_valid_id(version_id) || length == 0 {
            return Ok(false);
        }
        let stream_length = i64::from(length) - 1;
        Ok(stream_length >= 0
            && stream_length <= i64::from(SECTOR_BYTES) * i64::from(location.sector_count))
    }

    pub fn write_chunk_nbt(&self, chunk: ChunkPos, name: &str, tag: &Tag) -> io::Result<()> {
        self.write_chunk_nbt_with_compression(chunk, name, tag, RegionCompression::DEFAULT)
    }

    pub fn clear_chunk_nbt(&self, chunk: ChunkPos) -> io::Result<()> {
        if self.read_location(chunk)?.is_none() {
            return Ok(());
        }
        self.write_location(
            chunk,
            RegionLocation {
                sector_offset: 0,
                sector_count: 0,
            },
        )?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        self.write_timestamp(chunk, timestamp)?;
        if let Some(external_path) = self.external_chunk_path(chunk) {
            match fs::remove_file(external_path) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }

    pub fn flush(&self) -> io::Result<()> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)?
            .sync_all()
    }

    pub fn close(&self) -> io::Result<()> {
        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
        let file_len = file.metadata()?.len();
        let padded_len = file_len.div_ceil(SECTOR_BYTES as u64) * SECTOR_BYTES as u64;
        if file_len != padded_len {
            file.seek(SeekFrom::Start(padded_len - 1))?;
            file.write_all(&[0])?;
        }
        file.sync_all()
    }

    pub fn write_chunk_nbt_with_compression(
        &self,
        chunk: ChunkPos,
        name: &str,
        tag: &Tag,
        compression: RegionCompression,
    ) -> io::Result<()> {
        let compressed = encode_region_payload(name, tag, compression)?;

        // data_len = compression-type byte + compressed bytes
        let data_len = 1 + compressed.len();
        let mut chunk_bytes: Vec<u8> = Vec::with_capacity(4 + data_len);
        chunk_bytes.extend_from_slice(&(data_len as u32).to_be_bytes());
        chunk_bytes.push(compression.id());
        chunk_bytes.extend_from_slice(&compressed);

        let mut sector_count = chunk_bytes.len().div_ceil(SECTOR_BYTES as usize);
        if sector_count >= 256 {
            let external_path = self.external_chunk_path(chunk).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "region file path has no parent for external chunk stream",
                )
            })?;
            fs::write(external_path, &compressed)?;
            chunk_bytes.clear();
            chunk_bytes.extend_from_slice(&1u32.to_be_bytes());
            chunk_bytes.push(compression.id() | 0x80);
            sector_count = 1;
        } else if let Some(external_path) = self.external_chunk_path(chunk) {
            match fs::remove_file(external_path) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
        chunk_bytes.resize(sector_count * SECTOR_BYTES as usize, 0);

        let sector_start = self.allocate_sectors(sector_count)?;
        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;

        file.seek(SeekFrom::Start(sector_start as u64 * SECTOR_BYTES as u64))?;
        file.write_all(&chunk_bytes)?;
        drop(file);

        self.write_location(
            chunk,
            RegionLocation {
                sector_offset: sector_start,
                sector_count: sector_count as u8,
            },
        )?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        self.write_timestamp(chunk, timestamp)
    }
}

impl RegionFileStorage {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            region_cache: RefCell::new(BTreeMap::new()),
            region_lru: RefCell::new(VecDeque::new()),
        })
    }

    pub fn cached_region_count(&self) -> usize {
        self.region_cache.borrow().len()
    }

    pub fn get_region_file(&self, region_pos: RegionPos) -> io::Result<RegionFile> {
        if let Some(region) = self.cached_region_file(region_pos) {
            return Ok(region);
        }

        let region = RegionFile::open(&self.dir, region_pos)?;
        self.cache_region_file(region_pos, region.clone())?;
        Ok(region)
    }

    pub fn read_chunk_nbt(&self, chunk: ChunkPos) -> io::Result<Option<(String, Tag)>> {
        self.get_region_file(chunk.region())?.read_chunk_nbt(chunk)
    }

    pub fn write_chunk_nbt(&self, chunk: ChunkPos, name: &str, tag: &Tag) -> io::Result<()> {
        self.get_region_file(chunk.region())?
            .write_chunk_nbt(chunk, name, tag)
    }

    pub fn clear_chunk_nbt(&self, chunk: ChunkPos) -> io::Result<()> {
        self.get_region_file(chunk.region())?.clear_chunk_nbt(chunk)
    }

    pub fn flush(&self) -> io::Result<()> {
        for region in self.region_cache.borrow().values() {
            region.flush()?;
        }
        Ok(())
    }

    pub fn close(&self) -> io::Result<()> {
        let mut first_error = None;
        for region in self.region_cache.borrow().values() {
            if let Err(err) = region.close() {
                first_error.get_or_insert(err);
            }
        }
        self.region_cache.borrow_mut().clear();
        self.region_lru.borrow_mut().clear();
        match first_error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    fn cached_region_file(&self, region_pos: RegionPos) -> Option<RegionFile> {
        let region = self.region_cache.borrow().get(&region_pos).cloned()?;
        let mut lru = self.region_lru.borrow_mut();
        if let Some(index) = lru.iter().position(|cached| *cached == region_pos) {
            lru.remove(index);
        }
        lru.push_front(region_pos);
        Some(region)
    }

    fn cache_region_file(&self, region_pos: RegionPos, region: RegionFile) -> io::Result<()> {
        self.region_cache.borrow_mut().insert(region_pos, region);

        let mut lru = self.region_lru.borrow_mut();
        if let Some(index) = lru.iter().position(|cached| *cached == region_pos) {
            lru.remove(index);
        }
        lru.push_front(region_pos);

        while lru.len() > REGION_FILE_STORAGE_CACHE_SIZE {
            if let Some(expired) = lru.pop_back() {
                if let Some(region) = self.region_cache.borrow_mut().remove(&expired) {
                    region.close()?;
                }
            }
        }
        Ok(())
    }
}

impl RegionIoWorker {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        Ok(Self {
            storage: RegionFileStorage::open(dir)?,
            pending_writes: BTreeMap::new(),
            old_chunk_mask_cache: RefCell::new(BTreeMap::new()),
            old_chunk_mask_lru: RefCell::new(VecDeque::new()),
            closed: false,
        })
    }

    pub fn pending_write_count(&self) -> usize {
        self.pending_writes.len()
    }

    pub fn store_chunk_nbt(&mut self, chunk: ChunkPos, name: impl Into<String>, tag: Tag) {
        self.invalidate_old_chunk_mask_cache(chunk.region());
        self.pending_writes.insert(
            chunk,
            PendingRegionWrite {
                name: name.into(),
                tag: Some(tag),
            },
        );
    }

    pub fn clear_chunk_nbt(&mut self, chunk: ChunkPos) {
        self.invalidate_old_chunk_mask_cache(chunk.region());
        self.pending_writes.insert(
            chunk,
            PendingRegionWrite {
                name: String::new(),
                tag: None,
            },
        );
    }

    pub fn load_chunk_nbt(&self, chunk: ChunkPos) -> io::Result<Option<(String, Tag)>> {
        if let Some(pending) = self.pending_writes.get(&chunk) {
            return Ok(pending.tag.clone().map(|tag| (pending.name.clone(), tag)));
        }
        self.storage.read_chunk_nbt(chunk)
    }

    pub fn scan_chunk_nbt<F>(&self, chunk: ChunkPos, mut visitor: F) -> io::Result<()>
    where
        F: FnMut(&str, &Tag) -> io::Result<()>,
    {
        if let Some(pending) = self.pending_writes.get(&chunk) {
            if let Some(tag) = pending.tag.as_ref() {
                visitor(&pending.name, tag)?;
            }
            return Ok(());
        }

        if let Some((name, tag)) = self.storage.read_chunk_nbt(chunk)? {
            visitor(&name, &tag)?;
        }
        Ok(())
    }

    pub fn synchronize(&mut self) -> io::Result<()> {
        let pending = std::mem::take(&mut self.pending_writes);
        for (chunk, write) in pending {
            if let Some(tag) = write.tag {
                self.storage.write_chunk_nbt(chunk, &write.name, &tag)?;
            } else {
                self.storage.clear_chunk_nbt(chunk)?;
            }
        }
        Ok(())
    }

    pub fn synchronize_with_flush(&mut self) -> io::Result<()> {
        let mut regions_to_flush = Vec::new();
        for chunk in self.pending_writes.keys().copied() {
            let region_pos = chunk.region();
            if !regions_to_flush.contains(&region_pos) {
                regions_to_flush.push(region_pos);
            }
        }
        self.synchronize()?;
        for region_pos in regions_to_flush {
            self.storage.get_region_file(region_pos)?.flush()?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> io::Result<()> {
        if self.closed {
            return Ok(());
        }

        self.synchronize_with_flush()?;
        self.old_chunk_mask_cache.get_mut().clear();
        self.old_chunk_mask_lru.get_mut().clear();
        self.storage.close()?;
        self.closed = true;
        Ok(())
    }

    pub fn old_chunk_mask_for_region(&self, region_pos: RegionPos) -> io::Result<Vec<bool>> {
        if let Some(mask) = self.cached_old_chunk_mask(region_pos) {
            return Ok(mask);
        }

        let mut mask = vec![false; (CHUNKS_PER_REGION_AXIS * CHUNKS_PER_REGION_AXIS) as usize];
        let min = region_pos.min_chunk_pos();
        let max = region_pos.max_chunk_pos();
        for z in min.z..=max.z {
            for x in min.x..=max.x {
                let chunk = ChunkPos { x, z };
                let Some((_name, tag)) = self.load_chunk_nbt(chunk)? else {
                    continue;
                };
                if chunk_tag_is_old_for_blending(&tag) {
                    mask[chunk.local_index()] = true;
                }
            }
        }
        self.cache_old_chunk_mask(region_pos, mask.clone());
        Ok(mask)
    }

    pub fn is_old_chunk_around(&self, pos: ChunkPos, range: i32) -> io::Result<bool> {
        let from = ChunkPos {
            x: pos.x - range,
            z: pos.z - range,
        };
        let to = ChunkPos {
            x: pos.x + range,
            z: pos.z + range,
        };

        for region_x in from.region().x..=to.region().x {
            for region_z in from.region().z..=to.region().z {
                let region_pos = RegionPos {
                    x: region_x,
                    z: region_z,
                };
                let mask = self.old_chunk_mask_for_region(region_pos)?;
                if !mask.iter().any(|old| *old) {
                    continue;
                }

                let min = region_pos.min_chunk_pos();
                let start_x = (from.x - min.x).max(0);
                let start_z = (from.z - min.z).max(0);
                let end_x = (to.x - min.x).min(CHUNKS_PER_REGION_AXIS - 1);
                let end_z = (to.z - min.z).min(CHUNKS_PER_REGION_AXIS - 1);

                for x in start_x..=end_x {
                    for z in start_z..=end_z {
                        let index = (z * CHUNKS_PER_REGION_AXIS + x) as usize;
                        if mask[index] {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    fn cached_old_chunk_mask(&self, region_pos: RegionPos) -> Option<Vec<bool>> {
        let mask = self
            .old_chunk_mask_cache
            .borrow()
            .get(&region_pos)
            .cloned()?;
        let mut lru = self.old_chunk_mask_lru.borrow_mut();
        if let Some(index) = lru.iter().position(|cached| *cached == region_pos) {
            lru.remove(index);
        }
        lru.push_front(region_pos);
        Some(mask)
    }

    fn cache_old_chunk_mask(&self, region_pos: RegionPos, mask: Vec<bool>) {
        self.old_chunk_mask_cache
            .borrow_mut()
            .insert(region_pos, mask);

        let mut lru = self.old_chunk_mask_lru.borrow_mut();
        if let Some(index) = lru.iter().position(|cached| *cached == region_pos) {
            lru.remove(index);
        }
        lru.push_front(region_pos);

        while lru.len() > OLD_CHUNK_REGION_CACHE_SIZE {
            if let Some(expired) = lru.pop_back() {
                self.old_chunk_mask_cache.borrow_mut().remove(&expired);
            }
        }
    }

    fn invalidate_old_chunk_mask_cache(&mut self, region_pos: RegionPos) {
        self.old_chunk_mask_cache.get_mut().remove(&region_pos);
        if let Some(index) = self
            .old_chunk_mask_lru
            .get_mut()
            .iter()
            .position(|cached| *cached == region_pos)
        {
            self.old_chunk_mask_lru.get_mut().remove(index);
        }
    }
}

pub fn chunk_tag_is_old_for_blending(tag: &Tag) -> bool {
    let Tag::Compound(fields) = tag else {
        return false;
    };
    let data_version = fields
        .iter()
        .find_map(|(name, tag)| match (name.as_str(), tag) {
            ("DataVersion", Tag::Int(value)) => Some(*value),
            _ => None,
        })
        .unwrap_or(0);
    data_version < OLD_CHUNK_DATA_VERSION_CUTOFF
        || fields
            .iter()
            .any(|(name, tag)| name == "blending_data" && matches!(tag, Tag::Compound(_)))
}

fn encode_region_payload(
    name: &str,
    tag: &Tag,
    compression: RegionCompression,
) -> io::Result<Vec<u8>> {
    match compression {
        RegionCompression::Gzip => {
            let mut compressed = Vec::new();
            let mut encoder =
                flate2::write::GzEncoder::new(&mut compressed, Compression::default());
            write_named_tag(&mut encoder, name, tag)?;
            encoder.finish()?;
            Ok(compressed)
        }
        RegionCompression::Deflate => {
            let mut compressed = Vec::new();
            let mut encoder = ZlibEncoder::new(&mut compressed, Compression::default());
            write_named_tag(&mut encoder, name, tag)?;
            encoder.finish()?;
            Ok(compressed)
        }
        RegionCompression::None => {
            let mut bytes = Vec::new();
            write_named_tag(&mut bytes, name, tag)?;
            Ok(bytes)
        }
        RegionCompression::Lz4 => {
            let mut compressed = Vec::new();
            let mut encoder = Lz4EncoderBuilder::new().build(&mut compressed)?;
            write_named_tag(&mut encoder, name, tag)?;
            let (_writer, result) = encoder.finish();
            result?;
            Ok(compressed)
        }
        RegionCompression::Custom => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "custom region compression cannot be written without an external stream",
        )),
    }
}

fn decode_region_payload(
    compression_type: u8,
    payload: &[u8],
) -> io::Result<Option<(String, Tag)>> {
    let tag = match compression_type {
        1 => {
            let mut bytes = Vec::new();
            GzDecoder::new(payload).read_to_end(&mut bytes)?;
            read_named_tag(&mut bytes.as_slice())
        }
        2 => {
            let mut bytes = Vec::new();
            ZlibDecoder::new(payload).read_to_end(&mut bytes)?;
            read_named_tag(&mut bytes.as_slice())
        }
        3 => read_named_tag(&mut payload.as_ref()),
        4 => {
            let mut bytes = Vec::new();
            Lz4Decoder::new(payload)?.read_to_end(&mut bytes)?;
            read_named_tag(&mut bytes.as_slice())
        }
        _ => return Ok(None),
    }?;
    Ok(Some(tag))
}

#[cfg(test)]
mod tests;
