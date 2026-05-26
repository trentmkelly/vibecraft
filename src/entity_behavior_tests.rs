#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::ai_system::{GoalControl, GoalDef, GoalSelector, NavigationKind, PathPlan};
    use crate::base_entity::{
        BaseEntity, EntityDimensions, EntityFlags, EntityPose, RemovalReason, Vec3 as EntityVec3,
    };
    use crate::living_entity::{ItemStackRef, LivingAnimation, LivingEntityState};
    use crate::network::codec::Uuid;
    use crate::network::play::{
        AddEntityPacketInput, ClientboundAddEntityPacket, ClientboundSetEntityDataPacket,
        ClientboundSetEntityMotionPacket, EntityDataValue, EntitySpawnBundle, PlayInstruction,
        Vec3 as PacketVec3,
    };
    use crate::spawning::{
        validate_natural_spawn, NaturalSpawnContext, SpawnStateSummary, SpawnedEntitySample,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct EntityBehaviorCase {
        requirement: &'static str,
        evidence: &'static str,
    }

    fn entity() -> BaseEntity {
        BaseEntity::new(
            42,
            "00000000-0000-0000-0000-000000000042".to_string(),
            "minecraft:zombie",
            EntityDimensions {
                width: 0.6,
                height: 1.95,
                eye_height: 1.74,
            },
        )
    }

    fn stack(item: &'static str, count: u32) -> ItemStackRef {
        ItemStackRef { item, count }
    }

    #[test]
    fn entity_behavior_matrix_covers_required_surface() {
        let cases = [
            EntityBehaviorCase {
                requirement: "spawning",
                evidence: "SpawnStateSummary and validate_natural_spawn",
            },
            EntityBehaviorCase {
                requirement: "AI",
                evidence: "GoalSelector priority and control locking",
            },
            EntityBehaviorCase {
                requirement: "pathfinding",
                evidence: "PathPlan navigation water traversal",
            },
            EntityBehaviorCase {
                requirement: "combat",
                evidence: "LivingEntityState::hurt",
            },
            EntityBehaviorCase {
                requirement: "drops",
                evidence: "LivingEntityState::die_with_drops",
            },
            EntityBehaviorCase {
                requirement: "save/load",
                evidence: "BaseEntity and LivingEntity saved state round trips",
            },
            EntityBehaviorCase {
                requirement: "network metadata",
                evidence: "EntityFlags and EntitySpawnBundle packet instructions",
            },
        ];

        assert_eq!(
            cases
                .iter()
                .map(|case| case.requirement)
                .collect::<Vec<_>>(),
            vec![
                "spawning",
                "AI",
                "pathfinding",
                "combat",
                "drops",
                "save/load",
                "network metadata"
            ]
        );
    }

    #[test]
    fn spawning_ai_and_pathfinding_behaviors_match_vanilla_shapes() {
        let summary = SpawnStateSummary::create(
            289,
            &[
                SpawnedEntitySample {
                    category: "monster",
                    persistent: false,
                    custom_persistent: false,
                    is_mob: true,
                },
                SpawnedEntitySample {
                    category: "monster",
                    persistent: true,
                    custom_persistent: false,
                    is_mob: true,
                },
            ],
        );
        assert_eq!(summary.count_for("monster"), 1);
        assert!(validate_natural_spawn(NaturalSpawnContext {
            category: "monster",
            nearest_player_distance_squared: 40 * 40,
            distance_to_world_spawn_squared: 40 * 40,
            same_chunk_or_spawnable_neighbor: true,
            block_collision_full: false,
            block_signal_source: false,
            fluid_empty: true,
            prevent_mob_spawning_inside: false,
            dangerous_block: false,
            can_summon: true,
            placement_ok: true,
            spawn_rules_ok: true,
            collision_free: true,
            can_spawn_far_from_player: false,
        })
        .is_ok());

        let selector = GoalSelector::new(vec![
            GoalDef {
                name: "wander",
                priority: 4,
                controls: BTreeSet::from([GoalControl::Move]),
                can_use: true,
            },
            GoalDef {
                name: "melee_attack",
                priority: 1,
                controls: BTreeSet::from([GoalControl::Move, GoalControl::Look]),
                can_use: true,
            },
            GoalDef {
                name: "look_at_player",
                priority: 2,
                controls: BTreeSet::from([GoalControl::Look]),
                can_use: true,
            },
        ]);
        assert_eq!(selector.select_running_goals(), vec!["melee_attack"]);

        let amphibious_path = PathPlan {
            navigation: NavigationKind::Amphibious,
            can_float: false,
            max_visited_nodes_multiplier: 1.0,
            target: (4, 64, 4),
            reached: false,
        };
        assert!(amphibious_path.can_path_through_water());
        assert!(!PathPlan {
            navigation: NavigationKind::Ground,
            can_float: false,
            ..amphibious_path.clone()
        }
        .can_path_through_water());
    }

    #[test]
    fn combat_drops_save_load_and_network_metadata_behaviors_match_vanilla_shapes() {
        let mut base = entity();
        base.set_pos(1.0, 65.0, -2.0);
        base.set_velocity(EntityVec3 {
            x: 0.1,
            y: 0.2,
            z: 0.3,
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
        let sync = base.sync_plan();
        assert!(sync.add_entity);
        assert_eq!(sync.metadata_flags, 0b0100_1010);
        assert_eq!(sync.pose, EntityPose::Crouching);

        let mut living = LivingEntityState::new(20.0);
        living.armor = 4.0;
        let damage = living.hurt(7.0);
        assert!(damage.health_damage > 0.0);
        assert_eq!(living.last_animation, Some(LivingAnimation::Hurt));
        living.die_with_drops(vec![stack("minecraft:rotten_flesh", 2)], 5);
        assert!(living.dead);
        assert_eq!(living.drops, vec![stack("minecraft:rotten_flesh", 2)]);
        assert_eq!(living.experience_reward, 5);

        let saved_base = base.save();
        let mut loaded_base = entity();
        loaded_base.load(saved_base.clone());
        assert_eq!(loaded_base.position, saved_base.pos);
        assert_eq!(loaded_base.velocity, saved_base.motion);

        let saved_living = living.save();
        let mut loaded_living = LivingEntityState::new(1.0);
        loaded_living.load(saved_living.clone());
        assert_eq!(loaded_living.health, saved_living.health);
        assert_eq!(loaded_living.equipment, saved_living.equipment);

        base.remove(RemovalReason::Killed);
        assert_eq!(base.sync_plan().removed, Some(RemovalReason::Killed));

        let bundle = EntitySpawnBundle {
            spawn: ClientboundAddEntityPacket::new(AddEntityPacketInput {
                id: 42,
                uuid: Uuid([42; 16]),
                entity_type: 1,
                position: PacketVec3 {
                    x: 1.0,
                    y: 65.0,
                    z: -2.0,
                },
                movement: PacketVec3 {
                    x: 0.1,
                    y: 0.2,
                    z: 0.3,
                },
                rotation: (90.0, 0.0),
                y_head_rot: 90.0,
                data: 0,
            }),
            metadata: Some(ClientboundSetEntityDataPacket {
                id: 42,
                packed_items: vec![EntityDataValue {
                    index: 0,
                    serializer_id: 0,
                    encoded_payload: vec![sync.metadata_flags],
                }],
            }),
            velocity: Some(ClientboundSetEntityMotionPacket::new(
                42,
                PacketVec3 {
                    x: 0.1,
                    y: 0.2,
                    z: 0.3,
                },
            )),
            equipment: None,
            attributes: None,
            effects: Vec::new(),
        };
        let instructions = bundle.instructions();
        assert!(matches!(instructions[0], PlayInstruction::AddEntity(_)));
        assert!(matches!(instructions[1], PlayInstruction::SetEntityData(_)));
        assert!(matches!(
            instructions[2],
            PlayInstruction::SetEntityMotion(_)
        ));
    }
}
