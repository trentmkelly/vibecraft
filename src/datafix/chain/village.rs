//! `DataFixers.addFixers`, versions 1800 through 1905: new village mobs,
//! dye/sign/slab names, item lore components, cats and chunk status.

use crate::datafix::fixer::DataFixerBuilder;
use crate::datafix::fixes::{
    add_new_choices::add_new_choices, block_rename_fix::block_rename_fix, chunk_status_fix,
    dye_item_rename_fix, entity_cat_split_fix, item_lore_fix, item_rename_fix::item_rename_fix,
};
use crate::datafix::references as r;
use crate::datafix::schemas::{v1800, v1801, v1904};

use super::create_renamer_map;

const SIGN_AND_SLAB_BLOCKS: &[(&str, &str)] = &[
    ("minecraft:stone_slab", "minecraft:smooth_stone_slab"),
    ("minecraft:sign", "minecraft:oak_sign"),
    ("minecraft:wall_sign", "minecraft:oak_wall_sign"),
];

const SIGN_AND_SLAB_ITEMS: &[(&str, &str)] = &[
    ("minecraft:stone_slab", "minecraft:smooth_stone_slab"),
    ("minecraft:sign", "minecraft:oak_sign"),
];

/// Registers every Java schema and fix after 1624 through 1905, in Java order.
pub fn add_fixers(b: &mut DataFixerBuilder) {
    let v1800 = b.add_schema(1800, 0, v1800::build);
    b.add_fixer(
        v1800,
        true,
        add_new_choices("Added 1.14 mobs fix", r::ENTITY),
    );
    b.add_fixer(v1800, false, dye_item_rename_fix::fix());
    let v1801 = b.add_schema(1801, 0, v1801::build);
    b.add_fixer(
        v1801,
        true,
        add_new_choices("Added Illager Beast", r::ENTITY),
    );
    let v1802 = b.add_same_schema(1802, 0);
    b.add_fixer(
        v1802,
        false,
        block_rename_fix(
            "Rename sign blocks & stone slabs",
            create_renamer_map(SIGN_AND_SLAB_BLOCKS),
        ),
    );
    b.add_fixer(
        v1802,
        false,
        item_rename_fix(
            "Rename sign item & stone slabs",
            create_renamer_map(SIGN_AND_SLAB_ITEMS),
        ),
    );
    let v1803 = b.add_same_schema(1803, 0);
    b.add_fixer(v1803, false, item_lore_fix::fix());
    let v1904 = b.add_schema(1904, 0, v1904::build);
    b.add_fixer(v1904, true, add_new_choices("Added Cats", r::ENTITY));
    b.add_fixer(v1904, false, entity_cat_split_fix::fix());
    let v1905 = b.add_same_schema(1905, 0);
    b.add_fixer(v1905, false, chunk_status_fix::fix());
}
