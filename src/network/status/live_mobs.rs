use super::*;

const DEFAULT_MOB_HEALTH: f32 = 20.0;
const PLAYER_ATTACK_DAMAGE: f32 = 1.0;
const PLAYER_ATTACK_RANGE: f64 = 3.0;
const HOSTILE_FOLLOW_RANGE: f64 = 16.0;
const HOSTILE_STEP_PER_TICK: f64 = 0.115;
const PASSIVE_WANDER_STEP_PER_TICK: f64 = 0.035;
const MOVE_PACKET_SCALE: f64 = 4096.0;
const MAX_RELATIVE_MOVE_DELTA: i16 = 32_767;

#[derive(Debug, Clone, PartialEq)]
pub struct LiveMobEntity {
    pub entity_id: i32,
    pub entity_type: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub previous_x: f64,
    pub previous_y: f64,
    pub previous_z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub tick_count: u64,
}

#[derive(Debug, Default)]
pub struct LiveMobStore {
    mobs: BTreeMap<i32, LiveMobEntity>,
    synced_chunks: BTreeSet<ChunkPos>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobAttackResult {
    Miss,
    Hurt { entity_id: i32 },
    Killed { entity_id: i32 },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LiveMobTickStats {
    pub synced_chunks: usize,
    pub total_mobs: usize,
    pub moved_mobs: usize,
}

impl LiveMobStore {
    pub fn sync_loaded_chunks(
        &mut self,
        loaded_chunks: &BTreeSet<(i32, i32)>,
        chunk_cache: &GeneratedChunkCache,
        world_root: &Path,
        world_seed: i64,
    ) -> usize {
        let mut registered = 0;
        for &(x, z) in loaded_chunks {
            let pos = ChunkPos { x, z };
            if self.synced_chunks.contains(&pos) {
                continue;
            }
            let chunk = chunk_cache.get_or_load(x, z, world_root, world_seed);
            for (index, entity) in chunk.entities.iter().enumerate() {
                if let Some(mob) = live_mob_from_chunk_entity(pos, index, entity) {
                    self.mobs.entry(mob.entity_id).or_insert(mob);
                    registered += 1;
                }
            }
            self.synced_chunks.insert(pos);
        }
        registered
    }

    pub fn attack(
        &mut self,
        entity_id: i32,
        player_position: Vec3,
        damage: f32,
    ) -> MobAttackResult {
        let Some(mob) = self.mobs.get_mut(&entity_id) else {
            return MobAttackResult::Miss;
        };
        if distance_squared((mob.x, mob.y, mob.z), (player_position.x, player_position.y, player_position.z))
            > PLAYER_ATTACK_RANGE * PLAYER_ATTACK_RANGE
        {
            return MobAttackResult::Miss;
        }
        mob.health -= damage;
        if mob.health <= 0.0 {
            self.mobs.remove(&entity_id);
            MobAttackResult::Killed { entity_id }
        } else {
            MobAttackResult::Hurt { entity_id }
        }
    }

    pub fn tick_ai(&mut self, tick_count: u64, player_position: Vec3) -> LiveMobTickStats {
        let mut moved_mobs = 0;
        for mob in self.mobs.values_mut() {
            mob.tick_count = mob.tick_count.wrapping_add(1);
            mob.previous_x = mob.x;
            mob.previous_y = mob.y;
            mob.previous_z = mob.z;
            let full_goal_tick = (mob.tick_count + mob.entity_id as u64).is_multiple_of(2)
                || mob.tick_count <= 1;
            if hostile_mob(&mob.entity_type) {
                if step_toward_player(mob, player_position, full_goal_tick) {
                    moved_mobs += 1;
                }
            } else if full_goal_tick && wander_passive_mob(mob, tick_count) {
                moved_mobs += 1;
            }
        }
        LiveMobTickStats {
            synced_chunks: self.synced_chunks.len(),
            total_mobs: self.mobs.len(),
            moved_mobs,
        }
    }

    pub fn moved_mobs(&self) -> impl Iterator<Item = &LiveMobEntity> {
        self.mobs.values().filter(|mob| {
            mob.x != mob.previous_x || mob.y != mob.previous_y || mob.z != mob.previous_z
        })
    }
}

pub fn handle_live_mob_attack<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    store: &Arc<Mutex<LiveMobStore>>,
    packet: ServerboundAttackPacket,
    play_state: &PlaySessionState,
) -> io::Result<MobAttackResult> {
    let player_position = Vec3 {
        x: play_state.x,
        y: play_state.y,
        z: play_state.z,
    };
    let result = lock_status_mutex(store).attack(packet.entity_id, player_position, PLAYER_ATTACK_DAMAGE);
    match result {
        MobAttackResult::Miss => {}
        MobAttackResult::Hurt { entity_id } => {
            write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
                |payload| ClientboundEntityEventPacket {
                    entity_id,
                    event_id: 2,
                }
                .write(payload),
            )?;
        }
        MobAttackResult::Killed { entity_id } => {
            write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_ENTITY_EVENT_PACKET_ID,
                |payload| ClientboundEntityEventPacket {
                    entity_id,
                    event_id: 3,
                }
                .write(payload),
            )?;
            write_framed_packet_with_compression(
                writer,
                compression,
                CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID,
                |payload| ClientboundRemoveEntitiesPacket {
                    entity_ids: vec![entity_id],
                }
                .write(payload),
            )?;
        }
    }
    Ok(result)
}

