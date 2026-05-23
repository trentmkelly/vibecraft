use super::*;

pub fn chunk_generation_mob_entity_nbt(snap: ChunkGenerationMobEntitySnapPlan, uuid: &str) -> Tag {
    let health = chunk_generation_mob_default_health(snap.entity_type);
    let mut fields = vec![
        ("id".to_string(), Tag::String(snap.entity_type.to_string())),
        ("UUID".to_string(), Tag::String(uuid.to_string())),
        (
            "Pos".to_string(),
            Tag::List(vec![
                Tag::Double(snap.x),
                Tag::Double(snap.y),
                Tag::Double(snap.z),
            ]),
        ),
        (
            "Rotation".to_string(),
            Tag::List(vec![Tag::Float(snap.yaw), Tag::Float(snap.pitch)]),
        ),
        (
            "Motion".to_string(),
            Tag::List(vec![Tag::Double(0.0), Tag::Double(0.0), Tag::Double(0.0)]),
        ),
        ("fall_distance".to_string(), Tag::Double(0.0)),
        ("Fire".to_string(), Tag::Short(0)),
        ("Air".to_string(), Tag::Short(300)),
        ("OnGround".to_string(), Tag::Byte(0)),
        ("Invulnerable".to_string(), Tag::Byte(0)),
        ("PortalCooldown".to_string(), Tag::Int(0)),
        ("Health".to_string(), Tag::Float(health)),
        ("HurtTime".to_string(), Tag::Short(0)),
        ("HurtByTimestamp".to_string(), Tag::Int(0)),
        ("DeathTime".to_string(), Tag::Short(0)),
        ("AbsorptionAmount".to_string(), Tag::Float(0.0)),
        (
            "current_impulse_context_reset_grace_time".to_string(),
            Tag::Int(0),
        ),
        ("CanPickUpLoot".to_string(), Tag::Byte(0)),
        ("PersistenceRequired".to_string(), Tag::Byte(0)),
        ("LeftHanded".to_string(), Tag::Byte(0)),
    ];
    if chunk_generation_mob_is_ageable(snap.entity_type) {
        fields.push(("Age".to_string(), Tag::Int(0)));
        fields.push(("ForcedAge".to_string(), Tag::Int(0)));
        fields.push(("AgeLocked".to_string(), Tag::Byte(0)));
    }
    if chunk_generation_mob_is_animal(snap.entity_type) {
        fields.push(("InLove".to_string(), Tag::Int(0)));
    }
    if chunk_generation_mob_is_abstract_horse(snap.entity_type) {
        fields.push(("EatingHaystack".to_string(), Tag::Byte(0)));
        fields.push(("Bred".to_string(), Tag::Byte(0)));
        fields.push(("Temper".to_string(), Tag::Int(0)));
        fields.push(("Tame".to_string(), Tag::Byte(0)));
    }
    if chunk_generation_mob_is_neutral(snap.entity_type) {
        fields.push(("anger_end_time".to_string(), Tag::Long(0)));
    }
    if chunk_generation_mob_is_patrolling_monster(snap.entity_type) {
        fields.push(("PatrolLeader".to_string(), Tag::Byte(0)));
        fields.push(("Patrolling".to_string(), Tag::Byte(0)));
    }
    if chunk_generation_mob_is_raider(snap.entity_type) {
        fields.push(("Wave".to_string(), Tag::Int(0)));
        fields.push(("CanJoinRaid".to_string(), Tag::Byte(0)));
    }
    append_chunk_generation_mob_specific_save_fields(snap.entity_type, &mut fields);
    Tag::Compound(fields)
}

