use super::*;

pub fn chat_type_nbt(chat_type: &ChatTypeEntry) -> Tag {
    Tag::Compound(vec![
        (
            "chat".to_string(),
            chat_decoration_nbt(chat_type.chat_translation_key, chat_type.chat_parameters),
        ),
        (
            "narration".to_string(),
            chat_decoration_nbt(
                chat_type.narration_translation_key,
                chat_type.narration_parameters,
            ),
        ),
    ])
}

pub fn chat_decoration_nbt(translation_key: &str, parameters: &[&str]) -> Tag {
    Tag::Compound(vec![
        (
            "translation_key".to_string(),
            Tag::String(translation_key.to_string()),
        ),
        (
            "parameters".to_string(),
            Tag::List(
                parameters
                    .iter()
                    .map(|parameter| Tag::String((*parameter).to_string()))
                    .collect(),
            ),
        ),
    ])
}

pub fn trim_pattern_nbt(pattern: &str) -> Tag {
    Tag::Compound(vec![
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:{pattern}")),
        ),
        (
            "description".to_string(),
            Tag::Compound(vec![(
                "translate".to_string(),
                Tag::String(format!("trim_pattern.minecraft.{pattern}")),
            )]),
        ),
        ("decal".to_string(), Tag::Byte(0)),
    ])
}

pub fn jukebox_song_nbt(song: &JukeboxSongEntry) -> Tag {
    Tag::Compound(vec![
        (
            "sound_event".to_string(),
            Tag::String(song.sound_event.to_string()),
        ),
        (
            "description".to_string(),
            Tag::Compound(vec![(
                "translate".to_string(),
                Tag::String(format!("jukebox_song.minecraft.{}", song.id)),
            )]),
        ),
        (
            "length_in_seconds".to_string(),
            Tag::Float(song.length_seconds),
        ),
        (
            "comparator_output".to_string(),
            Tag::Int(song.comparator_output),
        ),
    ])
}

pub fn banner_pattern_nbt(pattern: &str) -> Tag {
    Tag::Compound(vec![
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:{pattern}")),
        ),
        (
            "translation_key".to_string(),
            Tag::String(format!("block.minecraft.banner.{pattern}")),
        ),
    ])
}

pub fn instrument_nbt(instrument: &InstrumentEntry) -> Tag {
    Tag::Compound(vec![
        (
            "sound_event".to_string(),
            Tag::String(instrument.sound_event.to_string()),
        ),
        ("use_duration".to_string(), Tag::Float(7.0)),
        ("range".to_string(), Tag::Float(256.0)),
        (
            "description".to_string(),
            Tag::Compound(vec![(
                "translate".to_string(),
                Tag::String(format!("instrument.minecraft.{}", instrument.id)),
            )]),
        ),
    ])
}

pub fn single_texture_variant_nbt(asset_id: &str) -> Tag {
    Tag::Compound(vec![(
        "asset_id".to_string(),
        Tag::String(asset_id.to_string()),
    )])
}

pub fn animal_texture_variant_nbt(kind: &str, texture_name: &str, model: &str) -> Tag {
    let mut fields = vec![
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:entity/{kind}/{texture_name}")),
        ),
        (
            "baby_asset_id".to_string(),
            Tag::String(format!("minecraft:entity/{kind}/{texture_name}_baby")),
        ),
    ];
    if model != "normal" {
        fields.push(("model".to_string(), Tag::String(model.to_string())));
    }
    Tag::Compound(fields)
}

pub fn wolf_variant_nbt(file_name: &str) -> Tag {
    let assets = wolf_assets_nbt(file_name, "");
    let baby_assets = wolf_assets_nbt(file_name, "_baby");
    Tag::Compound(vec![
        ("assets".to_string(), assets),
        ("baby_assets".to_string(), baby_assets),
    ])
}

pub fn wolf_assets_nbt(file_name: &str, suffix: &str) -> Tag {
    Tag::Compound(vec![
        (
            "wild".to_string(),
            Tag::String(format!("minecraft:entity/wolf/{file_name}{suffix}")),
        ),
        (
            "tame".to_string(),
            Tag::String(format!("minecraft:entity/wolf/{file_name}_tame{suffix}")),
        ),
        (
            "angry".to_string(),
            Tag::String(format!("minecraft:entity/wolf/{file_name}_angry{suffix}")),
        ),
    ])
}

