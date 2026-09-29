//! Port of `net.minecraft.util.datafix.fixes.EntityHealthFix`.

use crate::datafix::dynamic::{as_f32, get, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new EntityHealthFix(schema, changesType)`: `Health`/`HealF` become a float `Health`.
pub fn fix() -> Fix {
    Fix::everywhere("EntityHealthFix", Target::Type(r::ENTITY), fix_tag)
}

/// `EntityHealthFix.fixTag`.
fn fix_tag(input: &mut Tag) {
    let old_heal_f = get(input, "HealF").and_then(as_f32);
    let old_health = get(input, "Health").and_then(as_f32);
    let health = match (old_heal_f, old_health) {
        (Some(heal_f), _) => {
            remove(input, "HealF");
            heal_f
        }
        (None, Some(health)) => health,
        (None, None) => return,
    };
    set(input, "Health", Tag::Float(health));
}