fn append_chunk_generation_mob_specific_save_fields(
    entity_type: &str,
    fields: &mut Vec<(String, Tag)>,
) {
    match entity_type {
        "minecraft:armadillo" => {
            fields.push(("state".to_string(), Tag::String("idle".to_string())));
        }
        "minecraft:axolotl" => {
            fields.push(("Variant".to_string(), Tag::Int(0)));
            fields.push(("FromBucket".to_string(), Tag::Byte(0)));
        }
        "minecraft:bee" => {
            fields.push(("HasNectar".to_string(), Tag::Byte(0)));
            fields.push(("HasStung".to_string(), Tag::Byte(0)));
            fields.push(("TicksSincePollination".to_string(), Tag::Int(0)));
            fields.push(("CannotEnterHiveTicks".to_string(), Tag::Int(0)));
            fields.push(("CropsGrownSincePollination".to_string(), Tag::Int(0)));
        }
        "minecraft:bat" => {
            fields.push(("BatFlags".to_string(), Tag::Byte(0)));
        }
        "minecraft:bogged" => {
            fields.push(("sheared".to_string(), Tag::Byte(0)));
        }
        "minecraft:camel" => {
            fields.push(("LastPoseTick".to_string(), Tag::Long(0)));
        }
        "minecraft:cat" | "minecraft:wolf" => {
            let variant = if entity_type == "minecraft:cat" {
                "minecraft:black"
            } else {
                "minecraft:pale"
            };
            fields.push(("variant".to_string(), Tag::String(variant.to_string())));
            fields.push((
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string()),
            ));
            fields.push(("CollarColor".to_string(), Tag::Byte(14)));
        }
        "minecraft:chicken" => {
            fields.push(("IsChickenJockey".to_string(), Tag::Byte(0)));
            fields.push(("EggLayTime".to_string(), Tag::Int(6000)));
            fields.push((
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string()),
            ));
            fields.push((
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string()),
            ));
        }
        "minecraft:cow" => {
            fields.push((
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string()),
            ));
            fields.push((
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string()),
            ));
        }
        "minecraft:creeper" => {
            fields.push(("powered".to_string(), Tag::Byte(0)));
            fields.push(("Fuse".to_string(), Tag::Short(30)));
            fields.push(("ExplosionRadius".to_string(), Tag::Byte(3)));
            fields.push(("ignited".to_string(), Tag::Byte(0)));
        }
        "minecraft:endermite" => {
            fields.push(("Lifetime".to_string(), Tag::Int(0)));
        }
        "minecraft:ghast" => {
            fields.push(("ExplosionPower".to_string(), Tag::Byte(1)));
        }
        "minecraft:goat" => {
            fields.push(("IsScreamingGoat".to_string(), Tag::Byte(0)));
            fields.push(("HasLeftHorn".to_string(), Tag::Byte(1)));
            fields.push(("HasRightHorn".to_string(), Tag::Byte(1)));
        }
        "minecraft:horse" => {
            fields.push(("Variant".to_string(), Tag::Int(0)));
        }
        "minecraft:hoglin" => {
            fields.push(("IsImmuneToZombification".to_string(), Tag::Byte(0)));
            fields.push(("TimeInOverworld".to_string(), Tag::Int(0)));
            fields.push(("CannotBeHunted".to_string(), Tag::Byte(0)));
        }
        "minecraft:iron_golem" => {
            fields.push(("PlayerCreated".to_string(), Tag::Byte(0)));
        }
        "minecraft:llama" => {
            fields.push(("ChestedHorse".to_string(), Tag::Byte(0)));
            fields.push(("Variant".to_string(), Tag::Int(0)));
            fields.push(("Strength".to_string(), Tag::Int(0)));
        }
        "minecraft:rabbit" => {
            fields.push(("RabbitType".to_string(), Tag::Int(0)));
            fields.push(("MoreCarrotTicks".to_string(), Tag::Int(0)));
        }
        "minecraft:ravager" => {
            fields.push(("AttackTick".to_string(), Tag::Int(0)));
            fields.push(("StunTick".to_string(), Tag::Int(0)));
            fields.push(("RoarTick".to_string(), Tag::Int(0)));
        }
        "minecraft:donkey" | "minecraft:mule" => {
            fields.push(("ChestedHorse".to_string(), Tag::Byte(0)));
        }
        "minecraft:cod" => {
            fields.push(("FromBucket".to_string(), Tag::Byte(0)));
        }
        "minecraft:dolphin" => {
            fields.push(("GotFish".to_string(), Tag::Byte(0)));
            fields.push(("Moistness".to_string(), Tag::Int(2400)));
        }
        "minecraft:fox" => {
            fields.push(("Sleeping".to_string(), Tag::Byte(0)));
            fields.push(("Sitting".to_string(), Tag::Byte(0)));
            fields.push(("Crouching".to_string(), Tag::Byte(0)));
        }
        "minecraft:frog" => {
            fields.push((
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string()),
            ));
        }
        "minecraft:glow_squid" => {
            fields.push(("DarkTicksRemaining".to_string(), Tag::Int(0)));
        }
        "minecraft:ocelot" => {
            fields.push(("Trusting".to_string(), Tag::Byte(0)));
        }
        "minecraft:mooshroom" => {
            fields.push(("Type".to_string(), Tag::String("red".to_string())));
        }
        "minecraft:panda" => {
            fields.push(("MainGene".to_string(), Tag::String("normal".to_string())));
            fields.push(("HiddenGene".to_string(), Tag::String("normal".to_string())));
        }
        "minecraft:parrot" => {
            fields.push(("Variant".to_string(), Tag::Int(0)));
        }
        "minecraft:pufferfish" => {
            fields.push(("FromBucket".to_string(), Tag::Byte(0)));
            fields.push(("PuffState".to_string(), Tag::Int(0)));
        }
        "minecraft:phantom" => {
            fields.push(("size".to_string(), Tag::Int(0)));
        }
        "minecraft:pig" => {
            fields.push((
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string()),
            ));
            fields.push((
                "sound_variant".to_string(),
                Tag::String("minecraft:classic".to_string()),
            ));
        }
        "minecraft:piglin_brute" => {
            fields.push(("IsImmuneToZombification".to_string(), Tag::Byte(0)));
            fields.push(("TimeInOverworld".to_string(), Tag::Int(0)));
        }
        "minecraft:piglin" => {
            fields.push(("IsImmuneToZombification".to_string(), Tag::Byte(0)));
            fields.push(("TimeInOverworld".to_string(), Tag::Int(0)));
            fields.push(("IsBaby".to_string(), Tag::Byte(0)));
            fields.push(("CannotHunt".to_string(), Tag::Byte(0)));
        }
        "minecraft:sheep" => {
            fields.push(("Sheared".to_string(), Tag::Byte(0)));
            fields.push(("Color".to_string(), Tag::Byte(0)));
        }
        "minecraft:shulker" => {
            fields.push(("AttachFace".to_string(), Tag::Byte(0)));
            fields.push(("Peek".to_string(), Tag::Byte(0)));
            fields.push(("Color".to_string(), Tag::Byte(16)));
        }
        "minecraft:skeleton" | "minecraft:stray" => {
            append_chunk_generation_skeleton_save_fields(fields);
        }
        "minecraft:skeleton_horse" => {
            fields.push(("SkeletonTrap".to_string(), Tag::Byte(0)));
            fields.push(("SkeletonTrapTime".to_string(), Tag::Int(0)));
        }
        "minecraft:slime" | "minecraft:magma_cube" => {
            fields.push(("Size".to_string(), Tag::Int(0)));
            fields.push(("wasOnGround".to_string(), Tag::Byte(0)));
        }
        "minecraft:snow_golem" => {
            fields.push(("Pumpkin".to_string(), Tag::Byte(1)));
        }
        "minecraft:trader_llama" => {
            fields.push(("ChestedHorse".to_string(), Tag::Byte(0)));
            fields.push(("Variant".to_string(), Tag::Int(0)));
            fields.push(("Strength".to_string(), Tag::Int(0)));
            fields.push(("DespawnDelay".to_string(), Tag::Int(47999)));
        }
        "minecraft:salmon" => {
            fields.push(("FromBucket".to_string(), Tag::Byte(0)));
            fields.push(("type".to_string(), Tag::String("medium".to_string())));
        }
        "minecraft:tropical_fish" => {
            fields.push(("FromBucket".to_string(), Tag::Byte(0)));
            fields.push(("Variant".to_string(), Tag::Int(0)));
        }
        "minecraft:turtle" => {
            fields.push((
                "home_pos".to_string(),
                Tag::List(vec![Tag::Int(0), Tag::Int(0), Tag::Int(0)]),
            ));
            fields.push(("has_egg".to_string(), Tag::Byte(0)));
        }
        "minecraft:zoglin" => {
            fields.push(("IsBaby".to_string(), Tag::Byte(0)));
        }
        "minecraft:zombie_nautilus" => {
            fields.push((
                "variant".to_string(),
                Tag::String("minecraft:temperate".to_string()),
            ));
        }
        "minecraft:drowned" | "minecraft:husk" | "minecraft:zombie" => {
            append_chunk_generation_zombie_save_fields(fields);
        }
        "minecraft:zombie_villager" => {
            append_chunk_generation_zombie_save_fields(fields);
            fields.push(("VillagerDataFinalized".to_string(), Tag::Byte(0)));
            fields.push(("ConversionTime".to_string(), Tag::Int(-1)));
            fields.push(("Xp".to_string(), Tag::Int(0)));
        }
        _ => {}
    }
}

