use super::codec::{decode, encode, from_saved_data_tag, to_saved_data_tag};
use super::criteria::{PlayerVitals, RecordedVitals};
use super::display_slot::DisplaySlot;
use super::live::{CommandScoreboardSeed, SharedScoreboard};
use super::*;
use crate::command::{
    execute_command_with_functions, LevelBasedPermissionSet, PermissionLevel, ServerCommandState,
};
use crate::network::play::{
    CLIENTBOUND_RESET_SCORE_PACKET_ID, CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
    CLIENTBOUND_SET_OBJECTIVE_PACKET_ID, CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID,
    CLIENTBOUND_SET_SCORE_PACKET_ID,
};
use crate::network::varint::read_var_i32;
use crate::network::world_broadcast::WorldPacketBus;
use crate::storage::nbt::Tag;

fn objective(name: &str, criteria: &str) -> ScoreboardObjective {
    ScoreboardObjective {
        name: name.to_string(),
        criteria: criteria.to_string(),
        display_name: name.to_string(),
        render_type: "integer".to_string(),
        display_auto_update: false,
        number_format: None,
    }
}

fn ids(payloads: &[Vec<u8>]) -> Vec<i32> {
    payloads
        .iter()
        .map(|payload| read_var_i32(&mut payload.as_slice()).unwrap())
        .collect()
}

fn sidebar() -> DisplaySlot {
    DisplaySlot::by_name("sidebar").unwrap()
}

#[test]
fn displaying_an_objective_starts_tracking_with_objective_display_and_scores() {
    let mut scoreboard = ServerScoreboard::default();
    scoreboard.add_objective(objective("kills", "dummy"));
    scoreboard.set_score("Steve", "kills", 3, None);
    // Untracked objectives send no score packet (`trackedObjectives.contains`).
    assert!(scoreboard.take_outbox().is_empty());

    scoreboard.set_display_objective(sidebar(), Some("kills"));
    assert_eq!(
        ids(&scoreboard.take_outbox()),
        [
            CLIENTBOUND_SET_OBJECTIVE_PACKET_ID,
            CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
            CLIENTBOUND_SET_SCORE_PACKET_ID,
        ]
    );

    scoreboard.set_score("Steve", "kills", 4, None);
    assert_eq!(ids(&scoreboard.take_outbox()), [CLIENTBOUND_SET_SCORE_PACKET_ID]);
    // An unchanged value on an existing score sends nothing (`ScoreAccess.set`).
    scoreboard.set_score("Steve", "kills", 4, None);
    assert!(scoreboard.take_outbox().is_empty());
}

#[test]
fn moving_a_slot_and_removing_an_objective_follow_java_tracking_rules() {
    let mut scoreboard = ServerScoreboard::default();
    scoreboard.add_objective(objective("a", "dummy"));
    scoreboard.add_objective(objective("b", "dummy"));
    scoreboard.set_display_objective(sidebar(), Some("a"));
    scoreboard.take_outbox();

    // Replacing the only slot of `a` stops tracking it, then starts tracking `b`.
    scoreboard.set_display_objective(sidebar(), Some("b"));
    let sent = scoreboard.take_outbox();
    assert_eq!(
        ids(&sent),
        [
            CLIENTBOUND_SET_OBJECTIVE_PACKET_ID,
            CLIENTBOUND_SET_OBJECTIVE_PACKET_ID,
            CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
        ]
    );

    scoreboard.remove_objective("b");
    assert_eq!(ids(&scoreboard.take_outbox()), [CLIENTBOUND_SET_OBJECTIVE_PACKET_ID]);
    assert!(scoreboard.data().display_slots.is_empty());
}

#[test]
fn resetting_a_holders_last_score_sends_a_full_reset() {
    let mut scoreboard = ServerScoreboard::default();
    scoreboard.add_objective(objective("a", "dummy"));
    scoreboard.add_objective(objective("b", "dummy"));
    scoreboard.set_display_objective(sidebar(), Some("a"));
    scoreboard.set_score("Steve", "a", 1, None);
    scoreboard.set_score("Steve", "b", 2, None);
    scoreboard.take_outbox();

    scoreboard.reset_single_player_score("Steve", "a");
    assert_eq!(ids(&scoreboard.take_outbox()), [CLIENTBOUND_RESET_SCORE_PACKET_ID]);
    assert_eq!(scoreboard.score_value("Steve", "b"), Some(2));

    // The last remaining score removes the holder entirely (`onPlayerRemoved`).
    scoreboard.reset_single_player_score("Steve", "b");
    let sent = scoreboard.take_outbox();
    assert_eq!(ids(&sent), [CLIENTBOUND_RESET_SCORE_PACKET_ID]);
    // owner "Steve" then `false` (no objective name) closes the packet.
    assert_eq!(sent[0].last(), Some(&0));
}

