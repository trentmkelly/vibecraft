//! `DataFixers.addFixers`, data versions 1450 to 1510: the flattening era.

use crate::datafix::dynamic::{ensure_namespaced, get_str};
use crate::datafix::fixer::DataFixerBuilder;
use crate::datafix::fixes::{
    add_new_choices::add_new_choices, block_entity_banner_color_fix, block_entity_block_state_fix,
    block_entity_custom_name_to_component_fix, block_entity_jukebox_fix, block_name_flattening_fix,
    block_rename_fix::block_rename_fix, block_state_structure_template_fix,
    chunk_paletted_storage_fix, chunk_to_protochunk_fix, colorless_shulker_entity_fix,
    entity_block_state_fix, entity_cod_salmon_fix, entity_custom_name_to_component_fix,
    entity_item_frame_direction_fix, entity_painting_motive_fix, entity_pufferfish_rename_fix,
    entity_wolf_color_fix, heightmap_renaming_fix, igloo_metadata_removal_fix,
    item_custom_name_to_component_fix, item_rename_fix::item_rename_fix, item_stack_map_id_fix,
    item_stack_spawn_egg_fix, item_stack_the_flattening_fix, level_flat_generator_info_fix,
    named_entity_write_read_fix::named_entity_write_read_fix, remove_block_entity_tag_fix,
    renamed_coral_fix, stats_counter_fix, villager_trade_fix,
};
use crate::datafix::legacy_component_data_fix_utils::create_text_component_json;
use crate::datafix::references as r;
use crate::datafix::schemas::{
    v1451, v1451_1, v1451_2, v1451_3, v1451_4, v1451_5, v1451_6, v1458, v1460, v1466, v1470, v1481,
    v1483, v1486, v1488,
};
use crate::datafix::typed::typed_mut;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::{create_renamer, create_renamer_map};

/// `RemoveBlockEntityTagFix` argument of the 1451.5 schema.
const BLOCK_ENTITIES_WITHOUT_ITEM_TAG: &[&str] = &["minecraft:noteblock", "minecraft:flower_pot"];

const SEAGRASS: &[(&str, &str)] = &[
    ("minecraft:sea_grass", "minecraft:seagrass"),
    ("minecraft:tall_sea_grass", "minecraft:tall_seagrass"),
];

const PRISMARINE: &[(&str, &str)] = &[
    (
        "minecraft:prismarine_bricks_slab",
        "minecraft:prismarine_brick_slab",
    ),
    (
        "minecraft:prismarine_bricks_stairs",
        "minecraft:prismarine_brick_stairs",
    ),
];

/// `NamespacedSchema.ensureNamespaced(id).equals(expected)`.
fn is_namespaced(id: &str, expected: &str) -> bool {
    ensure_namespaced(id) == expected
}