fn append_chunk_generation_skeleton_save_fields(fields: &mut Vec<(String, Tag)>) {
    fields.push(("StrayConversionTime".to_string(), Tag::Int(-1)));
}

fn append_chunk_generation_zombie_save_fields(fields: &mut Vec<(String, Tag)>) {
    fields.push(("IsBaby".to_string(), Tag::Byte(0)));
    fields.push(("CanBreakDoors".to_string(), Tag::Byte(0)));
    fields.push(("InWaterTime".to_string(), Tag::Int(-1)));
    fields.push(("DrownedConversionTime".to_string(), Tag::Int(-1)));
}

fn chunk_generation_mob_default_health(entity_type: &str) -> f32 {
    match entity_type {
        "minecraft:armadillo" => 12.0,
        "minecraft:bat" | "minecraft:parrot" => 6.0,
        "minecraft:cat" | "minecraft:ocelot" | "minecraft:rabbit" | "minecraft:sheep" => 8.0,
        "minecraft:chicken" | "minecraft:cod" | "minecraft:pufferfish" | "minecraft:salmon" => 4.0,
        "minecraft:cow" | "minecraft:goat" | "minecraft:mooshroom" | "minecraft:pig" => 10.0,
        "minecraft:creeper"
        | "minecraft:drowned"
        | "minecraft:enderman"
        | "minecraft:husk"
        | "minecraft:spider"
        | "minecraft:stray"
        | "minecraft:witch"
        | "minecraft:zombie"
        | "minecraft:zombified_piglin" => 20.0,
        "minecraft:fox" => 10.0,
        "minecraft:frog" => 10.0,
        "minecraft:glow_squid" | "minecraft:squid" | "minecraft:tropical_fish" => 10.0,
        "minecraft:hoglin" => 40.0,
        "minecraft:piglin" => 16.0,
        "minecraft:polar_bear" => 30.0,
        "minecraft:wolf" => 8.0,
        _ => 20.0,
    }
}

