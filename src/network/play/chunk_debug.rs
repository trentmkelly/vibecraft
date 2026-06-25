use super::*;

const DEBUG_ENTITY_NAME_FIRST_PARTS: &[&str] = &[
    "Slim", "Far", "River", "Silly", "Fat", "Thin", "Fish", "Bat", "Dark", "Oak", "Sly", "Bush",
    "Zen", "Bark", "Cry", "Slack", "Soup", "Grim", "Hook", "Dirt", "Mud", "Sad", "Hard",
    "Crook", "Sneak", "Stink", "Weird", "Fire", "Soot", "Soft", "Rough", "Cling", "Scar",
];
const DEBUG_ENTITY_NAME_SECOND_PARTS: &[&str] = &[
    "Fox", "Tail", "Jaw", "Whisper", "Twig", "Root", "Finder", "Nose", "Brow", "Blade", "Fry",
    "Seek", "Wart", "Tooth", "Foot", "Leaf", "Stone", "Fall", "Face", "Tongue", "Voice", "Lip",
    "Mouth", "Snail", "Toe", "Ear", "Hair", "Beard", "Shirt", "Fist",
];

pub fn debug_entity_name_for_uuid(uuid: Uuid) -> String {
    let seed = (java_uuid_hash_code(uuid) >> 2) as i64;
    let mut random = crate::random_source::LegacyRandom::new(seed);
    let first = debug_name_part(&mut random, DEBUG_ENTITY_NAME_FIRST_PARTS);
    let second = debug_name_part(&mut random, DEBUG_ENTITY_NAME_SECOND_PARTS);
    format!("{first}{second}")
}

pub fn debug_entity_name(
    is_player: bool,
    plain_text_name: &str,
    custom_name: Option<&str>,
    uuid: Uuid,
) -> String {
    if is_player {
        return plain_text_name.to_string();
    }
    custom_name
        .map(str::to_string)
        .unwrap_or_else(|| debug_entity_name_for_uuid(uuid))
}

fn debug_name_part<'a>(
    random: &mut crate::random_source::LegacyRandom,
    names: &'a [&'a str],
) -> &'a str {
    let index = random.next_i32_bound(names.len() as i32) as usize;
    names[index]
}

fn java_uuid_hash_code(uuid: Uuid) -> i32 {
    let mut most_significant_bytes = [0; 8];
    most_significant_bytes.copy_from_slice(&uuid.0[..8]);
    let mut least_significant_bytes = [0; 8];
    least_significant_bytes.copy_from_slice(&uuid.0[8..]);
    let most_significant = i64::from_be_bytes(most_significant_bytes);
    let least_significant = i64::from_be_bytes(least_significant_bytes);
    let hilo = most_significant ^ least_significant;
    ((hilo >> 32) as i32) ^ (hilo as i32)
}

impl DebugSubscriptionUpdate {
    const VALUE_SUBSCRIPTION_RANGE: std::ops::Range<i32> = 1..16;

    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_debug_subscription_id(writer, self.subscription_id)?;
        match self.value_payload.as_deref() {
            Some(payload) => {
                write_bool(writer, true)?;
                writer.write_all(payload)
            }
            None => write_bool(writer, false),
        }
    }
}

impl DebugSubscriptionEvent {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_debug_subscription_id(writer, self.subscription_id)?;
        writer.write_all(&self.value_payload)
    }
}

impl ClientboundDebugBlockValuePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_block_position(writer, self.block_pos.x, self.block_pos.y, self.block_pos.z)?;
        self.update.write(writer)
    }
}

impl ClientboundDebugChunkValuePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_i64(writer, pack_chunk_pos_as_long(self.chunk_pos))?;
        self.update.write(writer)
    }
}

impl ClientboundDebugEntityValuePacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write_var_i32(writer, self.entity_id)?;
        self.update.write(writer)
    }
}

impl ClientboundDebugEventPacket {
    pub fn write<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        self.event.write(writer)
    }
}

fn write_debug_subscription_id<W: Write>(writer: &mut W, subscription_id: i32) -> io::Result<()> {
    if !DebugSubscriptionUpdate::VALUE_SUBSCRIPTION_RANGE.contains(&subscription_id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "debug subscription has no value stream codec",
        ));
    }
    write_var_i32(writer, subscription_id)
}
