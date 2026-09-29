//! The player-side effects of an advancement completing: `AdvancementRewards.grant`
//! and the `chat.type.advancement.*` announcement.

use std::io;
use std::sync::{Arc, Mutex};

use super::super::player_messaging_live::{
    json_component_to_tag, player_display_name_tag, publish_system_chat,
};
use super::super::xp_orb_live::write_set_experience;
use super::super::*;
use crate::advancement_system::{AdvancementDefinition, AdvancementFrame, AdvancementRewards};
use crate::chat_formatting::ChatFormatting;
use crate::item_entity::{DroppedItem, WorldItemEntities};
use crate::live_block_entities::registry_ids::sound_event_protocol_id;
use crate::log::log_info;
use crate::loot_system::{LootContext, LootParamSet};
use crate::network::play::{
    ClientboundSoundPacket, SoundEventHolder, SoundSource, Vec3, CLIENTBOUND_SOUND_PACKET_ID,
};
use crate::player_advancements::AdvancementEvent;
use crate::registry_pipeline::server_resources::LoadedResources;
use crate::xp_orb_entity::XpOrbRandom;

/// `SoundEvents.ITEM_PICKUP`.
const ITEM_PICKUP_SOUND: &str = "minecraft:entity.item.pickup";
/// Radius in which `Level.playSound(null, ...)` is heard (`volume * 16`).
const PICKUP_SOUND_RADIUS: f64 = 16.0;
/// `LivingEntity.getEyeHeight - 0.3` for a standing player (`Player.drop` spawn height).
const DROP_HEIGHT_OFFSET: f64 = 1.62 - 0.3;

/// What the reward and announcement code needs from the session.
pub(super) struct RewardContext<'a> {
    pub profile: &'a NameAndId,
    pub bus: &'a WorldPacketBus,
    pub world_items: &'a Arc<Mutex<WorldItemEntities>>,
    pub resources: &'a LoadedResources,
}

/// Runs one queued [`AdvancementEvent`].
pub(super) fn handle_event(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: &RewardContext<'_>,
    event: &AdvancementEvent,
) -> io::Result<()> {
    match event {
        AdvancementEvent::GrantRewards(id) => {
            let Some(holder) = context.resources.content.advancements.get(id) else {
                return Ok(());
            };
            grant_rewards(
                stream,
                compression,
                play_state,
                context,
                &holder.value().rewards,
            )
        }
        AdvancementEvent::Announce(id) => {
            let Some(holder) = context.resources.content.advancements.get(id) else {
                return Ok(());
            };
            announce(context, holder.value())
        }
    }
}

/// `AdvancementRewards.grant(player)`.
fn grant_rewards(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: &RewardContext<'_>,
    rewards: &AdvancementRewards,
) -> io::Result<()> {
    give_experience_points(stream, compression, play_state, rewards.experience)?;
    let times_changed_before = play_state.inventory_menu.player_inventory().times_changed();
    for table in &rewards.loot {
        grant_loot_table(stream, compression, play_state, context, table)?;
    }
    // `if (changes) player.containerMenu.broadcastChanges()`.
    write_pickup_inventory_sync(stream, compression, play_state, times_changed_before)?;
    if !rewards.recipes.is_empty() {
        award_recipes(stream, compression, play_state, context, &rewards.recipes)?;
    }
    // TODO(advancement-reward-function): `rewards.function` runs `ServerFunctionManager.execute`
    // with the player as a suppressed-output, game-master source; that needs a player
    // command source outside the chat-command path.
    Ok(())
}

/// `ServerPlayer.giveExperiencePoints(points)`: score, level/progress and a resend.
fn give_experience_points(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    points: i32,
) -> io::Result<()> {
    if points == 0 {
        return Ok(());
    }
    let mut experience = crate::experience_system::PlayerExperience {
        level: play_state.xp_level,
        progress: play_state.xp_progress,
        total: play_state.xp_total,
        take_xp_delay: play_state.combat.take_xp_delay,
    };
    experience.give_points(points);
    play_state.xp_level = experience.level;
    play_state.xp_progress = experience.progress;
    play_state.xp_total = experience.total;
    // `Player.increaseScore(points)`.
    play_state.score = play_state.score.saturating_add(points);
    write_set_experience(stream, compression, play_state)
}

