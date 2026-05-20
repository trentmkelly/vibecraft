#![allow(dead_code)]

use std::collections::BTreeMap;
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
pub struct PendingRegionWrite {
    pub name: String,
    pub tag: Tag,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionIoWorker {
    dir: PathBuf,
    pending_writes: BTreeMap<ChunkPos, PendingRegionWrite>,
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

        let mut file = OpenOptions::new().read(true).write(true).open(&self.path)?;
        let file_len = file.seek(SeekFrom::End(0))?;
        let sector_start = (file_len as usize).div_ceil(SECTOR_BYTES as usize) as u32;
        let sector_start = sector_start.max(2); // sectors 0-1 are the header

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

impl RegionIoWorker {
    pub fn open(dir: impl Into<PathBuf>) -> io::Result<Self> {
        let dir = dir.into();
        fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            pending_writes: BTreeMap::new(),
        })
    }

    pub fn pending_write_count(&self) -> usize {
        self.pending_writes.len()
    }

    pub fn store_chunk_nbt(&mut self, chunk: ChunkPos, name: impl Into<String>, tag: Tag) {
        self.pending_writes.insert(
            chunk,
            PendingRegionWrite {
                name: name.into(),
                tag,
            },
        );
    }

    pub fn load_chunk_nbt(&self, chunk: ChunkPos) -> io::Result<Option<(String, Tag)>> {
        if let Some(pending) = self.pending_writes.get(&chunk) {
            return Ok(Some((pending.name.clone(), pending.tag.clone())));
        }
        RegionFile::open(&self.dir, chunk.region())?.read_chunk_nbt(chunk)
    }

    pub fn synchronize(&mut self) -> io::Result<()> {
        let pending = std::mem::take(&mut self.pending_writes);
        for (chunk, write) in pending {
            RegionFile::open(&self.dir, chunk.region())?.write_chunk_nbt(
                chunk,
                &write.name,
                &write.tag,
            )?;
        }
        Ok(())
    }
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
mod tests {
    use super::{
        ChunkPos, RegionCompression, RegionFile, RegionIoWorker, RegionLocation, RegionPos,
        HEADER_BYTES,
    };
    use crate::storage::nbt::Tag;
    use std::fs;
    use std::io::{Seek, SeekFrom, Write};

    #[test]
    fn computes_region_positions_and_local_indexes_like_mca_files() {
        assert_eq!(ChunkPos { x: 0, z: 0 }.region(), RegionPos { x: 0, z: 0 });
        assert_eq!(ChunkPos { x: 31, z: 31 }.region(), RegionPos { x: 0, z: 0 });
        assert_eq!(ChunkPos { x: 32, z: 0 }.region(), RegionPos { x: 1, z: 0 });
        assert_eq!(
            ChunkPos { x: -1, z: -1 }.region(),
            RegionPos { x: -1, z: -1 }
        );
        assert_eq!(ChunkPos { x: -1, z: -1 }.local_index(), 1023);
        assert_eq!(RegionPos { x: -1, z: 2 }.file_name(), "r.-1.2.mca");
    }

    #[test]
    fn creates_header_and_round_trips_location_and_timestamp() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-region-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
        assert_eq!(fs::metadata(region.path()).unwrap().len(), HEADER_BYTES);

        let chunk = ChunkPos { x: 3, z: 4 };
        assert_eq!(region.read_location(chunk).unwrap(), None);
        region
            .write_location(
                chunk,
                RegionLocation {
                    sector_offset: 2,
                    sector_count: 1,
                },
            )
            .unwrap();
        region.write_timestamp(chunk, 1234).unwrap();
        assert_eq!(
            region.read_location(chunk).unwrap(),
            Some(RegionLocation {
                sector_offset: 2,
                sector_count: 1,
            })
        );
        assert_eq!(region.read_timestamp(chunk).unwrap(), 1234);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn region_file_open_sanitizes_invalid_header_locations() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-region-invalid-header-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("r.0.0.mca");
        let file = fs::File::create(&path).unwrap();
        file.set_len(HEADER_BYTES + super::SECTOR_BYTES as u64)
            .unwrap();
        drop(file);

        write_raw_location(&path, ChunkPos { x: 0, z: 0 }, 1, 1);
        write_raw_location(&path, ChunkPos { x: 1, z: 0 }, 2, 0);
        write_raw_location(&path, ChunkPos { x: 2, z: 0 }, 99, 1);
        write_raw_location(&path, ChunkPos { x: 3, z: 0 }, 2, 1);

        let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();

        assert_eq!(region.read_location(ChunkPos { x: 0, z: 0 }).unwrap(), None);
        assert_eq!(region.read_location(ChunkPos { x: 1, z: 0 }).unwrap(), None);
        assert_eq!(region.read_location(ChunkPos { x: 2, z: 0 }).unwrap(), None);
        assert_eq!(
            region.read_location(ChunkPos { x: 3, z: 0 }).unwrap(),
            Some(RegionLocation {
                sector_offset: 2,
                sector_count: 1,
            })
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn region_io_worker_pending_writes_shadow_disk_and_coalesce() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-region-worker-pending-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);

