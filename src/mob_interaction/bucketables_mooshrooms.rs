#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BucketEntityData {
    pub no_ai: bool,
    pub silent: bool,
    pub no_gravity: bool,
    pub glowing: bool,
    pub invulnerable: bool,
    pub health_saved: bool,
}

pub fn save_default_bucket_data(
    no_ai: bool,
    silent: bool,
    no_gravity: bool,
    glowing: bool,
    invulnerable: bool,
) -> BucketEntityData {
    BucketEntityData {
        no_ai,
        silent,
        no_gravity,
        glowing,
        invulnerable,
        health_saved: true,
    }
}

pub fn ready_for_shearing(alive: bool, baby: bool, already_sheared: bool) -> bool {
    alive && !baby && !already_sheared
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MooshroomInteraction {
    FillStew,
    FillSuspiciousStew,
    ShearIntoCow,
    StoreSuspiciousEffects,
    Delegate,
}

pub fn mooshroom_interaction(
    item: &str,
    baby: bool,
    brown: bool,
    has_stew_effects: bool,
    item_has_suspicious_effect: bool,
) -> MooshroomInteraction {
    if item == "minecraft:bowl" && !baby {
        if has_stew_effects {
            MooshroomInteraction::FillSuspiciousStew
        } else {
            MooshroomInteraction::FillStew
        }
    } else if item == "minecraft:shears" && ready_for_shearing(true, baby, false) {
        MooshroomInteraction::ShearIntoCow
    } else if brown && !baby && item_has_suspicious_effect {
        MooshroomInteraction::StoreSuspiciousEffects
    } else {
        MooshroomInteraction::Delegate
    }
}
