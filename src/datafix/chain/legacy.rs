//! `DataFixers.addFixers`, data versions 99 to 1446: the pre-flattening era.

use crate::datafix::dynamic::ensure_namespaced;
use crate::datafix::fixer::DataFixerBuilder;
use crate::datafix::fixes::{
    add_new_choices::add_new_choices, bed_item_color_fix, block_entity_id_fix,
    block_entity_shulker_box_color_fix, chunk_bed_block_entity_injecter_fix,
    entity_armor_stand_silent_fix, entity_elder_guardian_split_fix,
    entity_equipment_to_armor_and_hand_fix, entity_health_fix, entity_horse_saddle_fix,
    entity_horse_split_fix, entity_id_fix, entity_minecart_identifiers_fix,
    entity_painting_item_frame_direction_fix, entity_redundant_chance_tags_fix,
    entity_riding_to_passengers_fix, entity_shulker_color_fix, entity_skeleton_split_fix,
    entity_string_uuid_fix, entity_tipped_arrow_fix, entity_zombie_split_fix,
    entity_zombie_villager_type_fix, item_banner_color_fix, item_id_fix, item_potion_fix,
    item_rename_fix::item_rename_fix, item_shulker_box_color_fix, item_spawn_egg_fix,
    item_water_potion_fix, mob_spawner_entity_identifiers_fix, options_force_vbo_fix,
    options_key_lwjgl3_fix, options_key_translation_fix, options_lower_case_language_fix,
    sign_text_strict_json_fix, villager_set_can_pick_up_loot_fix,
    write_and_read_fix::write_and_read_fix, written_book_pages_strict_json_fix,
};
use crate::datafix::references as r;
use crate::datafix::schemas::{
    v100, v102, v1022, v106, v107, v1125, v135, v143, v501, v700, v701, v702, v703, v704, v705,
    v808, v99,
};

use super::create_renamer;

/// Registers the schemas and fixes for data versions 99 to 1446.
#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
pub fn add_fixers(b: &mut DataFixerBuilder) {
    b.add_schema(99, 0, v99::build);
    let v100 = b.add_schema(100, 0, v100::build);
    b.add_fixer(v100, true, entity_equipment_to_armor_and_hand_fix::fix());
    let v101 = b.add_same_schema(101, 0);
    b.add_fixer(v101, true, villager_set_can_pick_up_loot_fix::fix());
    let v102 = b.add_schema(102, 0, v102::build);
    b.add_fixer(v102, true, item_id_fix::fix());
    b.add_fixer(v102, false, item_potion_fix::fix());
    let v105 = b.add_same_schema(105, 0);
    b.add_fixer(v105, true, item_spawn_egg_fix::fix());
    let v106 = b.add_schema(106, 0, v106::build);
    b.add_fixer(v106, true, mob_spawner_entity_identifiers_fix::fix());
    let v107 = b.add_schema(107, 0, v107::build);
    b.add_fixer(v107, true, entity_minecart_identifiers_fix::fix());
    let v108 = b.add_same_schema(108, 0);
    b.add_fixer(v108, true, entity_string_uuid_fix::fix());
    let v109 = b.add_same_schema(109, 0);
    b.add_fixer(v109, true, entity_health_fix::fix());
    let v110 = b.add_same_schema(110, 0);
    b.add_fixer(v110, true, entity_horse_saddle_fix::fix());
    let v111 = b.add_same_schema(111, 0);
    b.add_fixer(v111, true, entity_painting_item_frame_direction_fix::fix());
    let v113 = b.add_same_schema(113, 0);
    b.add_fixer(v113, true, entity_redundant_chance_tags_fix::fix());
    let v135 = b.add_schema(135, 0, v135::build);
    b.add_fixer(v135, true, entity_riding_to_passengers_fix::fix());
    let v143 = b.add_schema(143, 0, v143::build);
    b.add_fixer(v143, true, entity_tipped_arrow_fix::fix());
    let v147 = b.add_same_schema(147, 0);
    b.add_fixer(v147, true, entity_armor_stand_silent_fix::fix());
    let v165 = b.add_same_schema(165, 0);
    b.add_fixer(v165, false, sign_text_strict_json_fix::fix());
    b.add_fixer(v165, false, written_book_pages_strict_json_fix::fix());
    let v501 = b.add_schema(501, 0, v501::build);
    b.add_fixer(
        v501,
        true,
        add_new_choices("Add 1.10 entities fix", r::ENTITY),
    );
    let v502 = b.add_same_schema(502, 0);
    b.add_fixer(
        v502,
        false,
        item_rename_fix("cooked_fished item renamer", |item| {
            if ensure_namespaced(item) == "minecraft:cooked_fished" {
                "minecraft:cooked_fish".to_string()
            } else {
                item.to_string()
            }
        }),
    );
    b.add_fixer(v502, false, entity_zombie_villager_type_fix::fix());
    let v505 = b.add_same_schema(505, 0);
    b.add_fixer(v505, false, options_force_vbo_fix::fix());
    let v700 = b.add_schema(700, 0, v700::build);
    b.add_fixer(v700, true, entity_elder_guardian_split_fix::fix());
    let v701 = b.add_schema(701, 0, v701::build);
    b.add_fixer(v701, true, entity_skeleton_split_fix::fix());
    let v702 = b.add_schema(702, 0, v702::build);
    b.add_fixer(v702, true, entity_zombie_split_fix::fix());
    let v703 = b.add_schema(703, 0, v703::build);
    b.add_fixer(v703, true, entity_horse_split_fix::fix());
    let v704 = b.add_schema(704, 0, v704::build);
    b.add_fixer(v704, true, block_entity_id_fix::fix());
    let v705 = b.add_schema(705, 0, v705::build);
    b.add_fixer(v705, true, entity_id_fix::fix());
    let v804 = b.add_same_schema(804, 0);
    b.add_fixer(v804, true, item_banner_color_fix::fix());
    let v806 = b.add_same_schema(806, 0);
    b.add_fixer(v806, false, item_water_potion_fix::fix());
    let v808 = b.add_schema(808, 0, v808::build);
    b.add_fixer(
        v808,
        true,
        add_new_choices("added shulker box", r::BLOCK_ENTITY),
    );
    let v808_1 = b.add_same_schema(808, 1);
    b.add_fixer(v808_1, false, entity_shulker_color_fix::fix());
    let v813 = b.add_same_schema(813, 0);
    b.add_fixer(v813, false, item_shulker_box_color_fix::fix());
    b.add_fixer(v813, false, block_entity_shulker_box_color_fix::fix());
    let v816 = b.add_same_schema(816, 0);
    b.add_fixer(v816, false, options_lower_case_language_fix::fix());
    let v820 = b.add_same_schema(820, 0);
    b.add_fixer(
        v820,
        false,
        item_rename_fix(
            "totem item renamer",
            create_renamer("minecraft:totem", "minecraft:totem_of_undying"),
        ),
    );
    let v1022 = b.add_schema(1022, 0, v1022::build);
    b.add_fixer(
        v1022,
        true,
        write_and_read_fix("added shoulder entities to players", r::PLAYER),
    );
    let v1125 = b.add_schema(1125, 0, v1125::build);
    b.add_fixer(v1125, true, chunk_bed_block_entity_injecter_fix::fix());
    b.add_fixer(v1125, false, bed_item_color_fix::fix());
    let v1344 = b.add_same_schema(1344, 0);
    b.add_fixer(v1344, false, options_key_lwjgl3_fix::fix());
    let v1446 = b.add_same_schema(1446, 0);
    b.add_fixer(v1446, false, options_key_translation_fix::fix());
}
