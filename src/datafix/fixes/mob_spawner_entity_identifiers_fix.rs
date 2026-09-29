//! Port of `net.minecraft.util.datafix.fixes.MobSpawnerEntityIdentifiersFix`.

use crate::datafix::decode::decode;
use crate::datafix::dynamic::{get, get_str, get_str_or, list_items, remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::template::Tmpl;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

/// `new MobSpawnerEntityIdentifiersFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::everywhere_with(
        "MobSpawnerEntityIdentifiersFix",
        Target::Type(r::UNTAGGED_SPAWNER),
        |ctx, spawner| {
            let mut fixed = spawner.clone();
            fix_spawner(&mut fixed);
            // `newType.readTyped(..)`: keep the input when the new type cannot read it.
            if decode(ctx.output, &Tmpl::Ref(r::UNTAGGED_SPAWNER), &mut fixed).is_ok() {
                *spawner = fixed;
            }
        },
    )
}

/// `MobSpawnerEntityIdentifiersFix.fix` (the id is always `MobSpawner` there).
fn fix_spawner(input: &mut Tag) {
    if let Some(entity_id) = get_str(input, "EntityId").map(str::to_string) {
        let mut spawn_data = match get(input, "SpawnData") {
            Some(existing) => existing.clone(),
            None => Tag::Compound(Vec::new()),
        };
        let id = if entity_id.is_empty() {
            "Pig".to_string()
        } else {
            entity_id
        };
        set(&mut spawn_data, "id", Tag::String(id));
        set(input, "SpawnData", spawn_data);
        remove(input, "EntityId");
    }
    if let Some(potentials) = get(input, "SpawnPotentials").and_then(list_items) {
        let fixed = potentials.into_iter().map(fix_spawn_potential).collect();
        set(input, "SpawnPotentials", Tag::List(fixed));
    }
}

fn fix_spawn_potential(mut potential: Tag) -> Tag {
    let Some(entity_type) = get_str(&potential, "Type").map(str::to_string) else {
        return potential;
    };
    let mut spawn_data = match get(&potential, "Properties") {
        Some(properties) => properties.clone(),
        None => Tag::Compound(Vec::new()),
    };
    set(&mut spawn_data, "id", Tag::String(entity_type));
    set(&mut potential, "Entity", spawn_data);
    remove(&mut potential, "Type");
    remove(&mut potential, "Properties");
    let _ = get_str_or;
    potential
}
