use super::*;

const DEFAULT_MOB_HEALTH: f32 = 20.0;
const PLAYER_BASE_ATTACK_DAMAGE: f32 = 1.0;
const PLAYER_ENTITY_INTERACTION_RANGE: f64 = 3.0;
const PLAYER_ATTACK_VERIFICATION_BUFFER: f64 = 3.0;
const PLAYER_EYE_HEIGHT: f64 = 1.62;
const PLAYER_WIDTH: f64 = 0.6;
const PLAYER_HEIGHT: f64 = 1.8;
const HOSTILE_FOLLOW_RANGE: f64 = 16.0;
const HOSTILE_STEP_PER_TICK: f64 = 0.115;
const PASSIVE_WANDER_STEP_PER_TICK: f64 = 0.035;
const MOB_DEFAULT_ATTACK_REACH: f64 = 0.828_285_694_955_484_2;
const MOB_MELEE_ATTACK_INTERVAL_TICKS: i32 = 20;
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
    pub attack_cooldown_ticks: i32,
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LiveMobTickStats {
    pub synced_chunks: usize,
    pub registered_mobs: usize,
    pub total_mobs: usize,
    pub moved_mobs: usize,
    pub player_damage: f32,
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
        if !player_can_reach_mob_attack(mob, player_position) {
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

    pub fn tick_ai(
        &mut self,
        tick_count: u64,
        play_state: &mut PlaySessionState,
        registered_mobs: usize,
    ) -> LiveMobTickStats {
        let mut moved_mobs = 0;
        let mut player_damage = 0.0;
        let player_position = Vec3 {
            x: play_state.x,
            y: play_state.y,
            z: play_state.z,
        };
        for mob in self.mobs.values_mut() {
            mob.tick_count = mob.tick_count.wrapping_add(1);
            mob.attack_cooldown_ticks = (mob.attack_cooldown_ticks - 1).max(0);
            mob.previous_x = mob.x;
            mob.previous_y = mob.y;
            mob.previous_z = mob.z;
            let full_goal_tick = (mob.tick_count + mob.entity_id as u64).is_multiple_of(2)
                || mob.tick_count <= 1;
            if hostile_mob(&mob.entity_type) {
                if step_toward_player(mob, player_position, full_goal_tick) {
                    moved_mobs += 1;
                }
                if mob_can_melee_player(mob, player_position) && mob.attack_cooldown_ticks <= 0 {
                    let damage = live_mob_attack_damage(&mob.entity_type);
                    play_state.health = (play_state.health - damage).max(0.0);
                    mob.attack_cooldown_ticks = MOB_MELEE_ATTACK_INTERVAL_TICKS;
                    player_damage += damage;
                }
            } else if full_goal_tick && wander_passive_mob(mob, tick_count) {
                moved_mobs += 1;
            }
        }
        LiveMobTickStats {
            synced_chunks: self.synced_chunks.len(),
            registered_mobs,
            total_mobs: self.mobs.len(),
            moved_mobs,
            player_damage,
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
    let result =
        lock_status_mutex(store).attack(packet.entity_id, player_position, player_attack_damage(play_state));
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
) -> io::Result<bool> {
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
        let registered_mobs = mobs.sync_loaded_chunks(loaded_chunks, chunk_cache, world_root, world_seed);
        let sync_us = sync_started.elapsed().as_micros();
        let ai_started = Instant::now();
        // Java `GoalSelector.tick()` is split into cleanup, goal selection, and
        // running-goal ticks; this lightweight live path keeps one measured AI
        // phase but preserves the same "goal work happens during the entity tick"
        // placement in the server tick.
        let stats = mobs.tick_ai(tick_count, play_state, registered_mobs);
        let ai_us = ai_started.elapsed().as_micros();
        let moves: Vec<ClientboundMoveEntityPacket> = mobs
            .moved_mobs()
            .map(relative_move_packet)
            .collect();
        eprintln!("{}", format_live_mob_ai_timing(tick_count, stats, sync_us, ai_us));
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
    Ok(stats.player_damage > 0.0)
}

fn format_live_mob_ai_timing(
    tick_count: u64,
    stats: LiveMobTickStats,
    sync_us: u128,
    ai_us: u128,
) -> String {
    format!(
        "[mob-ai-timing] tick={tick_count} mobs={} registered={} moved={} synced_chunks={} sync={}us ai={}us",
        stats.total_mobs,
        stats.registered_mobs,
        stats.moved_mobs,
        stats.synced_chunks,
        sync_us,
        ai_us
    )
}

pub struct LiveMobClientTickContext<'a> {
    pub store: &'a Arc<Mutex<LiveMobStore>>,
    pub loaded_chunks: &'a BTreeSet<(i32, i32)>,
    pub chunk_cache: &'a GeneratedChunkCache,
    pub world_root: &'a Path,
    pub world_seed: i64,
    pub play_state: &'a mut PlaySessionState,
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
        attack_cooldown_ticks: 0,
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

fn player_can_reach_mob_attack(mob: &LiveMobEntity, player_position: Vec3) -> bool {
    // Java `ServerGamePacketListenerImpl.handleAttack` calls
    // `Player.isWithinAttackRange(mainHandItem, targetBounds, 3.0)`. Empty-hand
    // attacks use `AttackRange.defaultFor(player)`: max reach is the player's
    // entity_interaction_range (3.0 in survival/adventure), plus the listener's
    // 3.0 verification buffer. Distance is eye-to-target-AABB, not center-to-center.
    let max_range = PLAYER_ENTITY_INTERACTION_RANGE + PLAYER_ATTACK_VERIFICATION_BUFFER;
    let eye = Vec3 {
        x: player_position.x,
        y: player_position.y + PLAYER_EYE_HEIGHT,
        z: player_position.z,
    };
    mob_aabb_distance_squared(mob, eye) <= max_range * max_range
}

fn mob_aabb_distance_squared(mob: &LiveMobEntity, point: Vec3) -> f64 {
    let (width, height) = entity_dimensions(&mob.entity_type);
    let half_width = width / 2.0;
    let dx = axis_distance_to_interval(point.x, mob.x - half_width, mob.x + half_width);
    let dy = axis_distance_to_interval(point.y, mob.y, mob.y + height);
    let dz = axis_distance_to_interval(point.z, mob.z - half_width, mob.z + half_width);
    dx * dx + dy * dy + dz * dz
}

fn axis_distance_to_interval(point: f64, min: f64, max: f64) -> f64 {
    if point < min {
        min - point
    } else if point > max {
        point - max
    } else {
        0.0
    }
}

fn player_attack_damage(play_state: &PlaySessionState) -> f32 {
    // Java 26.1.2 `Player.attack` reads the player's ATTACK_DAMAGE attribute.
    // The base value is 1.0, and tool/weapon item components add the material
    // modifier from `ToolMaterial.createToolAttributes/createSwordAttributes`.
    selected_main_hand_item(play_state).map_or(PLAYER_BASE_ATTACK_DAMAGE, |stack| {
        attack_damage_for_item(stack.item_id())
    })
}

fn selected_main_hand_item(state: &PlaySessionState) -> Option<&ItemStack> {
    let slot = usize::try_from(state.selected_slot).ok()?;
    (slot < HOTBAR_SIZE).then(|| state.inventory_menu.player_inventory().get(slot))
}

fn attack_damage_for_item(item_id: &str) -> f32 {
    match item_id {
        "minecraft:wooden_sword" | "minecraft:golden_sword" => 4.0,
        "minecraft:stone_sword" | "minecraft:copper_sword" => 5.0,
        "minecraft:iron_sword" => 6.0,
        "minecraft:diamond_sword" => 7.0,
        "minecraft:netherite_sword" => 8.0,
        "minecraft:wooden_axe" | "minecraft:golden_axe" => 7.0,
        "minecraft:copper_axe" | "minecraft:stone_axe" | "minecraft:iron_axe"
        | "minecraft:diamond_axe" => 9.0,
        "minecraft:netherite_axe" => 10.0,
        "minecraft:wooden_pickaxe" | "minecraft:golden_pickaxe" => 2.0,
        "minecraft:stone_pickaxe" | "minecraft:copper_pickaxe" => 3.0,
        "minecraft:iron_pickaxe" => 4.0,
        "minecraft:diamond_pickaxe" => 5.0,
        "minecraft:netherite_pickaxe" => 6.0,
        "minecraft:wooden_shovel" | "minecraft:golden_shovel" => 2.5,
        "minecraft:stone_shovel" | "minecraft:copper_shovel" => 3.5,
        "minecraft:iron_shovel" => 4.5,
        "minecraft:diamond_shovel" => 5.5,
        "minecraft:netherite_shovel" => 6.5,
        "minecraft:mace" => 6.0,
        "minecraft:trident" => 9.0,
        _ => PLAYER_BASE_ATTACK_DAMAGE,
    }
}

fn mob_can_melee_player(mob: &LiveMobEntity, player_position: Vec3) -> bool {
    let (mob_width, mob_height) = entity_dimensions(&mob.entity_type);
    let mob_half = mob_width / 2.0 + MOB_DEFAULT_ATTACK_REACH;
    let player_half = PLAYER_WIDTH / 2.0;
    aabb_intersects(
        Aabb {
            min_x: mob.x - mob_half,
            max_x: mob.x + mob_half,
            min_y: mob.y,
            max_y: mob.y + mob_height,
            min_z: mob.z - mob_half,
            max_z: mob.z + mob_half,
        },
        Aabb {
            min_x: player_position.x - player_half,
            max_x: player_position.x + player_half,
            min_y: player_position.y,
            max_y: player_position.y + PLAYER_HEIGHT,
            min_z: player_position.z - player_half,
            max_z: player_position.z + player_half,
        },
    )
}

#[derive(Debug, Clone, Copy)]
struct Aabb {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    min_z: f64,
    max_z: f64,
}

fn aabb_intersects(a: Aabb, b: Aabb) -> bool {
    a.max_x > b.min_x
        && a.min_x < b.max_x
        && a.max_y > b.min_y
        && a.min_y < b.max_y
        && a.max_z > b.min_z
        && a.min_z < b.max_z
}

fn live_mob_attack_damage(entity_type: &str) -> f32 {
    match entity_type {
        "minecraft:zombie" | "minecraft:husk" | "minecraft:drowned" => {
            crate::mob_interaction::zombie_attributes().attack_damage
        }
        "minecraft:zombified_piglin" => crate::mob_interaction::zombified_piglin_attributes().attack_damage,
        "minecraft:ravager" => crate::mob_interaction::ravager_attributes().attack_damage,
        "minecraft:enderman" => 7.0,
        "minecraft:blaze" => 6.0,
        "minecraft:zoglin" | "minecraft:hoglin" => 6.0,
        "minecraft:piglin_brute" => 7.0,
        "minecraft:cave_spider" => 2.0,
        "minecraft:silverfish" => 1.0,
        "minecraft:spider" | "minecraft:vex" => 4.0,
        "minecraft:slime" | "minecraft:magma_cube" => 2.0,
        _ => 2.0,
    }
}

fn entity_dimensions(entity_type: &str) -> (f64, f64) {
    // Java 26.1.2 `EntityType` builder `.sized(width, height)` values for the
    // mob families currently emitted by VibeCraft chunk generation. Unknown
    // live mobs fall back to the player-sized default instead of center reach.
    match entity_type {
        "minecraft:allay" => (0.35, 0.6),
        "minecraft:armadillo" => (0.7, 0.65),
        "minecraft:axolotl" => (0.75, 0.42),
        "minecraft:bat" => (0.5, 0.9),
        "minecraft:bee" => (0.7, 0.6),
        "minecraft:blaze" => (0.6, 1.8),
        "minecraft:bogged" | "minecraft:parched" | "minecraft:skeleton" | "minecraft:stray" => {
            (0.6, 1.99)
        }
        "minecraft:breeze" => (0.6, 1.77),
        "minecraft:camel" | "minecraft:camel_husk" => (1.7, 2.375),
        "minecraft:cat" | "minecraft:ocelot" => (0.6, 0.7),
        "minecraft:cave_spider" => (0.7, 0.5),
        "minecraft:chicken" => (0.4, 0.7),
        "minecraft:cod" | "minecraft:tropical_fish" => (0.5, 0.4),
        "minecraft:cow" | "minecraft:mooshroom" => (0.9, 1.4),
        "minecraft:creaking" => (0.9, 2.7),
        "minecraft:creeper" => (0.6, 1.7),
        "minecraft:dolphin" => (0.9, 0.6),
        "minecraft:donkey" | "minecraft:mule" => (1.3964844, 1.5),
        "minecraft:drowned" | "minecraft:husk" | "minecraft:piglin" | "minecraft:piglin_brute"
        | "minecraft:villager" | "minecraft:vindicator" | "minecraft:wandering_trader"
        | "minecraft:witch" | "minecraft:zombie" | "minecraft:zombie_villager"
        | "minecraft:zombified_piglin" => (0.6, 1.95),
        "minecraft:elder_guardian" => (1.9975, 1.9975),
        "minecraft:enderman" | "minecraft:warden" => (0.6, 2.9),
        "minecraft:endermite" | "minecraft:silverfish" => (0.4, 0.3),
        "minecraft:evoker" | "minecraft:illusioner" | "minecraft:pillager" => (0.6, 1.95),
        "minecraft:fox" => (0.6, 0.7),
        "minecraft:frog" => (0.5, 0.5),
        "minecraft:ghast" | "minecraft:happy_ghast" => (4.0, 4.0),
        "minecraft:giant" => (3.6, 12.0),
        "minecraft:glow_squid" | "minecraft:squid" => (0.8, 0.8),
        "minecraft:goat" => (0.9, 1.3),
        "minecraft:guardian" => (0.85, 0.85),
        "minecraft:hoglin" | "minecraft:zoglin" => (1.3964844, 1.4),
        "minecraft:horse" | "minecraft:skeleton_horse" | "minecraft:zombie_horse" => {
            (1.3964844, 1.6)
        }
        "minecraft:iron_golem" => (1.4, 2.7),
        "minecraft:llama" | "minecraft:trader_llama" => (0.9, 1.87),
        "minecraft:magma_cube" | "minecraft:slime" => (0.52, 0.52),
        "minecraft:panda" => (1.3, 1.25),
        "minecraft:parrot" => (0.5, 0.9),
        "minecraft:phantom" => (0.9, 0.5),
        "minecraft:pig" => (0.9, 0.9),
        "minecraft:polar_bear" => (1.4, 1.4),
        "minecraft:pufferfish" => (0.7, 0.7),
        "minecraft:rabbit" => (0.49, 0.6),
        "minecraft:ravager" => (1.95, 2.2),
        "minecraft:salmon" => (0.7, 0.4),
        "minecraft:sheep" => (0.9, 1.3),
        "minecraft:shulker" => (1.0, 1.0),
        "minecraft:sniffer" => (1.9, 1.75),
        "minecraft:snow_golem" => (0.7, 1.9),
        "minecraft:spider" => (1.4, 0.9),
        "minecraft:strider" => (0.9, 1.7),
        "minecraft:tadpole" => (0.4, 0.3),
        "minecraft:turtle" => (1.2, 0.4),
        "minecraft:vex" => (0.4, 0.8),
        "minecraft:wither" => (0.9, 3.5),
        "minecraft:wither_skeleton" => (0.7, 2.4),
        "minecraft:wolf" => (0.6, 0.85),
        "minecraft:zombie_nautilus" => (0.875, 0.95),
        _ => (0.6, 1.8),
    }
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
            attack_cooldown_ticks: 0,
        }
    }

    fn framed_packet_ids(output: &[u8]) -> Vec<i32> {
        let mut ids = Vec::new();
        let mut input = Cursor::new(output);
        while input.position() < output.len() as u64 {
            let frame_len = read_var_i32(&mut input).unwrap() as usize;
            let mut frame = vec![0; frame_len];
            input.read_exact(&mut frame).unwrap();
            ids.push(read_var_i32(&mut Cursor::new(frame)).unwrap());
        }
        ids
    }

    #[test]
    fn live_mob_client_tick_emits_movement_health_and_timing_path() {
        let store = Arc::new(Mutex::new(LiveMobStore::default()));
        lock_status_mutex(&store).mobs.insert(49, zombie(49, 0.9, 0.0));
        let mut play_state = PlaySessionState {
            x: 0.0,
            y: 64.0,
            z: 0.0,
            health: 20.0,
            ..PlaySessionState::default()
        };
        let loaded_chunks = BTreeSet::new();
        let chunk_cache = GeneratedChunkCache::default();
        let mut output = Vec::new();

        let health_changed = tick_live_mobs_for_client(
            &mut output,
            CompressionState::disabled(),
            LiveMobClientTickContext {
                store: &store,
                loaded_chunks: &loaded_chunks,
                chunk_cache: &chunk_cache,
                world_root: Path::new("."),
                world_seed: 0,
                play_state: &mut play_state,
                tick_count: 1,
            },
        )
        .unwrap();

        assert!(health_changed);
        assert_eq!(play_state.health, 17.0);
        assert_eq!(framed_packet_ids(&output), vec![CLIENTBOUND_MOVE_ENTITY_POS_PACKET_ID]);
        assert!(lock_status_mutex(&store).mobs.get(&49).unwrap().x < 0.9);
    }

    #[test]
    fn live_mob_attack_handler_emits_hurt_and_remove_packets() {
        let store = Arc::new(Mutex::new(LiveMobStore::default()));
        lock_status_mutex(&store).mobs.insert(51, zombie(51, 1.0, 0.0));
        let mut play_state = PlaySessionState {
            x: 0.0,
            y: 64.0,
            z: 0.0,
            selected_slot: 0,
            ..PlaySessionState::default()
        };
        play_state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:diamond_sword", 1));

        let mut hurt_output = Vec::new();
        let hurt = handle_live_mob_attack(
            &mut hurt_output,
            CompressionState::disabled(),
            &store,
            ServerboundAttackPacket { entity_id: 51 },
            &play_state,
        )
        .unwrap();
        assert_eq!(hurt, MobAttackResult::Hurt { entity_id: 51 });
        assert_eq!(framed_packet_ids(&hurt_output), vec![CLIENTBOUND_ENTITY_EVENT_PACKET_ID]);

        lock_status_mutex(&store).mobs.get_mut(&51).unwrap().health = 1.0;
        let mut kill_output = Vec::new();
        let kill = handle_live_mob_attack(
            &mut kill_output,
            CompressionState::disabled(),
            &store,
            ServerboundAttackPacket { entity_id: 51 },
            &play_state,
        )
        .unwrap();
        assert_eq!(kill, MobAttackResult::Killed { entity_id: 51 });
        assert_eq!(
            framed_packet_ids(&kill_output),
            vec![CLIENTBOUND_ENTITY_EVENT_PACKET_ID, CLIENTBOUND_REMOVE_ENTITIES_PACKET_ID]
        );
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
    fn live_mob_attack_reach_uses_java_eye_to_aabb_distance() {
        let mut store = LiveMobStore::default();
        store.mobs.insert(44, zombie(44, 6.29, 0.0));

        assert_eq!(
            store.attack(
                44,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                1.0
            ),
            MobAttackResult::Hurt { entity_id: 44 }
        );

        store.mobs.insert(45, zombie(45, 6.31, 0.0));
        assert_eq!(
            store.attack(
                45,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                1.0
            ),
            MobAttackResult::Miss
        );
    }

    #[test]
    fn live_mob_attack_reach_counts_large_entity_hitbox_like_java() {
        let mut store = LiveMobStore::default();
        let mut ghast = zombie(46, 7.9, 0.0);
        ghast.entity_type = "minecraft:ghast".to_string();
        store.mobs.insert(46, ghast);

        assert_eq!(
            store.attack(
                46,
                Vec3 {
                    x: 0.0,
                    y: 64.0,
                    z: 0.0
                },
                1.0
            ),
            MobAttackResult::Hurt { entity_id: 46 }
        );
    }

    #[test]
    fn hostile_mob_ai_uses_id_based_half_rate_goal_tick_and_moves_toward_player() {
        let mut store = LiveMobStore::default();
        store.mobs.insert(43, zombie(43, 4.0, 0.0));
        let mut play_state = PlaySessionState {
            x: 0.0,
            y: 64.0,
            z: 0.0,
            ..PlaySessionState::default()
        };

        let first = store.tick_ai(1, &mut play_state, 0);
        assert_eq!(first.moved_mobs, 1);
        assert_eq!(first.registered_mobs, 0);
        assert_eq!(first.player_damage, 0.0);
        let after_first = store.mobs.get(&43).unwrap().x;
        assert!(after_first < 4.0);

        let second = store.tick_ai(2, &mut play_state, 0);
        assert_eq!(second.moved_mobs, 0);
        assert_eq!(store.mobs.get(&43).unwrap().x, after_first);

        let third = store.tick_ai(3, &mut play_state, 0);
        assert_eq!(third.moved_mobs, 1);
        assert!(store.mobs.get(&43).unwrap().x < after_first);

        let packet = relative_move_packet(store.mobs.get(&43).unwrap());
        assert_eq!(packet.id, 43);
        assert!(packet.delta[0] < 0);
        assert_eq!(packet.delta[1], 0);
    }

    #[test]
    fn hostile_mob_melee_uses_java_attack_range_cooldown_and_attribute_damage() {
        let mut store = LiveMobStore::default();
        store.mobs.insert(47, zombie(47, 0.9, 0.0));
        let mut play_state = PlaySessionState {
            x: 0.0,
            y: 64.0,
            z: 0.0,
            health: 20.0,
            ..PlaySessionState::default()
        };

        let first = store.tick_ai(1, &mut play_state, 0);
        assert_eq!(first.player_damage, crate::mob_interaction::ZOMBIE_ATTACK_DAMAGE);
        assert_eq!(play_state.health, 17.0);
        assert_eq!(
            store.mobs.get(&47).unwrap().attack_cooldown_ticks,
            MOB_MELEE_ATTACK_INTERVAL_TICKS
        );

        let second = store.tick_ai(2, &mut play_state, 0);
        assert_eq!(second.player_damage, 0.0);
        assert_eq!(play_state.health, 17.0);
    }

    #[test]
    fn player_attack_damage_uses_java_tool_attribute_values() {
        let mut state = PlaySessionState::default();
        state.selected_slot = 0;
        assert_eq!(player_attack_damage(&state), PLAYER_BASE_ATTACK_DAMAGE);

        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:diamond_sword", 1));
        assert_eq!(player_attack_damage(&state), 7.0);

        state
            .inventory_menu
            .player_inventory_mut()
            .set(0, ItemStack::new("minecraft:netherite_axe", 1));
        assert_eq!(player_attack_damage(&state), 10.0);
    }

    #[test]
    fn live_mob_ai_timing_line_reports_counts_and_phase_durations() {
        let line = format_live_mob_ai_timing(
            37,
            LiveMobTickStats {
                synced_chunks: 4,
                registered_mobs: 2,
                total_mobs: 9,
                moved_mobs: 3,
                player_damage: 0.0,
            },
            11,
            23,
        );

        assert_eq!(
            line,
            "[mob-ai-timing] tick=37 mobs=9 registered=2 moved=3 synced_chunks=4 sync=11us ai=23us"
        );
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