fn chunk_generation_mob_is_ageable(entity_type: &str) -> bool {
    matches!(
        entity_type,
        "minecraft:armadillo"
            | "minecraft:axolotl"
            | "minecraft:camel"
            | "minecraft:cat"
            | "minecraft:chicken"
            | "minecraft:cow"
            | "minecraft:dolphin"
            | "minecraft:donkey"
            | "minecraft:fox"
            | "minecraft:frog"
            | "minecraft:glow_squid"
            | "minecraft:goat"
            | "minecraft:hoglin"
            | "minecraft:horse"
            | "minecraft:llama"
            | "minecraft:mooshroom"
            | "minecraft:mule"
            | "minecraft:ocelot"
            | "minecraft:panda"
            | "minecraft:parrot"
            | "minecraft:pig"
            | "minecraft:polar_bear"
            | "minecraft:rabbit"
            | "minecraft:sheep"
            | "minecraft:squid"
            | "minecraft:strider"
            | "minecraft:trader_llama"
            | "minecraft:turtle"
            | "minecraft:wolf"
            | "minecraft:zombie_horse"
    )
}

fn chunk_generation_mob_is_animal(entity_type: &str) -> bool {
    matches!(
        entity_type,
        "minecraft:armadillo"
            | "minecraft:axolotl"
            | "minecraft:camel"
            | "minecraft:cat"
            | "minecraft:chicken"
            | "minecraft:cow"
            | "minecraft:donkey"
            | "minecraft:fox"
            | "minecraft:frog"
            | "minecraft:goat"
            | "minecraft:hoglin"
            | "minecraft:horse"
            | "minecraft:llama"
            | "minecraft:mooshroom"
            | "minecraft:mule"
            | "minecraft:ocelot"
            | "minecraft:panda"
            | "minecraft:parrot"
            | "minecraft:pig"
            | "minecraft:polar_bear"
            | "minecraft:rabbit"
            | "minecraft:sheep"
            | "minecraft:strider"
            | "minecraft:trader_llama"
            | "minecraft:turtle"
            | "minecraft:wolf"
            | "minecraft:zombie_horse"
    )
}

fn chunk_generation_mob_is_abstract_horse(entity_type: &str) -> bool {
    matches!(
        entity_type,
        "minecraft:camel"
            | "minecraft:donkey"
            | "minecraft:horse"
            | "minecraft:llama"
            | "minecraft:mule"
            | "minecraft:skeleton_horse"
            | "minecraft:trader_llama"
            | "minecraft:zombie_horse"
    )
}