pub fn zombie_nautilus_variant_nbt(variant: &str) -> Tag {
    let (asset_id, model) = match variant {
        "warm" => ("minecraft:entity/nautilus/zombie_nautilus_coral", "warm"),
        _ => ("minecraft:entity/nautilus/zombie_nautilus", "normal"),
    };
    Tag::Compound(vec![
        ("asset_id".to_string(), Tag::String(asset_id.to_string())),
        ("model".to_string(), Tag::String(model.to_string())),
    ])
}

pub fn painting_variant_nbt(id: &str) -> Tag {
    Tag::Compound(vec![
        ("width".to_string(), Tag::Int(1)),
        ("height".to_string(), Tag::Int(1)),
        (
            "asset_id".to_string(),
            Tag::String(format!("minecraft:{id}")),
        ),
    ])
}

pub fn cow_sound_variant_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.cow.ambient"),
        sound_field("hurt_sound", "minecraft:entity.cow.hurt"),
        sound_field("death_sound", "minecraft:entity.cow.death"),
        sound_field("step_sound", "minecraft:entity.cow.step"),
    ])
}

pub fn chicken_sound_variant_nbt() -> Tag {
    let sounds = chicken_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

pub fn chicken_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.chicken.ambient"),
        sound_field("hurt_sound", "minecraft:entity.chicken.hurt"),
        sound_field("death_sound", "minecraft:entity.chicken.death"),
        sound_field("step_sound", "minecraft:entity.chicken.step"),
    ])
}

pub fn pig_sound_variant_nbt() -> Tag {
    let sounds = pig_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

pub fn pig_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.pig.ambient"),
        sound_field("hurt_sound", "minecraft:entity.pig.hurt"),
        sound_field("death_sound", "minecraft:entity.pig.death"),
        sound_field("step_sound", "minecraft:entity.pig.step"),
        sound_field("eat_sound", "minecraft:entity.generic.eat"),
    ])
}

pub fn cat_sound_variant_nbt() -> Tag {
    let sounds = cat_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

pub fn cat_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.cat.ambient"),
        sound_field("stray_ambient_sound", "minecraft:entity.cat.stray_ambient"),
        sound_field("hiss_sound", "minecraft:entity.cat.hiss"),
        sound_field("hurt_sound", "minecraft:entity.cat.hurt"),
        sound_field("death_sound", "minecraft:entity.cat.death"),
        sound_field("eat_sound", "minecraft:entity.generic.eat"),
        sound_field("beg_for_food_sound", "minecraft:entity.cat.beg_for_food"),
        sound_field("purr_sound", "minecraft:entity.cat.purr"),
        sound_field("purreow_sound", "minecraft:entity.cat.purreow"),
    ])
}

pub fn wolf_sound_variant_nbt() -> Tag {
    let sounds = wolf_sound_set_nbt();
    Tag::Compound(vec![
        ("adult_sounds".to_string(), sounds.clone()),
        ("baby_sounds".to_string(), sounds),
    ])
}

pub fn wolf_sound_set_nbt() -> Tag {
    Tag::Compound(vec![
        sound_field("ambient_sound", "minecraft:entity.wolf.ambient"),
        sound_field("death_sound", "minecraft:entity.wolf.death"),
        sound_field("growl_sound", "minecraft:entity.wolf.growl"),
        sound_field("hurt_sound", "minecraft:entity.wolf.hurt"),
        sound_field("pant_sound", "minecraft:entity.wolf.pant"),
        sound_field("whine_sound", "minecraft:entity.wolf.whine"),
        sound_field("step_sound", "minecraft:entity.wolf.step"),
    ])
}

pub fn sound_field(name: &str, sound: &str) -> (String, Tag) {
    (name.to_string(), Tag::String(sound.to_string()))
}

pub fn write_network_nbt<W: Write>(writer: &mut W, tag: &Tag) -> io::Result<()> {
    writer.write_all(&[tag.id()])?;
    tag.write_payload(writer)
}

