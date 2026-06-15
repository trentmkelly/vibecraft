use super::*;

pub(super) fn entity_ref(id: &str) -> EntityRef {
    EntityRef {
        id: id.to_string(),
        display_name: id.to_string(),
    }
}

pub(super) fn is_number(input: &str) -> bool {
    input.parse::<f64>().is_ok()
}

pub(super) fn parse_name_list(input: &str) -> Vec<NameAndId> {
    input
        .split(',')
        .filter(|name| !name.is_empty())
        .map(NameAndId::create_offline)
        .collect()
}

pub(super) fn parse_entity_list(input: &str) -> Vec<EntityRef> {
    input
        .split(',')
        .filter(|id| !id.is_empty())
        .map(entity_ref)
        .collect()
}

pub(super) fn parse_identifier(input: &str) -> Result<String, CommandError> {
    if input.is_empty()
        || input.bytes().any(|byte| {
            !byte.is_ascii_alphanumeric() && !matches!(byte, b'_' | b'-' | b'.' | b'/' | b':')
        })
    {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(input.to_string())
    }
}

pub(super) fn parse_uuid_string(input: &str) -> Result<String, CommandError> {
    let bytes = input.as_bytes();
    if bytes.len() != 36 || [8, 13, 18, 23].iter().any(|index| bytes[*index] != b'-') {
        return Err(CommandError::InvalidSyntax);
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| ![8, 13, 18, 23].contains(&index) && !byte.is_ascii_hexdigit())
    {
        return Err(CommandError::InvalidSyntax);
    }
    Ok(input.to_ascii_lowercase())
}

pub(super) fn name_uuid_from_bytes(input: &[u8]) -> String {
    let mut bytes = md5_digest(input);
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format_uuid_bytes(bytes)
}

pub(super) fn format_uuid_bytes(bytes: [u8; 16]) -> String {
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

pub(super) fn md5_digest(input: &[u8]) -> [u8; 16] {
    const S: [u32; 64] = [
        7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5,
        9, 14, 20, 5, 9, 14, 20, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10,
        15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21,
    ];
    const K: [u32; 64] = [
        0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613,
        0xfd469501, 0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193,
        0xa679438e, 0x49b40821, 0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d,
        0x02441453, 0xd8a1e681, 0xe7d3fbc8, 0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed,
        0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a, 0xfffa3942, 0x8771f681, 0x6d9d6122,
        0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70, 0x289b7ec6, 0xeaa127fa,
        0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665, 0xf4292244,
        0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
        0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb,
        0xeb86d391,
    ];

    let mut message = input.to_vec();
    let bit_len = (message.len() as u64) * 8;
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_le_bytes());

    let mut a0 = 0x67452301u32;
    let mut b0 = 0xefcdab89u32;
    let mut c0 = 0x98badcfeu32;
    let mut d0 = 0x10325476u32;

    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 16];
        for (index, word) in words.iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_le_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }

        let mut a = a0;
        let mut b = b0;
        let mut c = c0;
        let mut d = d0;

        for i in 0..64 {
            let (f, g) = match i {
                0..=15 => ((b & c) | ((!b) & d), i),
                16..=31 => ((d & b) | ((!d) & c), (5 * i + 1) % 16),
                32..=47 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let next = b.wrapping_add(
                a.wrapping_add(f)
                    .wrapping_add(K[i])
                    .wrapping_add(words[g])
                    .rotate_left(S[i]),
            );
            a = d;
            d = c;
            c = b;
            b = next;
        }

        a0 = a0.wrapping_add(a);
        b0 = b0.wrapping_add(b);
        c0 = c0.wrapping_add(c);
        d0 = d0.wrapping_add(d);
    }

    let mut digest = [0u8; 16];
    digest[0..4].copy_from_slice(&a0.to_le_bytes());
    digest[4..8].copy_from_slice(&b0.to_le_bytes());
    digest[8..12].copy_from_slice(&c0.to_le_bytes());
    digest[12..16].copy_from_slice(&d0.to_le_bytes());
    digest
}