pub fn tick_live_mobs_for_client<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    context: LiveMobClientTickContext<'_>,
) -> io::Result<()> {
    let LiveMobClientTickContext {
        store,
        loaded_chunks,
        chunk_cache,
        world_root,
        world_seed,
        play_state,
        tick_count,
    } = context;
    let total_started = Instant::now();
    let sync_started = Instant::now();
    let (stats, moves) = {
        let mut mobs = lock_status_mutex(store);
        mobs.sync_loaded_chunks(loaded_chunks, chunk_cache, world_root, world_seed);
        let sync_ms = sync_started.elapsed().as_micros();
        let ai_started = Instant::now();
        let stats = mobs.tick_ai(
            tick_count,
            Vec3 {
                x: play_state.x,
                y: play_state.y,
                z: play_state.z,
            },
        );
        let ai_us = ai_started.elapsed().as_micros();
        let moves: Vec<ClientboundMoveEntityPacket> = mobs
            .moved_mobs()
            .map(relative_move_packet)
            .collect();
        eprintln!(
            "[mob-ai-timing] tick={tick_count} mobs={} moved={} synced_chunks={} sync={}us ai={}us",
            stats.total_mobs, stats.moved_mobs, stats.synced_chunks, sync_ms, ai_us
        );
        (stats, moves)
    };
    let write_started = Instant::now();
    for packet in moves {
        write_framed_packet_with_compression(
            writer,
            compression,
            CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID,
            |payload| packet.write_pos(payload),
        )?;
    }
    eprintln!(
        "[mob-ai-timing] tick={tick_count} total={}us write={}us packets={}",
        total_started.elapsed().as_micros(),
        write_started.elapsed().as_micros(),
        stats.moved_mobs
    );
    Ok(())
}

pub struct LiveMobClientTickContext<'a> {
    pub store: &'a Arc<Mutex<LiveMobStore>>,
    pub loaded_chunks: &'a BTreeSet<(i32, i32)>,
    pub chunk_cache: &'a GeneratedChunkCache,
    pub world_root: &'a Path,
    pub world_seed: i64,
    pub play_state: &'a PlaySessionState,
    pub tick_count: u64,
}

