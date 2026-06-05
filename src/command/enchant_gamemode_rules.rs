use super::*;

pub(super) fn enchant_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["enchant", targets, enchantment_id] => {
            enchant_targets(state, parse_name_list(targets), enchantment_id, 1)
        }
        ["enchant", targets, enchantment_id, level] => enchant_targets(
            state,
            parse_name_list(targets),
            enchantment_id,
            parse_non_negative_i32(level)?,
        ),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn enchant_targets(
    state: &mut ServerCommandState,
    targets: Vec<NameAndId>,
    enchantment_id: &str,
    level: i32,
) -> Result<CommandResult, CommandError> {
    let enchantment_id = parse_resource_identifier(enchantment_id)?;
    let Some(enchantment_def) = enchantment(&enchantment_id) else {
        return Err(CommandError::InvalidSyntax);
    };
    if level > enchantment_def.max_level {
        return Err(CommandError::EnchantLevelTooHigh);
    }

    let mut success = 0;
    for target_profile in &targets {
        let target = entity_ref(&target_profile.name);
        if matches!(entity_kind(state, &target), EntityKind::NonLiving) {
            if targets.len() == 1 {
                return Err(CommandError::EnchantNotLivingEntity);
            }
            continue;
        }
        let Some(item) = held_item(state, target_profile) else {
            if targets.len() == 1 {
                return Err(CommandError::EnchantNoItem);
            }
            continue;
        };
        if !item_supports_enchantment(item, enchantment_def.supported_items)
            || !existing_enchantments_compatible(state, &target, item, &enchantment_id)
        {
            if targets.len() == 1 {
                return Err(CommandError::EnchantIncompatible);
            }
            continue;
        }
        upsert_item_enchantment(
            state,
            CommandItemEnchantment {
                target,
                item: item.to_string(),
                enchantment: enchantment_id.clone(),
                level,
            },
        );
        success += 1;
    }
    if success == 0 {
        return Err(CommandError::EnchantFailed);
    }
    Ok(CommandResult {
        success_count: success,
        feedback_key: if targets.len() == 1 {
            "commands.enchant.success.single"
        } else {
            "commands.enchant.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn held_item<'a>(state: &'a ServerCommandState, player: &NameAndId) -> Option<&'a str> {
    state
        .player_inventories
        .iter()
        .find(|inventory| inventory.player.uuid == player.uuid)
        .and_then(|inventory| inventory.items.first())
        .filter(|item| item.count > 0)
        .map(|item| item.item.as_str())
}

/// Whether `item` is a member of the enchantment's `supported_items` tag. Delegates to
/// the exact tag-hierarchy resolution in `item_tags` (the previous `ends_with`
/// heuristics were approximate and keyed on the pre-1.21 tag names); a literal item id
/// (non-`#`) is matched directly, mirroring vanilla `HolderSet` direct entries.
pub(super) fn item_supports_enchantment(item: &str, supported_items: &str) -> bool {
    if supported_items.starts_with('#') {
        crate::item_tags::item_in_tag(item, supported_items)
    } else {
        supported_items == item
    }
}

pub(super) fn existing_enchantments_compatible(
    state: &ServerCommandState,
    target: &EntityRef,
    item: &str,
    new_enchantment: &str,
) -> bool {
    let Some(new_def) = enchantment(new_enchantment) else {
        return false;
    };
    state
        .item_enchantments
        .iter()
        .filter(|existing| existing.target.id == target.id && existing.item == item)
        .all(|existing| {
            enchantment(&existing.enchantment)
                .is_some_and(|existing_def| are_compatible(existing_def, new_def))
        })
}

pub(super) fn upsert_item_enchantment(
    state: &mut ServerCommandState,
    enchantment: CommandItemEnchantment,
) {
    if let Some(existing) = state.item_enchantments.iter_mut().find(|existing| {
        existing.target.id == enchantment.target.id
            && existing.item == enchantment.item
            && existing.enchantment == enchantment.enchantment
    }) {
        existing.level = enchantment.level;
    } else {
        state.item_enchantments.push(enchantment);
    }
}

pub(super) fn gamemode_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["gamemode", mode] => {
            let player = state
                .command_source_player
                .clone()
                .ok_or(CommandError::InvalidSyntax)?;
            set_gamemode_for_targets(state, parse_gamemode(mode)?, &[player])
        }
        ["gamemode", mode, targets @ ..] if !targets.is_empty() => {
            let targets = targets
                .iter()
                .map(|target| NameAndId::create_offline(target))
                .collect::<Vec<_>>();
            set_gamemode_for_targets(state, parse_gamemode(mode)?, &targets)
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn set_gamemode_for_targets(
    state: &mut ServerCommandState,
    mode: GameMode,
    targets: &[NameAndId],
) -> Result<CommandResult, CommandError> {
    let mut changed = 0;
    for target in targets {
        if player_gamemode(state, target) != mode {
            set_player_gamemode(state, target.clone(), mode);
            changed += 1;
        }
    }
    Ok(CommandResult {
        success_count: changed,
        feedback_key: if targets.len() == 1 {
            "commands.gamemode.success.self"
        } else {
            "commands.gamemode.success.other"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn gamerule_command(
    state: &mut ServerCommandState,
    parts: &[&str],
) -> Result<CommandResult, CommandError> {
    match parts {
        ["gamerule", rule] => {
            let value = game_rule_value(state, rule)?;
            Ok(CommandResult {
                success_count: value.command_result(),
                feedback_key: "commands.gamerule.query",
                broadcast_to_admins: false,
            })
        }
        ["gamerule", rule, value] => {
            let normalized = normalize_game_rule_name(rule);
            let definition = game_rule_definition(&normalized)?;
            let current = game_rule_value(state, &normalized)?;
            let parsed = parse_game_rule_value(value, &current, definition)?;
            set_game_rule_value(state, normalized, parsed);
            state.game_rule_syncs.push(GameRuleSyncEvent {
                rule: format!("minecraft:{}", definition.name),
                value: parsed.sync_value(),
            });
            Ok(CommandResult {
                success_count: parsed.command_result(),
                feedback_key: "commands.gamerule.set",
                broadcast_to_admins: true,
            })
        }
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn kill_entities(
    state: &mut ServerCommandState,
    targets: Vec<EntityRef>,
) -> Result<CommandResult, CommandError> {
    let count = targets.len() as i32;
    state.killed_entities.extend(targets);
    Ok(CommandResult {
        success_count: count,
        feedback_key: if count == 1 {
            "commands.kill.success.single"
        } else {
            "commands.kill.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn discover_reload_packs(state: &ServerCommandState) -> Vec<String> {
    let mut selected = state.selected_data_packs.clone();
    for pack in &state.available_data_packs {
        if !state.disabled_data_packs.contains(pack) && !selected.contains(pack) {
            selected.push(pack.clone());
        }
    }
    selected
}
