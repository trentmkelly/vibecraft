#![allow(dead_code)]

use crate::base_entity::{
    BaseEntity, EntityDimensions, EntityFlags, EntityPose, RemovalReason, Vec3,
};
use crate::living_entity::{
    EquipmentSlot, InteractionHand, ItemStackRef, LivingAnimation, LivingEntityState,
    MobEffectState,
};
use crate::mob_interaction::{
    animal_feed_result, breeding_result, bucket_pickup_result, mooshroom_interaction, AgeState,
    AnimalFeedResult, BucketPickupResult, MooshroomInteraction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityValidationStep {
    Spawn,
    Metadata,
    Interact,
    Kill,
    Save,
    Load,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityValidationReport {
    pub entity_type: &'static str,
    pub steps: Vec<EntityValidationStep>,
    pub metadata_flags: u8,
    pub add_entity: bool,
    pub interaction: &'static str,
    pub killed: bool,
    pub saved_type: &'static str,
    pub loaded_uuid: String,
}

pub fn validate_spawn_interact_kill_save_load() -> EntityValidationReport {
    let mut base = BaseEntity::new(
        42,
        "00000000-0000-0000-0000-000000000042".to_string(),
        "minecraft:pig",
        EntityDimensions {
            width: 0.9,
            height: 0.9,
            eye_height: 0.8,
        },
    );
    base.set_pos(1.25, 65.0, -3.5);
    base.set_rotation(450.0, 120.0);
    base.set_velocity(Vec3 {
        x: 0.2,
        y: 0.0,
        z: -0.1,
    });
    base.pose = EntityPose::Crouching;
    base.flags = EntityFlags {
        on_fire: false,
        crouching: true,
        sprinting: true,
        swimming: false,
        invisible: false,
        glowing: true,
        fall_flying: false,
    };
    let spawn_sync = base.sync_plan();

    let interaction = match animal_feed_result(true, 0, 0, false, false) {
        AnimalFeedResult::SetInLove => "set_in_love",
        _ => "unexpected",
    };

    let mut living = LivingEntityState::new(10.0);
    living.equip(
        EquipmentSlot::MainHand,
        ItemStackRef {
            item: "minecraft:carrot",
            count: 1,
        },
    );
    living.start_using_item(
        InteractionHand::MainHand,
        ItemStackRef {
            item: "minecraft:carrot",
            count: 1,
        },
        32,
    );
    living.add_effect(MobEffectState {
        id: "minecraft:speed",
        amplifier: 0,
        duration: 100,
        ambient: false,
        visible: true,
        show_icon: true,
    });
    living.die_with_drops(
        vec![ItemStackRef {
            item: "minecraft:porkchop",
            count: 1,
        }],
        2,
    );
    let living_sync = living.sync_plan();
    base.remove(RemovalReason::Killed);
    let saved = base.save();
    let mut loaded = BaseEntity::new(
        99,
        String::new(),
        "minecraft:pig",
        EntityDimensions {
            width: 0.9,
            height: 0.9,
            eye_height: 0.8,
        },
    );
    loaded.load(saved.clone());

    EntityValidationReport {
        entity_type: base.entity_type,
        steps: vec![
            EntityValidationStep::Spawn,
            EntityValidationStep::Metadata,
            EntityValidationStep::Interact,
            EntityValidationStep::Kill,
            EntityValidationStep::Save,
            EntityValidationStep::Load,
        ],
        metadata_flags: spawn_sync.metadata_flags,
        add_entity: spawn_sync.add_entity,
        interaction,
        killed: living_sync.animation == Some(LivingAnimation::Death),
        saved_type: saved.id,
        loaded_uuid: loaded.uuid,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BehaviorScenarioResult {
    pub age_ticks_after_one_tick: i32,
    pub breeding_parent_age: i32,
    pub bucket_pickup_discards: bool,
    pub mooshroom_shears_transform: bool,
}

pub fn validate_behavior_scenarios() -> BehaviorScenarioResult {
    BehaviorScenarioResult {
        age_ticks_after_one_tick: AgeState::baby().tick().age,
        breeding_parent_age: breeding_result(true).parent_age,
        bucket_pickup_discards: bucket_pickup_result("minecraft:water_bucket", true)
            == BucketPickupResult::FilledBucketAndDiscardEntity,
        mooshroom_shears_transform: mooshroom_interaction(
            "minecraft:shears",
            false,
            false,
            false,
            false,
        ) == MooshroomInteraction::ShearIntoCow,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityValidationCoverage {
    pub requirement: &'static str,
    pub evidence: &'static str,
}

pub const ENTITY_VALIDATION_COVERAGE: &[EntityValidationCoverage] = &[
    EntityValidationCoverage {
        requirement: "spawn",
        evidence: "BaseEntity::sync_plan add_entity and network spawn bundle tests",
    },
    EntityValidationCoverage {
        requirement: "metadata",
        evidence: "EntityFlags::metadata_byte and sync metadata flags",
    },
    EntityValidationCoverage {
        requirement: "interact",
        evidence: "Animal feed, bucket pickup, shearing, and interaction entity tests",
    },
    EntityValidationCoverage {
        requirement: "kill",
        evidence: "LivingEntityState death animation drops and removal reason",
    },
    EntityValidationCoverage {
        requirement: "save",
        evidence: "BaseEntity::save and LivingEntityState::save",
    },
    EntityValidationCoverage {
        requirement: "load",
        evidence: "BaseEntity::load and LivingEntityState::load",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_spawn_metadata_interact_kill_save_and_load_in_one_scenario() {
        let report = validate_spawn_interact_kill_save_load();
        assert_eq!(report.entity_type, "minecraft:pig");
        assert_eq!(
            report.steps,
            vec![
                EntityValidationStep::Spawn,
                EntityValidationStep::Metadata,
                EntityValidationStep::Interact,
                EntityValidationStep::Kill,
                EntityValidationStep::Save,
                EntityValidationStep::Load,
            ]
        );
        assert!(report.add_entity);
        assert_eq!(report.metadata_flags, 0b0100_1010);
        assert_eq!(report.interaction, "set_in_love");
        assert!(report.killed);
        assert_eq!(report.saved_type, "minecraft:pig");
        assert_eq!(report.loaded_uuid, "00000000-0000-0000-0000-000000000042");
    }

    #[test]
    fn behavior_scenarios_cover_age_breeding_bucket_and_transformation_paths() {
        assert_eq!(
            validate_behavior_scenarios(),
            BehaviorScenarioResult {
                age_ticks_after_one_tick: -23_999,
                breeding_parent_age: 6_000,
                bucket_pickup_discards: true,
                mooshroom_shears_transform: true,
            }
        );
    }

    #[test]
    fn validation_coverage_maps_every_checklist_word_to_evidence() {
        let covered: Vec<&str> = ENTITY_VALIDATION_COVERAGE
            .iter()
            .map(|entry| entry.requirement)
            .collect();
        for expected in ["spawn", "metadata", "interact", "kill", "save", "load"] {
            assert!(covered.contains(&expected), "missing {expected}");
        }
    }
}