pub fn write_clientbound_login_packet<W: Write>(
    writer: &mut W,
    packet: &ClientboundLoginPacket,
) -> io::Result<()> {
    packet.write(writer)
}

pub fn game_mode_legacy_id(game_mode: GameMode) -> i32 {
    match game_mode {
        GameMode::Survival => 0,
        GameMode::Creative => 1,
        GameMode::Adventure => 2,
        GameMode::Spectator => 3,
    }
}

pub fn game_mode_from_legacy_id(id: i32) -> GameMode {
    match id {
        1 => GameMode::Creative,
        2 => GameMode::Adventure,
        3 => GameMode::Spectator,
        _ => GameMode::Survival,
    }
}

pub fn game_mode_from_name(name: &str) -> GameMode {
    if let Ok(id) = name.parse::<i32>() {
        return game_mode_from_legacy_id(id);
    }
    match name {
        "creative" => GameMode::Creative,
        "adventure" => GameMode::Adventure,
        "spectator" => GameMode::Spectator,
        _ => GameMode::Survival,
    }
}

pub fn write_vec3<W: Write>(writer: &mut W, x: f64, y: f64, z: f64) -> io::Result<()> {
    writer.write_all(&x.to_be_bytes())?;
    writer.write_all(&y.to_be_bytes())?;
    writer.write_all(&z.to_be_bytes())
}

pub fn read_f64<R: Read>(reader: &mut R) -> io::Result<f64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes)?;
    Ok(f64::from_be_bytes(bytes))
}

pub fn read_f32<R: Read>(reader: &mut R) -> io::Result<f32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(f32::from_be_bytes(bytes))
}

pub fn read_i16<R: Read>(reader: &mut R) -> io::Result<i16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(i16::from_be_bytes(bytes))
}

pub fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

pub fn read_bool<R: Read>(reader: &mut R) -> io::Result<bool> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0] != 0)
}

pub fn write_bool<W: Write>(writer: &mut W, value: bool) -> io::Result<()> {
    writer.write_all(&[u8::from(value)])
}

/// Writes a ClientboundGameEventPacket with the given type and float parameter.
/// Java: ClientboundGameEventPacket — byte event type, float param
pub fn write_game_event(
    stream: &mut TcpStream,
    compression: CompressionState,
    event_type: u8,
    param: f32,
) -> io::Result<()> {
    write_game_event_to_writer(stream, compression, event_type, param)
}

pub fn write_game_event_to_writer<W: Write>(
    writer: &mut W,
    compression: CompressionState,
    event_type: u8,
    param: f32,
) -> io::Result<()> {
    write_framed_packet_with_compression(
        writer,
        compression,
        CLIENTBOUND_GAME_EVENT_PACKET_ID,
        |payload| {
            payload.write_all(&[event_type])?;
            payload.write_all(&param.to_be_bytes())
        },
    )
}

pub fn write_framed_packet<W, F>(writer: &mut W, packet_id: i32, write_body: F) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    write_var_i32(&mut payload, packet_id)?;
    write_body(&mut payload)?;
    write_packet(writer, &payload)
}

/// Returns the number of bytes needed to encode `value` as a Minecraft VarInt.
///
/// Packet IDs are always non-negative, so the sign extension applied to
/// negative values by [`write_var_i32`] is not relevant here.
pub fn var_int_encoded_len(value: i32) -> usize {
    let uval = value as u32;
    match uval {
        0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1F_FFFF => 3,
        0x20_0000..=0xFFF_FFFF => 4,
        _ => 5,
    }
}

pub fn write_framed_packet_with_compression<W, F>(
    writer: &mut W,
    compression: CompressionState,
    packet_id: i32,
    write_body: F,
) -> io::Result<()>
where
    W: Write,
    F: FnOnce(&mut Vec<u8>) -> io::Result<()>,
{
    let mut payload = Vec::new();
    write_var_i32(&mut payload, packet_id)?;
    write_body(&mut payload)?;
    // Emit a TRACE-level packet dump when tracing is active.  The guard avoids
    // the format overhead when tracing is off.
    if crate::log::global_level() >= crate::log::LogLevel::Trace {
        let id_len = var_int_encoded_len(packet_id);
        crate::log::log_packet_send(packet_id, &payload[id_len..]);
    }
    let frame = compression.encode_packet(&payload)?;
    writer.write_all(&frame)
}

