//! Port of `net.minecraft.util.datafix.fixes.EntityRidingToPassengersFix`.

use crate::datafix::dynamic::{remove, set};
use crate::datafix::fix::Fix;
use crate::datafix::references as r;
use crate::datafix::typed::typed;
use crate::datafix::walk::Target;
use crate::storage::nbt::Tag;

use super::write_and_read_fix::write_and_read_fix;

/// `new EntityRidingToPassengersFix(schema, changesType)`.
pub fn fix() -> Fix {
    Fix::sequence(
        "EntityRidingToPassengersFix",
        vec![
            Fix::everywhere(
                "EntityRidingToPassengerFix",
                Target::Type(r::ENTITY_TREE),
                riding_to_passengers,
            ),
            write_and_read_fix("player RootVehicle injecter", r::PLAYER),
        ],
    )
}

/// Turns a `Riding` chain (rider containing the mount) into a `Passengers` tree
/// rooted at the bottom-most mount.
fn riding_to_passengers(tree: &mut Tag) {
    let mut passenger: Option<Tag> = None;
    let mut updating = std::mem::replace(tree, Tag::End);
    loop {
        // An unreadable `Riding` field is absent from the typed value.
        let riding = typed(&updating, "Riding").cloned();
        remove(&mut updating, "Riding");
        if let Some(passenger) = passenger.take() {
            set(&mut updating, "Passengers", Tag::List(vec![passenger]));
        }
        passenger = Some(updating);
        match riding {
            Some(mount @ Tag::Compound(_)) => updating = mount,
            _ => break,
        }
    }
    if let Some(root) = passenger {
        *tree = root;
    }
}
