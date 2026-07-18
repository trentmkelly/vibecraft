//! Synthetic zero-sized filesystem metadata matching Minecraft's dummy attributes.

#![allow(dead_code)]

use std::time::SystemTime;

/// Java `DummyFileAttributes.DIRECTORY` and `.FILE` implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DummyFileAttributes {
    Directory,
    File,
}

impl DummyFileAttributes {
    pub const DIRECTORY: Self = Self::Directory;
    pub const FILE: Self = Self::File;

    pub const fn is_regular_file(self) -> bool {
        matches!(self, Self::File)
    }

    pub const fn is_directory(self) -> bool {
        matches!(self, Self::Directory)
    }

    pub const fn is_symbolic_link(self) -> bool {
        false
    }

    pub const fn is_other(self) -> bool {
        false
    }

    pub const fn size(self) -> u64 {
        0
    }

    pub const fn last_modified_time(self) -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    pub const fn last_access_time(self) -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    pub const fn creation_time(self) -> SystemTime {
        SystemTime::UNIX_EPOCH
    }

    /// Java returns `null` for the untyped file key.
    pub const fn file_key(self) -> Option<()> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::DummyFileAttributes;
    use std::time::SystemTime;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn dummy_file_attributes_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/DummyFileAttributes.java");
        assert_eq!(JAVA.lines().count(), 66);
        for fragment in [
            "public abstract class DummyFileAttributes implements BasicFileAttributes",
            "public static final BasicFileAttributes DIRECTORY",
            "public static final BasicFileAttributes FILE",
            "private static final FileTime EPOCH = FileTime.fromMillis(0L)",
            "public boolean isSymbolicLink()",
            "public long size()",
            "public @Nullable Object fileKey()",
        ] {
            assert!(
                JAVA.contains(fragment),
                "missing DummyFileAttributes source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn directory_and_file_variants_have_java_dummy_metadata() {
        let directory = DummyFileAttributes::DIRECTORY;
        assert!(directory.is_directory());
        assert!(!directory.is_regular_file());
        let file = DummyFileAttributes::FILE;
        assert!(file.is_regular_file());
        assert!(!file.is_directory());
        for attributes in [directory, file] {
            assert!(!attributes.is_symbolic_link());
            assert!(!attributes.is_other());
            assert_eq!(attributes.size(), 0);
            assert_eq!(attributes.last_modified_time(), SystemTime::UNIX_EPOCH);
            assert_eq!(attributes.last_access_time(), SystemTime::UNIX_EPOCH);
            assert_eq!(attributes.creation_time(), SystemTime::UNIX_EPOCH);
            assert_eq!(attributes.file_key(), None);
        }
    }
}