fn chunk_generation_mob_is_neutral(entity_type: &str) -> bool {
    matches!(
        entity_type,
        "minecraft:bee"
            | "minecraft:enderman"
            | "minecraft:iron_golem"
            | "minecraft:polar_bear"
            | "minecraft:wolf"
            | "minecraft:zombified_piglin"
    )
}

fn chunk_generation_mob_is_patrolling_monster(entity_type: &str) -> bool {
    matches!(
        entity_type,
        "minecraft:evoker"
            | "minecraft:illusioner"
            | "minecraft:pillager"
            | "minecraft:ravager"
            | "minecraft:vindicator"
            | "minecraft:witch"
    )
}

fn chunk_generation_mob_is_raider(entity_type: &str) -> bool {
    chunk_generation_mob_is_patrolling_monster(entity_type)
}

pub fn queue_chunk_generation_mob_entity(
    chunk: &mut LevelChunk,
    snap: ChunkGenerationMobEntitySnapPlan,
    uuid: &str,
) -> bool {
    chunk.add_entity_nbt(chunk_generation_mob_entity_nbt(snap, uuid))
}

pub fn apply_chunk_generation_mob_batch_to_chunk(
    chunk: &mut LevelChunk,
    batch: ChunkGenerationMobSpawnBatchPlan,
    dimension_has_ceiling: bool,
    random: &mut RandomSourceKind,
    uuids: &[&str],
) -> usize {
    let mut x = batch.start_x;
    let mut z = batch.start_z;
    let start_x = x;
    let start_z = z;
    let min_block_x = chunk.pos.x * 16;
    let min_block_z = chunk.pos.z * 16;
    let mut spawned = 0;

    for _mob_index in 0..batch.count {
        let mut success = false;
        for _attempt in 0..4 {
            if !success {
                let position = chunk_generation_mob_top_non_colliding_pos(
                    chunk,
                    batch.entity_type,
                    x,
                    z,
                    dimension_has_ceiling,
                );
                if chunk_generation_spawn_position_ok(chunk, batch.entity_type, position.pos) {
                    let snap = chunk_generation_mob_entity_snap_plan(
                        chunk.pos,
                        batch.entity_type,
                        position.pos,
                        random,
                    );
                    let collision = chunk_generation_mob_collision_plan(snap);
                    if chunk_generation_mob_no_collision(chunk, collision) {
                        let spawn_rules_pos = BlockPos {
                            x: snap.x.floor() as i32,
                            y: position.pos.y,
                            z: snap.z.floor() as i32,
                        };
                        if chunk_generation_mob_spawn_rules_ok(
                            chunk,
                            batch.entity_type,
                            spawn_rules_pos,
                        ) {
                            let generated_uuid;
                            let uuid = if let Some(uuid) = uuids.get(spawned) {
                                *uuid
                            } else {
                                generated_uuid = create_insecure_uuid(random);
                                generated_uuid.as_str()
                            };
                            if queue_chunk_generation_mob_entity(chunk, snap, uuid) {
                                spawned += 1;
                                success = true;
                            }
                        }
                    }
                }
            }

            x += random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            z += random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            while x < min_block_x
                || x >= min_block_x + 16
                || z < min_block_z
                || z >= min_block_z + 16
            {
                x = start_x + random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
                z = start_z + random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            }
        }
    }

    spawned
}

