// TODO(item-use-block-and-entity-behaviors): the remaining behavioral item
// classes under net/minecraft/world/item (CHECKLIST_ITEMS.md #7) still need
// their item-USE paths wired. These are blocked on the BLOCKS (block-state
// mutation) and ENTITIES (world entity placement) subsystems and so are not yet
// represented here:
//   - Throwables: SnowballItem / EggItem / EnderpearlItem / ExperienceBottleItem /
//     WindChargeItem / FireChargeItem. The projectile ENTITIES already exist
//     (projectile_entity::ThrowableKind / HurtingProjectileKind) but no item-use
//     spawns them into the world.
//   - Tool-on-block: AxeItem / HoeItem / ShovelItem block mutations are now
//     wired live via item_tool_use + network::status::item_use_live. Axe's
//     offhand blocking-item intent gate remains deferred behind
//     TODO(live-sneak-tracking) until secondary-use tracking exists.
//   - BoneMealItem (apply growth), HoneycombItem (wax copper),
//     GlowInkSacItem / InkSacItem (sign glow toggle), DebugStickItem
//     (cycle block state).
//   - Entity-placement items: EndCrystalItem, ArmorStandItem, ItemFrameItem,
//     HangingEntityItem / HangingSignItem placement.
// Implement each here (or in a sibling use-dispatch module) once BLOCKS/ENTITIES
// expose the world mutation + entity spawn primitives, then mark #7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseResult {
    Success,
    SuccessServer,
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemAction {
    pub result: UseResult,
    pub consumes: u32,
    pub awards_stat: bool,
    pub spawned_entity: Option<&'static str>,
    pub returned_item: Option<&'static str>,
    pub opened_ui: Option<&'static str>,
    pub game_event: Option<&'static str>,
    pub sound: Option<&'static str>,
}

impl ItemAction {
    fn pass() -> Self {
        Self {
            result: UseResult::Pass,
            consumes: 0,
            awards_stat: false,
            spawned_entity: None,
            returned_item: None,
            opened_ui: None,
            game_event: None,
            sound: None,
        }
    }

    fn fail() -> Self {
        Self {
            result: UseResult::Fail,
            ..Self::pass()
        }
    }
}