pub(super) fn parse_sound_source(input: &str) -> Result<SoundSource, CommandError> {
    match input {
        "master" => Ok(SoundSource::Master),
        "music" => Ok(SoundSource::Music),
        "record" => Ok(SoundSource::Record),
        "weather" => Ok(SoundSource::Weather),
        "block" => Ok(SoundSource::Block),
        "hostile" => Ok(SoundSource::Hostile),
        "neutral" => Ok(SoundSource::Neutral),
        "player" => Ok(SoundSource::Player),
        "ambient" => Ok(SoundSource::Ambient),
        "voice" => Ok(SoundSource::Voice),
        "ui" => Ok(SoundSource::Ui),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn parse_f64(input: &str) -> Result<f64, CommandError> {
    input
        .parse::<f64>()
        .map_err(|_| CommandError::InvalidSyntax)
}

pub(super) fn parse_non_negative_i32(input: &str) -> Result<i32, CommandError> {
    let value = parse_i32(input)?;
    if value >= 0 {
        Ok(value)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_f32(input: &str) -> Result<f32, CommandError> {
    input
        .parse::<f32>()
        .map_err(|_| CommandError::InvalidSyntax)
}

pub(super) fn parse_block_pos(x: &str, y: &str, z: &str) -> Result<BlockPos, CommandError> {
    Ok(BlockPos {
        x: x.parse::<i32>().map_err(|_| CommandError::InvalidSyntax)?,
        y: y.parse::<i32>().map_err(|_| CommandError::InvalidSyntax)?,
        z: z.parse::<i32>().map_err(|_| CommandError::InvalidSyntax)?,
    })
}

pub(super) fn block_pos_containing(position: Vec3) -> BlockPos {
    BlockPos {
        x: position.x.floor() as i32,
        y: position.y.floor() as i32,
        z: position.z.floor() as i32,
    }
}

pub(super) fn wrap_degrees(value: f32) -> f32 {
    let wrapped = (value % 360.0 + 540.0) % 360.0 - 180.0;
    if wrapped == -180.0 {
        180.0
    } else {
        wrapped
    }
}

pub(super) fn parse_non_negative_f32(input: &str) -> Result<f32, CommandError> {
    let value = input
        .parse::<f32>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if value < 0.0 {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(value)
    }
}

pub(super) fn parse_positive_f32(input: &str) -> Result<f32, CommandError> {
    let value = parse_non_negative_f32(input)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub(super) fn parse_bounded_f32(input: &str, min: f32, max: f32) -> Result<f32, CommandError> {
    let value = parse_non_negative_f32(input)?;
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(CommandError::InvalidSyntax)
    }
}

pub fn visible_command_usages(permissions: LevelBasedPermissionSet) -> Vec<&'static str> {
    known_command_usages()
        .iter()
        .copied()
        .filter(|(command, _usage)| permissions.can_run(command) == CommandAvailability::Available)
        .map(|(_command, usage)| usage)
        .collect()
}

pub fn command_usage(command: &str, permissions: LevelBasedPermissionSet) -> Option<&'static str> {
    known_command_usages()
        .iter()
        .find(|(root, _usage)| *root == command)
        .and_then(|(root, usage)| {
            (permissions.can_run(root) == CommandAvailability::Available).then_some(*usage)
        })
}

const KNOWN_COMMAND_USAGES: &[(&str, &str)] = &[
        (
            "advancement",
            "/advancement <grant|revoke> <targets> <everything|only|from|until|through>",
        ),
        (
            "attribute",
            "/attribute <target> <attribute> get|base|get|set|reset|modifier",
        ),
        ("ban", "/ban <targets> [reason]"),
        ("ban-ip", "/ban-ip <target> [reason]"),
        ("banlist", "/banlist [ips|players]"),
        // VibeCraft-only debug helper; vanilla 26.1.2 has no `/biome` command.
        ("biome", "/biome"),
        ("bossbar", "/bossbar <add|remove|list|set|get> ..."),
        ("chase", "/chase <follow|lead|stop> [host|bind_address] [port]"),
        ("clear", "/clear [targets] [item] [maxCount]"),
        (
            "clone",
            "/clone [from <sourceDimension>] <begin> <end> [to <targetDimension>] [strict] <destination> [replace|masked|filtered <filter>] [force|move|normal]",
        ),
        (
            "damage",
            "/damage <target> <amount> [damageType] [at <location>|by <entity> [from <cause>]]",
        ),
        (
            "datapack",
            "/datapack <list|enable|disable|create>",
        ),
        ("debug", "/debug <start|stop|function>"),
        ("debugconfig", "/debugconfig <config|unconfig|dialog>"),
        ("debugmobspawning", "/debugmobspawning <category> <pos>"),
        ("debugpath", "/debugpath <to>"),
        ("defaultgamemode", "/defaultgamemode <gamemode>"),
        ("difficulty", "/difficulty [difficulty]"),
        ("dialog", "/dialog <show|clear> <targets> [dialog]"),
        ("effect", "/effect <give|clear> ..."),
        ("enchant", "/enchant <targets> <enchantment> [level]"),
        ("execute", "/execute ... run <command>"),
        ("experience", "/experience <add|set|query> ..."),
        ("fetchprofile", "/fetchprofile <name|id|entity> <target>"),
        ("fill", "/fill <from> <to> <block> [mode]"),
        ("fillbiome", "/fillbiome <from> <to> <biome> [replace <filter>]"),
        ("forceload", "/forceload <add|remove|query> ..."),
        ("function", "/function <name|#tag> [arguments]"),
        ("gamemode", "/gamemode <gamemode> [target]"),
        ("gamerule", "/gamerule <rule> [value]"),
        ("give", "/give <targets> <item> [count]"),
        ("item", "/item <replace|modify> <block|entity> ..."),
        ("locate", "/locate <structure|biome|poi> <target>"),
        ("loot", "/loot <give|insert|replace|spawn> ... <fish|loot|kill|mine> ..."),
        ("place", "/place <feature|jigsaw|structure|template> ..."),
        ("raid", "/raid <start|stop|check|sound|spawnleader|setomen|glow>"),
        ("help", "/help [command]"),
        ("jfr", "/jfr <start|stop>"),
        ("kick", "/kick <targets> [reason]"),
        ("kill", "/kill [targets]"),
        ("list", "/list [uuids]"),
        ("msg", "/msg <targets> <message>"),
        ("op", "/op <targets>"),
        (
            "playsound",
            "/playsound <sound> [source] [targets] [pos] [volume] [pitch] [minVolume]",
        ),
        (
            "particle",
            "/particle <name> [pos] [delta] [speed] [count] [force|normal] [viewers]",
        ),
        ("perf", "/perf <start|stop>"),
        ("pardon", "/pardon <targets>"),
        ("pardon-ip", "/pardon-ip <target>"),
        ("publish", "/publish [allowCommands] [gamemode] [port]"),
        ("random", "/random value|roll|reset ..."),
        ("recipe", "/recipe <give|take> <targets> <recipe|*>"),
        ("reload", "/reload"),
        ("return", "/return <value>|fail|run <command>"),
        (
            "rotate",
            "/rotate <target> <rotation>|facing <entity|location>",
        ),
        ("ride", "/ride <target> mount <vehicle>|dismount"),
        ("save-all", "/save-all [flush]"),
        ("save-off", "/save-off"),
        ("save-on", "/save-on"),
        ("say", "/say <message>"),
        (
            "schedule",
            "/schedule function <function|#tag> <time> [append|replace]|clear <id>",
        ),
        (
            "scoreboard",
            "/scoreboard objectives|players ...",
        ),
        ("seed", "/seed"),
        (
            "serverpack",
            "/serverpack push <url> [uuid] [hash]|pop <uuid>",
        ),
        ("setidletimeout", "/setidletimeout <minutes>"),
        (
            "setblock",
            "/setblock <pos> <block> [destroy|keep|replace|strict]",
        ),
        ("setworldspawn", "/setworldspawn [pos] [rotation]"),
        ("spectate", "/spectate [target] [player]"),
        (
            "spawn_armor_trims",
            "/spawn_armor_trims <pattern|*_lag_my_game>",
        ),
        ("spawnpoint", "/spawnpoint [targets] [pos] [rotation]"),
        (
            "spreadplayers",
            "/spreadplayers <center> <spreadDistance> <maxRange> [under <maxHeight>] <respectTeams> <targets>",
        ),
        ("stop", "/stop"),
        ("stopsound", "/stopsound <targets> [source|*] [sound]"),
        (
            "stopwatch",
            "/stopwatch <create|query|restart|remove> <id> [scale]",
        ),
        ("summon", "/summon <entity> [pos] [nbt]"),
        ("swing", "/swing [targets] [mainhand|offhand]"),
        ("tag", "/tag <targets> <add|remove|list> [name]"),
        ("teleport", "/teleport <targets|location> ..."),
        (
            "team",
            "/team <list|add|remove|empty|join|leave|modify> ...",
        ),
        ("teammsg", "/teammsg <message>"),
        ("tell", "/tell <targets> <message>"),
        ("tellraw", "/tellraw <targets> <message>"),
        ("tick", "/tick query|rate|step|sprint|freeze|unfreeze"),
        ("time", "/time <set|add|query|pause|resume|rate> ..."),
        ("title", "/title <targets> <clear|reset|title|subtitle|actionbar|times> ..."),
        ("trigger", "/trigger <objective> [add|set] [value]"),
        ("tm", "/tm <message>"),
        ("tp", "/tp <targets|location> ..."),
        ("transfer", "/transfer <hostname> [port] [players]"),
        ("version", "/version"),
        (
            "warden_spawn_tracker",
            "/warden_spawn_tracker <clear|set> [warning_level]",
        ),
        ("waypoint", "/waypoint <list|modify> ..."),
        ("weather", "/weather <clear|rain|thunder> [duration]"),
        ("whitelist", "/whitelist <on|off|list|add|remove|reload>"),
        ("worldborder", "/worldborder <add|set|center|damage|get|warning> ..."),
        ("deop", "/deop <targets>"),
];

pub(super) fn known_command_usages() -> &'static [(&'static str, &'static str)] {
    KNOWN_COMMAND_USAGES
}

pub(super) fn parse_bool(input: &str) -> Result<bool, CommandError> {
    match input {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn parse_gamemode(input: &str) -> Result<GameMode, CommandError> {
    match input {
        "survival" => Ok(GameMode::Survival),
        "creative" => Ok(GameMode::Creative),
        "adventure" => Ok(GameMode::Adventure),
        "spectator" => Ok(GameMode::Spectator),
        _ => Err(CommandError::InvalidSyntax),
    }
}

pub(super) fn parse_difficulty(input: &str) -> Result<Difficulty, CommandError> {
    match input {
        "peaceful" => Ok(Difficulty::Peaceful),
        "easy" => Ok(Difficulty::Easy),
        "normal" => Ok(Difficulty::Normal),
        "hard" => Ok(Difficulty::Hard),
        _ => Err(CommandError::InvalidSyntax),
    }
}

impl Difficulty {
    pub(super) fn id(self) -> i32 {
        match self {
            Self::Peaceful => 0,
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 3,
        }
    }
}

pub(super) fn parse_publish_port(input: &str) -> Result<u16, CommandError> {
    input
        .parse::<u16>()
        .map_err(|_| CommandError::InvalidSyntax)
}

pub(super) fn require_gamemaster(permissions: LevelBasedPermissionSet) -> Result<(), CommandError> {
    if permissions.has_permission(Permission::CommandLevel(PermissionLevel::Gamemasters)) {
        Ok(())
    } else {
        Err(CommandError::PermissionDenied)
    }
}

pub(super) fn random_sample(
    state: &mut ServerCommandState,
    range: &str,
    sequence: Option<&str>,
    announced: bool,
) -> Result<CommandResult, CommandError> {
    let (min, max) = parse_int_range(range)?;
    let span = i64::from(max) - i64::from(min);
    if span == 0 {
        return Err(CommandError::RandomRangeTooSmall);
    }
    if span >= i64::from(i32::MAX) {
        return Err(CommandError::RandomRangeTooLarge);
    }

    let seed = match sequence {
        Some(id) => random_sequence_seed(state, id),
        None => state.world_seed as u64,
    };
    let next = lcg_next(seed);
    if let Some(id) = sequence {
        set_random_sequence_seed(state, id, next);
    } else {
        state.world_seed = next as i64;
    }

    let value = min + (next % (span as u64 + 1)) as i32;
    state.random_broadcasts.push(RandomSample {
        value,
        min,
        max,
        sequence: sequence.map(str::to_string),
        announced,
    });
    Ok(CommandResult {
        success_count: value,
        feedback_key: if announced {
            "commands.random.roll"
        } else {
            "commands.random.sample.success"
        },
        broadcast_to_admins: false,
    })
}

pub(super) fn random_reset_all(
    state: &mut ServerCommandState,
    seed: &str,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<CommandResult, CommandError> {
    state.random_seed_defaults = RandomSeedDefaults {
        salt: parse_i32(seed)?,
        include_world_seed,
        include_sequence_id,
    };
    let count = state.random_sequences.len() as i32;
    state.random_sequences.clear();
    Ok(CommandResult {
        success_count: count,
        feedback_key: "commands.random.reset.all.success",
        broadcast_to_admins: false,
    })
}

pub(super) fn reset_random_sequence(
    state: &mut ServerCommandState,
    sequence: &str,
    salt: Option<i32>,
    include_world_seed: bool,
    include_sequence_id: bool,
) -> Result<CommandResult, CommandError> {
    if sequence.is_empty() {
        return Err(CommandError::InvalidSyntax);
    }
    let salt = salt.unwrap_or(0);
    let mut seed = salt as u64;
    if include_world_seed {
        seed ^= state.world_seed as u64;
    }
    if include_sequence_id {
        seed ^= stable_hash(sequence);
    }
    set_random_sequence_seed(state, sequence, seed);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: "commands.random.reset.success",
        broadcast_to_admins: false,
    })
}

pub(super) fn random_sequence_seed(state: &mut ServerCommandState, sequence: &str) -> u64 {
    if let Some(existing) = state
        .random_sequences
        .iter()
        .find(|existing| existing.id == sequence)
    {
        existing.seed
    } else {
        let mut seed = state.random_seed_defaults.salt as u64;
        if state.random_seed_defaults.include_world_seed {
            seed ^= state.world_seed as u64;
        }
        if state.random_seed_defaults.include_sequence_id {
            seed ^= stable_hash(sequence);
        }
        state.random_sequences.push(RandomSequenceState {
            id: sequence.to_string(),
            seed,
        });
        seed
    }
}

pub(super) fn set_random_sequence_seed(state: &mut ServerCommandState, sequence: &str, seed: u64) {
    if let Some(existing) = state
        .random_sequences
        .iter_mut()
        .find(|existing| existing.id == sequence)
    {
        existing.seed = seed;
    } else {
        state.random_sequences.push(RandomSequenceState {
            id: sequence.to_string(),
            seed,
        });
    }
}

pub(super) fn lcg_next(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

pub(super) fn stable_hash(input: &str) -> u64 {
    input.bytes().fold(0xcbf29ce484222325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
    })
}

pub(super) fn parse_int_range(input: &str) -> Result<(i32, i32), CommandError> {
    if let Some((min, max)) = input.split_once("..") {
        let min = if min.is_empty() {
            i32::MIN
        } else {
            parse_i32(min)?
        };
        let max = if max.is_empty() {
            i32::MAX
        } else {
            parse_i32(max)?
        };
        if min > max {
            return Err(CommandError::InvalidSyntax);
        }
        Ok((min, max))
    } else {
        let value = parse_i32(input)?;
        Ok((value, value))
    }
}

pub(super) fn parse_i32(input: &str) -> Result<i32, CommandError> {
    input
        .parse::<i32>()
        .map_err(|_| CommandError::InvalidSyntax)
}

pub(super) fn queue_transfer(
    state: &mut ServerCommandState,
    host: &str,
    port: u16,
    targets: Vec<NameAndId>,
) -> Result<CommandResult, CommandError> {
    if targets.is_empty() {
        return Err(CommandError::NoPlayers);
    }
    let success_count = targets.len() as i32;
    state.transfer_requests.push(TransferRequest {
        host: host.to_string(),
        port,
        targets,
    });
    Ok(CommandResult {
        success_count,
        feedback_key: if success_count == 1 {
            "commands.transfer.success.single"
        } else {
            "commands.transfer.success.multiple"
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn parse_port(input: &str) -> Result<u16, CommandError> {
    let port = input
        .parse::<u16>()
        .map_err(|_| CommandError::InvalidSyntax)?;
    if port == 0 {
        Err(CommandError::InvalidSyntax)
    } else {
        Ok(port)
    }
}

pub(super) fn set_weather(
    state: &mut ServerCommandState,
    mode: WeatherMode,
    duration_ticks: Option<u32>,
) -> Result<CommandResult, CommandError> {
    state.weather = WeatherState {
        mode,
        duration_ticks,
    };
    Ok(CommandResult {
        success_count: duration_ticks.map(|ticks| ticks as i32).unwrap_or(-1),
        feedback_key: match mode {
            WeatherMode::Clear => "commands.weather.set.clear",
            WeatherMode::Rain => "commands.weather.set.rain",
            WeatherMode::Thunder => "commands.weather.set.thunder",
        },
        broadcast_to_admins: true,
    })
}

pub(super) fn tick_query_status_key(state: &ServerCommandState) -> &'static str {
    if state.tick_rate.is_sprinting() {
        "commands.tick.status.sprinting"
    } else if state.tick_rate.is_frozen() {
        "commands.tick.status.frozen"
    } else if state.tick_rate.tick_duration().as_nanos() < state.average_tick_time_nanos as u128 {
        "commands.tick.status.lagging"
    } else {
        "commands.tick.status.running"
    }
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            mode: WeatherMode::Clear,
            duration_ticks: None,
        }
    }
}

impl Default for RandomSeedDefaults {
    fn default() -> Self {
        Self {
            salt: 0,
            include_world_seed: true,
            include_sequence_id: true,
        }
    }
}

pub(super) fn tick_step(
    state: &mut ServerCommandState,
    ticks: u32,
) -> Result<CommandResult, CommandError> {
    let success = state.tick_rate.step_game_if_paused(ticks);
    Ok(CommandResult {
        success_count: 1,
        feedback_key: if success {
            "commands.tick.step.success"
        } else {
            "commands.tick.step.fail"
        },
        broadcast_to_admins: success,
    })
}

pub(super) fn parse_time_ticks(input: &str) -> Result<u32, CommandError> {
    let (number, multiplier) = if let Some(number) = input.strip_suffix('t') {
        (number, 1)
    } else if let Some(number) = input.strip_suffix('s') {
        (number, 20)
    } else if let Some(number) = input.strip_suffix('d') {
        (number, 24_000)
    } else {
        (input, 1)
    };
    let ticks = number
        .parse::<u32>()
        .ok()
        .and_then(|number| number.checked_mul(multiplier))
        .filter(|ticks| *ticks >= 1)
        .ok_or(CommandError::InvalidSyntax)?;
    Ok(ticks)
}

pub(super) fn parse_time_ticks_allow_zero(input: &str) -> Result<u32, CommandError> {
    let (number, multiplier) = match input.as_bytes().last().copied() {
        Some(b't') => (&input[..input.len() - 1], 1),
        Some(b's') => (&input[..input.len() - 1], 20),
        Some(b'd') => (&input[..input.len() - 1], 24_000),
        _ => (input, 1),
    };
    let ticks = number
        .parse::<u32>()
        .ok()
        .and_then(|value| value.checked_mul(multiplier))
        .ok_or(CommandError::InvalidSyntax)?;
    Ok(ticks)
}