#[test]
fn team_hooks_broadcast_create_join_leave_and_remove() {
    let mut scoreboard = ServerScoreboard::default();
    scoreboard.add_player_team(TeamState::new("red".into(), "Red".into()));
    scoreboard.add_player_team(TeamState::new("blue".into(), "Blue".into()));
    assert!(scoreboard.add_player_to_team("Steve", "red"));
    // Joining another team leaves the first (broadcast) and then joins.
    assert!(scoreboard.add_player_to_team("Steve", "blue"));
    assert!(!scoreboard.add_player_to_team("Steve", "missing"));
    scoreboard.remove_player_team("blue");
    let team_packets = CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID;
    assert_eq!(ids(&scoreboard.take_outbox()), [team_packets; 6]);
    assert!(scoreboard.data().members.is_empty());
}

#[test]
fn joining_players_receive_teams_then_displayed_objectives() {
    let mut scoreboard = ServerScoreboard::default();
    scoreboard.add_objective(objective("kills", "dummy"));
    scoreboard.add_player_team(TeamState::new("red".into(), "Red".into()));
    scoreboard.set_display_objective(sidebar(), Some("kills"));
    scoreboard.set_display_objective(DisplaySlot::by_name("list").unwrap(), Some("kills"));
    scoreboard.take_outbox();
    assert_eq!(
        ids(&scoreboard.join_packets()),
        [
            CLIENTBOUND_SET_PLAYER_TEAM_PACKET_ID,
            // `list` comes first in `DisplaySlot` order; `kills` is sent once.
            CLIENTBOUND_SET_OBJECTIVE_PACKET_ID,
            CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
            CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID,
        ]
    );
}

fn sample_data() -> ScoreboardData {
    let mut kills = objective("kills", "playerKillCount");
    kills.render_type = "hearts".to_string();
    kills.display_auto_update = true;
    kills.number_format = Some("fixed:pts".to_string());
    let mut red = TeamState::new("red".into(), "Red Team".into());
    red.color = "red".to_string();
    red.friendly_fire = false;
    red.prefix = "[R] ".to_string();
    red.nametag_visibility = "hideForOtherTeams".to_string();
    red.collision_rule = "pushOwnTeam".to_string();
    ScoreboardData {
        objectives: vec![kills, objective("plain", "dummy")],
        scores: vec![
            ScoreboardScore {
                owner: "Steve".into(),
                objective: "kills".into(),
                value: 7,
                locked: true,
                display_name: Some("Slayer".into()),
                number_format: Some("blank".into()),
            },
            ScoreboardScore {
                owner: "Alex".into(),
                objective: "plain".into(),
                value: 0,
                locked: false,
                display_name: None,
                number_format: None,
            },
        ],
        display_slots: vec![ScoreboardDisplaySlot {
            slot: "sidebar".into(),
            objective: "kills".into(),
        }],
        teams: vec![red, TeamState::new("blue".into(), "blue".into())],
        members: vec![TeamMember {
            player: "Steve".into(),
            team: "red".into(),
        }],
    }
}

#[test]
fn codec_round_trips_and_uses_java_field_names_and_defaults() {
    let data = sample_data();
    let tag = encode(&data);
    assert_eq!(decode(&tag).unwrap(), data);
    assert_eq!(from_saved_data_tag(&to_saved_data_tag(&data)).unwrap(), data);

    let Tag::Compound(root) = &tag else { panic!() };
    let names: Vec<&str> = root.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["Objectives", "PlayerScores", "DisplaySlots", "Teams"]);
    let Some((_, Tag::List(scores))) = root.iter().find(|(name, _)| name == "PlayerScores") else {
        panic!()
    };
    // Alex's default score omits Score/Locked (`optionalFieldOf` defaults); Steve's keeps them
    // and stores the display/format under Java's `display`/`format` keys.
    let Tag::Compound(alex) = &scores[1] else { panic!() };
    assert_eq!(alex.len(), 2);
    let Tag::Compound(steve) = &scores[0] else { panic!() };
    for key in ["Name", "Objective", "Score", "Locked", "display", "format"] {
        assert!(steve.iter().any(|(name, _)| name == key), "missing {key}");
    }
}

#[test]
fn codec_defaults_apply_to_absent_fields_and_bad_shapes_fail() {
    let minimal = Tag::Compound(vec![(
        "Objectives".into(),
        Tag::List(vec![Tag::Compound(vec![
            ("Name".into(), Tag::String("o".into())),
            ("DisplayName".into(), Tag::String("O".into())),
        ])]),
    )]);
    let data = decode(&minimal).unwrap();
    assert_eq!(data.objectives[0].criteria, "dummy");
    assert_eq!(data.objectives[0].render_type, "integer");
    assert!(!data.objectives[0].display_auto_update);
    assert_eq!(decode(&Tag::Compound(Vec::new())).unwrap(), ScoreboardData::default());

    let missing_name = Tag::Compound(vec![(
        "Objectives".into(),
        Tag::List(vec![Tag::Compound(Vec::new())]),
    )]);
    assert!(decode(&missing_name).is_err());
}

