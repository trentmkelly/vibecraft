#![allow(dead_code)]

use super::DEFAULT_MAX_NBT_DEPTH;

pub const DEFAULT_NBT_QUOTA: i64 = 2_097_152;
pub const UNCOMPRESSED_NBT_QUOTA: i64 = 104_857_600;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtAccounterError {
    NegativeSize(i64),
    QuotaExceeded { usage: i64, size: i64, quota: i64 },
    DepthExceeded { max_depth: usize },
    PopAtTopLevel,
}

impl std::fmt::Display for NbtAccounterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NbtAccounterError::NegativeSize(size) => {
                write!(f, "Tried to account NBT tag with negative size: {size}")
            }
            NbtAccounterError::QuotaExceeded { usage, size, quota } => write!(
                f,
                "Tried to read NBT tag that was too big; tried to allocate: {usage} + {size} bytes where max allowed: {quota}"
            ),
            NbtAccounterError::DepthExceeded { max_depth } => {
                write!(f, "Tried to read NBT tag with too high complexity, depth > {max_depth}")
            }
            NbtAccounterError::PopAtTopLevel => {
                f.write_str("NBT-Accounter tried to pop stack-depth at top-level")
            }
        }
    }
}

impl std::error::Error for NbtAccounterError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtAccounter {
    quota: i64,
    usage: i64,
    max_depth: usize,
    depth: usize,
}

impl NbtAccounter {
    pub fn new(quota: i64, max_depth: usize) -> Self {
        Self {
            quota,
            usage: 0,
            max_depth,
            depth: 0,
        }
    }

    pub fn create(quota: i64) -> Self {
        Self::new(quota, DEFAULT_MAX_NBT_DEPTH)
    }

    pub fn default_quota() -> Self {
        Self::new(DEFAULT_NBT_QUOTA, DEFAULT_MAX_NBT_DEPTH)
    }

    pub fn uncompressed_quota() -> Self {
        Self::new(UNCOMPRESSED_NBT_QUOTA, DEFAULT_MAX_NBT_DEPTH)
    }

    pub fn unlimited_heap() -> Self {
        Self::new(i64::MAX, DEFAULT_MAX_NBT_DEPTH)
    }

    pub fn account_entries(
        &mut self,
        bytes_per_entry: i64,
        count: i64,
    ) -> Result<(), NbtAccounterError> {
        self.account_bytes(bytes_per_entry.saturating_mul(count))
    }

    pub fn account_bytes(&mut self, size: i64) -> Result<(), NbtAccounterError> {
        if size < 0 {
            return Err(NbtAccounterError::NegativeSize(size));
        }
        if self.usage.saturating_add(size) > self.quota {
            return Err(NbtAccounterError::QuotaExceeded {
                usage: self.usage,
                size,
                quota: self.quota,
            });
        }
        self.usage += size;
        Ok(())
    }

    pub fn push_depth(&mut self) -> Result<(), NbtAccounterError> {
        if self.depth >= self.max_depth {
            return Err(NbtAccounterError::DepthExceeded {
                max_depth: self.max_depth,
            });
        }
        self.depth += 1;
        Ok(())
    }

    pub fn pop_depth(&mut self) -> Result<(), NbtAccounterError> {
        if self.depth == 0 {
            return Err(NbtAccounterError::PopAtTopLevel);
        }
        self.depth -= 1;
        Ok(())
    }

    pub fn usage(&self) -> i64 {
        self.usage
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}
