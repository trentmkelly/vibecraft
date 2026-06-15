use std::collections::{BTreeMap, BTreeSet};

use crate::storage::nbt::snbt_operations::uuid_string_to_int_array;
use crate::storage::nbt::Tag;

const DEFAULT_VISIBLE: bool = false;
const DEFAULT_VALUE: i32 = 0;
const DEFAULT_MAX: i32 = 100;
const DEFAULT_COLOR: BossBarColor = BossBarColor::White;
const DEFAULT_OVERLAY: BossBarOverlay = BossBarOverlay::Progress;

#[derive(Debug, Clone, PartialEq)]
pub struct CustomBossEvents {
    events: BTreeMap<String, CustomBossEventPacked>,
    dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomBossEventPacked {
    pub name: Tag,
    pub visible: bool,
    pub value: i32,
    pub max: i32,
    pub color: BossBarColor,
    pub overlay: BossBarOverlay,
    pub darken_screen: bool,
    pub play_boss_music: bool,
    pub create_world_fog: bool,
    pub players: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossBarOverlay {
    Progress,
    Notched6,
    Notched10,
    Notched12,
    Notched20,
}

impl Default for CustomBossEvents {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomBossEvents {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            dirty: false,
        }
    }

    pub fn from_tag(tag: &Tag) -> Option<Self> {
        let Tag::Compound(fields) = tag else {
            return None;
        };
        let mut events = BTreeMap::new();
        for (id, event_tag) in fields {
            events.insert(id.clone(), CustomBossEventPacked::from_tag(event_tag)?);
        }
        Some(Self {
            events,
            dirty: false,
        })
    }

    pub fn to_tag(&self) -> Tag {
        Tag::Compound(
            self.events
                .iter()
                .map(|(id, event)| (id.clone(), event.to_tag()))
                .collect(),
        )
    }

    pub fn create(&mut self, id: impl Into<String>, name: Tag) -> &mut CustomBossEventPacked {
        let event = CustomBossEventPacked::new(name);
        self.dirty = true;
        self.events.entry(id.into()).or_insert(event)
    }

    pub fn remove(&mut self, id: &str) -> Option<CustomBossEventPacked> {
        let removed = self.events.remove(id);
        if removed.is_some() {
            self.dirty = true;
        }
        removed
    }

    pub fn get(&self, id: &str) -> Option<&CustomBossEventPacked> {
        self.events.get(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.events.keys().map(String::as_str)
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl CustomBossEventPacked {
    pub fn new(name: Tag) -> Self {
        Self {
            name,
            visible: DEFAULT_VISIBLE,
            value: DEFAULT_VALUE,
            max: DEFAULT_MAX,
            color: DEFAULT_COLOR,
            overlay: DEFAULT_OVERLAY,
            darken_screen: false,
            play_boss_music: false,
            create_world_fog: false,
            players: BTreeSet::new(),
        }
    }

    pub fn progress(&self) -> f32 {
        let value = (self.value as f32) / (self.max as f32);
        if value < 0.0 {
            0.0
        } else {
            value.min(1.0)
        }
    }

    pub fn from_tag(tag: &Tag) -> Option<Self> {
        let Tag::Compound(fields) = tag else {
            return None;
        };
        let name = compound_get(fields, "Name")?.clone();
        Some(Self {
            name,
            visible: compound_bool(fields, "Visible").unwrap_or(DEFAULT_VISIBLE),
            value: compound_i32(fields, "Value").unwrap_or(DEFAULT_VALUE),
            max: compound_i32(fields, "Max").unwrap_or(DEFAULT_MAX),
            color: compound_string(fields, "Color")
                .and_then(BossBarColor::from_serialized_name)
                .unwrap_or(DEFAULT_COLOR),
            overlay: compound_string(fields, "Overlay")
                .and_then(BossBarOverlay::from_serialized_name)
                .unwrap_or(DEFAULT_OVERLAY),
            darken_screen: compound_bool(fields, "DarkenScreen").unwrap_or(false),
            play_boss_music: compound_bool(fields, "PlayBossMusic").unwrap_or(false),
            create_world_fog: compound_bool(fields, "CreateWorldFog").unwrap_or(false),
            players: compound_uuid_set(fields, "Players").unwrap_or_default(),
        })
    }

    pub fn to_tag(&self) -> Tag {
        let mut fields = vec![
            ("Name".to_string(), self.name.clone()),
            ("Visible".to_string(), Tag::Byte(i8::from(self.visible))),
            ("Value".to_string(), Tag::Int(self.value)),
            ("Max".to_string(), Tag::Int(self.max)),
            (
                "Color".to_string(),
                Tag::String(self.color.serialized_name().to_string()),
            ),
            (
                "Overlay".to_string(),
                Tag::String(self.overlay.serialized_name().to_string()),
            ),
            (
                "DarkenScreen".to_string(),
                Tag::Byte(i8::from(self.darken_screen)),
            ),
            (
                "PlayBossMusic".to_string(),
                Tag::Byte(i8::from(self.play_boss_music)),
            ),
            (
                "CreateWorldFog".to_string(),
                Tag::Byte(i8::from(self.create_world_fog)),
            ),
        ];
        if !self.players.is_empty() {
            fields.push((
                "Players".to_string(),
                Tag::List(
                    self.players
                        .iter()
                        .filter_map(|uuid| uuid_string_to_int_array(uuid).map(Tag::IntArray))
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }
}

impl BossBarColor {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Pink => "pink",
            Self::Blue => "blue",
            Self::Red => "red",
            Self::Green => "green",
            Self::Yellow => "yellow",
            Self::Purple => "purple",
            Self::White => "white",
        }
    }

    pub fn from_serialized_name(name: &str) -> Option<Self> {
        match name {
            "pink" => Some(Self::Pink),
            "blue" => Some(Self::Blue),
            "red" => Some(Self::Red),
            "green" => Some(Self::Green),
            "yellow" => Some(Self::Yellow),
            "purple" => Some(Self::Purple),
            "white" => Some(Self::White),
            _ => None,
        }
    }
}

impl BossBarOverlay {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::Progress => "progress",
            Self::Notched6 => "notched_6",
            Self::Notched10 => "notched_10",
            Self::Notched12 => "notched_12",
            Self::Notched20 => "notched_20",
        }
    }

    pub fn from_serialized_name(name: &str) -> Option<Self> {
        match name {
            "progress" => Some(Self::Progress),
            "notched_6" => Some(Self::Notched6),
            "notched_10" => Some(Self::Notched10),
            "notched_12" => Some(Self::Notched12),
            "notched_20" => Some(Self::Notched20),
            _ => None,
        }
    }
}

fn compound_get<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    fields
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then_some(value))
}

fn compound_bool(fields: &[(String, Tag)], name: &str) -> Option<bool> {
    match compound_get(fields, name)? {
        Tag::Byte(value) => Some(*value != 0),
        _ => None,
    }
}

fn compound_i32(fields: &[(String, Tag)], name: &str) -> Option<i32> {
    match compound_get(fields, name)? {
        Tag::Int(value) => Some(*value),
        _ => None,
    }
}

fn compound_string<'a>(fields: &'a [(String, Tag)], name: &str) -> Option<&'a str> {
    match compound_get(fields, name)? {
        Tag::String(value) => Some(value),
        _ => None,
    }
}

