//! Port of `net.minecraft.util.datafix.fixes.HeightmapRenamingFix`.

use crate::datafix::dynamic::{get_mut, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new HeightmapRenamingFix(schema, changesType)`: heightmaps get their modern names.
pub fn fix() -> Fix {
    Fix::everywhere("HeightmapRenamingFix", Target::Type(r::CHUNK), |chunk| {
        if let Some(level) = get_mut(chunk, "Level") {
            fix_level(level);
        }
    })
}

fn rename_heightmap(heightmaps: &mut Tag, old: &str, new_names: &[&str]) {
    if let Some(value) = remove(heightmaps, old) {
        for new in new_names {
            set(heightmaps, new, value.clone());
        }
    }
}

fn fix_level(level: &mut Tag) {
    let Some(heightmaps) = get_mut(level, "Heightmaps") else {
        return;
    };
    rename_heightmap(heightmaps, "LIQUID", &["WORLD_SURFACE_WG"]);
    rename_heightmap(heightmaps, "SOLID", &["OCEAN_FLOOR_WG", "OCEAN_FLOOR"]);
    rename_heightmap(heightmaps, "LIGHT", &["LIGHT_BLOCKING"]);
    rename_heightmap(
        heightmaps,
        "RAIN",
        &["MOTION_BLOCKING", "MOTION_BLOCKING_NO_LEAVES"],
    );
}