fn live_mob_from_chunk_entity(
    chunk_pos: ChunkPos,
    index: usize,
    entity: &Tag,
) -> Option<LiveMobEntity> {
    let Tag::Compound(fields) = entity else {
        return None;
    };
    let entity_type = tag_string_field(fields, "id")?.to_string();
    let [x, y, z] = tag_double_triplet_field(fields, "Pos")?;
    let [yaw, pitch] = tag_float_pair_field(fields, "Rotation").unwrap_or([0.0, 0.0]);
    Some(LiveMobEntity {
        entity_id: generated_chunk_entity_runtime_id(chunk_pos, index),
        entity_type,
        x,
        y,
        z,
        previous_x: x,
        previous_y: y,
        previous_z: z,
        yaw,
        pitch,
        health: DEFAULT_MOB_HEALTH,
        tick_count: 0,
    })
}

fn relative_move_packet(mob: &LiveMobEntity) -> ClientboundMoveEntityPacket {
    let delta = [
        relative_delta(mob.x - mob.previous_x),
        relative_delta(mob.y - mob.previous_y),
        relative_delta(mob.z - mob.previous_z),
    ];
    ClientboundMoveEntityPacket::pos(mob.entity_id, delta, true)
}

fn relative_delta(delta: f64) -> i16 {
    (delta * MOVE_PACKET_SCALE)
        .round()
        .clamp(-(MAX_RELATIVE_MOVE_DELTA as f64), MAX_RELATIVE_MOVE_DELTA as f64)
        as i16
}

fn step_toward_player(mob: &mut LiveMobEntity, player_position: Vec3, full_goal_tick: bool) -> bool {
    if !full_goal_tick {
        return false;
    }
    let dx = player_position.x - mob.x;
    let dz = player_position.z - mob.z;
    let distance_sq = dx * dx + dz * dz;
    if !(0.0001..=(HOSTILE_FOLLOW_RANGE * HOSTILE_FOLLOW_RANGE)).contains(&distance_sq) {
        return false;
    }
    let distance = distance_sq.sqrt();
    mob.x += dx / distance * HOSTILE_STEP_PER_TICK;
    mob.z += dz / distance * HOSTILE_STEP_PER_TICK;
    mob.yaw = dx.atan2(dz).to_degrees() as f32;
    true
}

fn wander_passive_mob(mob: &mut LiveMobEntity, tick_count: u64) -> bool {
    let phase = ((tick_count as i64 + mob.entity_id as i64).rem_euclid(80) as f64) / 80.0;
    let angle = phase * std::f64::consts::TAU;
    mob.x += angle.cos() * PASSIVE_WANDER_STEP_PER_TICK;
    mob.z += angle.sin() * PASSIVE_WANDER_STEP_PER_TICK;
    mob.yaw = angle.to_degrees() as f32;
    true
}

fn hostile_mob(entity_type: &str) -> bool {
    matches!(
        entity_type,
        "minecraft:zombie"
            | "minecraft:husk"
            | "minecraft:drowned"
            | "minecraft:zombified_piglin"
            | "minecraft:skeleton"
            | "minecraft:stray"
            | "minecraft:bogged"
            | "minecraft:creeper"
            | "minecraft:spider"
            | "minecraft:cave_spider"
            | "minecraft:slime"
            | "minecraft:witch"
            | "minecraft:pillager"
            | "minecraft:vindicator"
            | "minecraft:ravager"
            | "minecraft:zoglin"
            | "minecraft:zombie_nautilus"
    )
}

fn distance_squared(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    let dz = a.2 - b.2;
    dx * dx + dy * dy + dz * dz
}

fn tag_double_triplet_field(fields: &[(String, Tag)], name: &str) -> Option<[f64; 3]> {
    match fields.iter().find(|(field_name, _)| field_name == name)?.1 {
        Tag::List(ref values) if values.len() == 3 => Some([
            tag_number_as_f64(values.first()?)?,
            tag_number_as_f64(values.get(1)?)?,
            tag_number_as_f64(values.get(2)?)?,
        ]),
        _ => None,
    }
}

fn tag_float_pair_field(fields: &[(String, Tag)], name: &str) -> Option<[f32; 2]> {
    match fields.iter().find(|(field_name, _)| field_name == name)?.1 {
        Tag::List(ref values) if values.len() == 2 => Some([
            tag_number_as_f64(values.first()?)? as f32,
            tag_number_as_f64(values.get(1)?)? as f32,
        ]),
        _ => None,
    }
}

