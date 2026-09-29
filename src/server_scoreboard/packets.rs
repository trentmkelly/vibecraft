//! Clientbound scoreboard packet construction (Java `ServerScoreboard` senders).
//!
//! Every function returns a *plain payload* (`VarInt id` + body, no length prefix and no
//! compression) ready for [`crate::network::world_broadcast::WorldPacketBus`], the same shape
//! the other world broadcasts use. The command/scoreboard model stores display names as plain
//! text, so they go on the wire as literal components (a `TAG_String`, which is how
//! `ComponentSerialization` encodes a style-less literal).

use crate::chat_formatting::ChatFormatting;
use crate::command::{ScoreboardObjective, TeamState};
use super::display_slot::DisplaySlot;
use crate::network::play::{
    ClientboundResetScorePacket, ClientboundSetDisplayObjectivePacket,
    ClientboundSetObjectivePacket, ClientboundSetPlayerTeamPacket, ClientboundSetScorePacket,
    NumberFormat, ObjectiveMethod, ObjectiveRenderType, TeamCollisionRule, TeamPacketMethod,
    TeamPacketParameters, TeamVisibility, CLIENTBOUND_RESET_SCORE_PACKET_ID,
    CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID, CLIENTBOUND_SET_OBJECTIVE_PACKET_ID,
    CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID, CLIENTBOUND_SET_SCORE_PACKET_ID,
};
use crate::network::varint::write_var_i32;
use crate::storage::nbt::{parse_snbt, Tag};

use super::number_format::{parse_number_format, ParsedNumberFormat};

/// Literal text component as sent on the wire.
fn literal(text: &str) -> Tag {
    Tag::String(text.to_string())
}

/// `ObjectiveCriteria.RenderType` from its serialized name (`integer` / `hearts`).
fn render_type(name: &str) -> ObjectiveRenderType {
    if name == "hearts" {
        ObjectiveRenderType::Hearts
    } else {
        ObjectiveRenderType::Integer
    }
}

/// Model number-format string (`blank`, `fixed:<text>`, `styled:<snbt style>`) to the wire type.
fn wire_number_format(format: Option<&str>) -> Option<NumberFormat> {
    Some(match parse_number_format(format?)? {
        ParsedNumberFormat::Blank => NumberFormat::Blank,
        ParsedNumberFormat::Fixed(text) => NumberFormat::Fixed {
            value: literal(&text),
        },
        ParsedNumberFormat::Styled(style) => NumberFormat::Styled {
            style: parse_snbt(&style)
                .ok()
                .filter(|tag| matches!(tag, Tag::Compound(_)))
                .unwrap_or_else(|| Tag::Compound(Vec::new())),
        },
    })
}

/// Encodes `id` followed by the packet body written by `body`.
fn frame(id: i32, body: impl FnOnce(&mut Vec<u8>) -> std::io::Result<()>) -> Vec<u8> {
    let mut payload = Vec::new();
    // Writing into a `Vec` cannot fail.
    let _ = write_var_i32(&mut payload, id);
    let _ = body(&mut payload);
    payload
}

/// `ClientboundSetObjectivePacket.METHOD_ADD` / `METHOD_CHANGE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ObjectiveMethodKind {
    Add,
    Change,
}

/// `new ClientboundSetObjectivePacket(objective, METHOD_ADD | METHOD_CHANGE)`.
pub(super) fn set_objective(
    objective: &ScoreboardObjective,
    method: ObjectiveMethodKind,
) -> Vec<u8> {
    let display_name = literal(&objective.display_name);
    let render_type = render_type(&objective.render_type);
    let number_format = wire_number_format(objective.number_format.as_deref());
    let method = match method {
        ObjectiveMethodKind::Add => ObjectiveMethod::Add {
            display_name,
            render_type,
            number_format,
        },
        ObjectiveMethodKind::Change => ObjectiveMethod::Change {
            display_name,
            render_type,
            number_format,
        },
    };
    let packet = ClientboundSetObjectivePacket {
        objective_name: objective.name.clone(),
        method,
    };
    frame(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID, |out| packet.write(out))
}

/// `new ClientboundSetObjectivePacket(objective, METHOD_REMOVE)` (only the name is sent).
pub(super) fn remove_objective(name: &str) -> Vec<u8> {
    let packet = ClientboundSetObjectivePacket {
        objective_name: name.to_string(),
        method: ObjectiveMethod::Remove,
    };
    frame(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID, |out| packet.write(out))
}