#[test]
fn loading_tracks_displayed_objectives_and_drops_unknown_scores() {
    let mut data = sample_data();
    data.scores.push(ScoreboardScore {
        owner: "Ghost".into(),
        objective: "gone".into(),
        value: 1,
        locked: false,
        display_name: None,
        number_format: None,
    });
    let mut scoreboard = ServerScoreboard::load(data);
    assert!(scoreboard.take_outbox().is_empty());
    assert!(!scoreboard.is_dirty());
    assert_eq!(scoreboard.score_value("Ghost", "gone"), None);
    assert_eq!(scoreboard.data().members.len(), 1);
    // `kills` was tracked by the load (Java `setDisplayObjective`), so score changes broadcast.
    scoreboard.set_score("Steve", "kills", 8, None);
    assert_eq!(ids(&scoreboard.take_outbox()), [CLIENTBOUND_SET_SCORE_PACKET_ID]);
    assert!(scoreboard.take_dirty_data().is_some());
    assert!(scoreboard.take_dirty_data().is_none());
}

fn run(scoreboard: &SharedScoreboard, bus: &WorldPacketBus, command: &str) {
    let mut state = ServerCommandState::default();
    let seed = CommandScoreboardSeed::seed(scoreboard, &mut state);
    let permissions = LevelBasedPermissionSet::new(PermissionLevel::Owners);
    execute_command_with_functions(&mut state, permissions, command)
        .unwrap_or_else(|error| panic!("{command}: {error:?}"));
    seed.apply(scoreboard, &state, bus);
}

#[test]
fn commands_share_one_scoreboard_and_broadcast_java_packets() {
    let scoreboard = SharedScoreboard::default();
    let bus = WorldPacketBus::default();
    let inbox = bus.subscribe(1);
    let drain = |inbox: &crate::network::world_broadcast::Subscription| {
        let mut framed = Vec::new();
        inbox.drain_into(&mut framed, crate::network::compression::CompressionState::disabled()).unwrap();
        framed
    };

    run(&scoreboard, &bus, "scoreboard objectives add kills dummy");
    run(&scoreboard, &bus, "scoreboard players set Steve kills 5");
    assert!(drain(&inbox).is_empty(), "untracked objective sends nothing");

    run(&scoreboard, &bus, "scoreboard objectives setdisplay sidebar kills");
    assert!(!drain(&inbox).is_empty());
    run(&scoreboard, &bus, "scoreboard players add Steve kills 2");
    assert!(!drain(&inbox).is_empty());
    assert_eq!(scoreboard.lock().score_value("Steve", "kills"), Some(7));

    run(&scoreboard, &bus, "team add red");
    run(&scoreboard, &bus, "team join red Alex");
    run(&scoreboard, &bus, "team modify red color red");
    let data = scoreboard.lock().data().clone();
    assert_eq!(data.team("red").unwrap().color, "red");
    assert_eq!(data.team_of("Alex"), Some("red"));

    run(&scoreboard, &bus, "team leave Alex");
    run(&scoreboard, &bus, "scoreboard players reset Steve");
    run(&scoreboard, &bus, "scoreboard objectives remove kills");
    let data = scoreboard.lock().data().clone();
    assert!(data.objectives.is_empty() && data.scores.is_empty());
    assert!(data.members.is_empty());
    assert!(scoreboard.lock().is_dirty());
}

#[test]
fn trigger_scores_persist_between_commands() {
    let scoreboard = SharedScoreboard::default();
    let bus = WorldPacketBus::default();
    run(&scoreboard, &bus, "scoreboard objectives add t trigger");
    run(&scoreboard, &bus, "scoreboard players enable Steve t");
    assert_eq!(scoreboard.lock().data().score("Steve", "t").map(|s| s.locked), Some(false));
}

#[test]
fn criteria_scores_follow_death_count_and_recorded_vitals() {
    let mut scoreboard = ServerScoreboard::default();
    scoreboard.add_objective(objective("deaths", "deathCount"));
    scoreboard.add_objective(objective("hp", "health"));
    scoreboard.add_objective(objective("plain", "dummy"));
    scoreboard.increment_death_count("Steve");
    scoreboard.increment_death_count("Steve");
    assert_eq!(scoreboard.score_value("Steve", "deaths"), Some(2));
    assert_eq!(scoreboard.score_value("Steve", "plain"), None);

    let vitals = PlayerVitals {
        health_and_absorption: 19.5,
        food_level: 20,
        air_supply: 300,
        armor: 0.0,
        experience: 0,
        level: 0,
    };
    let mut recorded = RecordedVitals::default();
    recorded.update(&mut scoreboard, "Steve", vitals);
    // `Mth.ceil(19.5)`.
    assert_eq!(scoreboard.score_value("Steve", "hp"), Some(20));
    scoreboard.take_outbox();
    recorded.update(&mut scoreboard, "Steve", vitals);
    assert!(scoreboard.take_outbox().is_empty());
    recorded.update(&mut scoreboard, "Steve", PlayerVitals { health_and_absorption: 4.0, ..vitals });
    assert_eq!(scoreboard.score_value("Steve", "hp"), Some(4));
}