        let chunk = ChunkPos { x: 2, z: 3 };
        let region = RegionFile::open(&dir, chunk.region()).unwrap();
        region
            .write_chunk_nbt(
                chunk,
                "",
                &Tag::Compound(vec![("old".to_string(), Tag::Int(1))]),
            )
            .unwrap();

        let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
        assert_eq!(
            worker.load_chunk_nbt(chunk).unwrap(),
            Some((
                "".to_string(),
                Tag::Compound(vec![("old".to_string(), Tag::Int(1))])
            ))
        );

        worker.store_chunk_nbt(
            chunk,
            "",
            Tag::Compound(vec![("pending".to_string(), Tag::Int(2))]),
        );
        worker.store_chunk_nbt(
            chunk,
            "",
            Tag::Compound(vec![("latest".to_string(), Tag::Int(3))]),
        );

        assert_eq!(worker.pending_write_count(), 1);
        assert_eq!(
            worker.load_chunk_nbt(chunk).unwrap(),
            Some((
                "".to_string(),
                Tag::Compound(vec![("latest".to_string(), Tag::Int(3))])
            ))
        );
        assert_eq!(
            RegionFile::open(&dir, chunk.region())
                .unwrap()
                .read_chunk_nbt(chunk)
                .unwrap(),
            Some((
                "".to_string(),
                Tag::Compound(vec![("old".to_string(), Tag::Int(1))])
            ))
        );

        worker.synchronize().unwrap();
        assert_eq!(worker.pending_write_count(), 0);
        assert_eq!(
            RegionFile::open(&dir, chunk.region())
                .unwrap()
                .read_chunk_nbt(chunk)
                .unwrap(),
            Some((
                "".to_string(),
                Tag::Compound(vec![("latest".to_string(), Tag::Int(3))])
            ))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn region_compression_versions_match_vanilla_ids_and_options() {
        assert_eq!(RegionCompression::DEFAULT, RegionCompression::Deflate);
        assert_eq!(RegionCompression::from_id(1), Some(RegionCompression::Gzip));
        assert_eq!(
            RegionCompression::from_id(2),
            Some(RegionCompression::Deflate)
        );
        assert_eq!(RegionCompression::from_id(3), Some(RegionCompression::None));
        assert_eq!(RegionCompression::from_id(4), Some(RegionCompression::Lz4));
        assert_eq!(
            RegionCompression::from_id(127),
            Some(RegionCompression::Custom)
        );
        assert_eq!(RegionCompression::from_id(99), None);
        assert_eq!(RegionCompression::Deflate.option_name(), Some("deflate"));
        assert_eq!(RegionCompression::None.option_name(), Some("none"));
        assert_eq!(RegionCompression::Lz4.option_name(), Some("lz4"));
        assert_eq!(RegionCompression::Gzip.option_name(), None);
        assert_eq!(
            RegionCompression::from_option_name("lz4"),
            Some(RegionCompression::Lz4)
        );
        assert!(RegionCompression::is_valid_id(4));
        assert!(!RegionCompression::is_valid_id(5));
    }

    #[test]
    fn region_file_round_trips_zlib_none_and_lz4_payloads() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-region-compression-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);

