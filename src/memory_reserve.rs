//! Emergency memory reserve matching Minecraft's `MemoryReserve`.

#![allow(dead_code)]

use std::sync::{Mutex, OnceLock};

const RESERVE_SIZE: usize = 10_485_760;

fn reserve_slot() -> &'static Mutex<Option<Vec<u8>>> {
    static RESERVE: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();
    RESERVE.get_or_init(|| Mutex::new(None))
}

fn with_reserve<R>(operation: impl FnOnce(&mut Option<Vec<u8>>) -> R) -> R {
    match reserve_slot().lock() {
        Ok(mut reserve) => operation(&mut reserve),
        Err(poisoned) => operation(&mut poisoned.into_inner()),
    }
}

pub fn allocate() {
    with_reserve(|reserve| *reserve = Some(vec![0; RESERVE_SIZE]));
}

pub fn release() {
    with_reserve(|reserve| {
        reserve.take();
    });
}

#[cfg(test)]
fn is_allocated() -> bool {
    with_reserve(|reserve| reserve.is_some())
}

#[cfg(test)]
mod tests {
    use super::{allocate, is_allocated, release};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn memory_reserve_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/MemoryReserve.java");
        assert_eq!(JAVA.lines().count(), 24);
        for fragment in [
            "private static byte @Nullable [] reserve",
            "reserve = new byte[10485760]",
            "if (reserve != null)",
            "reserve = null",
            "System.gc()",
        ] {
            assert!(JAVA.contains(fragment), "missing MemoryReserve source fragment: {fragment}");
        }
    }

    #[test]
    fn memory_reserve_allocates_and_releases_idempotently() {
        release();
        assert!(!is_allocated());
        allocate();
        assert!(is_allocated());
        allocate();
        assert!(is_allocated());
        release();
        assert!(!is_allocated());
        release();
        assert!(!is_allocated());
    }
}
