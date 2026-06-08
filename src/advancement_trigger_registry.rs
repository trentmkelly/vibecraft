use crate::advancement_criteria::CriterionTriggerModel;
use crate::registry::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CriteriaTriggerEntry {
    pub field_name: &'static str,
    pub serialized_name: &'static str,
    pub implementation: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CriteriaTriggersModel {
    entries: Vec<CriteriaTriggerEntry>,
}

impl CriteriaTriggersModel {
    pub fn java_bootstrap() -> CriterionTriggerModel {
        Self::trigger_model(Self::IMPOSSIBLE)
    }

    pub fn java_default_registry() -> Self {
        let mut registry = Self::default();
        for entry in Self::JAVA_ENTRIES {
            registry.register(*entry);
        }
        registry
    }

    pub fn register(&mut self, entry: CriteriaTriggerEntry) -> CriterionTriggerModel {
        self.entries.push(entry);
        Self::trigger_model(entry)
    }

    pub fn by_name(&self, name: &str) -> Option<CriteriaTriggerEntry> {
        self.entries
            .iter()
            .find(|entry| entry.serialized_name == name)
            .copied()
    }

    pub fn entries(&self) -> &[CriteriaTriggerEntry] {
        &self.entries
    }

    fn trigger_model(entry: CriteriaTriggerEntry) -> CriterionTriggerModel {
        CriterionTriggerModel::new(Identifier::parse(entry.serialized_name).unwrap())
    }

    pub const IMPOSSIBLE: CriteriaTriggerEntry = CriteriaTriggerEntry {
        field_name: "IMPOSSIBLE",
        serialized_name: "impossible",
        implementation: "ImpossibleTrigger",
    };

    pub const JAVA_ENTRIES: &'static [CriteriaTriggerEntry] = &[
        Self::IMPOSSIBLE,
        CriteriaTriggerEntry {
            field_name: "PLAYER_KILLED_ENTITY",
            serialized_name: "player_killed_entity",
            implementation: "KilledTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ENTITY_KILLED_PLAYER",
            serialized_name: "entity_killed_player",
            implementation: "KilledTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ENTER_BLOCK",
            serialized_name: "enter_block",
            implementation: "EnterBlockTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "INVENTORY_CHANGED",
            serialized_name: "inventory_changed",
            implementation: "InventoryChangeTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "RECIPE_UNLOCKED",
            serialized_name: "recipe_unlocked",
            implementation: "RecipeUnlockedTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "PLAYER_HURT_ENTITY",
            serialized_name: "player_hurt_entity",
            implementation: "PlayerHurtEntityTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ENTITY_HURT_PLAYER",
            serialized_name: "entity_hurt_player",
            implementation: "EntityHurtPlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ENCHANTED_ITEM",
            serialized_name: "enchanted_item",
            implementation: "EnchantedItemTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "FILLED_BUCKET",
            serialized_name: "filled_bucket",
            implementation: "FilledBucketTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "BREWED_POTION",
            serialized_name: "brewed_potion",
            implementation: "BrewedPotionTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "CONSTRUCT_BEACON",
            serialized_name: "construct_beacon",
            implementation: "ConstructBeaconTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "USED_ENDER_EYE",
            serialized_name: "used_ender_eye",
            implementation: "UsedEnderEyeTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "SUMMONED_ENTITY",
            serialized_name: "summoned_entity",
            implementation: "SummonedEntityTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "BRED_ANIMALS",
            serialized_name: "bred_animals",
            implementation: "BredAnimalsTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "LOCATION",
            serialized_name: "location",
            implementation: "PlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "SLEPT_IN_BED",
            serialized_name: "slept_in_bed",
            implementation: "PlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "CURED_ZOMBIE_VILLAGER",
            serialized_name: "cured_zombie_villager",
            implementation: "CuredZombieVillagerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "TRADE",
            serialized_name: "villager_trade",
            implementation: "TradeTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ITEM_DURABILITY_CHANGED",
            serialized_name: "item_durability_changed",
            implementation: "ItemDurabilityTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "LEVITATION",
            serialized_name: "levitation",
            implementation: "LevitationTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "CHANGED_DIMENSION",
            serialized_name: "changed_dimension",
            implementation: "ChangeDimensionTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "TICK",
            serialized_name: "tick",
            implementation: "PlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "TAME_ANIMAL",
            serialized_name: "tame_animal",
            implementation: "TameAnimalTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "PLACED_BLOCK",
            serialized_name: "placed_block",
            implementation: "ItemUsedOnLocationTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "CONSUME_ITEM",
            serialized_name: "consume_item",
            implementation: "ConsumeItemTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "EFFECTS_CHANGED",
            serialized_name: "effects_changed",
            implementation: "EffectsChangedTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "USED_TOTEM",
            serialized_name: "used_totem",
            implementation: "UsedTotemTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "NETHER_TRAVEL",
            serialized_name: "nether_travel",
            implementation: "DistanceTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "FISHING_ROD_HOOKED",
            serialized_name: "fishing_rod_hooked",
            implementation: "FishingRodHookedTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "CHANNELED_LIGHTNING",
            serialized_name: "channeled_lightning",
            implementation: "ChanneledLightningTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "SHOT_CROSSBOW",
            serialized_name: "shot_crossbow",
            implementation: "ShotCrossbowTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "SPEAR_MOBS_TRIGGER",
            serialized_name: "spear_mobs",
            implementation: "SpearMobsTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "KILLED_BY_ARROW",
            serialized_name: "killed_by_arrow",
            implementation: "KilledByArrowTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "RAID_WIN",
            serialized_name: "hero_of_the_village",
            implementation: "PlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "RAID_OMEN",
            serialized_name: "voluntary_exile",
            implementation: "PlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "HONEY_BLOCK_SLIDE",
            serialized_name: "slide_down_block",
            implementation: "SlideDownBlockTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "BEE_NEST_DESTROYED",
            serialized_name: "bee_nest_destroyed",
            implementation: "BeeNestDestroyedTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "TARGET_BLOCK_HIT",
            serialized_name: "target_hit",
            implementation: "TargetBlockTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ITEM_USED_ON_BLOCK",
            serialized_name: "item_used_on_block",
            implementation: "ItemUsedOnLocationTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "DEFAULT_BLOCK_USE",
            serialized_name: "default_block_use",
            implementation: "DefaultBlockInteractionTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ANY_BLOCK_USE",
            serialized_name: "any_block_use",
            implementation: "AnyBlockInteractionTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "GENERATE_LOOT",
            serialized_name: "player_generates_container_loot",
            implementation: "LootTableTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "THROWN_ITEM_PICKED_UP_BY_ENTITY",
            serialized_name: "thrown_item_picked_up_by_entity",
            implementation: "PickedUpItemTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "THROWN_ITEM_PICKED_UP_BY_PLAYER",
            serialized_name: "thrown_item_picked_up_by_player",
            implementation: "PickedUpItemTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "PLAYER_INTERACTED_WITH_ENTITY",
            serialized_name: "player_interacted_with_entity",
            implementation: "PlayerInteractTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "PLAYER_SHEARED_EQUIPMENT",
            serialized_name: "player_sheared_equipment",
            implementation: "PlayerInteractTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "START_RIDING_TRIGGER",
            serialized_name: "started_riding",
            implementation: "StartRidingTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "LIGHTNING_STRIKE",
            serialized_name: "lightning_strike",
            implementation: "LightningStrikeTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "USING_ITEM",
            serialized_name: "using_item",
            implementation: "UsingItemTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "FALL_FROM_HEIGHT",
            serialized_name: "fall_from_height",
            implementation: "DistanceTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "RIDE_ENTITY_IN_LAVA_TRIGGER",
            serialized_name: "ride_entity_in_lava",
            implementation: "DistanceTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "KILL_MOB_NEAR_SCULK_CATALYST",
            serialized_name: "kill_mob_near_sculk_catalyst",
            implementation: "KilledTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "ALLAY_DROP_ITEM_ON_BLOCK",
            serialized_name: "allay_drop_item_on_block",
            implementation: "ItemUsedOnLocationTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "AVOID_VIBRATION",
            serialized_name: "avoid_vibration",
            implementation: "PlayerTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "RECIPE_CRAFTED",
            serialized_name: "recipe_crafted",
            implementation: "RecipeCraftedTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "CRAFTER_RECIPE_CRAFTED",
            serialized_name: "crafter_recipe_crafted",
            implementation: "RecipeCraftedTrigger",
        },
        CriteriaTriggerEntry {
            field_name: "FALL_AFTER_EXPLOSION",
            serialized_name: "fall_after_explosion",
            implementation: "FallAfterExplosionTrigger",
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn criteria_triggers_registry_matches_java_registration_order_and_lookup() {
        let registry = CriteriaTriggersModel::java_default_registry();
        assert_eq!(registry.entries().len(), 58);
        assert_eq!(registry.entries()[0], CriteriaTriggersModel::IMPOSSIBLE);
        assert_eq!(
            registry.entries().last().unwrap(),
            &CriteriaTriggerEntry {
                field_name: "FALL_AFTER_EXPLOSION",
                serialized_name: "fall_after_explosion",
                implementation: "FallAfterExplosionTrigger",
            }
        );
        assert_eq!(
            registry.by_name("recipe_unlocked").unwrap(),
            CriteriaTriggerEntry {
                field_name: "RECIPE_UNLOCKED",
                serialized_name: "recipe_unlocked",
                implementation: "RecipeUnlockedTrigger",
            }
        );
        assert_eq!(
            registry.by_name("player_generates_container_loot").unwrap(),
            CriteriaTriggerEntry {
                field_name: "GENERATE_LOOT",
                serialized_name: "player_generates_container_loot",
                implementation: "LootTableTrigger",
            }
        );
        assert_eq!(registry.by_name("missing"), None);
    }

    #[test]
    fn criteria_triggers_model_preserves_shared_java_trigger_implementations() {
        let registry = CriteriaTriggersModel::java_default_registry();
        let player_triggers = registry
            .entries()
            .iter()
            .filter(|entry| entry.implementation == "PlayerTrigger")
            .map(|entry| entry.serialized_name)
            .collect::<Vec<_>>();
        assert_eq!(
            player_triggers,
            vec![
                "location",
                "slept_in_bed",
                "tick",
                "hero_of_the_village",
                "voluntary_exile",
                "avoid_vibration",
            ]
        );

        let crafted_triggers = registry
            .entries()
            .iter()
            .filter(|entry| entry.implementation == "RecipeCraftedTrigger")
            .map(|entry| entry.serialized_name)
            .collect::<Vec<_>>();
        assert_eq!(
            crafted_triggers,
            vec!["recipe_crafted", "crafter_recipe_crafted"]
        );
    }

    #[test]
    fn criteria_triggers_bootstrap_returns_impossible_like_java() {
        assert_eq!(
            CriteriaTriggersModel::java_bootstrap().id(),
            &Identifier::parse("minecraft:impossible").unwrap()
        );
    }
}