pub fn write_status_response_packet<W: Write>(writer: &mut W, json: &str) -> io::Result<()> {
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 0)?;
    write_string(&mut payload, json)?;
    write_packet(writer, &payload)
}

pub fn write_status_pong_packet<W: Write>(
    writer: &mut W,
    request: ServerboundPingRequestPacket,
) -> io::Result<()> {
    let response = ClientboundPongResponsePacket::from_request(request);
    let mut payload = Vec::new();
    write_var_i32(&mut payload, 1)?;
    response.write(&mut payload)?;
    write_packet(writer, &payload)
}

pub fn handle_legacy_status_tcp_connection(
    stream: &mut TcpStream,
    properties: &ServerProperties,
) -> io::Result<()> {
    stream.set_nonblocking(true)?;
    let mut request = Vec::new();
    let mut buffer = [0u8; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => request.extend_from_slice(&buffer[..count]),
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => break,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => {
                let _ = stream.set_nonblocking(false);
                return Err(err);
            }
        }
    }
    stream.set_nonblocking(false)?;
    let response = legacy_status_response(&request, properties)?;
    stream.write_all(&response)
}

#[cfg(test)]
pub fn handle_legacy_status_connection<W: Read + Write>(
    stream: &mut W,
    properties: &ServerProperties,
) -> io::Result<()> {
    let mut request = Vec::new();
    stream.read_to_end(&mut request)?;
    let response = legacy_status_response(&request, properties)?;
    stream.write_all(&response)
}

pub fn legacy_status_response(
    request: &[u8],
    properties: &ServerProperties,
) -> io::Result<Vec<u8>> {
    if request.first() != Some(&0xFE) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected legacy query packet",
        ));
    }

    let body = match &request[1..] {
        [] => legacy_version0_response(properties),
        [0x01] => legacy_version1_response(properties),
        [0x01, tail @ ..] if read_legacy_ping_host_payload(tail).is_some() => {
            legacy_version1_response(properties)
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "malformed legacy query packet",
            ))
        }
    };

    legacy_disconnect_packet(&body)
}

pub fn read_legacy_ping_host_payload(input: &[u8]) -> Option<()> {
    let mut input = Cursor::new(input);
    let mut packet_id = [0u8; 1];
    input.read_exact(&mut packet_id).ok()?;
    if packet_id[0] != 250 {
        return None;
    }

    let channel = read_legacy_string(&mut input).ok()?;
    if channel != "MC|PingHost" {
        return None;
    }

    let mut size = [0u8; 2];
    input.read_exact(&mut size).ok()?;
    let payload_size = u16::from_be_bytes(size) as u64;
    if input.get_ref().len() as u64 - input.position() != payload_size {
        return None;
    }

    let mut protocol = [0u8; 1];
    input.read_exact(&mut protocol).ok()?;
    if protocol[0] < 73 {
        return None;
    }

    let _host = read_legacy_string(&mut input).ok()?;
    let mut port = [0u8; 4];
    input.read_exact(&mut port).ok()?;
    (u32::from_be_bytes(port) <= u16::MAX as u32).then_some(())
}

pub fn legacy_version0_response(properties: &ServerProperties) -> String {
    format!("{}§{}§{}", properties.motd, 0, properties.max_players)
}

pub fn legacy_version1_response(properties: &ServerProperties) -> String {
    format!(
        "§1\0{}\0{}\0{}\0{}\0{}",
        127, VERSION_NAME, properties.motd, 0, properties.max_players
    )
}

pub fn legacy_disconnect_packet(reason: &str) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(3 + reason.len() * 2);
    out.push(255);
    write_legacy_string(&mut out, reason)?;
    Ok(out)
}

pub fn read_legacy_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut length = [0u8; 2];
    reader.read_exact(&mut length)?;
    let char_count = u16::from_be_bytes(length) as usize;
    let mut bytes = vec![0u8; char_count * 2];
    reader.read_exact(&mut bytes)?;
    String::from_utf16(
        &bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>(),
    )
    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

