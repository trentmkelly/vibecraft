#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChaseDimension {
    Overworld,
    Nether,
    End,
}

impl ChaseDimension {
    pub fn short_name(self) -> &'static str {
        match self {
            Self::Overworld => "o",
            Self::Nether => "n",
            Self::End => "e",
        }
    }

    pub fn identifier(self) -> &'static str {
        match self {
            Self::Overworld => "minecraft:overworld",
            Self::Nether => "minecraft:the_nether",
            Self::End => "minecraft:the_end",
        }
    }

    pub fn from_short_name(name: &str) -> Option<Self> {
        match name {
            "o" => Some(Self::Overworld),
            "n" => Some(Self::Nether),
            "e" => Some(Self::End),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChasePlayerPosition {
    pub dimension: ChaseDimension,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub y_rot: f32,
    pub x_rot: f32,
}

impl ChasePlayerPosition {
    pub fn format_server_message(self) -> String {
        format!(
            "t {} {:.2} {:.2} {:.2} {:.2} {:.2}\n",
            self.dimension.short_name(),
            self.x,
            self.y,
            self.z,
            self.y_rot,
            self.x_rot
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChaseTeleportTarget {
    pub dimension: ChaseDimension,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub y_rot: f32,
    pub x_rot: f32,
}

impl ChaseTeleportTarget {
    pub fn command(self) -> String {
        format!(
            "execute in {} run tp @s {:.3} {:.3} {:.3} {:.3} {:.3}",
            self.dimension.identifier(),
            self.x,
            self.y,
            self.z,
            self.y_rot,
            self.x_rot
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChaseClientModel {
    pub server_host: String,
    pub server_port: u16,
    pub wants_to_run: bool,
    pub thread_name: Option<String>,
    pub logs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChaseServerModel {
    pub bind_address: String,
    pub server_port: u16,
    pub broadcast_interval_ms: u64,
    pub wants_to_run: bool,
    pub server_socket_open: bool,
    pub client_socket_count: usize,
    pub logs: Vec<String>,
}

impl ChaseClientModel {
    pub const RECONNECT_INTERVAL_SECONDS: u64 = 5;

    pub fn new(server_host: impl Into<String>, server_port: u16) -> Self {
        Self {
            server_host: server_host.into(),
            server_port,
            wants_to_run: false,
            thread_name: None,
            logs: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        if self.thread_name.is_some() {
            self.logs.push(
                "Remote control client was asked to start, but it is already running. Will ignore."
                    .to_string(),
            );
        }
        self.wants_to_run = true;
        self.thread_name = Some("chase-client".to_string());
    }

    pub fn stop(&mut self) {
        self.wants_to_run = false;
        self.thread_name = None;
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }

    pub fn handle_message(&mut self, message: &str) -> Option<String> {
        let mut parts = message.split_whitespace();
        match parts.next() {
            Some("t") => Self::parse_target_parts(parts).map(ChaseTeleportTarget::command),
            Some(head) => {
                self.logs.push(format!("Unknown message type '{head}'"));
                None
            }
            None => {
                self.logs
                    .push(format!("Could not parse message '{message}', ignoring"));
                None
            }
        }
    }

    fn parse_target_parts<'a>(mut parts: impl Iterator<Item = &'a str>) -> Option<ChaseTeleportTarget> {
        let dimension = ChaseDimension::from_short_name(parts.next()?)?;
        let x = parts.next()?.parse::<f32>().ok()?;
        let y = parts.next()?.parse::<f32>().ok()?;
        let z = parts.next()?.parse::<f32>().ok()?;
        let y_rot = parts.next()?.parse::<f32>().ok()?;
        let x_rot = parts.next()?.parse::<f32>().ok()?;
        Some(ChaseTeleportTarget {
            dimension,
            x,
            y,
            z,
            y_rot,
            x_rot,
        })
    }
}

impl ChaseServerModel {
    pub const DEFAULT_BROADCAST_INTERVAL_MS: u64 = 100;

    pub fn new(bind_address: impl Into<String>, server_port: u16, broadcast_interval_ms: u64) -> Self {
        Self {
            bind_address: bind_address.into(),
            server_port,
            broadcast_interval_ms,
            wants_to_run: false,
            server_socket_open: false,
            client_socket_count: 0,
            logs: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        if self.server_socket_open {
            self.logs.push(
                "Remote control server was asked to start, but it is already running. Will ignore."
                    .to_string(),
            );
        } else {
            self.wants_to_run = true;
            self.server_socket_open = true;
            self.logs
                .push(format!("acceptor=chase-server-acceptor:{}", self.server_port));
            self.logs
                .push("sender=chase-server-sender".to_string());
        }
    }

    pub fn stop(&mut self) {
        self.wants_to_run = false;
        self.server_socket_open = false;
    }

    pub fn accept_client(&mut self) {
        self.client_socket_count += 1;
    }

    pub fn remove_closed_clients(&mut self, closed: usize) {
        self.client_socket_count = self.client_socket_count.saturating_sub(closed);
    }

    pub fn first_player_position(players: &[ChasePlayerPosition]) -> Option<ChasePlayerPosition> {
        players.first().copied()
    }

    pub fn next_broadcast(
        &self,
        old_position: Option<ChasePlayerPosition>,
        players: &[ChasePlayerPosition],
    ) -> Option<ChasePlayerPosition> {
        if self.client_socket_count == 0 {
            return None;
        }
        let position = Self::first_player_position(players)?;
        (Some(position) != old_position).then_some(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const CHASE_CLIENT_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/chase/ChaseClient.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const CHASE_SERVER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/chase/ChaseServer.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const CHASE_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ChaseCommand.java");

    #[test]
    fn chase_dimension_names_match_command_bimap() {
        assert_eq!(ChaseDimension::from_short_name("o"), Some(ChaseDimension::Overworld));
        assert_eq!(ChaseDimension::from_short_name("n"), Some(ChaseDimension::Nether));
        assert_eq!(ChaseDimension::from_short_name("e"), Some(ChaseDimension::End));
        assert_eq!(ChaseDimension::from_short_name("bad"), None);
        assert_eq!(ChaseDimension::Nether.identifier(), "minecraft:the_nether");
    }

    #[test]
    fn chase_client_parses_teleport_message_into_java_command() {
        let mut client = ChaseClientModel::new("example.test", 10000);
        assert_eq!(client.server_address(), "example.test:10000");
        client.start();
        assert_eq!(client.thread_name.as_deref(), Some("chase-client"));

        let command = client
            .handle_message("t n 1.2349 65.0 -2.5 180.0 22.25")
            .expect("teleport command");
        assert_eq!(
            command,
            "execute in minecraft:the_nether run tp @s 1.235 65.000 -2.500 180.000 22.250"
        );

        assert_eq!(client.handle_message("x ignored"), None);
        assert_eq!(client.logs.last().map(String::as_str), Some("Unknown message type 'x'"));
        client.stop();
        assert!(!client.wants_to_run);
        assert_eq!(client.thread_name, None);
    }

    #[test]
    fn chase_client_rejects_missing_bad_or_unknown_dimension_teleports() {
        let mut client = ChaseClientModel::new("localhost", 10000);
        assert_eq!(client.handle_message("t bad 1 2 3 4 5"), None);
        assert_eq!(client.handle_message("t o 1 2"), None);
        assert_eq!(client.handle_message(""), None);
        assert_eq!(
            client.logs.last().map(String::as_str),
            Some("Could not parse message '', ignoring")
        );
    }

    #[test]
    fn chase_server_formats_first_player_position_and_suppresses_repeats() {
        let mut server = ChaseServerModel::new("0.0.0.0", 10000, 100);
        server.start();
        server.accept_client();

        let first = ChasePlayerPosition {
            dimension: ChaseDimension::Overworld,
            x: 1.234,
            y: 64.0,
            z: -9.876,
            y_rot: 45.125,
            x_rot: -12.5,
        };
        let second = ChasePlayerPosition {
            dimension: ChaseDimension::End,
            x: 999.0,
            y: 80.0,
            z: 999.0,
            y_rot: 0.0,
            x_rot: 0.0,
        };

        let broadcast = server
            .next_broadcast(None, &[first, second])
            .expect("changed first player position");
        assert_eq!(broadcast, first);
        assert_eq!(broadcast.format_server_message(), "t o 1.23 64.00 -9.88 45.12 -12.50\n");
        assert_eq!(server.next_broadcast(Some(first), &[first, second]), None);
        server.remove_closed_clients(1);
        assert_eq!(server.next_broadcast(None, &[first]), None);
        server.stop();
        assert!(!server.wants_to_run);
        assert!(!server.server_socket_open);
    }

    #[test]
    fn chase_server_start_is_idempotent_like_java() {
        let mut server = ChaseServerModel::new("127.0.0.1", 12000, 100);
        server.start();
        server.start();
        assert!(server.server_socket_open);
        assert_eq!(
            server.logs.last().map(String::as_str),
            Some("Remote control server was asked to start, but it is already running. Will ignore.")
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn chase_sources_match_java_26_1_2() {
        for sentinel in [
            "private static final int RECONNECT_INTERVAL_SECONDS = 5;",
            "new Thread(this::run, \"chase-client\")",
            "Thread.sleep(5000L);",
            "scanner.useLocale(Locale.ROOT);",
            "if (\"t\".equals(head))",
            "String.format(\n                  Locale.ROOT,\n                  \"execute in %s run tp @s %.3f %.3f %.3f %.3f %.3f\"",
            "ChaseCommand.DIMENSION_NAMES.get(scanner.next())",
            "LevelBasedPermissionSet.OWNER",
        ] {
            assert!(
                CHASE_CLIENT_JAVA.contains(sentinel),
                "ChaseClient.java is missing sentinel: {sentinel}"
            );
        }
        for sentinel in [
            "private final int broadcastIntervalMs;",
            "new Thread(this::runAcceptor, \"chase-server-acceptor\")",
            "new Thread(this::runSender, \"chase-server-sender\")",
            "CopyOnWriteArrayList<Socket>",
            "if (playerPosition != null && !playerPosition.equals(oldPlayerPosition))",
            "playerPosition.format().getBytes(StandardCharsets.US_ASCII)",
            "this.clientSockets.stream().filter(Socket::isClosed).collect(Collectors.toList())",
            "String dimensionName = (String)ChaseCommand.DIMENSION_NAMES.inverse().get(player.level().dimension());",
            "String.format(Locale.ROOT, \"t %s %.2f %.2f %.2f %.2f %.2f\\n\"",
        ] {
            assert!(
                CHASE_SERVER_JAVA.contains(sentinel),
                "ChaseServer.java is missing sentinel: {sentinel}"
            );
        }
        for sentinel in [
            "private static final String DEFAULT_CONNECT_HOST = \"localhost\";",
            "private static final String DEFAULT_BIND_ADDRESS = \"0.0.0.0\";",
            "private static final int DEFAULT_PORT = 10000;",
            "private static final int BROADCAST_INTERVAL_MS = 100;",
            "ImmutableBiMap.of(\"o\", Level.OVERWORLD, \"n\", Level.NETHER, \"e\", Level.END)",
        ] {
            assert!(
                CHASE_COMMAND_JAVA.contains(sentinel),
                "ChaseCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
