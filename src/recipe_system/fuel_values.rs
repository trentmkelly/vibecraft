//! `FuelValues` — the fuel-item → burn-time registry. Split from
//! `recipe_system.rs` to keep it under the 1200-line limit.
//!
//! `vanilla_from_tags` is the production 1:1 builder; it has no live caller yet
//! because the furnace block entity is not ticked in production (see
//! `TODO(cooking-server-wiring)` in `vault_banner_furnace.rs`).
#![allow(dead_code)]

use super::{FuelValues, ItemTagMap};

impl FuelValues {
    /// `FuelValues.vanillaBurnTimes` — the 1:1 burn-time registry, built from the
    /// loaded vanilla item tags. Mirrors the Java `Builder`: each item/tag is added
    /// (tags expanded to their members, last write wins), then every
    /// `#minecraft:non_flammable_wood` item (crimson/warped wood) is removed.
    /// `base_unit` is the per-item smelt time (200 ticks in vanilla).
    pub fn vanilla_from_tags(tags: &ItemTagMap, base_unit: i32) -> Self {
        use std::collections::HashMap;
        let u = base_unit;
        let mut map: HashMap<&'static str, i32> = HashMap::new();
        let put = |map: &mut HashMap<&'static str, i32>, item: &'static str, t: i32| {
            map.insert(item, t);
        };
        let put_tag = |map: &mut HashMap<&'static str, i32>, tag: &str, t: i32| {
            for &item in tags.resolve(tag) {
                map.insert(item, t);
            }
        };

        put(&mut map, "minecraft:lava_bucket", u * 100);
        put(&mut map, "minecraft:coal_block", u * 8 * 10);
        put(&mut map, "minecraft:blaze_rod", u * 12);
        put(&mut map, "minecraft:coal", u * 8);
        put(&mut map, "minecraft:charcoal", u * 8);
        put_tag(&mut map, "minecraft:logs", u * 3 / 2);
        put_tag(&mut map, "minecraft:bamboo_blocks", u * 3 / 2);
        put_tag(&mut map, "minecraft:planks", u * 3 / 2);
        put(&mut map, "minecraft:bamboo_mosaic", u * 3 / 2);
        put_tag(&mut map, "minecraft:wooden_stairs", u * 3 / 2);
        put(&mut map, "minecraft:bamboo_mosaic_stairs", u * 3 / 2);
        put_tag(&mut map, "minecraft:wooden_slabs", u * 3 / 4);
        put(&mut map, "minecraft:bamboo_mosaic_slab", u * 3 / 4);
        put_tag(&mut map, "minecraft:wooden_trapdoors", u * 3 / 2);
        put_tag(&mut map, "minecraft:wooden_pressure_plates", u * 3 / 2);
        put_tag(&mut map, "minecraft:wooden_shelves", u * 3 / 2);
        put_tag(&mut map, "minecraft:wooden_fences", u * 3 / 2);
        put_tag(&mut map, "minecraft:fence_gates", u * 3 / 2);
        put(&mut map, "minecraft:note_block", u * 3 / 2);
        put(&mut map, "minecraft:bookshelf", u * 3 / 2);
        put(&mut map, "minecraft:chiseled_bookshelf", u * 3 / 2);
        put(&mut map, "minecraft:lectern", u * 3 / 2);
        put(&mut map, "minecraft:jukebox", u * 3 / 2);
        put(&mut map, "minecraft:chest", u * 3 / 2);
        put(&mut map, "minecraft:trapped_chest", u * 3 / 2);
        put(&mut map, "minecraft:crafting_table", u * 3 / 2);
        put(&mut map, "minecraft:daylight_detector", u * 3 / 2);
        put_tag(&mut map, "minecraft:banners", u * 3 / 2);
        put(&mut map, "minecraft:bow", u * 3 / 2);
        put(&mut map, "minecraft:fishing_rod", u * 3 / 2);
        put(&mut map, "minecraft:ladder", u * 3 / 2);
        put_tag(&mut map, "minecraft:signs", u);
        put_tag(&mut map, "minecraft:hanging_signs", u * 4);
        put(&mut map, "minecraft:wooden_shovel", u);
        put(&mut map, "minecraft:wooden_sword", u);
        put(&mut map, "minecraft:wooden_spear", u);
        put(&mut map, "minecraft:wooden_hoe", u);
        put(&mut map, "minecraft:wooden_axe", u);
        put(&mut map, "minecraft:wooden_pickaxe", u);
        put_tag(&mut map, "minecraft:wooden_doors", u);
        put_tag(&mut map, "minecraft:boats", u * 6);
        put_tag(&mut map, "minecraft:wool", u / 2);
        put_tag(&mut map, "minecraft:wooden_buttons", u / 2);
        put(&mut map, "minecraft:stick", u / 2);
        put_tag(&mut map, "minecraft:saplings", u / 2);
        put(&mut map, "minecraft:bowl", u / 2);
        put_tag(&mut map, "minecraft:wool_carpets", 1 + u / 3);
        put(&mut map, "minecraft:dried_kelp_block", 1 + u * 20);
        put(&mut map, "minecraft:crossbow", u * 3 / 2);
        put(&mut map, "minecraft:bamboo", u / 4);
        put(&mut map, "minecraft:dead_bush", u / 2);
        put(&mut map, "minecraft:short_dry_grass", u / 2);
        put(&mut map, "minecraft:tall_dry_grass", u / 2);
        put(&mut map, "minecraft:scaffolding", u / 4);
        put(&mut map, "minecraft:loom", u * 3 / 2);
        put(&mut map, "minecraft:barrel", u * 3 / 2);
        put(&mut map, "minecraft:cartography_table", u * 3 / 2);
        put(&mut map, "minecraft:fletching_table", u * 3 / 2);
        put(&mut map, "minecraft:smithing_table", u * 3 / 2);
        put(&mut map, "minecraft:composter", u * 3 / 2);
        put(&mut map, "minecraft:azalea", u / 2);
        put(&mut map, "minecraft:flowering_azalea", u / 2);
        put(&mut map, "minecraft:mangrove_roots", u * 3 / 2);
        put(&mut map, "minecraft:leaf_litter", u / 2);

