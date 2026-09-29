use super::super::*;
use super::*;

#[test]
pub fn visible_spawn_terrain_uses_deterministic_rolling_grass_layers() {
    let words = visible_spawn_palette_words();
    let columns = visible_spawn_test_columns();

    assert!(
        columns.max_height > 95,
        "spawn terrain should be visibly non-flat"
    );
    assert!(visible_spawn_terrain_block_count(0, 0, 8) > 4000);
    assert!(visible_spawn_terrain_block_count(0, 0, 9) > 512);
    assert!(visible_spawn_terrain_block_count(0, 0, 10) > 0);
    assert_ne!(words.iter().filter(|word| **word != 0).count(), 0);
    assert_high_visible_spawn_column(&words, columns.high_column);
    assert_outcrop_visible_spawn_column(&words, columns.outcrop_column);
    assert_featured_visible_spawn_column(&words, columns.featured_column);
    assert!(
        visible_spawn_terrain_height(0, 0, columns.ridge_column.0, columns.ridge_column.1) >= 96
    );
}

fn visible_spawn_palette_words() -> Vec<u64> {
    let mut payload = Vec::new();
    write_visible_spawn_terrain_block_state_container(&mut payload, 0, 0, 9).unwrap();
    let mut input = Cursor::new(payload);

    let mut bits = [0_u8; 1];
    input.read_exact(&mut bits).unwrap();
    assert_eq!(bits[0], 4);
    assert_eq!(read_var_i32(&mut input).unwrap(), 11);
    assert_eq!(read_var_i32(&mut input).unwrap(), 0);
    assert_eq!(read_var_i32(&mut input).unwrap(), STONE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), GRANITE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), DIORITE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), ANDESITE_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), BEDROCK_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), DIRT_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), GRASS_BLOCK_STATE_ID);
    assert_eq!(
        read_var_i32(&mut input).unwrap(),
        SHORT_GRASS_BLOCK_STATE_ID
    );
    assert_eq!(read_var_i32(&mut input).unwrap(), DANDELION_BLOCK_STATE_ID);
    assert_eq!(read_var_i32(&mut input).unwrap(), POPPY_BLOCK_STATE_ID);

    let mut raw = Vec::new();
    input.read_to_end(&mut raw).unwrap();
    assert_eq!(raw.len(), 2048);
    let words = raw
        .chunks_exact(8)
        .map(|chunk| {
            u64::from_be_bytes([
                chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6], chunk[7],
            ])
        })
        .collect::<Vec<_>>();
    words
}

struct VisibleSpawnTestColumns {
    high_column: (usize, usize),
    featured_column: (usize, usize, i32),
    outcrop_column: (usize, usize),
    ridge_column: (usize, usize),
    max_height: i32,
}

fn visible_spawn_test_columns() -> VisibleSpawnTestColumns {
    let high_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find(|(x, z)| {
            (90..96).contains(&visible_spawn_terrain_height(0, 0, *x, *z))
                && visible_spawn_surface_top_block_id(0, 0, *x, *z) == GRASS_BLOCK_STATE_ID
        })
        .expect("spawn chunk should contain a high visible hill top");
    let featured_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find_map(|(x, z)| {
            (visible_spawn_surface_top_block_id(0, 0, x, z) == GRASS_BLOCK_STATE_ID
                && (80..95).contains(&visible_spawn_terrain_height(0, 0, x, z)))
            .then(|| visible_spawn_surface_feature_id(0, 0, x, z).map(|id| (x, z, id)))
            .flatten()
        })
        .expect("spawn chunk should contain visible surface vegetation");
    let outcrop_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find(|(x, z)| {
            visible_spawn_surface_top_block_id(0, 0, *x, *z) != GRASS_BLOCK_STATE_ID
                && (80..96).contains(&visible_spawn_terrain_height(0, 0, *x, *z))
        })
        .expect("spawn chunk should contain a visible non-grass outcrop");
    let max_height = (0..16)
        .flat_map(|z| (0..16).map(move |x| visible_spawn_terrain_height(0, 0, x, z)))
        .max()
        .expect("spawn chunk should contain terrain columns");
    let ridge_column = (0..16)
        .flat_map(|z| (0..16).map(move |x| (x, z)))
        .find(|(x, z)| visible_spawn_terrain_height(0, 0, *x, *z) == max_height)
        .expect("spawn chunk should contain a visible ridge");
    VisibleSpawnTestColumns {
        high_column,
        featured_column,
        outcrop_column,
        ridge_column,
        max_height,
    }
}

fn assert_high_visible_spawn_column(words: &[u64], high_column: (usize, usize)) {
    let high_local_y =
        (visible_spawn_terrain_height(0, 0, high_column.0, high_column.1) - 80) as usize;
    assert_eq!(
        palette_index_at(words, high_column.0, high_local_y - 1, high_column.1),
        6
    );
    assert_eq!(
        palette_index_at(words, high_column.0, high_local_y, high_column.1),
        7
    );
}

fn assert_outcrop_visible_spawn_column(words: &[u64], outcrop_column: (usize, usize)) {
    let expected_outcrop_palette =
        match visible_spawn_surface_top_block_id(0, 0, outcrop_column.0, outcrop_column.1) {
            STONE_BLOCK_STATE_ID => 1,
            GRANITE_BLOCK_STATE_ID => 2,
            DIORITE_BLOCK_STATE_ID => 3,
            ANDESITE_BLOCK_STATE_ID => 4,
            DIRT_BLOCK_STATE_ID => 6,
            _ => unreachable!("outcrop column must be non-grass"),
        };
    assert_eq!(
        palette_index_at(
            words,
            outcrop_column.0,
            (visible_spawn_terrain_height(0, 0, outcrop_column.0, outcrop_column.1) - 80) as usize,
            outcrop_column.1
        ),
        expected_outcrop_palette
    );
}

fn assert_featured_visible_spawn_column(words: &[u64], featured_column: (usize, usize, i32)) {
    let feature_y = (visible_spawn_terrain_height(0, 0, featured_column.0, featured_column.1) + 1
        - 80) as usize;
    let expected_feature_palette = match featured_column.2 {
        SHORT_GRASS_BLOCK_STATE_ID => 8,
        DANDELION_BLOCK_STATE_ID => 9,
        POPPY_BLOCK_STATE_ID => 10,
        _ => unreachable!("feature id must be in the emitted palette"),
    };
    assert_eq!(
        palette_index_at(words, featured_column.0, feature_y, featured_column.1),
        expected_feature_palette
    );
}

#[test]
pub fn all_air_persisted_chunks_are_not_reused_for_spawn_terrain() {
    let empty = LevelChunk::empty(ChunkPos { x: 0, z: 0 });
    assert!(!chunk_has_non_air_blocks(&empty));

    let generated =
        crate::worldgen::generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "normal")
            .expect("normal preset should generate visible terrain");
    assert!(chunk_has_non_air_blocks(&generated));
}

