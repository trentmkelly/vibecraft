use super::*;
use crate::command::{execute_builtin_command, CommandError, CommandResult};
use crate::command::{LevelBasedPermissionSet, PermissionLevel};
use std::fs;

fn loopback_pair() -> (TcpStream, TcpStream) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let (server, _) = listener.accept().unwrap();
    (client, server)
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vibecraft-access-live-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn properties(pairs: &[(&str, &str)]) -> ServerProperties {
    let mut properties =
        ServerProperties::load_or_default(Path::new("definitely-missing-test-server.properties"))
            .unwrap();
    for (key, value) in pairs {
        properties.set(key, *value);
    }
    properties
}

struct Online {
    profile: NameAndId,
    _guard: ActiveLoginGuard,
    inbox: Subscription,
    _peer: TcpStream,
}

fn join(registry: &ActiveLoginRegistry, name: &str) -> Online {
    let profile = NameAndId::create_offline(name);
    let (client, peer) = loopback_pair();
    let (guard, _) = registry.register_replacing(&profile.uuid, name, &client).unwrap();
    guard.mark_in_play();
    let inbox = registry.world_bus.subscribe(guard.token);
    Online { profile, _guard: guard, inbox, _peer: peer }
}

impl Online {
    fn disconnect_text(&self) -> Option<String> {
        let mut framed = Vec::new();
        self.inbox.drain_into(&mut framed, CompressionState::disabled()).unwrap();
        (!framed.is_empty()).then(|| String::from_utf8_lossy(&framed).into_owned())
    }
}

struct Server {
    dir: PathBuf,
    registry: ActiveLoginRegistry,
    access: Arc<Mutex<PlayerAccess>>,
    properties: ServerProperties,
}

impl Server {
    fn new(name: &str, pairs: &[(&str, &str)]) -> Self {
        let dir = temp_dir(name);
        let mut access = PlayerAccess::load_from_dir(&dir).unwrap();
        let properties = properties(pairs);
        access.set_using_whitelist(properties.white_list);
        Self {
            dir,
            registry: ActiveLoginRegistry::default(),
            access: Arc::new(Mutex::new(access)),
            properties,
        }
    }

    /// Runs `line` as the console the way `ServerCommandRunner::execute` does.
    fn run(&self, line: &str) -> (Result<CommandResult, CommandError>, ServerCommandState) {
        let properties_file = self.dir.join("server.properties");
        let context = AccessContext {
            access: &self.access,
            properties: &self.properties,
            sessions: self.registry.live_sessions(),
            properties_file: Some(&properties_file),
        };
        let mut state = ServerCommandState {
            online_players: self.registry.live_sessions().in_play_profiles(),
            ..ServerCommandState::default()
        };
        let seed = seed_access_state(&mut state, &self.access, context.sessions);
        let permissions = LevelBasedPermissionSet::new(PermissionLevel::Owners);
        let result = execute_builtin_command(&mut state, permissions, line);
        apply_access_changes(&context, &seed, &state);
        apply_console_disconnects(context.sessions, &state);
        (result, state)
    }