pub(super) fn apply_chunk_generation_mob_batch_to_chunk_timed(
    chunk: &mut LevelChunk,
    batch: ChunkGenerationMobSpawnBatchPlan,
    dimension_has_ceiling: bool,
    random: &mut RandomSourceKind,
    uuids: &[&str],
    timings: &mut LiveMobGenerationTimings,
) {
    let mut x = batch.start_x;
    let mut z = batch.start_z;
    let start_x = x;
    let start_z = z;
    let min_block_x = chunk.pos.x * 16;
    let min_block_z = chunk.pos.z * 16;

    for _mob_index in 0..batch.count {
        let mut success = false;
        for _attempt in 0..4 {
            timings.attempts += 1;
            if !success {
                let started = Instant::now();
                let position = chunk_generation_mob_top_non_colliding_pos(
                    chunk,
                    batch.entity_type,
                    x,
                    z,
                    dimension_has_ceiling,
                );
                add_timing(&mut timings.top_position_ms, started);

                let started = Instant::now();
                let position_ok =
                    chunk_generation_spawn_position_ok(chunk, batch.entity_type, position.pos);
                add_timing(&mut timings.position_ok_ms, started);

                if position_ok {
                    let started = Instant::now();
                    let snap = chunk_generation_mob_entity_snap_plan(
                        chunk.pos,
                        batch.entity_type,
                        position.pos,
                        random,
                    );
                    let collision = chunk_generation_mob_collision_plan(snap);
                    let no_collision = chunk_generation_mob_no_collision(chunk, collision);
                    add_timing(&mut timings.snap_collision_ms, started);

                    if no_collision {
                        let spawn_rules_pos = BlockPos {
                            x: snap.x.floor() as i32,
                            y: position.pos.y,
                            z: snap.z.floor() as i32,
                        };
                        let started = Instant::now();
                        let spawn_rules_ok = chunk_generation_mob_spawn_rules_ok(
                            chunk,
                            batch.entity_type,
                            spawn_rules_pos,
                        );
                        add_timing(&mut timings.spawn_rules_ms, started);

                        if spawn_rules_ok {
                            let generated_uuid;
                            let uuid = if let Some(uuid) = uuids.get(timings.mobs_spawned) {
                                *uuid
                            } else {
                                generated_uuid = create_insecure_uuid(random);
                                generated_uuid.as_str()
                            };
                            let started = Instant::now();
                            if queue_chunk_generation_mob_entity(chunk, snap, uuid) {
                                timings.mobs_spawned += 1;
                                success = true;
                            }
                            add_timing(&mut timings.queue_ms, started);
                        }
                    }
                }
            }

            let started = Instant::now();
            x += random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            z += random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            while x < min_block_x
                || x >= min_block_x + 16
                || z < min_block_z
                || z >= min_block_z + 16
            {
                x = start_x + random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
                z = start_z + random_next_i32_bound(random, 5) - random_next_i32_bound(random, 5);
            }
            add_timing(&mut timings.random_walk_ms, started);
        }
    }
}

pub fn create_insecure_uuid(random: &mut RandomSourceKind) -> String {
    let most = (random_source_next_i64(random) & -61441_i64) | 16384_i64;
    let least = (random_source_next_i64(random) & 4_611_686_018_427_387_903_i64) | i64::MIN;
    format_uuid_from_longs(most, least)
}

fn format_uuid_from_longs(most: i64, least: i64) -> String {
    let raw = format!("{:016x}{:016x}", most as u64, least as u64);
    format!(
        "{}-{}-{}-{}-{}",
        &raw[0..8],
        &raw[8..12],
        &raw[12..16],
        &raw[16..20],
        &raw[20..32]
    )
}

fn random_source_next_i64(random: &mut RandomSourceKind) -> i64 {
    match random {
        RandomSourceKind::Legacy(random) => random.next_i64(),
        RandomSourceKind::Xoroshiro(random) => random.next_i64(),
    }
}

pub(super) fn spawn_pathfindable_land_block(block: &str) -> bool {
    is_surface_air(block)
}

pub(super) fn spawn_valid_ground_block(block: &str, entity_type: &str) -> bool {
    match block {
        "minecraft:ice" | "minecraft:packed_ice" | "minecraft:blue_ice" => {
            entity_type == "minecraft:polar_bear"
        }
        _ => matches!(spawn_block_kind(block), SpawnBlockKind::Solid),
    }
}

pub(super) fn spawn_valid_empty_block(block: &str) -> bool {
    matches!(
        spawn_block_kind(block),
        SpawnBlockKind::Air | SpawnBlockKind::NonSolid
    )
}

pub(super) fn spawn_redstone_conductor_block(block: &str) -> bool {
    matches!(spawn_block_kind(block), SpawnBlockKind::Solid)
}

pub(super) fn spawn_colliding_block(block: &str) -> bool {
    matches!(spawn_block_kind(block), SpawnBlockKind::Solid)
}

pub(super) fn animals_spawnable_on(block: &str) -> bool {
    block == "minecraft:grass_block"
}

pub(super) fn goats_spawnable_on(block: &str) -> bool {
    animals_spawnable_on(block)
        || matches!(
            block,
            "minecraft:stone"
                | "minecraft:snow"
                | "minecraft:snow_block"
                | "minecraft:packed_ice"
                | "minecraft:gravel"
        )
}

pub(super) fn rabbits_spawnable_on(block: &str) -> bool {
    animals_spawnable_on(block)
        || matches!(
            block,
            "minecraft:snow" | "minecraft:snow_block" | "minecraft:sand"
        )
}