fn compound_uuid_set(fields: &[(String, Tag)], name: &str) -> Option<BTreeSet<String>> {
    let Tag::List(values) = compound_get(fields, name)? else {
        return None;
    };
    values
        .iter()
        .map(|tag| match tag {
            Tag::IntArray(values) if values.len() == 4 => Some(uuid_int_array_to_string(values)),
            _ => None,
        })
        .collect()
}

fn uuid_int_array_to_string(values: &[i32]) -> String {
    let msb = ((values[0] as u64) << 32) | u64::from(values[1] as u32);
    let lsb = ((values[2] as u64) << 32) | u64::from(values[3] as u32);
    let raw = format!("{msb:016x}{lsb:016x}");
    format!(
        "{}-{}-{}-{}-{}",
        &raw[0..8],
        &raw[8..12],
        &raw[12..16],
        &raw[16..20],
        &raw[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSTOM_BOSS_EVENTS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/bossevents/CustomBossEvents.java");
    const CUSTOM_BOSS_EVENT_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/bossevents/CustomBossEvent.java");
    const BOSS_EVENT_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/world/BossEvent.java");

    #[test]
    fn custom_boss_events_java_saved_data_sentinels_match_source() {
        for sentinel in [
            "private static final Codec<Map<Identifier, CustomBossEvent.Packed>> EVENTS_CODEC = Codec.unboundedMap(Identifier.CODEC, CustomBossEvent.Packed.CODEC);",
            "Identifier.withDefaultNamespace(\"custom_boss_events\")",
            "DataFixTypes.SAVED_DATA_CUSTOM_BOSS_EVENTS",
            "events.forEach((id, packed) -> r.events.put(id, CustomBossEvent.load(UUID.randomUUID(), id, packed, r::setDirty)))",
            "Util.mapValues(c.events, CustomBossEvent::pack)",
        ] {
            assert!(
                CUSTOM_BOSS_EVENTS_JAVA.contains(sentinel),
                "missing CustomBossEvents sentinel {sentinel}"
            );
        }
        for sentinel in [
            "private static final int DEFAULT_MAX = 100;",
            "ComponentSerialization.CODEC.fieldOf(\"Name\")",
            "Codec.BOOL.optionalFieldOf(\"Visible\", false)",
            "Codec.INT.optionalFieldOf(\"Value\", 0)",
            "Codec.INT.optionalFieldOf(\"Max\", 100)",
            "BossEvent.BossBarColor.CODEC.optionalFieldOf(\"Color\", BossEvent.BossBarColor.WHITE)",
            "BossEvent.BossBarOverlay.CODEC.optionalFieldOf(\"Overlay\", BossEvent.BossBarOverlay.PROGRESS)",
            "UUIDUtil.CODEC_SET.optionalFieldOf(\"Players\", Set.of())",
            "this.setProgress(Mth.clamp((float)value / this.max, 0.0F, 1.0F));",
        ] {
            assert!(
                CUSTOM_BOSS_EVENT_JAVA.contains(sentinel),
                "missing CustomBossEvent sentinel {sentinel}"
            );
        }
        for sentinel in [
            "PINK(\"pink\", ChatFormatting.RED)",
            "WHITE(\"white\", ChatFormatting.WHITE)",
            "PROGRESS(\"progress\")",
            "NOTCHED_20(\"notched_20\")",
        ] {
            assert!(
                BOSS_EVENT_JAVA.contains(sentinel),
                "missing BossEvent sentinel {sentinel}"
            );
        }
    }

    #[test]
    fn custom_boss_event_packed_defaults_and_progress_match_java() {
        let tag = Tag::Compound(vec![(
            "minecraft:raid".to_string(),
            Tag::Compound(vec![(
                "Name".to_string(),
                Tag::String("{\"text\":\"Raid\"}".to_string()),
            )]),
        )]);
        let Some(events) = CustomBossEvents::from_tag(&tag) else {
            panic!("custom boss events should decode");
        };
        let Some(event) = events.get("minecraft:raid") else {
            panic!("custom boss event should be present");
        };
        assert_eq!(event.name, Tag::String("{\"text\":\"Raid\"}".to_string()));
        assert!(!event.visible);
        assert_eq!(event.value, 0);
        assert_eq!(event.max, 100);
        assert_eq!(event.color, BossBarColor::White);
        assert_eq!(event.overlay, BossBarOverlay::Progress);
        assert_eq!(event.progress(), 0.0);
        assert!(!events.is_dirty());
    }

    #[test]
    fn custom_boss_events_round_trip_java_codec_shape() {
        let uuid = "00112233-4455-6677-8899-aabbccddeeff";
        let uuid_ints = vec![0x0011_2233, 0x4455_6677, -0x7766_5545, -0x3322_1101];
        let tag = Tag::Compound(vec![(
            "minecraft:event".to_string(),
            Tag::Compound(vec![
                (
                    "Name".to_string(),
                    Tag::Compound(vec![("text".to_string(), Tag::String("Event".to_string()))]),
                ),
                ("Visible".to_string(), Tag::Byte(1)),
                ("Value".to_string(), Tag::Int(75)),
                ("Max".to_string(), Tag::Int(150)),
                ("Color".to_string(), Tag::String("purple".to_string())),
                ("Overlay".to_string(), Tag::String("notched_10".to_string())),
                ("DarkenScreen".to_string(), Tag::Byte(1)),
                ("PlayBossMusic".to_string(), Tag::Byte(1)),
                ("CreateWorldFog".to_string(), Tag::Byte(0)),
                ("Players".to_string(), Tag::List(vec![Tag::IntArray(uuid_ints)])),
            ]),
        )]);

        let Some(events) = CustomBossEvents::from_tag(&tag) else {
            panic!("custom boss events should decode");
        };
        assert_eq!(events.ids().collect::<Vec<_>>(), vec!["minecraft:event"]);
        let Some(event) = events.get("minecraft:event") else {
            panic!("custom boss event should be present");
        };
        assert!(event.visible);
        assert_eq!(event.value, 75);
        assert_eq!(event.max, 150);
        assert_eq!(event.color, BossBarColor::Purple);
        assert_eq!(event.overlay, BossBarOverlay::Notched10);
        assert!(event.darken_screen);
        assert!(event.play_boss_music);
        assert!(!event.create_world_fog);
        assert!(event.players.contains(uuid));
        assert_eq!(event.progress(), 0.5);
        assert_eq!(CustomBossEvents::from_tag(&events.to_tag()), Some(events));
    }

    #[test]
    fn custom_boss_events_create_remove_and_uuid_codec_match_java() {
        let mut events = CustomBossEvents::new();
        events.create(
            "minecraft:test",
            Tag::Compound(vec![("text".to_string(), Tag::String("Test".to_string()))]),
        );
        assert!(events.is_dirty());
        let Some(event) = events.get("minecraft:test") else {
            panic!("custom boss event should be present");
        };
        assert_eq!(event.color.serialized_name(), "white");
        assert_eq!(event.overlay.serialized_name(), "progress");

        let mut event = CustomBossEventPacked::new(Tag::String("Name".to_string()));
        event
            .players
            .insert("00112233-4455-6677-8899-aabbccddeeff".to_string());
        assert!(matches!(
            event.to_tag(),
            Tag::Compound(fields) if fields.iter().any(|(name, tag)| {
                name == "Players"
                    && matches!(
                        tag,
                        Tag::List(values)
                            if values == &vec![Tag::IntArray(vec![
                                0x0011_2233,
                                0x4455_6677,
                                -0x7766_5545,
                                -0x3322_1101,
                            ])]
                    )
            })
        ));

        assert!(events.remove("minecraft:test").is_some());
        assert!(events.get("minecraft:test").is_none());
    }
}