    fn file(&self, name: &str) -> String {
        fs::read_to_string(self.dir.join(name)).unwrap_or_default()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn op_and_deop_update_and_persist_ops_json_with_the_configured_level() {
    let server = Server::new("op", &[("op-permission-level", "3")]);
    let steve = NameAndId::create_offline("Steve");

    server.run("op Steve").0.unwrap();
    assert_eq!(lock_status_mutex(&server.access).op_level(&steve.uuid), Some(3));
    let ops = server.file("ops.json");
    assert!(ops.contains("\"level\": 3") && ops.contains("\"bypassesPlayerLimit\": false"), "{ops}");
    assert!(matches!(server.run("op Steve").0, Err(CommandError::OpFailed)));

    server.run("deop Steve").0.unwrap();
    assert!(!lock_status_mutex(&server.access).is_op(&steve.uuid));
    assert_eq!(server.file("ops.json"), "[]");
    assert!(matches!(server.run("deop Steve").0, Err(CommandError::DeOpFailed)));
}

#[test]
fn op_keeps_an_existing_player_limit_bypass_and_uses_cached_uuids() {
    let server = Server::new("op-cache", &[]);
    let cached = NameAndId { uuid: "11111111-2222-3333-4444-555555555555".into(), name: "Alex".into() };
    lock_status_mutex(&server.access).cache_user(cached.clone());

    server.run("op alex").0.unwrap();

    assert!(lock_status_mutex(&server.access).is_op(&cached.uuid));
    assert!(server.file("ops.json").contains(&cached.uuid));
}

#[test]
fn ban_persists_with_source_and_reason_and_kicks_the_online_target() {
    let server = Server::new("ban", &[]);
    let steve = join(&server.registry, "Steve");
    let alex = join(&server.registry, "Alex");

    server.run("ban Steve griefing spawn").0.unwrap();

    let banned = server.file("banned-players.json");
    assert!(banned.contains("\"name\": \"Steve\"") && banned.contains("\"reason\": \"griefing spawn\""), "{banned}");
    assert!(banned.contains("\"source\": \"Server\"") && banned.contains("\"expires\": \"forever\""), "{banned}");
    assert!(steve.disconnect_text().unwrap().contains("multiplayer.disconnect.banned"));
    assert!(alex.disconnect_text().is_none());
    assert!(lock_status_mutex(&server.access).is_player_banned(&steve.profile.uuid));

    server.run("pardon Steve").0.unwrap();
    assert!(!lock_status_mutex(&server.access).is_player_banned(&steve.profile.uuid));
    assert_eq!(server.file("banned-players.json"), "[]");
}

#[test]
fn ban_without_a_reason_omits_the_reason_key_like_gson() {
    let server = Server::new("ban-null", &[]);
    server.run("ban Griefer").0.unwrap();
    assert!(!server.file("banned-players.json").contains("reason"));
}

#[test]
fn ban_ip_by_player_name_persists_and_disconnects_players_on_that_address() {
    let server = Server::new("banip", &[]);
    let steve = join(&server.registry, "Steve");

    server.run("ban-ip Steve rude").0.unwrap();

    assert!(server.file("banned-ips.json").contains("\"ip\": \"127.0.0.1\""));
    assert!(steve.disconnect_text().unwrap().contains("multiplayer.disconnect.ip_banned"));
    server.run("pardon-ip 127.0.0.1").0.unwrap();
    assert!(!lock_status_mutex(&server.access).is_ip_banned("127.0.0.1"));
    assert_eq!(server.file("banned-ips.json"), "[]");
}

#[test]
fn whitelist_on_kicks_unlisted_players_only_with_enforce_whitelist() {
    let server = Server::new("wl-on", &[("enforce-whitelist", "true")]);
    let steve = join(&server.registry, "Steve");
    let alex = join(&server.registry, "Alex");
    server.run("whitelist add Alex").0.unwrap();
    assert!(server.file("whitelist.json").contains("\"name\": \"Alex\""));
    assert!(steve.disconnect_text().is_none(), "whitelist is still off");

    server.run("whitelist on").0.unwrap();

    assert!(lock_status_mutex(&server.access).using_whitelist());
    assert!(server.file("server.properties").contains("white-list=true"));
    assert!(steve.disconnect_text().unwrap().contains("multiplayer.disconnect.not_whitelisted"));
    assert!(alex.disconnect_text().is_none());
    assert!(matches!(server.run("whitelist on").0, Err(CommandError::WhitelistAlreadyOn)));

    server.run("whitelist off").0.unwrap();
    assert!(!lock_status_mutex(&server.access).using_whitelist());
    assert!(server.file("server.properties").contains("white-list=false"));
}

#[test]
fn whitelist_on_does_not_kick_without_enforce_whitelist() {
    let server = Server::new("wl-noenforce", &[]);
    let steve = join(&server.registry, "Steve");
    server.run("whitelist on").0.unwrap();
    assert!(steve.disconnect_text().is_none());
}

#[test]
fn whitelist_remove_and_reload_kick_players_no_longer_listed() {
    let server = Server::new("wl-remove", &[("enforce-whitelist", "true"), ("white-list", "true")]);
    let steve = join(&server.registry, "Steve");
    let alex = join(&server.registry, "Alex");
    server.run("whitelist add Steve").0.unwrap();
    server.run("whitelist add Alex").0.unwrap();

    server.run("whitelist remove Steve").0.unwrap();
    assert!(steve.disconnect_text().unwrap().contains("not_whitelisted"));
    assert!(alex.disconnect_text().is_none());
    assert!(!server.file("whitelist.json").contains("Steve"));

    // Hot-editing whitelist.json and reloading drops Alex (operators are not exempt).
    fs::write(server.dir.join("whitelist.json"), "[]").unwrap();
    server.run("whitelist reload").0.unwrap();
    assert!(lock_status_mutex(&server.access).whitelisted().is_empty());
    assert!(alex.disconnect_text().unwrap().contains("not_whitelisted"));
}

#[test]
fn login_gate_reports_ban_reason_and_expiry_components() {
    let server = Server::new("login-ban", &[]);
    let griefer = NameAndId::create_offline("Griefer");
    let mut access = lock_status_mutex(&server.access);
    access.ban_player(BanEntry {
        user: griefer.clone(),
        created: "2026-01-01 00:00:00 +0000".into(),
        source: "Server".into(),
        expires: Some("2999-01-01 00:00:00 +0000".into()),
        reason: Some("spam".into()),
    });
    access.ban_ip(BanEntry {
        user: "203.0.113.7".into(),
        created: "2026-01-01 00:00:00 +0000".into(),
        source: "Server".into(),
        expires: None,
        reason: None,
    });

    let json = can_player_login(&access, &server.properties, &griefer, "1.1.1.1", 0).unwrap();
    assert!(json.contains("multiplayer.disconnect.banned.reason") && json.contains("spam"), "{json}");
    assert!(json.contains("multiplayer.disconnect.banned.expiration"), "{json}");
    assert!(json.contains("2999-01-01 at 00:00:00 UTC") || json.contains("2998-12-31 at"), "{json}");

    let steve = NameAndId::create_offline("Steve");
    let json = can_player_login(&access, &server.properties, &steve, "203.0.113.7", 0).unwrap();
    assert!(json.contains("multiplayer.disconnect.banned_ip.reason"), "{json}");
    assert!(json.contains("multiplayer.disconnect.banned.reason.default"), "{json}");
    assert!(!json.contains("expiration"), "{json}");
}

#[test]
fn login_gate_expired_bans_no_longer_apply() {
    let server = Server::new("login-expired", &[]);
    let steve = NameAndId::create_offline("Steve");
    let mut access = lock_status_mutex(&server.access);
    access.ban_player(BanEntry {
        user: steve.clone(),
        created: "2000-01-01 00:00:00 +0000".into(),
        source: "Server".into(),
        expires: Some("2000-02-01 00:00:00 +0000".into()),
        reason: None,
    });
    assert!(can_player_login(&access, &server.properties, &steve, "1.1.1.1", 0).is_none());
}

#[test]
fn login_gate_whitelist_uses_the_live_flag_and_exempts_operators() {
    let server = Server::new("login-wl", &[("enforce-whitelist", "false")]);
    let steve = NameAndId::create_offline("Steve");
    let alex = NameAndId::create_offline("Alex");
    let boss = NameAndId::create_offline("Boss");
    let mut access = lock_status_mutex(&server.access);
    access.whitelist(alex.clone());
    access.op(OpEntry { user: boss.clone(), level: 4, bypasses_player_limit: false });
    assert!(can_player_login(&access, &server.properties, &steve, "1.1.1.1", 0).is_none());

    access.set_using_whitelist(true);
    let json = can_player_login(&access, &server.properties, &steve, "1.1.1.1", 0).unwrap();
    assert!(json.contains("multiplayer.disconnect.not_whitelisted"), "{json}");
    assert!(can_player_login(&access, &server.properties, &alex, "1.1.1.1", 0).is_none());
    assert!(can_player_login(&access, &server.properties, &boss, "1.1.1.1", 0).is_none());
}

#[test]
fn login_gate_enforces_max_players_unless_the_operator_bypasses_the_limit() {
    let server = Server::new("login-full", &[("max-players", "2")]);
    let steve = NameAndId::create_offline("Steve");
    let vip = NameAndId::create_offline("Vip");
    let mut access = lock_status_mutex(&server.access);
    access.op(OpEntry { user: vip.clone(), level: 4, bypasses_player_limit: true });

    assert!(can_player_login(&access, &server.properties, &steve, "1.1.1.1", 1).is_none());
    let json = can_player_login(&access, &server.properties, &steve, "1.1.1.1", 2).unwrap();
    assert!(json.contains("multiplayer.disconnect.server_full"), "{json}");
    assert!(can_player_login(&access, &server.properties, &vip, "1.1.1.1", 2).is_none());
}

#[test]
fn permission_update_sends_the_level_event_before_the_command_tree() {
    let (client, mut server) = loopback_pair();
    let mut client = client;
    let mut sync = CommandTreeSync::default();
    sync.observe_permission_level(2);

    write_permission_level_update(&mut client, CompressionState::disabled(), &mut sync).unwrap();

    server.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    let mut first = [0u8; 7];
    server.read_exact(&mut first).unwrap();
    // length 6 = id(1) + entity id(4) + event(1); event 24 + level 2 = 26 on the local player.
    assert_eq!(first, [6, CLIENTBOUND_ENTITY_EVENT_PACKET_ID as u8, 0, 0, 0, 1, 26]);
    let mut next_id = [0u8; 4];
    server.read_exact(&mut next_id).unwrap();
    assert!(!sync.needs_resend());
}