/// One reward loot table: `getRandomItems(params)`, each stack into the inventory or
/// dropped at the player's feet for them alone.
fn grant_loot_table(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: &RewardContext<'_>,
    table: &crate::registry::Identifier,
) -> io::Result<()> {
    let Some(loot_table) = context.resources.content.loot_tables.get(&table.to_string()) else {
        return Ok(());
    };
    let mut loot_context = LootContext::new(
        LootParamSet::AdvancementReward,
        super::super::player_death::live_random_seed(),
    );
    let mut random = XpOrbRandom::new(super::super::player_death::live_random_seed());
    for stack in loot_table.evaluate(&mut loot_context) {
        let Some(name) = item_static_name(&stack.item).filter(|_| stack.count > 0) else {
            continue;
        };
        let item = ItemStack::new(name, stack.count);
        let leftover = match play_state.inventory_menu.player_inventory_mut().add(item) {
            crate::player_inventory::InventoryAddResult::PartiallyAdded { remaining } => remaining,
            _ => 0,
        };
        if leftover < stack.count {
            play_pickup_sound(stream, compression, play_state, context, &mut random)?;
        }
        if leftover > 0 {
            drop_for_player(play_state, context, name, leftover)?;
        }
    }
    Ok(())
}

/// `level.playSound(null, x, y, z, ITEM_PICKUP, PLAYERS, 0.2F, ...)`.
fn play_pickup_sound(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &PlaySessionState,
    context: &RewardContext<'_>,
    random: &mut XpOrbRandom,
) -> io::Result<()> {
    let Some(id) = sound_event_protocol_id(ITEM_PICKUP_SOUND) else {
        return Ok(());
    };
    let pitch = ((random.next_double() - random.next_double()) * 0.7 + 1.0) * 2.0;
    let packet = ClientboundSoundPacket {
        sound: SoundEventHolder::Registered { id },
        source_id: SoundSource::Players as i32,
        position: Vec3 {
            x: play_state.x,
            y: play_state.y,
            z: play_state.z,
        },
        volume: 0.2,
        pitch: pitch as f32,
        seed: random.next_int(i32::MAX) as i64,
        entity_id: None,
    };
    let mut payload = Vec::new();
    write_var_i32(&mut payload, CLIENTBOUND_SOUND_PACKET_ID)?;
    packet.write_position(&mut payload)?;
    let own = context.bus.tokens_within(
        [play_state.x, play_state.y, play_state.z],
        PICKUP_SOUND_RADIUS,
    );
    // The listening player is subscribed to the bus, so they hear it with everyone else.
    if own.is_empty() {
        write_framed_packet_with_compression(stream, compression, CLIENTBOUND_SOUND_PACKET_ID, |p| {
            packet.write_position(p)
        })
    } else {
        for token in own {
            context.bus.publish_to(token, &payload);
        }
        Ok(())
    }
}

/// `Player.drop(stack, false)` followed by `setNoPickUpDelay` + `setTarget(player)`.
fn drop_for_player(
    play_state: &PlaySessionState,
    context: &RewardContext<'_>,
    name: &'static str,
    count: i32,
) -> io::Result<()> {
    let Some(item_pid) = item_protocol_id(name) else {
        return Ok(());
    };
    let entity_id = lock_status_mutex(context.world_items).alloc_entity_id();
    let item = DroppedItem {
        entity_id,
        item: name,
        count,
        x: play_state.x,
        y: play_state.y + DROP_HEIGHT_OFFSET,
        z: play_state.z,
        vel_x: 0.0,
        vel_y: 0.0,
        vel_z: 0.0,
        pickup_delay: 0,
        age: 0,
        target_uuid: Some(context.profile.uuid.clone()),
        health: crate::item_entity::ITEM_DEFAULT_HEALTH,
    };
    let mut frames = Vec::new();
    write_item_entity_spawn_packets(&mut frames, CompressionState::disabled(), &item, item_pid)?;
    lock_status_mutex(context.world_items).entities.push(item);
    context.bus.publish_frames(&frames)
}

