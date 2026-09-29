//! `ScoreboardSaveData.Packed.CODEC` as NBT (`data/minecraft/scoreboard.dat`).
//!
//! Field names, defaults and omission rules follow the `RecordCodecBuilder` codecs in Java:
//! `Objective.Packed`, `Scoreboard.PackedScore` (+ `Score.Packed`), `PlayerTeam.Packed` and the
//! `DisplaySlots` map. `optionalFieldOf(name, default)` omits a field that equals its default
//! when encoding and supplies the default when decoding; booleans decode leniently from any
//! numeric tag. The saved-data file nests the record under `data` beside `DataVersion`
//! (`SavedDataStorage`), which [`to_saved_data_tag`]/[`from_saved_data_tag`] handle.

use crate::chat_formatting::ChatFormatting;
use crate::command::{ScoreboardDisplaySlot, ScoreboardObjective, ScoreboardScore, TeamState};
use super::display_slot::DisplaySlot;
use crate::storage::nbt::Tag;

use super::number_format::{number_format_from_tag, number_format_to_tag, plain_text};
use super::{ScoreboardData, TeamMember};

/// Wraps the codec output as `SavedDataStorage` stores it (the storage adds `DataVersion`).
pub fn to_saved_data_tag(data: &ScoreboardData) -> Tag {
    Tag::Compound(vec![("data".to_string(), encode(data))])
}

/// Reads the codec record out of a saved-data root tag.
pub fn from_saved_data_tag(root: &Tag) -> Result<ScoreboardData, String> {
    let Tag::Compound(fields) = root else {
        return Err("scoreboard.dat root is not a compound".to_string());
    };
    let (_, data) = fields
        .iter()
        .find(|(name, _)| name == "data")
        .ok_or_else(|| "scoreboard.dat has no data".to_string())?;
    decode(data)
}

fn string(text: &str) -> Tag {
    Tag::String(text.to_string())
}

fn bool_tag(value: bool) -> Tag {
    Tag::Byte(i8::from(value))
}

/// `ScoreboardSaveData.Packed.CODEC` encode.
pub fn encode(data: &ScoreboardData) -> Tag {
    let mut fields = Vec::new();
    if !data.objectives.is_empty() {
        let list = data.objectives.iter().map(encode_objective).collect();
        fields.push(("Objectives".to_string(), Tag::List(list)));
    }
    if !data.scores.is_empty() {
        let list = data.scores.iter().map(encode_score).collect();
        fields.push(("PlayerScores".to_string(), Tag::List(list)));
    }
    if !data.display_slots.is_empty() {
        let slots = data
            .display_slots
            .iter()
            .map(|slot| (slot.slot.clone(), string(&slot.objective)))
            .collect();
        fields.push(("DisplaySlots".to_string(), Tag::Compound(slots)));
    }
    if !data.teams.is_empty() {
        let list = data
            .teams
            .iter()
            .map(|team| encode_team(team, &data.team_players(&team.name)))
            .collect();
        fields.push(("Teams".to_string(), Tag::List(list)));
    }
    Tag::Compound(fields)
}

/// `Objective.Packed.CODEC`.
fn encode_objective(objective: &ScoreboardObjective) -> Tag {
    let mut fields = vec![("Name".to_string(), string(&objective.name))];
    if objective.criteria != "dummy" {
        fields.push(("CriteriaName".to_string(), string(&objective.criteria)));
    }
    fields.push(("DisplayName".to_string(), string(&objective.display_name)));
    if objective.render_type != "integer" {
        fields.push(("RenderType".to_string(), string(&objective.render_type)));
    }
    if objective.display_auto_update {
        fields.push(("display_auto_update".to_string(), bool_tag(true)));
    }
    if let Some(format) = objective.number_format.as_deref().and_then(number_format_to_tag) {
        fields.push(("format".to_string(), format));
    }
    Tag::Compound(fields)
}

