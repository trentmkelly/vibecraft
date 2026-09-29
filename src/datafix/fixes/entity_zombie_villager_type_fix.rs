//! Port of `net.minecraft.util.datafix.fixes.EntityZombieVillagerTypeFix`.

use rand::Rng;

use crate::datafix::dynamic::{get, get_bool_or, get_i32_or, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::storage::nbt::Tag;

use super::named_entity_fix::named_entity_fix;

const PROFESSION_MAX: i32 = 6;

/// `new EntityZombieVillagerTypeFix(schema, changesType)`.
pub fn fix() -> Fix {
    named_entity_fix("EntityZombieVillagerTypeFix", r::ENTITY, "Zombie", fix_tag)
}

/// `EntityZombieVillagerTypeFix.fixTag`.
fn fix_tag(input: &mut Tag) {
    if !get_bool_or(input, "IsVillager", false) {
        return;
    }
    if get(input, "ZombieType").is_none() {
        let mut zombie_type = villager_profession(get_i32_or(input, "VillagerProfession", -1));
        if zombie_type == -1 {
            zombie_type = villager_profession(rand::thread_rng().gen_range(0..PROFESSION_MAX));
        }
        set(input, "ZombieType", Tag::Int(zombie_type));
    }
    remove(input, "IsVillager");
}

fn villager_profession(profession: i32) -> i32 {
    if (0..PROFESSION_MAX).contains(&profession) {
        profession
    } else {
        -1
    }
}