/// `ServerPlayer.awardRecipesByKey`: unlock each known recipe and tell the client.
fn award_recipes(
    stream: &mut ClientStream,
    compression: CompressionState,
    play_state: &mut PlaySessionState,
    context: &RewardContext<'_>,
    recipes: &[crate::registry::Identifier],
) -> io::Result<()> {
    for recipe in recipes {
        play_state
            .inventory_menu
            .unlock_recipe_by_key(&recipe.to_string());
    }
    write_pickup_recipe_unlocks(stream, compression, play_state, &context.resources.content.recipes)
}

/// The completion message for `definition`
/// (`AdvancementType.createAnnouncement` broadcast without an overlay).
fn announce(context: &RewardContext<'_>, definition: &AdvancementDefinition) -> io::Result<()> {
    let Some(display) = &definition.display else {
        return Ok(());
    };
    let name_json = advancement_name_json(display);
    let Some(name_tag) = json_component_to_tag(&name_json.to_string()) else {
        return Ok(());
    };
    let message = Tag::Compound(vec![
        (
            "translate".to_string(),
            Tag::String(display.frame.announcement_translation_key()),
        ),
        (
            "with".to_string(),
            Tag::List(vec![player_display_name_tag(context.profile), name_tag]),
        ),
    ]);
    publish_system_chat(context.bus, message)?;
    // `MinecraftServer.sendSystemMessage` logs the plain text.
    log_info(&crate::language::translate(
        &display.frame.announcement_translation_key(),
        &[
            context.profile.name.clone(),
            format!("[{}]", plain_text(&display.title_json)),
        ],
    ));
    Ok(())
}

/// `Advancement.decorateName(display)`: the bracketed, coloured title whose hover shows
/// the coloured title, a newline and the description.
pub(super) fn advancement_name_json(
    display: &crate::advancement_system::AdvancementDisplay,
) -> serde_json::Value {
    let color = frame_color_name(display.frame);
    let tooltip = {
        let mut title = component_object(&display.title_json);
        title.insert("color".to_string(), serde_json::Value::from(color));
        let mut extra = match title.remove("extra") {
            Some(serde_json::Value::Array(extra)) => extra,
            _ => Vec::new(),
        };
        extra.push(serde_json::Value::from("\n"));
        extra.push(display.description_json.clone());
        title.insert("extra".to_string(), serde_json::Value::Array(extra));
        serde_json::Value::Object(title)
    };
    let mut title = component_object(&display.title_json);
    title.insert(
        "hover_event".to_string(),
        serde_json::json!({ "action": "show_text", "value": tooltip }),
    );
    serde_json::json!({
        "translate": "chat.square_brackets",
        "with": [serde_json::Value::Object(title)],
        "color": color,
    })
}

/// The `ChatFormatting` name of `AdvancementType.getChatColor`.
fn frame_color_name(frame: AdvancementFrame) -> &'static str {
    match frame.chat_color() {
        ChatFormatting::DarkPurple => "dark_purple",
        _ => "green",
    }
}

/// A component as an editable JSON object (`Component.copy()`).
fn component_object(component: &serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match component {
        serde_json::Value::Object(object) => object.clone(),
        serde_json::Value::String(text) => {
            let mut object = serde_json::Map::new();
            object.insert("text".to_string(), serde_json::Value::from(text.clone()));
            object
        }
        other => {
            let mut object = serde_json::Map::new();
            object.insert("text".to_string(), serde_json::Value::from(""));
            object.insert(
                "extra".to_string(),
                serde_json::Value::Array(vec![other.clone()]),
            );
            object
        }
    }
}

/// `Component.getString()` for a text or translatable component.
fn plain_text(component: &serde_json::Value) -> String {
    match component {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Object(object) => {
            if let Some(text) = object.get("text").and_then(serde_json::Value::as_str) {
                text.to_string()
            } else if let Some(key) = object.get("translate").and_then(serde_json::Value::as_str) {
                crate::language::translate(key, &[])
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}