/// `Scoreboard.PackedScore.CODEC` with the `Score.Packed` map codec merged in.
fn encode_score(score: &ScoreboardScore) -> Tag {
    let mut fields = vec![
        ("Name".to_string(), string(&score.owner)),
        ("Objective".to_string(), string(&score.objective)),
    ];
    if score.value != 0 {
        fields.push(("Score".to_string(), Tag::Int(score.value)));
    }
    if score.locked {
        fields.push(("Locked".to_string(), bool_tag(true)));
    }
    if let Some(display) = &score.display_name {
        fields.push(("display".to_string(), string(display)));
    }
    if let Some(format) = score.number_format.as_deref().and_then(number_format_to_tag) {
        fields.push(("format".to_string(), format));
    }
    Tag::Compound(fields)
}

/// `PlayerTeam.Packed.CODEC`.
fn encode_team(team: &TeamState, players: &[String]) -> Tag {
    let mut fields = vec![
        ("Name".to_string(), string(&team.name)),
        ("DisplayName".to_string(), string(&team.display_name)),
    ];
    if team.color != "reset" {
        fields.push(("TeamColor".to_string(), string(&team.color)));
    }
    if !team.friendly_fire {
        fields.push(("AllowFriendlyFire".to_string(), bool_tag(false)));
    }
    if !team.see_friendly_invisibles {
        fields.push(("SeeFriendlyInvisibles".to_string(), bool_tag(false)));
    }
    if !team.prefix.is_empty() {
        fields.push(("MemberNamePrefix".to_string(), string(&team.prefix)));
    }
    if !team.suffix.is_empty() {
        fields.push(("MemberNameSuffix".to_string(), string(&team.suffix)));
    }
    if team.nametag_visibility != "always" {
        fields.push(("NameTagVisibility".to_string(), string(&team.nametag_visibility)));
    }
    if team.death_message_visibility != "always" {
        fields.push((
            "DeathMessageVisibility".to_string(),
            string(&team.death_message_visibility),
        ));
    }
    if team.collision_rule != "always" {
        fields.push(("CollisionRule".to_string(), string(&team.collision_rule)));
    }
    if !players.is_empty() {
        let list = players.iter().map(|player| string(player)).collect();
        fields.push(("Players".to_string(), Tag::List(list)));
    }
    Tag::Compound(fields)
}

// ---- decoding ------------------------------------------------------------------------------

type Fields<'a> = &'a [(String, Tag)];

fn compound(tag: &Tag) -> Result<Fields<'_>, String> {
    match tag {
        Tag::Compound(fields) => Ok(fields),
        _ => Err("expected a compound".to_string()),
    }
}

fn field<'a>(fields: Fields<'a>, name: &str) -> Option<&'a Tag> {
    fields.iter().find(|(key, _)| key == name).map(|(_, tag)| tag)
}

fn required_string(fields: Fields<'_>, name: &str) -> Result<String, String> {
    match field(fields, name) {
        Some(Tag::String(text)) => Ok(text.clone()),
        _ => Err(format!("missing string field {name}")),
    }
}

fn optional_string(fields: Fields<'_>, name: &str, default: &str) -> Result<String, String> {
    match field(fields, name) {
        None => Ok(default.to_string()),
        Some(Tag::String(text)) => Ok(text.clone()),
        Some(_) => Err(format!("field {name} is not a string")),
    }
}

/// A component field: a plain string or a `{text:"..."}` literal.
fn optional_component(fields: Fields<'_>, name: &str, default: &str) -> Result<String, String> {
    Ok(field(fields, name).map_or_else(|| default.to_string(), plain_text))
}

/// `Codec.BOOL`: any numeric tag, non-zero is true.
fn optional_bool(fields: Fields<'_>, name: &str, default: bool) -> Result<bool, String> {
    match field(fields, name) {
        None => Ok(default),
        Some(Tag::Byte(value)) => Ok(*value != 0),
        Some(Tag::Short(value)) => Ok(*value != 0),
        Some(Tag::Int(value)) => Ok(*value != 0),
        Some(Tag::Long(value)) => Ok(*value != 0),
        Some(_) => Err(format!("field {name} is not a boolean")),
    }
}

fn optional_number_format(fields: Fields<'_>) -> Result<Option<String>, String> {
    match field(fields, "format") {
        None => Ok(None),
        Some(tag) => number_format_from_tag(tag)
            .map(Some)
            .ok_or_else(|| "invalid number format".to_string()),
    }
}

