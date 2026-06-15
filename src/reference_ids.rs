#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReferenceResourceKey {
    pub registry: &'static str,
    pub location: &'static str,
}

impl ReferenceResourceKey {
    pub const fn block(location: &'static str) -> Self {
        Self {
            registry: "minecraft:block",
            location,
        }
    }

    pub const fn item(location: &'static str) -> Self {
        Self {
            registry: "minecraft:item",
            location,
        }
    }
}

// Source: decompiled-server-26.1.2/net/minecraft/references/BlockIds.java
pub const BLOCK_IDS: &[ReferenceResourceKey] = &[
    ReferenceResourceKey::block("minecraft:pumpkin"),
    ReferenceResourceKey::block("minecraft:pumpkin_stem"),
    ReferenceResourceKey::block("minecraft:attached_pumpkin_stem"),
    ReferenceResourceKey::block("minecraft:melon"),
    ReferenceResourceKey::block("minecraft:melon_stem"),
    ReferenceResourceKey::block("minecraft:attached_melon_stem"),
    ReferenceResourceKey::block("minecraft:dirt"),
];

// Source: decompiled-server-26.1.2/net/minecraft/references/ItemIds.java
pub const ITEM_IDS: &[ReferenceResourceKey] = &[
    ReferenceResourceKey::item("minecraft:pumpkin_seeds"),
    ReferenceResourceKey::item("minecraft:melon_seeds"),
];

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK_IDS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/references/BlockIds.java");
    const ITEM_IDS_JAVA: &str = vibecraft_java_source!("/net/minecraft/references/ItemIds.java");

    #[test]
    fn block_ids_match_java_reference_keys_and_existing_block_registry() {
        assert_java_contains(
            BLOCK_IDS_JAVA,
            &[
                "public static final ResourceKey<Block> PUMPKIN = createKey(\"pumpkin\");",
                "public static final ResourceKey<Block> PUMPKIN_STEM = createKey(\"pumpkin_stem\");",
                "public static final ResourceKey<Block> ATTACHED_PUMPKIN_STEM = createKey(\"attached_pumpkin_stem\");",
                "public static final ResourceKey<Block> MELON = createKey(\"melon\");",
                "public static final ResourceKey<Block> MELON_STEM = createKey(\"melon_stem\");",
                "public static final ResourceKey<Block> ATTACHED_MELON_STEM = createKey(\"attached_melon_stem\");",
                "public static final ResourceKey<Block> DIRT = createKey(\"dirt\");",
                "return ResourceKey.create(Registries.BLOCK, Identifier.withDefaultNamespace(name));",
            ],
        );
        assert_eq!(
            BLOCK_IDS,
            &[
                ReferenceResourceKey::block("minecraft:pumpkin"),
                ReferenceResourceKey::block("minecraft:pumpkin_stem"),
                ReferenceResourceKey::block("minecraft:attached_pumpkin_stem"),
                ReferenceResourceKey::block("minecraft:melon"),
                ReferenceResourceKey::block("minecraft:melon_stem"),
                ReferenceResourceKey::block("minecraft:attached_melon_stem"),
                ReferenceResourceKey::block("minecraft:dirt"),
            ]
        );
        for key in BLOCK_IDS {
            assert_eq!(key.registry, "minecraft:block");
            assert!(
                crate::block_metadata::registry_entry_by_id(key.location).is_some(),
                "missing referenced block id {}",
                key.location
            );
        }
    }

    #[test]
    fn item_ids_match_java_reference_keys_and_existing_item_registry() {
        assert_java_contains(
            ITEM_IDS_JAVA,
            &[
                "public static final ResourceKey<Item> PUMPKIN_SEEDS = createKey(\"pumpkin_seeds\");",
                "public static final ResourceKey<Item> MELON_SEEDS = createKey(\"melon_seeds\");",
                "return ResourceKey.create(Registries.ITEM, Identifier.withDefaultNamespace(name));",
            ],
        );
        assert_eq!(
            ITEM_IDS,
            &[
                ReferenceResourceKey::item("minecraft:pumpkin_seeds"),
                ReferenceResourceKey::item("minecraft:melon_seeds"),
            ]
        );
        for key in ITEM_IDS {
            assert_eq!(key.registry, "minecraft:item");
            assert_eq!(
                crate::item_catalog::item_static_name(key.location),
                Some(key.location),
                "missing referenced item id {}",
                key.location
            );
        }
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