        // `.remove(ItemTags.NON_FLAMMABLE_WOOD)` — crimson/warped wood never burns.
        for &item in tags.resolve("minecraft:non_flammable_wood") {
            map.remove(item);
        }

        Self {
            entries: map.into_iter().collect(),
        }
    }

    /// A small hardcoded fuel table for furnace-mechanics tests that only need a few
    /// direct fuels (coal/lava); the full 1:1 table is [`vanilla_from_tags`].
    #[cfg(test)]
    pub fn vanilla() -> Self {
        Self::vanilla_with_base_unit(200)
    }

    #[cfg(test)]
    pub fn vanilla_with_base_unit(base_unit: i32) -> Self {
        let mut entries = Vec::new();
        let mut add = |item, ticks| push_fuel(&mut entries, item, ticks);

        add("minecraft:lava_bucket", base_unit * 100);
        add("minecraft:coal_block", base_unit * 8 * 10);
        add("minecraft:blaze_rod", base_unit * 12);
        add("minecraft:coal", base_unit * 8);
        add("minecraft:charcoal", base_unit * 8);
        add("minecraft:oak_log", base_unit * 3 / 2);
        add("minecraft:bamboo_block", base_unit * 3 / 2);
        add("minecraft:oak_planks", base_unit * 3 / 2);
        add("minecraft:bamboo_mosaic", base_unit * 3 / 2);
        add("minecraft:oak_stairs", base_unit * 3 / 2);
        add("minecraft:bamboo_mosaic_stairs", base_unit * 3 / 2);
        add("minecraft:oak_slab", base_unit * 3 / 4);
        add("minecraft:bamboo_mosaic_slab", base_unit * 3 / 4);
        add("minecraft:oak_trapdoor", base_unit * 3 / 2);
        add("minecraft:oak_pressure_plate", base_unit * 3 / 2);
        add("minecraft:oak_shelf", base_unit * 3 / 2);
        add("minecraft:oak_fence", base_unit * 3 / 2);
        add("minecraft:oak_fence_gate", base_unit * 3 / 2);
        add("minecraft:note_block", base_unit * 3 / 2);
        add("minecraft:bookshelf", base_unit * 3 / 2);
        add("minecraft:chiseled_bookshelf", base_unit * 3 / 2);
        add("minecraft:lectern", base_unit * 3 / 2);
        add("minecraft:jukebox", base_unit * 3 / 2);
        add("minecraft:chest", base_unit * 3 / 2);
        add("minecraft:trapped_chest", base_unit * 3 / 2);
        add("minecraft:crafting_table", base_unit * 3 / 2);
        add("minecraft:daylight_detector", base_unit * 3 / 2);
        add("minecraft:white_banner", base_unit * 3 / 2);
        add("minecraft:bow", base_unit * 3 / 2);
        add("minecraft:fishing_rod", base_unit * 3 / 2);
        add("minecraft:ladder", base_unit * 3 / 2);
        add("minecraft:oak_sign", base_unit);
        add("minecraft:oak_hanging_sign", base_unit * 4);
        add("minecraft:wooden_shovel", base_unit);
        add("minecraft:wooden_sword", base_unit);
        add("minecraft:wooden_spear", base_unit);
        add("minecraft:wooden_hoe", base_unit);
        add("minecraft:wooden_axe", base_unit);
        add("minecraft:wooden_pickaxe", base_unit);
        add("minecraft:oak_door", base_unit);
        add("minecraft:oak_boat", base_unit * 6);
        add("minecraft:white_wool", base_unit / 2);
        add("minecraft:oak_button", base_unit / 2);
        add("minecraft:stick", base_unit / 2);
        add("minecraft:oak_sapling", base_unit / 2);
        add("minecraft:bowl", base_unit / 2);
        add("minecraft:white_carpet", 1 + base_unit / 3);
        add("minecraft:dried_kelp_block", 1 + base_unit * 20);
        add("minecraft:crossbow", base_unit * 3 / 2);
        add("minecraft:bamboo", base_unit / 4);
        add("minecraft:dead_bush", base_unit / 2);
        add("minecraft:short_dry_grass", base_unit / 2);
        add("minecraft:tall_dry_grass", base_unit / 2);
        add("minecraft:scaffolding", base_unit / 4);
        add("minecraft:loom", base_unit * 3 / 2);
        add("minecraft:barrel", base_unit * 3 / 2);
        add("minecraft:cartography_table", base_unit * 3 / 2);
        add("minecraft:fletching_table", base_unit * 3 / 2);
        add("minecraft:smithing_table", base_unit * 3 / 2);
        add("minecraft:composter", base_unit * 3 / 2);
        add("minecraft:azalea", base_unit / 2);
        add("minecraft:flowering_azalea", base_unit / 2);
        add("minecraft:mangrove_roots", base_unit * 3 / 2);
        add("minecraft:leaf_litter", base_unit / 2);

        Self { entries }
    }

    pub fn burn_duration(&self, item: Option<&str>) -> i32 {
        let Some(item) = item else {
            return 0;
        };
        self.entries
            .iter()
            .find_map(|(candidate, ticks)| (*candidate == item).then_some(*ticks))
            .unwrap_or(0)
    }

    pub fn is_fuel(&self, item: &str) -> bool {
        self.burn_duration(Some(item)) > 0
    }

    pub fn fuel_items(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.entries.iter().map(|(item, _)| *item)
    }
}