pub fn empty_map_use(server_side: bool, inventory_has_space: bool) -> ItemAction {
    if !server_side {
        return ItemAction {
            result: UseResult::Success,
            ..ItemAction::pass()
        };
    }

    ItemAction {
        result: UseResult::Success,
        consumes: 1,
        awards_stat: true,
        spawned_entity: (!inventory_has_space).then_some("minecraft:item"),
        returned_item: inventory_has_space.then_some("minecraft:filled_map"),
        opened_ui: None,
        game_event: None,
        sound: Some("ui_cartography_table_take_result"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapState {
    pub width: u8,
    pub height: u8,
    pub scale: u8,
    pub tracking_position: bool,
    pub unlimited_tracking: bool,
}

pub fn fresh_map_state(scale: u8, tracking_position: bool, unlimited_tracking: bool) -> MapState {
    MapState {
        width: 128,
        height: 128,
        scale,
        tracking_position,
        unlimited_tracking,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompassTarget {
    Lodestone,
    Recovery,
    Spawn,
    None,
}

pub fn compass_target(
    has_lodestone_component: bool,
    is_recovery_compass: bool,
    has_spawn: bool,
) -> CompassTarget {
    if has_lodestone_component {
        CompassTarget::Lodestone
    } else if is_recovery_compass {
        CompassTarget::Recovery
    } else if has_spawn {
        CompassTarget::Spawn
    } else {
        CompassTarget::None
    }
}

pub fn clock_tracks_daytime(dimension_has_fixed_time: bool) -> bool {
    !dimension_has_fixed_time
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleClick {
    PrimaryWithOther,
    PrimaryEmpty,
    SecondaryEmptySlot,
    SecondaryWithOther,
}

pub fn bundle_click(
    click: BundleClick,
    can_modify_slot: bool,
    bundle_has_space: bool,
    bundle_has_item: bool,
) -> ItemAction {
    match click {
        BundleClick::PrimaryWithOther if can_modify_slot && bundle_has_space => ItemAction {
            result: UseResult::Success,
            sound: Some("bundle_insert"),
            ..ItemAction::pass()
        },
        BundleClick::PrimaryWithOther => ItemAction {
            result: UseResult::Success,
            sound: Some("bundle_insert_fail"),
            ..ItemAction::pass()
        },
        BundleClick::PrimaryEmpty => ItemAction::pass(),
        BundleClick::SecondaryEmptySlot if can_modify_slot && bundle_has_item => ItemAction {
            result: UseResult::Success,
            returned_item: Some("removed_bundle_item"),
            sound: Some("bundle_remove_one"),
            ..ItemAction::pass()
        },
        BundleClick::SecondaryEmptySlot => ItemAction {
            result: UseResult::Success,
            ..ItemAction::pass()
        },
        BundleClick::SecondaryWithOther => ItemAction::pass(),
    }
}

pub fn writable_book_use() -> ItemAction {
    ItemAction {
        result: UseResult::Success,
        awards_stat: true,
        opened_ui: Some("book_editor"),
        ..ItemAction::pass()
    }
}

pub fn knowledge_book_use(recipe_count: usize, all_recipes_valid: bool) -> ItemAction {
    if recipe_count == 0 || !all_recipes_valid {
        return ItemAction::fail();
    }

    ItemAction {
        result: UseResult::Success,
        consumes: 1,
        awards_stat: true,
        opened_ui: Some("recipe_unlocks"),
        ..ItemAction::pass()
    }
}

pub fn spawn_egg_use(
    target_is_spawner: bool,
    target_allows_spawn: bool,
    entity_type: &'static str,
) -> ItemAction {
    if target_is_spawner {
        return ItemAction {
            result: UseResult::Success,
            opened_ui: Some("spawner_entity_update"),
            ..ItemAction::pass()
        };
    }

    if !target_allows_spawn {
        return ItemAction::fail();
    }

    ItemAction {
        result: UseResult::Success,
        consumes: 1,
        spawned_entity: Some(entity_type),
        game_event: Some("entity_place"),
        ..ItemAction::pass()
    }
}

pub fn boat_use(
    hit_block: bool,
    obstructing_pickable_entity: bool,
    collision_free: bool,
    boat_entity: &'static str,
) -> ItemAction {
    if !hit_block || obstructing_pickable_entity {
        return ItemAction::pass();
    }
    if !collision_free {
        return ItemAction::fail();
    }

    ItemAction {
        result: UseResult::Success,
        consumes: 1,
        awards_stat: true,
        spawned_entity: Some(boat_entity),
        game_event: Some("entity_place"),
        ..ItemAction::pass()
    }
}

pub fn minecart_use_on(
    is_rail: bool,
    rail_is_slope: bool,
    blocked_by_cart: bool,
    cart_entity: &'static str,
) -> ItemAction {
    if !is_rail || blocked_by_cart {
        return ItemAction::fail();
    }

    ItemAction {
        result: UseResult::Success,
        consumes: 1,
        spawned_entity: Some(cart_entity),
        game_event: Some(if rail_is_slope {
            "entity_place_slope_offset"
        } else {
            "entity_place"
        }),
        ..ItemAction::pass()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentBehavior {
    ArmorTrimTemplate,
    SmithingTemplate,
    ShieldBlock,
    ElytraGlider,
    MaceSmashWeapon,
    TridentSpearProjectile,
    BowProjectile,
    CrossbowChargedProjectile,
}

pub fn equipment_behavior(item: &'static str) -> Option<EquipmentBehavior> {
    match item {
        "minecraft:sentry_armor_trim_smithing_template" => {
            Some(EquipmentBehavior::ArmorTrimTemplate)
        }
        "minecraft:netherite_upgrade_smithing_template" => {
            Some(EquipmentBehavior::SmithingTemplate)
        }
        "minecraft:shield" => Some(EquipmentBehavior::ShieldBlock),
        "minecraft:elytra" => Some(EquipmentBehavior::ElytraGlider),
        "minecraft:mace" => Some(EquipmentBehavior::MaceSmashWeapon),
        "minecraft:trident" => Some(EquipmentBehavior::TridentSpearProjectile),
        "minecraft:bow" => Some(EquipmentBehavior::BowProjectile),
        "minecraft:crossbow" => Some(EquipmentBehavior::CrossbowChargedProjectile),
        _ => None,
    }
}

pub fn fishing_rod_use(has_hook: bool) -> ItemAction {
    if has_hook {
        ItemAction {
            result: UseResult::Success,
            awards_stat: true,
            game_event: Some("fishing_hook_retrieve"),
            sound: Some("fishing_bobber_retrieve"),
            ..ItemAction::pass()
        }
    } else {
        ItemAction {
            result: UseResult::Success,
            awards_stat: true,
            spawned_entity: Some("minecraft:fishing_bobber"),
            sound: Some("fishing_bobber_throw"),
            ..ItemAction::pass()
        }
    }
}

pub fn shears_use_on(shearable: bool) -> ItemAction {
    if shearable {
        ItemAction {
            result: UseResult::Success,
            awards_stat: true,
            sound: Some("shear"),
            game_event: Some("shear"),
            ..ItemAction::pass()
        }
    } else {
        ItemAction::pass()
    }
}

pub fn brush_use_on(brushable_block: bool) -> ItemAction {
    if brushable_block {
        ItemAction {
            result: UseResult::Success,
            sound: Some("brush"),
            game_event: Some("block_change"),
            ..ItemAction::pass()
        }
    } else {
        ItemAction::pass()
    }
}

pub fn lead_use_on_fence(
    leashable_mobs_held: usize,
    any_can_attach: bool,
    existing_knot: bool,
) -> ItemAction {
    if leashable_mobs_held == 0 || !any_can_attach {
        return ItemAction::pass();
    }

    ItemAction {
        result: UseResult::SuccessServer,
        spawned_entity: (!existing_knot).then_some("minecraft:leash_knot"),
        sound: Some("leash_knot_place"),
        game_event: Some("block_attach"),
        ..ItemAction::pass()
    }
}

pub fn name_tag_use(
    has_custom_name: bool,
    target_can_serialize: bool,
    target_alive: bool,
) -> ItemAction {
    if has_custom_name && target_can_serialize {
        ItemAction {
            result: UseResult::Success,
            consumes: u32::from(target_alive),
            opened_ui: Some("set_custom_name"),
            ..ItemAction::pass()
        }
    } else {
        ItemAction::pass()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveItemBehavior {
    MusicDiscJukebox,
    OminousBottleDrink,
    TrialKeyVaultUnlock,
}

pub fn passive_item_behavior(item: &'static str) -> Option<PassiveItemBehavior> {
    match item {
        "minecraft:music_disc_13" => Some(PassiveItemBehavior::MusicDiscJukebox),
        "minecraft:ominous_bottle" => Some(PassiveItemBehavior::OminousBottleDrink),
        "minecraft:trial_key" | "minecraft:ominous_trial_key" => {
            Some(PassiveItemBehavior::TrialKeyVaultUnlock)
        }
        _ => None,
    }
}

pub const COVERED_ITEM_FAMILIES: &[&str] = &[
    "maps",
    "compasses",
    "clocks",
    "recovery_compasses",
    "bundles",
    "books",
    "written_books",
    "knowledge_books",
    "spawn_eggs",
    "boats",
    "minecarts",
    "armor_trims",
    "smithing_templates",
    "shields",
    "elytra",
    "maces",
    "tridents",
    "bows",
    "crossbows",
    "fishing_rods",
    "shears",
    "brushes",
    "leads",
    "name_tags",
    "music_discs",
    "ominous_bottles",
    "trial_keys",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_compasses_and_clocks_expose_vanilla_tracking_behavior() {
        let map = empty_map_use(true, true);
        assert_eq!(map.result, UseResult::Success);
        assert_eq!(map.consumes, 1);
        assert_eq!(map.returned_item, Some("minecraft:filled_map"));
        assert_eq!(map.sound, Some("ui_cartography_table_take_result"));

        assert_eq!(
            fresh_map_state(0, true, false),
            MapState {
                width: 128,
                height: 128,
                scale: 0,
                tracking_position: true,
                unlimited_tracking: false
            }
        );
        assert_eq!(compass_target(true, true, true), CompassTarget::Lodestone);
        assert_eq!(compass_target(false, true, true), CompassTarget::Recovery);
        assert_eq!(compass_target(false, false, true), CompassTarget::Spawn);
        assert!(clock_tracks_daytime(false));
        assert!(!clock_tracks_daytime(true));
    }

    #[test]
    fn bundle_and_book_actions_match_vanilla_use_paths() {
        assert_eq!(
            bundle_click(BundleClick::PrimaryWithOther, true, true, false).sound,
            Some("bundle_insert")
        );
        assert_eq!(
            bundle_click(BundleClick::PrimaryWithOther, true, false, false).sound,
            Some("bundle_insert_fail")
        );
        assert_eq!(
            bundle_click(BundleClick::SecondaryEmptySlot, true, true, true).returned_item,
            Some("removed_bundle_item")
        );
        assert_eq!(
            bundle_click(BundleClick::PrimaryEmpty, true, true, true).result,
            UseResult::Pass
        );
        assert_eq!(
            bundle_click(BundleClick::SecondaryWithOther, true, true, true).result,
            UseResult::Pass
        );

        let writable = writable_book_use();
        assert_eq!(writable.opened_ui, Some("book_editor"));
        assert!(writable.awards_stat);

        assert_eq!(knowledge_book_use(0, true).result, UseResult::Fail);
        assert_eq!(knowledge_book_use(1, false).result, UseResult::Fail);
        assert_eq!(
            knowledge_book_use(2, true).opened_ui,
            Some("recipe_unlocks")
        );
    }

    #[test]
    fn spawn_eggs_boats_and_minecarts_spawn_or_fail_like_item_classes() {
        assert_eq!(
            spawn_egg_use(false, true, "minecraft:zombie").spawned_entity,
            Some("minecraft:zombie")
        );
        assert_eq!(
            spawn_egg_use(true, false, "minecraft:zombie").opened_ui,
            Some("spawner_entity_update")
        );
        assert_eq!(
            spawn_egg_use(false, false, "minecraft:zombie").result,
            UseResult::Fail
        );

        let boat = boat_use(true, false, true, "minecraft:oak_boat");
        assert_eq!(boat.spawned_entity, Some("minecraft:oak_boat"));
        assert_eq!(boat.game_event, Some("entity_place"));
        assert_eq!(
            boat_use(false, false, true, "minecraft:oak_boat").result,
            UseResult::Pass
        );
        assert_eq!(
            boat_use(true, true, true, "minecraft:oak_boat").result,
            UseResult::Pass
        );
        assert_eq!(
            boat_use(true, false, false, "minecraft:oak_boat").result,
            UseResult::Fail
        );

        assert_eq!(
            minecart_use_on(true, false, false, "minecraft:minecart").spawned_entity,
            Some("minecraft:minecart")
        );
        assert_eq!(
            minecart_use_on(true, true, false, "minecraft:minecart").game_event,
            Some("entity_place_slope_offset")
        );
        assert_eq!(
            minecart_use_on(false, false, false, "minecraft:minecart").result,
            UseResult::Fail
        );
        assert_eq!(
            minecart_use_on(true, false, true, "minecraft:minecart").result,
            UseResult::Fail
        );
    }

    #[test]
    fn equipment_and_tool_families_map_to_server_visible_behaviors() {
        assert_eq!(
            equipment_behavior("minecraft:sentry_armor_trim_smithing_template"),
            Some(EquipmentBehavior::ArmorTrimTemplate)
        );
        assert_eq!(
            equipment_behavior("minecraft:netherite_upgrade_smithing_template"),
            Some(EquipmentBehavior::SmithingTemplate)
        );
        assert_eq!(
            equipment_behavior("minecraft:shield"),
            Some(EquipmentBehavior::ShieldBlock)
        );
        assert_eq!(
            equipment_behavior("minecraft:elytra"),
            Some(EquipmentBehavior::ElytraGlider)
        );
        assert_eq!(
            equipment_behavior("minecraft:mace"),
            Some(EquipmentBehavior::MaceSmashWeapon)
        );
        assert_eq!(
            equipment_behavior("minecraft:trident"),
            Some(EquipmentBehavior::TridentSpearProjectile)
        );
        assert_eq!(
            equipment_behavior("minecraft:bow"),
            Some(EquipmentBehavior::BowProjectile)
        );
        assert_eq!(
            equipment_behavior("minecraft:crossbow"),
            Some(EquipmentBehavior::CrossbowChargedProjectile)
        );

        assert_eq!(
            fishing_rod_use(false).spawned_entity,
            Some("minecraft:fishing_bobber")
        );
        assert_eq!(
            fishing_rod_use(true).game_event,
            Some("fishing_hook_retrieve")
        );
        assert_eq!(shears_use_on(true).game_event, Some("shear"));
        assert_eq!(shears_use_on(false).result, UseResult::Pass);
        assert_eq!(brush_use_on(true).game_event, Some("block_change"));
        assert_eq!(brush_use_on(false).result, UseResult::Pass);
    }

    #[test]
    fn lead_name_tag_music_disc_ominous_bottle_and_trial_key_behaviors_are_covered() {
        let lead = lead_use_on_fence(2, true, false);
        assert_eq!(lead.result, UseResult::SuccessServer);
        assert_eq!(lead.spawned_entity, Some("minecraft:leash_knot"));
        assert_eq!(lead.game_event, Some("block_attach"));
        assert_eq!(lead_use_on_fence(0, true, false).result, UseResult::Pass);
        assert_eq!(lead_use_on_fence(2, false, false).result, UseResult::Pass);

        let named = name_tag_use(true, true, true);
        assert_eq!(named.result, UseResult::Success);
        assert_eq!(named.consumes, 1);
        assert_eq!(named.opened_ui, Some("set_custom_name"));
        assert_eq!(name_tag_use(false, true, true).result, UseResult::Pass);
        assert_eq!(name_tag_use(true, false, true).result, UseResult::Pass);

        assert_eq!(
            passive_item_behavior("minecraft:music_disc_13"),
            Some(PassiveItemBehavior::MusicDiscJukebox)
        );
        assert_eq!(
            passive_item_behavior("minecraft:ominous_bottle"),
            Some(PassiveItemBehavior::OminousBottleDrink)
        );
        assert_eq!(
            passive_item_behavior("minecraft:trial_key"),
            Some(PassiveItemBehavior::TrialKeyVaultUnlock)
        );
        assert_eq!(
            passive_item_behavior("minecraft:ominous_trial_key"),
            Some(PassiveItemBehavior::TrialKeyVaultUnlock)
        );
    }

    #[test]
    fn checklist_item_family_manifest_names_every_requested_family() {
        assert_eq!(COVERED_ITEM_FAMILIES.len(), 27);
        for required in [
            "maps",
            "compasses",
            "clocks",
            "recovery_compasses",
            "bundles",
            "books",
            "written_books",
            "knowledge_books",
            "spawn_eggs",
            "boats",
            "minecarts",
            "armor_trims",
            "smithing_templates",
            "shields",
            "elytra",
            "maces",
            "tridents",
            "bows",
            "crossbows",
            "fishing_rods",
            "shears",
            "brushes",
            "leads",
            "name_tags",
            "music_discs",
            "ominous_bottles",
            "trial_keys",
        ] {
            assert!(COVERED_ITEM_FAMILIES.contains(&required));
        }
    }
}