/// `new ClientboundSetDisplayObjectivePacket(slot, objective)`; a missing objective is `""`.
pub(super) fn set_display_objective(slot: DisplaySlot, objective: Option<&str>) -> Vec<u8> {
    let packet = ClientboundSetDisplayObjectivePacket {
        slot: slot.id(),
        objective_name: objective.unwrap_or_default().to_string(),
    };
    frame(CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID, |out| {
        packet.write(out)
    })
}

/// `new ClientboundSetScorePacket(owner, objective, value, display, numberFormat)`.
pub(super) fn set_score(
    owner: &str,
    objective: &str,
    value: i32,
    display: Option<&str>,
    number_format: Option<&str>,
) -> Vec<u8> {
    let packet = ClientboundSetScorePacket {
        owner: owner.to_string(),
        objective_name: objective.to_string(),
        score: value,
        display: display.map(literal),
        number_format: wire_number_format(number_format),
    };
    frame(CLIENTBOUND_SET_SCORE_PACKET_ID, |out| packet.write(out))
}

/// `new ClientboundResetScorePacket(owner, objective)`; `None` resets every objective.
pub(super) fn reset_score(owner: &str, objective: Option<&str>) -> Vec<u8> {
    let packet = ClientboundResetScorePacket {
        owner: owner.to_string(),
        objective_name: objective.map(str::to_string),
    };
    frame(CLIENTBOUND_RESET_SCORE_PACKET_ID, |out| packet.write(out))
}

/// `ChatFormatting` ordinal for the packet's `writeEnum(color)`; unknown names are `RESET`.
fn color_ordinal(color: &str) -> i32 {
    let format = ChatFormatting::get_by_name(Some(color)).unwrap_or(ChatFormatting::Reset);
    ChatFormatting::VALUES
        .iter()
        .position(|candidate| *candidate == format)
        .unwrap_or(ChatFormatting::VALUES.len() - 1) as i32
}

fn visibility(name: &str) -> TeamVisibility {
    match name {
        "never" => TeamVisibility::Never,
        "hideForOtherTeams" => TeamVisibility::HideForOtherTeams,
        "hideForOwnTeam" => TeamVisibility::HideForOwnTeam,
        _ => TeamVisibility::Always,
    }
}

fn collision_rule(name: &str) -> TeamCollisionRule {
    match name {
        "never" => TeamCollisionRule::Never,
        "pushOtherTeams" => TeamCollisionRule::PushOtherTeams,
        "pushOwnTeam" => TeamCollisionRule::PushOwnTeam,
        _ => TeamCollisionRule::Always,
    }
}

/// `ClientboundSetPlayerTeamPacket.Parameters(team)`.
fn team_parameters(team: &TeamState) -> TeamPacketParameters {
    TeamPacketParameters {
        display_name: literal(&team.display_name),
        options: team.packed_options(),
        nametag_visibility: visibility(&team.nametag_visibility),
        collision_rule: collision_rule(&team.collision_rule),
        color_id: color_ordinal(&team.color),
        prefix: literal(&team.prefix),
        suffix: literal(&team.suffix),
    }
}

fn team_packet(name: &str, method: TeamPacketMethod) -> Vec<u8> {
    let packet = ClientboundSetPlayerTeamPacket {
        name: name.to_string(),
        method,
    };
    frame(CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID, |out| packet.write(out))
}

/// `ClientboundSetPlayerTeamPacket.createAddOrModifyPacket(team, createNew)`: a new team
/// (`Some(players)`) lists its current members, a modification (`None`) carries none.
pub(super) fn team_add_or_modify(team: &TeamState, players: Option<Vec<String>>) -> Vec<u8> {
    let parameters = team_parameters(team);
    let method = match players {
        Some(players) => TeamPacketMethod::Create {
            parameters,
            players,
        },
        None => TeamPacketMethod::Update { parameters },
    };
    team_packet(&team.name, method)
}

/// `ClientboundSetPlayerTeamPacket.createRemovePacket(team)`.
pub(super) fn team_remove(team: &str) -> Vec<u8> {
    team_packet(team, TeamPacketMethod::Remove)
}

/// `ClientboundSetPlayerTeamPacket.createPlayerPacket(team, player, action)`.
pub(super) fn team_player(team: &str, player: &str, add: bool) -> Vec<u8> {
    let players = vec![player.to_string()];
    team_packet(
        team,
        if add {
            TeamPacketMethod::AddPlayers { players }
        } else {
            TeamPacketMethod::RemovePlayers { players }
        },
    )
}
