//! Port of `net.minecraft.util.datafix.fixes.DyeItemRenameFix`.

use crate::datafix::fix::Fix;

use super::item_rename_fix::item_rename_fix;

/// `DyeItemRenameFix.RENAMED_IDS`.
pub const RENAMED_IDS: &[(&str, &str)] = &[
    ("minecraft:cactus_green", "minecraft:green_dye"),
    ("minecraft:rose_red", "minecraft:red_dye"),
    ("minecraft:dandelion_yellow", "minecraft:yellow_dye"),
];

/// `ItemRenameFix.create(schema, "Rename dye items", createRenamer(RENAMED_IDS))`.
pub fn fix() -> Fix {
    item_rename_fix("Rename dye items", |name| {
        RENAMED_IDS
            .iter()
            .find(|(old, _)| *old == name)
            .map_or(name, |(_, new)| *new)
            .to_string()
    })
}

#[cfg(test)]
mod tests {
    use crate::datafix::fixer::DataFixerBuilder;
    use crate::datafix::references as r;
    use crate::datafix::template::dsl;
    use crate::storage::nbt::Tag;

    use super::{fix, RENAMED_IDS};

    #[test]
    fn renames_only_the_three_java_dye_ids() {
        let mut builder = DataFixerBuilder::new();
        let schema = builder.add_schema(1800, 0, |schema| {
            schema.register_type(r::ITEM_NAME, dsl::string());
        });
        builder.add_fixer(schema, false, fix());
        let fixer = builder.build(1800);

        for &(old, new) in RENAMED_IDS {
            assert_eq!(
                fixer.update(r::ITEM_NAME, &Tag::String(old.to_string()), 0, 1800),
                Ok(Tag::String(new.to_string())),
                "{old} should rename to {new}"
            );
        }
        assert_eq!(
            fixer.update(
                r::ITEM_NAME,
                &Tag::String("minecraft:other_item".to_string()),
                0,
                1800
            ),
            Ok(Tag::String("minecraft:other_item".to_string()))
        );
    }
}
