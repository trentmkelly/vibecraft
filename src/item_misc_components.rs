#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvidesTrimMaterial {
    pub material: &'static str,
}

impl ProvidesTrimMaterial {
    pub fn new(material: &'static str) -> Self {
        Self { material }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeededContainerLoot {
    pub loot_table: &'static str,
    pub seed: i64,
}

impl SeededContainerLoot {
    pub const DEFAULT_SEED: i64 = 0;
    pub const UNKNOWN_CONTENTS_TRANSLATION: &'static str = "item.container.loot_table.unknown";

    pub fn new(loot_table: &'static str, seed: i64) -> Self {
        Self { loot_table, seed }
    }

    pub fn with_default_seed(loot_table: &'static str) -> Self {
        Self::new(loot_table, Self::DEFAULT_SEED)
    }

    pub fn tooltip(&self) -> &'static str {
        Self::UNKNOWN_CONTENTS_TRANSLATION
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageResistant {
    pub types: Vec<&'static str>,
}

impl DamageResistant {
    pub fn new(types: Vec<&'static str>) -> Self {
        Self { types }
    }

    pub fn is_resistant_to(&self, damage_type: &str) -> bool {
        self.types.contains(&damage_type)
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const PROVIDES_TRIM_MATERIAL_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/ProvidesTrimMaterial.java");
    const SEEDED_CONTAINER_LOOT_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/SeededContainerLoot.java");
    const DAMAGE_RESISTANT_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/DamageResistant.java");

    #[test]
    fn provides_trim_material_wraps_trim_material_holder_like_java() {
        for sentinel in [
            "public record ProvidesTrimMaterial(Holder<TrimMaterial> material)",
            "TrimMaterial.CODEC.xmap(ProvidesTrimMaterial::new, ProvidesTrimMaterial::material)",
            "TrimMaterial.STREAM_CODEC",
            ".map(ProvidesTrimMaterial::new, ProvidesTrimMaterial::material)",
        ] {
            assert!(
                PROVIDES_TRIM_MATERIAL_JAVA.contains(sentinel),
                "missing ProvidesTrimMaterial sentinel {sentinel}"
            );
        }

        assert_eq!(
            ProvidesTrimMaterial::new("minecraft:diamond"),
            ProvidesTrimMaterial {
                material: "minecraft:diamond"
            }
        );
    }

    #[test]
    fn seeded_container_loot_defaults_and_tooltip_match_java() {
        for sentinel in [
            "public record SeededContainerLoot(ResourceKey<LootTable> lootTable, long seed) implements TooltipProvider",
            "Component.translatable(\"item.container.loot_table.unknown\")",
            "LootTable.KEY_CODEC.fieldOf(\"loot_table\")",
            "Codec.LONG.optionalFieldOf(\"seed\", 0L)",
            "consumer.accept(UNKNOWN_CONTENTS);",
        ] {
            assert!(
                SEEDED_CONTAINER_LOOT_JAVA.contains(sentinel),
                "missing SeededContainerLoot sentinel {sentinel}"
            );
        }

        let default_seed = SeededContainerLoot::with_default_seed("minecraft:chests/simple_dungeon");
        assert_eq!(default_seed.loot_table, "minecraft:chests/simple_dungeon");
        assert_eq!(default_seed.seed, 0);
        assert_eq!(
            default_seed.tooltip(),
            "item.container.loot_table.unknown"
        );

        assert_eq!(
            SeededContainerLoot::new("minecraft:chests/end_city_treasure", 42),
            SeededContainerLoot {
                loot_table: "minecraft:chests/end_city_treasure",
                seed: 42
            }
        );
    }

    #[test]
    fn damage_resistant_matches_damage_type_holder_membership_like_java() {
        for sentinel in [
            "public record DamageResistant(HolderSet<DamageType> types)",
            "RegistryCodecs.homogeneousList(Registries.DAMAGE_TYPE).fieldOf(\"types\")",
            "ByteBufCodecs.holderSet(Registries.DAMAGE_TYPE)",
            "return this.types.contains(source.typeHolder());",
        ] {
            assert!(
                DAMAGE_RESISTANT_JAVA.contains(sentinel),
                "missing DamageResistant sentinel {sentinel}"
            );
        }

        let resistant = DamageResistant::new(vec![
            "minecraft:in_fire",
            "minecraft:on_fire",
            "minecraft:lava",
        ]);
        assert!(resistant.is_resistant_to("minecraft:lava"));
        assert!(!resistant.is_resistant_to("minecraft:fall"));
    }
}
