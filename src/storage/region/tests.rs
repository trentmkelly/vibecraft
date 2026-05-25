use super::{
    chunk_tag_is_old_for_blending, ChunkPos, RegionCompression, RegionFile, RegionFileStorage,
    RegionIoWorker, RegionLocation, RegionPos, CHUNKS_PER_REGION_AXIS, HEADER_BYTES,
    OLD_CHUNK_DATA_VERSION_CUTOFF, OLD_CHUNK_REGION_CACHE_SIZE, REGION_FILE_STORAGE_CACHE_SIZE,
};
use crate::storage::nbt::Tag;
use std::fs::{self, OpenOptions};
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
    assert_eq!(
        RegionPos { x: -1, z: 2 }.min_chunk_pos(),
        ChunkPos { x: -32, z: 64 }
    );
    assert_eq!(
        RegionPos { x: -1, z: 2 }.max_chunk_pos(),
        ChunkPos { x: -1, z: 95 }
    );
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
fn sync_write_policy_propagates_to_region_storage_and_worker() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("rustcraft-region-sync-writes-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);

    let default_region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
    assert!(!default_region.sync_writes());

    let sync_region = RegionFile::open_with_sync(&dir, RegionPos { x: 1, z: 0 }, true).unwrap();
    assert!(sync_region.sync_writes());

    let storage = RegionFileStorage::open_with_sync(dir.clone(), true).unwrap();
    assert!(storage.sync_writes());
    assert!(storage
        .get_region_file(RegionPos { x: 2, z: 0 })
        .unwrap()
        .sync_writes());

    let chunk = ChunkPos { x: 96, z: 0 };
    let tag = Tag::Compound(vec![("synced".to_string(), Tag::Int(1))]);
    let mut worker = RegionIoWorker::open_with_sync(dir.clone(), true).unwrap();
    assert!(worker.sync_writes());
    worker.store_chunk_nbt(chunk, "", tag.clone());
    worker.synchronize().unwrap();

    assert_eq!(
        RegionFile::open(&dir, chunk.region())
            .unwrap()
            .read_chunk_nbt(chunk)
            .unwrap(),
        Some(("".to_string(), tag))
    );

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
fn region_io_worker_pending_clear_shadows_and_deletes_stored_chunk() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-clear-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let chunk = ChunkPos { x: 0, z: 0 };
    let region = RegionFile::open(&dir, chunk.region()).unwrap();
    region
        .write_chunk_nbt_with_compression(
            chunk,
            "",
            &Tag::Compound(vec![("stored".to_string(), Tag::Int(1))]),
            RegionCompression::None,
        )
        .unwrap();
    fs::write(dir.join("c.0.0.mcc"), [1, 2, 3]).unwrap();

    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    worker.clear_chunk_nbt(chunk);

    assert_eq!(worker.pending_write_count(), 1);
    assert_eq!(worker.load_chunk_nbt(chunk).unwrap(), None);
    assert!(RegionFile::open(&dir, chunk.region())
        .unwrap()
        .does_chunk_exist(chunk));
    assert!(dir.join("c.0.0.mcc").is_file());

    worker.synchronize().unwrap();

    let region = RegionFile::open(&dir, chunk.region()).unwrap();
    assert_eq!(worker.pending_write_count(), 0);
    assert_eq!(region.read_location(chunk).unwrap(), None);
    assert_eq!(region.read_chunk_nbt(chunk).unwrap(), None);
    assert!(!region.does_chunk_exist(chunk));
    assert!(!dir.join("c.0.0.mcc").exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_file_reuses_freed_sectors_for_new_chunk_writes() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-sector-reuse-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
    let first = ChunkPos { x: 0, z: 0 };
    let second = ChunkPos { x: 1, z: 0 };
    let third = ChunkPos { x: 2, z: 0 };
    let fourth = ChunkPos { x: 3, z: 0 };

    region
        .write_chunk_nbt(
            first,
            "",
            &Tag::Compound(vec![("first".to_string(), Tag::Int(1))]),
        )
        .unwrap();
    region
        .write_chunk_nbt(
            second,
            "",
            &Tag::Compound(vec![("second".to_string(), Tag::Int(2))]),
        )
        .unwrap();
    assert_eq!(
        region.read_location(first).unwrap().unwrap().sector_offset,
        2
    );
    assert_eq!(
        region.read_location(second).unwrap().unwrap().sector_offset,
        3
    );

    region.clear_chunk_nbt(first).unwrap();
    region
        .write_chunk_nbt(
            third,
            "",
            &Tag::Compound(vec![("third".to_string(), Tag::Int(3))]),
        )
        .unwrap();
    assert_eq!(
        region.read_location(third).unwrap().unwrap().sector_offset,
        2
    );

    region
        .write_chunk_nbt(
            second,
            "",
            &Tag::Compound(vec![("second-new".to_string(), Tag::Int(4))]),
        )
        .unwrap();
    assert_eq!(
        region.read_location(second).unwrap().unwrap().sector_offset,
        4
    );
    region
        .write_chunk_nbt(
            fourth,
            "",
            &Tag::Compound(vec![("fourth".to_string(), Tag::Int(5))]),
        )
        .unwrap();
    assert_eq!(
        region.read_location(fourth).unwrap().unwrap().sector_offset,
        3
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_file_close_pads_to_full_sector_and_forces_file() {
    let mut dir = std::env::temp_dir();
    dir.push(format!("rustcraft-region-close-pad-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);

    let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
    let mut file = OpenOptions::new().write(true).open(region.path()).unwrap();
    file.seek(SeekFrom::Start(HEADER_BYTES + 17)).unwrap();
    file.write_all(&[1]).unwrap();
    drop(file);
    assert_ne!(
        fs::metadata(region.path()).unwrap().len() % super::SECTOR_BYTES as u64,
        0
    );

    region.close().unwrap();

    assert_eq!(
        fs::metadata(region.path()).unwrap().len() % super::SECTOR_BYTES as u64,
        0
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_file_storage_caches_regions_with_lru_eviction_flush_and_close() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-file-storage-cache-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let storage = RegionFileStorage::open(dir.clone()).unwrap();
    let first = RegionPos { x: 0, z: 0 };
    storage.get_region_file(first).unwrap();
    storage.get_region_file(first).unwrap();
    assert_eq!(storage.cached_region_count(), 1);

    let first_path = dir.join(first.file_name());
    let mut file = OpenOptions::new().write(true).open(&first_path).unwrap();
    file.seek(SeekFrom::Start(HEADER_BYTES + 17)).unwrap();
    file.write_all(&[1]).unwrap();
    drop(file);
    assert_ne!(
        fs::metadata(&first_path).unwrap().len() % super::SECTOR_BYTES as u64,
        0
    );

    for x in 1..=(REGION_FILE_STORAGE_CACHE_SIZE as i32 + 1) {
        storage.get_region_file(RegionPos { x, z: 0 }).unwrap();
    }
    assert_eq!(
        storage.cached_region_count(),
        REGION_FILE_STORAGE_CACHE_SIZE
    );
    assert!(!storage.region_cache.borrow().contains_key(&first));
    assert_eq!(
        fs::metadata(&first_path).unwrap().len() % super::SECTOR_BYTES as u64,
        0
    );

    let chunk = ChunkPos { x: 32, z: 0 };
    storage
        .write_chunk_nbt(
            chunk,
            "",
            &Tag::Compound(vec![("cached".to_string(), Tag::Int(1))]),
        )
        .unwrap();
    storage.flush().unwrap();
    assert!(storage.region_cache.borrow().contains_key(&chunk.region()));
    storage.close().unwrap();
    assert_eq!(storage.cached_region_count(), 0);
    assert_eq!(
        RegionFile::open(&dir, chunk.region())
            .unwrap()
            .read_chunk_nbt(chunk)
            .unwrap(),
        Some((
            "".to_string(),
            Tag::Compound(vec![("cached".to_string(), Tag::Int(1))])
        ))
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_file_storage_close_attempts_all_cached_regions_after_error() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-file-storage-close-errors-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let storage = RegionFileStorage::open(dir.clone()).unwrap();
    let missing = RegionPos { x: 0, z: 0 };
    let partial = RegionPos { x: 1, z: 0 };
    storage.get_region_file(missing).unwrap();
    storage.get_region_file(partial).unwrap();

    let missing_path = dir.join(missing.file_name());
    let partial_path = dir.join(partial.file_name());
    fs::remove_file(&missing_path).unwrap();
    let mut file = OpenOptions::new().write(true).open(&partial_path).unwrap();
    file.seek(SeekFrom::Start(HEADER_BYTES + 17)).unwrap();
    file.write_all(&[1]).unwrap();
    drop(file);
    assert_ne!(
        fs::metadata(&partial_path).unwrap().len() % super::SECTOR_BYTES as u64,
        0
    );

    assert!(storage.close().is_err());

    assert_eq!(storage.cached_region_count(), 0);
    assert_eq!(
        fs::metadata(&partial_path).unwrap().len() % super::SECTOR_BYTES as u64,
        0
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_io_worker_scan_chunk_nbt_respects_pending_writes_and_deletes() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-scan-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let chunk = ChunkPos { x: 0, z: 0 };
    RegionFile::open(&dir, chunk.region())
        .unwrap()
        .write_chunk_nbt(
            chunk,
            "stored",
            &Tag::Compound(vec![("stored".to_string(), Tag::Int(1))]),
        )
        .unwrap();

    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    let mut scanned = Vec::new();
    worker
        .scan_chunk_nbt(chunk, |name, tag| {
            scanned.push((name.to_string(), tag.clone()));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        scanned,
        vec![(
            "stored".to_string(),
            Tag::Compound(vec![("stored".to_string(), Tag::Int(1))])
        )]
    );

    worker.store_chunk_nbt(
        chunk,
        "pending",
        Tag::Compound(vec![("pending".to_string(), Tag::Int(2))]),
    );
    scanned.clear();
    worker
        .scan_chunk_nbt(chunk, |name, tag| {
            scanned.push((name.to_string(), tag.clone()));
            Ok(())
        })
        .unwrap();
    assert_eq!(
        scanned,
        vec![(
            "pending".to_string(),
            Tag::Compound(vec![("pending".to_string(), Tag::Int(2))])
        )]
    );

    worker.clear_chunk_nbt(chunk);
    scanned.clear();
    worker
        .scan_chunk_nbt(chunk, |name, tag| {
            scanned.push((name.to_string(), tag.clone()));
            Ok(())
        })
        .unwrap();
    assert!(scanned.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_io_worker_synchronize_with_flush_drains_and_forces_regions() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-flush-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let first = ChunkPos { x: 0, z: 0 };
    let second = ChunkPos { x: 32, z: 0 };
    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    worker.store_chunk_nbt(
        first,
        "",
        Tag::Compound(vec![("first".to_string(), Tag::Int(1))]),
    );
    worker.store_chunk_nbt(
        second,
        "",
        Tag::Compound(vec![("second".to_string(), Tag::Int(2))]),
    );

    worker.synchronize_with_flush().unwrap();

    assert_eq!(worker.pending_write_count(), 0);
    assert_eq!(
        RegionFile::open(&dir, first.region())
            .unwrap()
            .read_chunk_nbt(first)
            .unwrap(),
        Some((
            "".to_string(),
            Tag::Compound(vec![("first".to_string(), Tag::Int(1))])
        ))
    );
    assert_eq!(
        RegionFile::open(&dir, second.region())
            .unwrap()
            .read_chunk_nbt(second)
            .unwrap(),
        Some((
            "".to_string(),
            Tag::Compound(vec![("second".to_string(), Tag::Int(2))])
        ))
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_io_worker_close_drains_pending_writes_and_clears_old_chunk_cache() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-close-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let chunk = ChunkPos { x: 0, z: 0 };
    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    worker.store_chunk_nbt(
        chunk,
        "",
        Tag::Compound(vec![("closed".to_string(), Tag::Int(1))]),
    );
    worker
        .old_chunk_mask_for_region(RegionPos { x: 0, z: 0 })
        .unwrap();
    assert_eq!(worker.pending_write_count(), 1);
    assert_eq!(worker.old_chunk_mask_cache.borrow().len(), 1);

    worker.close().unwrap();

    assert_eq!(worker.pending_write_count(), 0);
    assert!(worker.old_chunk_mask_cache.borrow().is_empty());
    assert!(worker.old_chunk_mask_lru.borrow().is_empty());
    assert_eq!(
        RegionFile::open(&dir, chunk.region())
            .unwrap()
            .read_chunk_nbt(chunk)
            .unwrap(),
        Some((
            "".to_string(),
            Tag::Compound(vec![("closed".to_string(), Tag::Int(1))])
        ))
    );

    worker.close().unwrap();
    assert_eq!(worker.pending_write_count(), 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn chunk_tag_old_for_blending_matches_ioworker_rule() {
    assert!(chunk_tag_is_old_for_blending(&Tag::Compound(vec![])));
    assert!(chunk_tag_is_old_for_blending(&Tag::Compound(vec![(
        "DataVersion".to_string(),
        Tag::Int(OLD_CHUNK_DATA_VERSION_CUTOFF - 1),
    )])));
    assert!(!chunk_tag_is_old_for_blending(&Tag::Compound(vec![(
        "DataVersion".to_string(),
        Tag::Int(OLD_CHUNK_DATA_VERSION_CUTOFF),
    )])));
    assert!(chunk_tag_is_old_for_blending(&Tag::Compound(vec![
        (
            "DataVersion".to_string(),
            Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        ("blending_data".to_string(), Tag::Compound(Vec::new())),
    ])));
    assert!(!chunk_tag_is_old_for_blending(&Tag::Compound(vec![
        (
            "DataVersion".to_string(),
            Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
        ),
        ("blending_data".to_string(), Tag::List(Vec::new())),
    ])));
    assert!(!chunk_tag_is_old_for_blending(&Tag::List(Vec::new())));
}

#[test]
fn region_io_worker_builds_old_chunk_mask_for_blender_scan() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-old-mask-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let old = ChunkPos { x: 0, z: 0 };
    let modern = ChunkPos { x: 1, z: 0 };
    let blending = ChunkPos { x: 31, z: 31 };
    let deleted_pending = ChunkPos { x: 2, z: 0 };
    let pending_old = ChunkPos { x: 3, z: 0 };
    let region = RegionFile::open(&dir, RegionPos { x: 0, z: 0 }).unwrap();
    region
        .write_chunk_nbt(
            old,
            "",
            &Tag::Compound(vec![(
                "DataVersion".to_string(),
                Tag::Int(OLD_CHUNK_DATA_VERSION_CUTOFF - 1),
            )]),
        )
        .unwrap();
    region
        .write_chunk_nbt(
            modern,
            "",
            &Tag::Compound(vec![(
                "DataVersion".to_string(),
                Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
            )]),
        )
        .unwrap();
    region
        .write_chunk_nbt(
            blending,
            "",
            &Tag::Compound(vec![
                (
                    "DataVersion".to_string(),
                    Tag::Int(crate::storage::datafix::TARGET_DATA_VERSION),
                ),
                ("blending_data".to_string(), Tag::Compound(Vec::new())),
            ]),
        )
        .unwrap();
    region
        .write_chunk_nbt(
            deleted_pending,
            "",
            &Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(1))]),
        )
        .unwrap();

    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    worker.clear_chunk_nbt(deleted_pending);
    worker.store_chunk_nbt(
        pending_old,
        "",
        Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(1))]),
    );

    let mask = worker
        .old_chunk_mask_for_region(RegionPos { x: 0, z: 0 })
        .unwrap();

    assert_eq!(mask.len(), 1024);
    assert!(mask[old.local_index()]);
    assert!(!mask[modern.local_index()]);
    assert!(mask[blending.local_index()]);
    assert!(!mask[deleted_pending.local_index()]);
    assert!(mask[pending_old.local_index()]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_io_worker_queries_old_chunks_around_across_regions() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-old-around-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let boundary_old = ChunkPos { x: 32, z: 0 };
    RegionFile::open(&dir, boundary_old.region())
        .unwrap()
        .write_chunk_nbt(
            boundary_old,
            "",
            &Tag::Compound(vec![(
                "DataVersion".to_string(),
                Tag::Int(OLD_CHUNK_DATA_VERSION_CUTOFF - 1),
            )]),
        )
        .unwrap();

    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    assert!(!worker
        .is_old_chunk_around(ChunkPos { x: 31, z: 0 }, 0)
        .unwrap());
    assert!(worker
        .is_old_chunk_around(ChunkPos { x: 31, z: 0 }, 1)
        .unwrap());

    worker.clear_chunk_nbt(boundary_old);
    assert!(!worker
        .is_old_chunk_around(ChunkPos { x: 31, z: 0 }, 1)
        .unwrap());

    let negative_old = ChunkPos { x: -1, z: -1 };
    worker.store_chunk_nbt(
        negative_old,
        "",
        Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(1))]),
    );
    assert!(worker
        .is_old_chunk_around(ChunkPos { x: 0, z: 0 }, 1)
        .unwrap());
    assert!(!worker
        .is_old_chunk_around(ChunkPos { x: 0, z: 0 }, 0)
        .unwrap());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn region_io_worker_caches_old_chunk_masks_with_lru_eviction_and_pending_invalidation() {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "rustcraft-region-worker-old-cache-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);

    let mut worker = RegionIoWorker::open(dir.clone()).unwrap();
    let region_pos = RegionPos { x: 0, z: 0 };
    let old_chunk = ChunkPos { x: 0, z: 0 };

    let first_mask = worker.old_chunk_mask_for_region(region_pos).unwrap();
    assert!(!first_mask[old_chunk.local_index()]);
    assert_eq!(worker.old_chunk_mask_cache.borrow().len(), 1);

    worker.store_chunk_nbt(
        old_chunk,
        "",
        Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(1))]),
    );
    assert!(worker.old_chunk_mask_for_region(region_pos).unwrap()[old_chunk.local_index()]);

    let empty_mask = vec![false; (CHUNKS_PER_REGION_AXIS * CHUNKS_PER_REGION_AXIS) as usize];
    for x in 1..=(OLD_CHUNK_REGION_CACHE_SIZE as i32 + 1) {
        worker.cache_old_chunk_mask(RegionPos { x, z: 0 }, empty_mask.clone());
    }
    assert_eq!(
        worker.old_chunk_mask_cache.borrow().len(),
        OLD_CHUNK_REGION_CACHE_SIZE
    );
    assert!(!worker
        .old_chunk_mask_cache
        .borrow()
        .contains_key(&region_pos));

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