fn tag_number_as_f64(tag: &Tag) -> Option<f64> {
    match tag {
        Tag::Double(value) => Some(*value),
        Tag::Float(value) => Some(f64::from(*value)),
        Tag::Int(value) => Some(f64::from(*value)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zombie(entity_id: i32, x: f64, z: f64) -> LiveMobEntity {
        LiveMobEntity {
            entity_id,
            entity_type: "minecraft:zombie".to_string(),
            x,
            y: 64.0,
            z,
            previous_x: x,
            previous_y: 64.0,
            previous_z: z,
            yaw: 0.0,
            pitch: 0.0,
            health: DEFAULT_MOB_HEALTH,
            tick_count: 0,
        }
    }

    #[test]
    fn live_mob_attack_applies_range_and_removes_dead_target() {
        let mut store = LiveMobStore::default();
        store.mobs.insert(42, zombie(42, 1.0, 0.0));

        assert_eq!(
            store.attack(
                42,
                Vec3 {
                    x: 10.0,
                    y: 64.0,
                    z: 0.0
                },
                5.0
            ),
            MobAttackResult::Miss
        );
        assert_eq!(store.mobs.get(&42).unwrap().health, DEFAULT_MOB_HEALTH);

        assert_eq!(
            store.attack(
                42,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                5.0
            ),
            MobAttackResult::Hurt { entity_id: 42 }
        );
        assert_eq!(store.mobs.get(&42).unwrap().health, 15.0);

        assert_eq!(
            store.attack(
                42,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                20.0
            ),
            MobAttackResult::Killed { entity_id: 42 }
        );
        assert!(!store.mobs.contains_key(&42));
    }

    #[test]
    fn hostile_mob_ai_uses_id_based_half_rate_goal_tick_and_moves_toward_player() {
        let mut store = LiveMobStore::default();
        store.mobs.insert(43, zombie(43, 4.0, 0.0));

        let first = store.tick_ai(
            1,
            Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
        );
        assert_eq!(first.moved_mobs, 1);
        let after_first = store.mobs.get(&43).unwrap().x;
        assert!(after_first < 4.0);

        let second = store.tick_ai(
            2,
            Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
        );
        assert_eq!(second.moved_mobs, 0);
        assert_eq!(store.mobs.get(&43).unwrap().x, after_first);

        let third = store.tick_ai(
            3,
            Vec3 {
                x: 0.0,
                y: 64.0,
                z: 0.0,
            },
        );
        assert_eq!(third.moved_mobs, 1);
        assert!(store.mobs.get(&43).unwrap().x < after_first);

        let packet = relative_move_packet(store.mobs.get(&43).unwrap());
        assert_eq!(packet.id, 43);
        assert!(packet.delta[0] < 0);
        assert_eq!(packet.delta[1], 0);
    }

    #[test]
    fn live_mob_from_chunk_entity_uses_generated_runtime_id_and_nbt_position() {
        let entity = Tag::Compound(vec![
            ("id".to_string(), Tag::String("minecraft:zombie".to_string())),
            (
                "Pos".to_string(),
                Tag::List(vec![Tag::Double(32.5), Tag::Double(70.0), Tag::Double(-15.25)]),
            ),
            (
                "Rotation".to_string(),
                Tag::List(vec![Tag::Float(90.0), Tag::Float(15.0)]),
            ),
        ]);
        let mob = live_mob_from_chunk_entity(ChunkPos { x: 2, z: -1 }, 3, &entity).unwrap();

        assert_eq!(
            mob.entity_id,
            generated_chunk_entity_runtime_id(ChunkPos { x: 2, z: -1 }, 3)
        );
        assert_eq!(mob.entity_type, "minecraft:zombie");
        assert_eq!((mob.x, mob.y, mob.z), (32.5, 70.0, -15.25));
        assert_eq!(mob.yaw, 90.0);
        assert_eq!(mob.pitch, 15.0);
    }
}