fn list<'a>(fields: Fields<'a>, name: &str) -> Result<&'a [Tag], String> {
    match field(fields, name) {
        None => Ok(&[]),
        Some(Tag::List(entries)) => Ok(entries),
        Some(_) => Err(format!("{name} must be a list")),
    }
}

/// `ScoreboardSaveData.Packed.CODEC` decode.
pub fn decode(tag: &Tag) -> Result<ScoreboardData, String> {
    let root = compound(tag)?;
    let mut data = ScoreboardData::default();
    for entry in list(root, "Objectives")? {
        data.objectives.push(decode_objective(entry)?);
    }
    for entry in list(root, "PlayerScores")? {
        data.scores.push(decode_score(entry)?);
    }
    match field(root, "DisplaySlots") {
        None => {}
        Some(Tag::Compound(slots)) => {
            for (slot, objective) in slots {
                let Tag::String(objective) = objective else {
                    return Err("DisplaySlots values must be strings".to_string());
                };
                let slot = DisplaySlot::by_name(slot)
                    .ok_or_else(|| format!("unknown display slot {slot}"))?;
                data.display_slots.push(ScoreboardDisplaySlot {
                    slot: slot.serialized_name().to_string(),
                    objective: objective.clone(),
                });
            }
        }
        Some(_) => return Err("DisplaySlots must be a compound".to_string()),
    }
    for entry in list(root, "Teams")? {
        let (team, players) = decode_team(entry)?;
        data.members.extend(players.into_iter().map(|player| TeamMember {
            player,
            team: team.name.clone(),
        }));
        data.teams.push(team);
    }
    Ok(data)
}

fn decode_objective(tag: &Tag) -> Result<ScoreboardObjective, String> {
    let fields = compound(tag)?;
    Ok(ScoreboardObjective {
        name: required_string(fields, "Name")?,
        criteria: optional_string(fields, "CriteriaName", "dummy")?,
        display_name: match field(fields, "DisplayName") {
            Some(display) => plain_text(display),
            None => return Err("missing DisplayName".to_string()),
        },
        render_type: optional_string(fields, "RenderType", "integer")?,
        display_auto_update: optional_bool(fields, "display_auto_update", false)?,
        number_format: optional_number_format(fields)?,
    })
}

fn decode_score(tag: &Tag) -> Result<ScoreboardScore, String> {
    let fields = compound(tag)?;
    let value = match field(fields, "Score") {
        None => 0,
        Some(Tag::Int(value)) => *value,
        Some(_) => return Err("Score must be an int".to_string()),
    };
    Ok(ScoreboardScore {
        owner: required_string(fields, "Name")?,
        objective: required_string(fields, "Objective")?,
        value,
        locked: optional_bool(fields, "Locked", false)?,
        display_name: field(fields, "display").map(plain_text),
        number_format: optional_number_format(fields)?,
    })
}

fn decode_team(tag: &Tag) -> Result<(TeamState, Vec<String>), String> {
    let fields = compound(tag)?;
    let name = required_string(fields, "Name")?;
    let display_name = optional_component(fields, "DisplayName", &name)?;
    let mut team = TeamState::new(name, display_name);
    if let Some(Tag::String(color)) = field(fields, "TeamColor") {
        let known = ChatFormatting::get_by_name(Some(color)).filter(|format| format.is_color());
        team.color = known.map_or_else(|| "reset".to_string(), |format| format.get_name());
    }
    team.friendly_fire = optional_bool(fields, "AllowFriendlyFire", true)?;
    team.see_friendly_invisibles = optional_bool(fields, "SeeFriendlyInvisibles", true)?;
    team.prefix = optional_component(fields, "MemberNamePrefix", "")?;
    team.suffix = optional_component(fields, "MemberNameSuffix", "")?;
    team.nametag_visibility = optional_string(fields, "NameTagVisibility", "always")?;
    team.death_message_visibility = optional_string(fields, "DeathMessageVisibility", "always")?;
    team.collision_rule = optional_string(fields, "CollisionRule", "always")?;
    let players = list(fields, "Players")?
        .iter()
        .map(|player| match player {
            Tag::String(player) => Ok(player.clone()),
            _ => Err("Players entries must be strings".to_string()),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((team, players))
}