        let tag = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        )]);

        for (index, compression) in [
            RegionCompression::Deflate,
            RegionCompression::None,
            RegionCompression::Lz4,
        ]
        .into_iter()
        .enumerate()
        {
            let region = RegionFile::open(
                &dir,
                RegionPos {
                    x: index as i32,
                    z: 0,
                },
            )
            .unwrap();
            let chunk = ChunkPos {
                x: index as i32 * 32,
                z: 0,
            };
            region
                .write_chunk_nbt_with_compression(chunk, "Data", &tag, compression)
                .unwrap();
            assert_eq!(
                region.read_chunk_nbt(chunk).unwrap(),
                Some(("Data".to_string(), tag.clone()))
            );

            let location = region.read_location(chunk).unwrap().unwrap();
            let bytes = fs::read(region.path()).unwrap();
            let offset = location.sector_offset as usize * super::SECTOR_BYTES as usize;
            assert_eq!(bytes[offset + 4], compression.id());
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn region_file_treats_corrupt_stream_headers_as_missing_chunks() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-region-corrupt-streams-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);

        let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
        let invalid_version = ChunkPos { x: 0, z: 0 };
        region
            .write_location(
                invalid_version,
                RegionLocation {
                    sector_offset: 2,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(region.path(), 2, 1, 99);
        assert_eq!(region.read_chunk_nbt(invalid_version).unwrap(), None);

        let oversized_stream = ChunkPos { x: 1, z: 0 };
        region
            .write_location(
                oversized_stream,
                RegionLocation {
                    sector_offset: 3,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(
            region.path(),
            3,
            super::SECTOR_BYTES,
            RegionCompression::Deflate.id(),
        );
        assert_eq!(region.read_chunk_nbt(oversized_stream).unwrap(), None);

        let truncated_header = ChunkPos { x: 2, z: 0 };
        region
            .write_location(
                truncated_header,
                RegionLocation {
                    sector_offset: 4,
                    sector_count: 1,
                },
            )
            .unwrap();
        assert_eq!(region.read_chunk_nbt(truncated_header).unwrap(), None);

        let _ = fs::remove_dir_all(&dir);
    }

    fn write_raw_chunk_header(path: &std::path::Path, sector: u32, length: u32, version: u8) {
        let mut file = fs::OpenOptions::new().write(true).open(path).unwrap();
        file.seek(SeekFrom::Start(sector as u64 * super::SECTOR_BYTES as u64))
            .unwrap();
        file.write_all(&length.to_be_bytes()).unwrap();
        file.write_all(&[version]).unwrap();
    }

    fn write_raw_location(path: &std::path::Path, chunk: ChunkPos, sector: u32, count: u8) {
        let mut file = fs::OpenOptions::new().write(true).open(path).unwrap();
        file.seek(SeekFrom::Start((chunk.local_index() * 4) as u64))
            .unwrap();
        let bytes = sector.to_be_bytes();
        file.write_all(&[bytes[1], bytes[2], bytes[3], count])
            .unwrap();
    }

    #[test]
    fn region_file_reads_external_chunk_streams() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-region-external-stream-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);

        let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
        let chunk = ChunkPos { x: 0, z: 0 };
        let tag = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        )]);
        let external =
            super::encode_region_payload("External", &tag, RegionCompression::Deflate).unwrap();

        region
            .write_location(
                chunk,
                RegionLocation {
                    sector_offset: 2,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(region.path(), 2, 1, RegionCompression::Deflate.id() | 0x80);
        fs::write(dir.join("c.0.0.mcc"), external).unwrap();

        assert_eq!(
            region.read_chunk_nbt(chunk).unwrap(),
            Some(("External".to_string(), tag))
        );

        let missing_external = ChunkPos { x: 1, z: 0 };
        region
            .write_location(
                missing_external,
                RegionLocation {
                    sector_offset: 3,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(region.path(), 3, 1, RegionCompression::Deflate.id() | 0x80);
        assert_eq!(region.read_chunk_nbt(missing_external).unwrap(), None);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn region_file_chunk_existence_matches_vanilla_stream_header_checks() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-region-existence-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
        let tag = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        )]);
        let valid = ChunkPos { x: 0, z: 0 };
        region.write_chunk_nbt(valid, "Valid", &tag).unwrap();
        assert!(region.does_chunk_exist(valid));

        let invalid_version = ChunkPos { x: 1, z: 0 };
        region
            .write_location(
                invalid_version,
                RegionLocation {
                    sector_offset: 3,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(region.path(), 3, 1, 99);
        assert!(!region.does_chunk_exist(invalid_version));

        let zero_length = ChunkPos { x: 2, z: 0 };
        region
            .write_location(
                zero_length,
                RegionLocation {
                    sector_offset: 4,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(region.path(), 4, 0, RegionCompression::Deflate.id());
        assert!(!region.does_chunk_exist(zero_length));

        let external = ChunkPos { x: 3, z: 0 };
        region
            .write_location(
                external,
                RegionLocation {
                    sector_offset: 5,
                    sector_count: 1,
                },
            )
            .unwrap();
        write_raw_chunk_header(region.path(), 5, 1, RegionCompression::Deflate.id() | 0x80);
        assert!(!region.does_chunk_exist(external));
        fs::write(dir.join("c.3.0.mcc"), [1, 2, 3]).unwrap();
        assert!(region.does_chunk_exist(external));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn region_file_writes_oversized_chunks_to_external_streams() {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "rustcraft-region-external-write-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);

        let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
        let chunk = ChunkPos { x: 0, z: 0 };
        let large_tag = crate::storage::nbt::Tag::Compound(vec![(
            "payload".to_string(),
            crate::storage::nbt::Tag::ByteArray(vec![7; 256 * super::SECTOR_BYTES as usize]),
        )]);

        region
            .write_chunk_nbt_with_compression(chunk, "Large", &large_tag, RegionCompression::None)
            .unwrap();

        let external_path = dir.join("c.0.0.mcc");
        assert!(external_path.is_file());
        let location = region.read_location(chunk).unwrap().unwrap();
        assert_eq!(location.sector_count, 1);
        assert_eq!(
            region.read_chunk_nbt(chunk).unwrap(),
            Some(("Large".to_string(), large_tag))
        );
        let bytes = fs::read(region.path()).unwrap();
        let offset = location.sector_offset as usize * super::SECTOR_BYTES as usize;
        assert_eq!(
            u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap()),
            1
        );
        assert_eq!(bytes[offset + 4], RegionCompression::None.id() | 0x80);

        let small_tag = crate::storage::nbt::Tag::Compound(vec![(
            "DataVersion".to_string(),
            crate::storage::nbt::Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        )]);
        region
            .write_chunk_nbt_with_compression(chunk, "Small", &small_tag, RegionCompression::None)
            .unwrap();

        assert!(!external_path.exists());
        assert_eq!(
            region.read_chunk_nbt(chunk).unwrap(),
            Some(("Small".to_string(), small_tag))
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
