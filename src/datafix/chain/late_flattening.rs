//! `DataFixers.addFixers`, data versions 1490 to 1624: melons, structure
//! templates, enchantment names, leaves, "the renamening" and coral fans.

use crate::datafix::fixer::DataFixerBuilder;
use crate::datafix::fixes::{
    advancements_fix, block_entity_keep_packed, block_rename_fix::block_rename_fix,
    chunk_structures_template_rename_fix, entity_the_renamening_fix,
    item_rename_fix::item_rename_fix, item_stack_enchantment_names_fix, leaves_fix,
    level_data_generator_options_fix, namespaced_type_rename_fix::namespaced_type_rename_fix,
    objective_render_type_fix, recipes_fix, recipes_renamening_fix, renamed_coral_fans_fix,
    scoreboard_display_name_fix::scoreboard_display_name_fix, stats_rename_fix::stats_rename_fix,
    trapped_chest_block_entity_fix,
};
use crate::datafix::references as r;
use crate::datafix::schemas::v1510;

use super::{create_renamer, create_renamer_map};

const MELON_ITEMS: &[(&str, &str)] = &[
    ("minecraft:melon_block", "minecraft:melon"),
    ("minecraft:melon", "minecraft:melon_slice"),
    (
        "minecraft:speckled_melon",
        "minecraft:glistering_melon_slice",
    ),
];

const SWIM_STATS: &[(&str, &str)] = &[
    ("minecraft:swim_one_cm", "minecraft:walk_on_water_one_cm"),
    ("minecraft:dive_one_cm", "minecraft:walk_under_water_one_cm"),
];

/// `createRenamer(EntityTheRenameningFix.RENAMED_BLOCKS)`.
fn renamed_blocks(name: &str) -> String {
    create_renamer_map(entity_the_renamening_fix::RENAMED_BLOCKS)(name)
}

/// `createRenamer(EntityTheRenameningFix.RENAMED_ITEMS)`: the block renames plus
/// the item-only renames.
fn renamed_items(name: &str) -> String {
    entity_the_renamening_fix::renamed_items()
        .into_iter()
        .find(|(old, _)| *old == name)
        .map_or(name, |(_, new)| new)
        .to_string()
}

/// Registers the schemas and fixes for data versions 1490 to 1624.
pub fn add_fixers(b: &mut DataFixerBuilder) {
    let v1490 = b.add_same_schema(1490, 0);
    b.add_fixer(
        v1490,
        false,
        block_rename_fix(
            "Rename melon_block",
            create_renamer("minecraft:melon_block", "minecraft:melon"),
        ),
    );
    b.add_fixer(
        v1490,
        false,
        item_rename_fix(
            "Rename melon_block/melon/speckled_melon",
            create_renamer_map(MELON_ITEMS),
        ),
    );
    let v1492 = b.add_same_schema(1492, 0);
    b.add_fixer(v1492, false, chunk_structures_template_rename_fix::fix());
    // `fileFixerUpper.addSchema(fixerUpper, 1493, ..)` also adds the schema to the
    // main chain; `LegacyStructureFileFix` belongs to the file fixer.
    b.add_same_schema(1493, 0);
    let v1494 = b.add_same_schema(1494, 0);
    b.add_fixer(v1494, false, item_stack_enchantment_names_fix::fix());
    let v1496 = b.add_same_schema(1496, 0);
    b.add_fixer(v1496, false, leaves_fix::fix());
    let v1500 = b.add_same_schema(1500, 0);
    b.add_fixer(v1500, false, block_entity_keep_packed::fix());
    let v1501 = b.add_same_schema(1501, 0);
    b.add_fixer(v1501, false, advancements_fix::fix());
    let v1502 = b.add_same_schema(1502, 0);
    b.add_fixer(
        v1502,
        false,
        namespaced_type_rename_fix(
            "Recipes fix",
            r::RECIPE,
            create_renamer_map(recipes_fix::RECIPES),
        ),
    );
    let v1506 = b.add_same_schema(1506, 0);
    b.add_fixer(v1506, false, level_data_generator_options_fix::fix());
    let v1510 = b.add_schema(1510, 0, v1510::build);
    b.add_fixer(
        v1510,
        false,
        block_rename_fix("Block renamening fix", renamed_blocks),
    );
    b.add_fixer(
        v1510,
        false,
        item_rename_fix("Item renamening fix", renamed_items),
    );
    b.add_fixer(
        v1510,
        false,
        namespaced_type_rename_fix(
            "Recipes renamening fix",
            r::RECIPE,
            create_renamer_map(recipes_renamening_fix::RECIPES),
        ),
    );
    b.add_fixer(v1510, true, entity_the_renamening_fix::fix());
    b.add_fixer(
        v1510,
        false,
        stats_rename_fix("SwimStatsRenameFix", SWIM_STATS),
    );
    let v1514 = b.add_same_schema(1514, 0);
    b.add_fixer(
        v1514,
        false,
        scoreboard_display_name_fix("ObjectiveDisplayNameFix", r::OBJECTIVE),
    );
    b.add_fixer(
        v1514,
        false,
        scoreboard_display_name_fix("TeamDisplayNameFix", r::TEAM),
    );
    b.add_fixer(v1514, false, objective_render_type_fix::fix());
    let v1515 = b.add_same_schema(1515, 0);
    b.add_fixer(
        v1515,
        false,
        block_rename_fix(
            "Rename coral fan blocks",
            create_renamer_map(renamed_coral_fans_fix::RENAMED_IDS),
        ),
    );
    let v1624 = b.add_same_schema(1624, 0);
    b.add_fixer(v1624, false, trapped_chest_block_entity_fix::fix());
}