#[cfg(test)]
fn push_fuel(entries: &mut Vec<(&'static str, i32)>, item: &'static str, ticks: i32) {
    if let Some((_, existing)) = entries.iter_mut().find(|(candidate, _)| *candidate == item) {
        *existing = ticks;
    } else {
        entries.push((item, ticks));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vanilla_fuel() -> FuelValues {
        let tag_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla-data/data/minecraft/tags/item");
        let tags = crate::recipe_system::load_item_tag_directory(&tag_dir);
        FuelValues::vanilla_from_tags(&tags, 200)
    }

    #[test]
    fn vanilla_from_tags_expands_tags_and_removes_non_flammable_wood() {
        let fuel = vanilla_fuel();

        // Direct items.
        assert_eq!(fuel.burn_duration(Some("minecraft:lava_bucket")), 20000);
        assert_eq!(fuel.burn_duration(Some("minecraft:coal")), 1600);
        assert_eq!(fuel.burn_duration(Some("minecraft:blaze_rod")), 2400);
        assert_eq!(fuel.burn_duration(Some("minecraft:stick")), 100);

        // LOGS expands to EVERY wood type, not just oak (the old representative bug).
        for log in [
            "minecraft:oak_log",
            "minecraft:birch_log",
            "minecraft:spruce_log",
            "minecraft:jungle_log",
            "minecraft:cherry_log",
        ] {
            assert_eq!(fuel.burn_duration(Some(log)), 300, "{log} should burn 300t");
        }
        // PLANKS, WOOL, BANNERS, BOATS all expand across colours/woods.
        assert_eq!(fuel.burn_duration(Some("minecraft:spruce_planks")), 300);
        assert_eq!(fuel.burn_duration(Some("minecraft:black_wool")), 100);
        assert_eq!(fuel.burn_duration(Some("minecraft:red_banner")), 300);
        assert_eq!(fuel.burn_duration(Some("minecraft:birch_boat")), 1200);
        assert_eq!(fuel.burn_duration(Some("minecraft:oak_slab")), 150);

        // NON_FLAMMABLE_WOOD (crimson/warped) is removed even though it's in #logs/#planks.
        assert_eq!(
            fuel.burn_duration(Some("minecraft:crimson_stem")),
            0,
            "crimson never burns"
        );
        assert_eq!(fuel.burn_duration(Some("minecraft:warped_planks")), 0);
        assert!(!fuel.is_fuel("minecraft:crimson_door"));
        assert!(fuel.fuel_items().any(|item| item == "minecraft:oak_log"));
        assert!(!fuel
            .fuel_items()
            .any(|item| item == "minecraft:warped_planks"));

        // Non-fuel.
        assert_eq!(fuel.burn_duration(Some("minecraft:stone")), 0);
    }
}
