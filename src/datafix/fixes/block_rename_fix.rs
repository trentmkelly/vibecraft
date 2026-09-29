//! Port of `net.minecraft.util.datafix.fixes.BlockRenameFix`.

use std::sync::Arc;

use crate::datafix::dynamic::{get_str, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `BlockRenameFix.create(schema, name, renamer)`: renames blocks in block names,
/// block states and flat block states (`name[properties]{nbt}`).
pub fn block_rename_fix(
    name: &str,
    renamer: impl Fn(&str) -> String + Send + Sync + 'static,
) -> Fix {
    let renamer = Arc::new(renamer);
    let for_block = Arc::clone(&renamer);
    let for_state = Arc::clone(&renamer);
    let for_flat = renamer;
    Fix::sequence(
        name,
        vec![
            Fix::everywhere(
                format!("{name} for block"),
                Target::Type(r::BLOCK_NAME),
                move |block| {
                    if let Tag::String(value) = block {
                        *value = for_block(value);
                    }
                },
            ),
            Fix::everywhere(
                format!("{name} for block_state"),
                Target::Type(r::BLOCK_STATE),
                move |state| {
                    if let Some(block) = get_str(state, "Name").map(&*for_state) {
                        set(state, "Name", Tag::String(block));
                    }
                },
            ),
            Fix::everywhere(
                format!("{name} for flat_block_state"),
                Target::Type(r::FLAT_BLOCK_STATE),
                move |flat| {
                    if let Tag::String(value) = flat {
                        *value = fix_flat_block_state(value, &*for_flat);
                    }
                },
            ),
        ],
    )
}

/// `BlockRenameFix.fixFlatBlockState`.
fn fix_flat_block_state(string: &str, rename: &dyn Fn(&str) -> String) -> String {
    let mut end = string.len();
    if let Some(start_properties) = string.find('[').filter(|index| *index > 0) {
        end = start_properties;
    }
    if let Some(start_nbt) = string.find('{').filter(|index| *index > 0) {
        end = end.min(start_nbt);
    }
    format!("{}{}", rename(&string[..end]), &string[end..])
}

#[cfg(test)]
mod tests {
    use super::fix_flat_block_state;

    fn rename(name: &str) -> String {
        if name == "minecraft:old" {
            "minecraft:new".to_string()
        } else {
            name.to_string()
        }
    }

    #[test]
    fn flat_block_states_keep_their_properties_and_nbt() {
        assert_eq!(
            fix_flat_block_state("minecraft:old", &rename),
            "minecraft:new"
        );
        assert_eq!(
            fix_flat_block_state("minecraft:old[a=b,c=d]", &rename),
            "minecraft:new[a=b,c=d]"
        );
        assert_eq!(
            fix_flat_block_state("minecraft:old{x:1}", &rename),
            "minecraft:new{x:1}"
        );
        assert_eq!(
            fix_flat_block_state("minecraft:old{x:1}[a=b]", &rename),
            "minecraft:new{x:1}[a=b]"
        );
        assert_eq!(
            fix_flat_block_state("minecraft:old[a={b}]", &rename),
            "minecraft:new[a={b}]"
        );
        // A leading bracket does not split (`startProperties > 0` in Java).
        assert_eq!(fix_flat_block_state("[weird", &rename), "[weird");
        assert_eq!(
            fix_flat_block_state("minecraft:other", &rename),
            "minecraft:other"
        );
    }
}