pub fn write_legacy_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    let utf16 = value.encode_utf16().collect::<Vec<_>>();
    let len = u16::try_from(utf16.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "legacy string too long"))?;
    writer.write_all(&len.to_be_bytes())?;
    for code_unit in utf16 {
        writer.write_all(&code_unit.to_be_bytes())?;
    }
    Ok(())
}

pub fn read_packet<R: Read>(reader: &mut R) -> io::Result<Vec<u8>> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > MAX_PACKET_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid packet length",
        ));
    }

    let mut payload = vec![0u8; length as usize];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

pub fn read_packet_with_compression<R: Read>(
    reader: &mut R,
    compression: CompressionState,
) -> io::Result<Vec<u8>> {
    match compression.threshold() {
        None => read_packet(reader),
        Some(_) => compression.decode_packet(reader),
    }
}

pub fn write_packet<W: Write>(writer: &mut W, payload: &[u8]) -> io::Result<()> {
    write_var_i32(writer, payload.len() as i32)?;
    writer.write_all(payload)
}

pub fn read_string<R: Read>(reader: &mut R, max_chars: usize) -> io::Result<String> {
    let length = read_var_i32(reader)?;
    if length < 0 || length as usize > max_chars * 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid string length",
        ));
    }

    let mut bytes = vec![0u8; length as usize];
    reader.read_exact(&mut bytes)?;
    let string =
        String::from_utf8(bytes).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if string.chars().count() > max_chars {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "string too long",
        ));
    }
    Ok(string)
}

pub fn write_string<W: Write>(writer: &mut W, value: &str) -> io::Result<()> {
    write_var_i32(writer, value.len() as i32)?;
    writer.write_all(value.as_bytes())
}

pub fn load_favicon(path: &Path) -> io::Result<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let bytes = fs::read(path)?;
    let (width, height) = png_dimensions(&bytes)?;
    if width != 64 || height != 64 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("server-icon.png must be 64x64, got {width}x{height}"),
        ));
    }
    Ok(Some(format!(
        "data:image/png;base64,{}",
        encode_base64(&bytes)
    )))
}

pub fn png_dimensions(bytes: &[u8]) -> io::Result<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "server-icon.png must be a PNG with an IHDR header",
        ));
    }

    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Ok((width, height))
}

pub fn status_json(properties: &ServerProperties, favicon: Option<&str>) -> String {
    let players = format!(
        "\"players\":{{\"max\":{},\"online\":0,\"sample\":[]}}",
        properties.max_players
    );

    let favicon = favicon
        .map(|value| format!(",\"favicon\":\"{}\"", escape_json_string(value)))
        .unwrap_or_default();

    // Java mirror: status `enforcesSecureChat` = `server.enforceSecureProfile()`
    // (MinecraftServer.java:1059) = `enforce-secure-profile && online-mode &&
    // services.canValidateProfileKeys()` (DedicatedServer.java:654-656). RustCraft
    // has not loaded a Mojang services PROFILE_KEY (no online profile-key
    // validation), so `canValidateProfileKeys()` is false and the whole expression
    // is false — matching vanilla offline behavior. (Previously hard-coded `true`,
    // which diverged from vanilla offline mode.)
    // TODO(secure-profile-enforcement): once online-mode auth (AUTH_CHAT) loads the
    // services key set, compute this via `player_profile_key::enforce_secure_profile`
    // (un-gate that module from cfg(test)) and add login-time signed-profile-key
    // rejection + unsigned-chat dropping. Until then `enforce-secure-profile` only
    // affects this advertisement, which is correctly false offline.
    format!(
        "{{\"version\":{{\"name\":\"{}\",\"protocol\":{}}},{},\"description\":{{\"text\":\"{}\"}}{},\"enforcesSecureChat\":false}}",
        VERSION_NAME,
        PROTOCOL_VERSION,
        players,
        escape_json_string(&properties.motd),
        favicon
    )
}

pub fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0b0000_0011) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 0b0000_1111) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 0b0011_1111) as usize] as char);
        } else {
            out.push('=');
        }
    }

    out
}

pub fn escape_json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}