/// Registers the schemas and fixes for data versions 1450 to 1488.
#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
pub fn add_fixers(b: &mut DataFixerBuilder) {
    let v1450 = b.add_same_schema(1450, 0);
    b.add_fixer(v1450, false, block_state_structure_template_fix::fix());
    let v1451 = b.add_schema(1451, 0, v1451::build);
    b.add_fixer(
        v1451,
        true,
        add_new_choices("AddTrappedChestFix", r::BLOCK_ENTITY),
    );
    let v1451_1 = b.add_schema(1451, 1, v1451_1::build);
    b.add_fixer(v1451_1, true, chunk_paletted_storage_fix::fix());
    let v1451_2 = b.add_schema(1451, 2, v1451_2::build);
    b.add_fixer(v1451_2, true, block_entity_block_state_fix::fix());
    let v1451_3 = b.add_schema(1451, 3, v1451_3::build);
    b.add_fixer(v1451_3, true, entity_block_state_fix::fix());
    b.add_fixer(v1451_3, false, item_stack_map_id_fix::fix());
    let v1451_4 = b.add_schema(1451, 4, v1451_4::build);
    b.add_fixer(v1451_4, true, block_name_flattening_fix::fix());
    b.add_fixer(v1451_4, false, item_stack_the_flattening_fix::fix());
    let v1451_5 = b.add_schema(1451, 5, v1451_5::build);
    b.add_fixer(
        v1451_5,
        true,
        remove_block_entity_tag_fix::fix(BLOCK_ENTITIES_WITHOUT_ITEM_TAG),
    );
    b.add_fixer(
        v1451_5,
        false,
        item_stack_spawn_egg_fix::fix("minecraft:spawn_egg"),
    );
    b.add_fixer(v1451_5, false, entity_wolf_color_fix::fix());
    b.add_fixer(v1451_5, false, block_entity_banner_color_fix::fix());
    b.add_fixer(v1451_5, false, level_flat_generator_info_fix::fix());
    let v1451_6 = b.add_schema(1451, 6, v1451_6::build);
    b.add_fixer(v1451_6, true, stats_counter_fix::fix());
    b.add_fixer(v1451_6, false, block_entity_jukebox_fix::fix());
    let v1451_7 = b.add_same_schema(1451, 7);
    b.add_fixer(v1451_7, false, villager_trade_fix::fix());
    let v1456 = b.add_same_schema(1456, 0);
    b.add_fixer(v1456, false, entity_item_frame_direction_fix::fix());
    let v1458 = b.add_schema(1458, 0, v1458::build);
    b.add_fixer(v1458, true, entity_custom_name_to_component_fix::fix());
    b.add_fixer(v1458, false, item_custom_name_to_component_fix::fix());
    b.add_fixer(
        v1458,
        true,
        block_entity_custom_name_to_component_fix::fix(),
    );
    let v1460 = b.add_schema(1460, 0, v1460::build);
    b.add_fixer(v1460, false, entity_painting_motive_fix::fix());
    let v1466 = b.add_schema(1466, 0, v1466::build);
    b.add_fixer(
        v1466,
        true,
        add_new_choices("Add DUMMY block entity", r::BLOCK_ENTITY),
    );
    b.add_fixer(v1466, true, chunk_to_protochunk_fix::fix());
    let v1470 = b.add_schema(1470, 0, v1470::build);
    b.add_fixer(
        v1470,
        true,
        add_new_choices("Add 1.13 entities fix", r::ENTITY),
    );
    let v1474 = b.add_same_schema(1474, 0);
    b.add_fixer(v1474, false, colorless_shulker_entity_fix::fix());
    b.add_fixer(
        v1474,
        false,
        block_rename_fix("Colorless shulker block fixer", |block| {
            if is_namespaced(block, "minecraft:purple_shulker_box") {
                "minecraft:shulker_box".to_string()
            } else {
                block.to_string()
            }
        }),
    );
    b.add_fixer(
        v1474,
        false,
        item_rename_fix("Colorless shulker item fixer", |item| {
            if is_namespaced(item, "minecraft:purple_shulker_box") {
                "minecraft:shulker_box".to_string()
            } else {
                item.to_string()
            }
        }),
    );
    let v1475 = b.add_same_schema(1475, 0);
    b.add_fixer(
        v1475,
        false,
        block_rename_fix(
            "Flowing fixer",
            create_renamer_map(&[
                ("minecraft:flowing_water", "minecraft:water"),
                ("minecraft:flowing_lava", "minecraft:lava"),
            ]),
        ),
    );
    let v1480 = b.add_same_schema(1480, 0);
    b.add_fixer(
        v1480,
        false,
        block_rename_fix(
            "Rename coral blocks",
            create_renamer_map(renamed_coral_fix::RENAMED_IDS),
        ),
    );
    b.add_fixer(
        v1480,
        false,
        item_rename_fix(
            "Rename coral items",
            create_renamer_map(renamed_coral_fix::RENAMED_IDS),
        ),
    );
    let v1481 = b.add_schema(1481, 0, v1481::build);
    b.add_fixer(v1481, true, add_new_choices("Add conduit", r::BLOCK_ENTITY));
    let v1483 = b.add_schema(1483, 0, v1483::build);
    b.add_fixer(v1483, true, entity_pufferfish_rename_fix::fix());
    b.add_fixer(
        v1483,
        false,
        item_rename_fix(
            "Rename pufferfish egg item",
            create_renamer_map(entity_pufferfish_rename_fix::RENAMED_IDS),
        ),
    );
    let v1484 = b.add_same_schema(1484, 0);
    b.add_fixer(
        v1484,
        false,
        item_rename_fix("Rename seagrass items", create_renamer_map(SEAGRASS)),
    );
    b.add_fixer(
        v1484,
        false,
        block_rename_fix("Rename seagrass blocks", create_renamer_map(SEAGRASS)),
    );
    b.add_fixer(v1484, false, heightmap_renaming_fix::fix());
    let v1486 = b.add_schema(1486, 0, v1486::build);
    b.add_fixer(v1486, true, entity_cod_salmon_fix::fix());
    b.add_fixer(
        v1486,
        false,
        item_rename_fix(
            "Rename cod/salmon egg items",
            create_renamer_map(entity_cod_salmon_fix::RENAMED_EGG_IDS),
        ),
    );
    let v1487 = b.add_same_schema(1487, 0);
    b.add_fixer(
        v1487,
        false,
        item_rename_fix(
            "Rename prismarine_brick(s)_* blocks",
            create_renamer_map(PRISMARINE),
        ),
    );
    b.add_fixer(
        v1487,
        false,
        block_rename_fix(
            "Rename prismarine_brick(s)_* items",
            create_renamer_map(PRISMARINE),
        ),
    );
    let v1488 = b.add_schema(1488, 0, v1488::build);
    b.add_fixer(
        v1488,
        false,
        block_rename_fix(
            "Rename kelp/kelptop",
            create_renamer_map(&[
                ("minecraft:kelp_top", "minecraft:kelp"),
                ("minecraft:kelp", "minecraft:kelp_plant"),
            ]),
        ),
    );
    b.add_fixer(
        v1488,
        false,
        item_rename_fix(
            "Rename kelptop",
            create_renamer("minecraft:kelp_top", "minecraft:kelp"),
        ),
    );
    b.add_fixer(
        v1488,
        true,
        named_entity_write_read_fix(
            "Command block block entity custom name fix",
            r::BLOCK_ENTITY,
            "minecraft:command_block",
            block_entity_custom_name_to_component_fix::fix_tag_custom_name,
        ),
    );
    b.add_fixer(v1488, false, command_block_minecart_custom_name_fix());
    b.add_fixer(v1488, false, igloo_metadata_removal_fix::fix());
}

/// The anonymous `DataFix` "Command block minecart custom name fix" of
/// `DataFixers`: the custom name of command block minecarts becomes a JSON text
/// component.
fn command_block_minecart_custom_name_fix() -> crate::datafix::fix::Fix {
    crate::datafix::fix::Fix::everywhere(
        "Command block minecart custom name fix",
        Target::Type(r::ENTITY),
        |entity| {
            if get_str(entity, "id") != Some("minecraft:commandblock_minecart") {
                return;
            }
            if let Some(Tag::String(name)) = typed_mut(entity, "CustomName") {
                *name = create_text_component_json(name);
            }
        },
    )
}
