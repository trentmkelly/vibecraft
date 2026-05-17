#![allow(dead_code)]

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

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
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
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
}

#[cfg(test)]
mod tests {
    use super::{ChunkPos, RegionCompression, RegionFile, RegionLocation, RegionPos, HEADER_BYTES};
    use std::fs;

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
}
