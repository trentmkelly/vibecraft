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

#[derive(Debug, Clone, PartialEq)]
pub struct CustomBossEvent {
    pub event_uuid: String,
    pub custom_id: String,
    pub packed: CustomBossEventPacked,
    live_players: BTreeSet<String>,
    dirty_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomBossEventDisplayName {
    pub bracketed_text: String,
    pub color: BossBarColor,
    pub hover_text: String,
    pub insertion: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomBossEventsRuntime {
    events: BTreeMap<String, CustomBossEvent>,
    dirty_count: usize,
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

impl CustomBossEvent {
    pub fn new(event_uuid: impl Into<String>, custom_id: impl Into<String>, name: Tag) -> Self {
        let mut event = Self {
            event_uuid: event_uuid.into(),
            custom_id: custom_id.into(),
            packed: CustomBossEventPacked::new(name),
            live_players: BTreeSet::new(),
            dirty_count: 0,
        };
        event.set_progress_from_value();
        event
    }

    pub fn load(
        event_uuid: impl Into<String>,
        custom_id: impl Into<String>,
        packed: CustomBossEventPacked,
    ) -> Self {
        let mut event = Self::new(event_uuid, custom_id, packed.name.clone());
        event.set_visible(packed.visible);
        event.set_value(packed.value);
        event.set_max(packed.max);
        event.set_color(packed.color);
        event.set_overlay(packed.overlay);
        event.set_darken_screen(packed.darken_screen);
        event.set_play_boss_music(packed.play_boss_music);
        event.set_create_world_fog(packed.create_world_fog);
        event.packed.players.extend(packed.players);
        event
    }

    pub fn pack(&self) -> CustomBossEventPacked {
        self.packed.clone()
    }

    pub fn value(&self) -> i32 {
        self.packed.value
    }

    pub fn max(&self) -> i32 {
        self.packed.max
    }

    pub fn progress(&self) -> f32 {
        self.packed.progress()
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty_count
    }

    pub fn stored_players(&self) -> &BTreeSet<String> {
        &self.packed.players
    }

    pub fn live_players(&self) -> &BTreeSet<String> {
        &self.live_players
    }

    pub fn add_player(&mut self, uuid: impl Into<String>) {
        let uuid = uuid.into();
        self.live_players.insert(uuid.clone());
        if self.packed.players.insert(uuid) {
            self.set_dirty();
        }
    }

    pub fn remove_player(&mut self, uuid: &str) {
        self.live_players.remove(uuid);
        if self.packed.players.remove(uuid) {
            self.set_dirty();
        }
    }

    pub fn remove_all_players(&mut self) {
        self.live_players.clear();
        if !self.packed.players.is_empty() {
            self.packed.players.clear();
            self.set_dirty();
        }
    }

    pub fn set_value(&mut self, value: i32) {
        self.packed.value = value;
        self.set_progress_from_value();
        self.set_dirty();
    }

    pub fn set_max(&mut self, max: i32) {
        self.packed.max = max;
        self.set_progress_from_value();
        self.set_dirty();
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.packed.visible = visible;
        self.set_dirty();
    }

    pub fn set_color(&mut self, color: BossBarColor) {
        self.packed.color = color;
        self.set_dirty();
    }

    pub fn set_overlay(&mut self, overlay: BossBarOverlay) {
        self.packed.overlay = overlay;
        self.set_dirty();
    }

    pub fn set_darken_screen(&mut self, darken_screen: bool) {
        self.packed.darken_screen = darken_screen;
        self.set_dirty();
    }

    pub fn set_play_boss_music(&mut self, play_boss_music: bool) {
        self.packed.play_boss_music = play_boss_music;
        self.set_dirty();
    }

    pub fn set_create_world_fog(&mut self, create_world_fog: bool) {
        self.packed.create_world_fog = create_world_fog;
        self.set_dirty();
    }

    pub fn display_name(&self) -> CustomBossEventDisplayName {
        let text = match &self.packed.name {
            Tag::String(value) => value.clone(),
            other => format!("{other:?}"),
        };
        CustomBossEventDisplayName {
            bracketed_text: format!("[{text}]"),
            color: self.packed.color,
            hover_text: self.custom_id.clone(),
            insertion: self.custom_id.clone(),
        }
    }

    pub fn set_players(&mut self, players: &[String]) -> bool {
        let desired = players.iter().cloned().collect::<BTreeSet<_>>();
        let to_remove = self
            .packed
            .players
            .difference(&desired)
            .cloned()
            .collect::<Vec<_>>();
        let to_add = desired
            .difference(&self.packed.players)
            .cloned()
            .collect::<Vec<_>>();

        let players_changed = !to_remove.is_empty() || !to_add.is_empty();

        for uuid in &to_remove {
            if self.live_players.contains(uuid) {
                self.remove_player(uuid);
            }
            self.packed.players.remove(uuid);
        }
        for uuid in to_add {
            self.add_player(uuid);
        }
        self.live_players = desired;

        if players_changed {
            self.set_dirty();
        }
        players_changed
    }

    pub fn on_player_connect(&mut self, uuid: impl Into<String>) {
        let uuid = uuid.into();
        if self.packed.players.contains(&uuid) {
            self.add_player(uuid);
        }
    }

    pub fn on_player_disconnect(&mut self, uuid: &str) {
        self.live_players.remove(uuid);
    }

    fn set_progress_from_value(&mut self) {
        let _ = self.progress();
    }

    fn set_dirty(&mut self) {
        self.dirty_count += 1;
    }
}

impl CustomBossEventsRuntime {
    pub fn new() -> Self {
        Self {
            events: BTreeMap::new(),
            dirty_count: 0,
        }
    }

    pub fn from_packed(
        events: impl IntoIterator<Item = (String, CustomBossEventPacked)>,
        mut uuid_factory: impl FnMut() -> String,
    ) -> Self {
        let mut runtime = Self::new();
        for (id, packed) in events {
            let event = CustomBossEvent::load(uuid_factory(), id.clone(), packed);
            runtime.dirty_count += event.dirty_count();
            runtime.events.insert(id, event);
        }
        runtime
    }

    pub fn create(
        &mut self,
        event_uuid: impl Into<String>,
        id: impl Into<String>,
        name: Tag,
    ) -> &mut CustomBossEvent {
        let id = id.into();
        let event = CustomBossEvent::new(event_uuid, id.clone(), name);
        self.events.insert(id.clone(), event);
        self.set_dirty();
        match self.events.get_mut(&id) {
            Some(event) => event,
            None => panic!("inserted event must exist"),
        }
    }

    pub fn remove(&mut self, id: &str) -> Option<CustomBossEvent> {
        let removed = self.events.remove(id);
        if removed.is_some() {
            self.set_dirty();
        }
        removed
    }

    pub fn get(&self, id: &str) -> Option<&CustomBossEvent> {
        self.events.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut CustomBossEvent> {
        self.events.get_mut(id)
    }

    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.events.keys().map(String::as_str)
    }

    pub fn pack(&self) -> BTreeMap<String, CustomBossEventPacked> {
        self.events
            .iter()
            .map(|(id, event)| (id.clone(), event.pack()))
            .collect()
    }

    pub fn on_player_connect(&mut self, uuid: &str) {
        for event in self.events.values_mut() {
            event.on_player_connect(uuid.to_string());
        }
    }

    pub fn on_player_disconnect(&mut self, uuid: &str) {
        for event in self.events.values_mut() {
            event.on_player_disconnect(uuid);
        }
    }

    pub fn dirty_count(&self) -> usize {
        self.dirty_count
    }

    fn set_dirty(&mut self) {
        self.dirty_count += 1;
    }
}

impl Default for CustomBossEventsRuntime {
    fn default() -> Self {
        Self::new()
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

    #[cfg(vibecraft_has_decompiled_sources)]
    const CUSTOM_BOSS_EVENTS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/bossevents/CustomBossEvents.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const CUSTOM_BOSS_EVENT_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/bossevents/CustomBossEvent.java");
    #[cfg(vibecraft_has_decompiled_sources)]
    const BOSS_EVENT_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/BossEvent.java");

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn custom_boss_events_java_saved_data_sentinels_match_source() {
        for sentinel in [
            "private static final Codec<Map<Identifier, CustomBossEvent.Packed>> EVENTS_CODEC = Codec.unboundedMap(Identifier.CODEC, CustomBossEvent.Packed.CODEC);",
            "Identifier.withDefaultNamespace(\"custom_boss_events\")",
            "DataFixTypes.SAVED_DATA_CUSTOM_BOSS_EVENTS",
            "events.forEach((id, packed) -> r.events.put(id, CustomBossEvent.load(UUID.randomUUID(), id, packed, r::setDirty)))",
            "Util.mapValues(c.events, CustomBossEvent::pack)",
            "public @Nullable CustomBossEvent get(final Identifier id)",
            "public CustomBossEvent create(final RandomSource random, final Identifier id, final Component name)",
            "CustomBossEvent result = new CustomBossEvent(Mth.createInsecureUUID(random), id, name, this::setDirty);",
            "public void remove(final CustomBossEvent event)",
            "public Collection<Identifier> getIds()",
            "public Collection<CustomBossEvent> getEvents()",
            "public void onPlayerConnect(final ServerPlayer player)",
            "public void onPlayerDisconnect(final ServerPlayer player)",
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
            "public void addPlayer(final ServerPlayer player)",
            "public void removePlayer(final ServerPlayer player)",
            "public void removeAllPlayers()",
            "public void setValue(final int value)",
            "public void setMax(final int max)",
            "this.setProgress(Mth.clamp((float)value / this.max, 0.0F, 1.0F));",
            "public final Component getDisplayName()",
            "public boolean setPlayers(final Collection<ServerPlayer> players)",
            "public static CustomBossEvent load(final UUID id, final Identifier customId, final CustomBossEvent.Packed packed, final Runnable setDirty)",
            "public CustomBossEvent.Packed pack()",
            "public void onPlayerConnect(final ServerPlayer player)",
            "public void onPlayerDisconnect(final ServerPlayer player)",
            "public void setDirty()",
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

    #[test]
    fn custom_boss_event_runtime_value_max_progress_and_display_name_match_java() {
        let mut event = CustomBossEvent::new(
            "00000000-0000-0000-0000-000000000001",
            "minecraft:raid",
            Tag::String("Raid".to_string()),
        );

        assert_eq!(event.progress(), 0.0);
        event.set_value(250);
        assert_eq!(event.progress(), 1.0);
        event.set_value(-10);
        assert_eq!(event.progress(), 0.0);
        event.set_max(200);
        event.set_value(50);
        assert_eq!(event.progress(), 0.25);
        event.set_color(BossBarColor::Purple);

        assert_eq!(
            event.display_name(),
            CustomBossEventDisplayName {
                bracketed_text: "[Raid]".to_string(),
                color: BossBarColor::Purple,
                hover_text: "minecraft:raid".to_string(),
                insertion: "minecraft:raid".to_string(),
            }
        );
        assert_eq!(event.dirty_count(), 5);
    }

    #[test]
    fn custom_boss_event_player_tracking_matches_stored_vs_live_java_sets() {
        let alex = "00000000-0000-0000-0000-0000000000aa".to_string();
        let steve = "00000000-0000-0000-0000-0000000000bb".to_string();
        let mut event = CustomBossEvent::new(
            "00000000-0000-0000-0000-000000000002",
            "minecraft:event",
            Tag::String("Event".to_string()),
        );

        event.add_player(alex.clone());
        assert!(event.stored_players().contains(&alex));
        assert!(event.live_players().contains(&alex));
        assert_eq!(event.dirty_count(), 1);

        event.on_player_disconnect(&alex);
        assert!(event.stored_players().contains(&alex));
        assert!(!event.live_players().contains(&alex));
        assert_eq!(event.dirty_count(), 1);

        event.on_player_connect(alex.clone());
        assert!(event.live_players().contains(&alex));
        assert_eq!(event.dirty_count(), 1);

        assert!(event.set_players(std::slice::from_ref(&steve)));
        assert!(!event.stored_players().contains(&alex));
        assert!(event.stored_players().contains(&steve));
        assert!(!event.live_players().contains(&alex));
        assert!(event.live_players().contains(&steve));
        assert!(event.dirty_count() >= 3);

        assert!(!event.set_players(std::slice::from_ref(&steve)));
        event.remove_all_players();
        assert!(event.stored_players().is_empty());
        assert!(event.live_players().is_empty());
    }

    #[test]
    fn custom_boss_events_runtime_load_create_remove_and_connect_disconnect_match_java_manager() {
        let player = "00000000-0000-0000-0000-0000000000cc".to_string();
        let mut packed = CustomBossEventPacked::new(Tag::String("Loaded".to_string()));
        packed.visible = true;
        packed.value = 25;
        packed.max = 50;
        packed.players.insert(player.clone());

        let mut next_uuid = 0;
        let mut events = CustomBossEventsRuntime::from_packed(
            [("minecraft:loaded".to_string(), packed.clone())],
            || {
                next_uuid += 1;
                format!("00000000-0000-0000-0000-{next_uuid:012}")
            },
        );
        assert_eq!(events.ids().collect::<Vec<_>>(), vec!["minecraft:loaded"]);
        let loaded = events.get("minecraft:loaded").expect("loaded bossbar");
        assert_eq!(loaded.event_uuid, "00000000-0000-0000-0000-000000000001");
        assert_eq!(loaded.pack().players, packed.players);
        assert!(loaded.live_players().is_empty());
        assert_eq!(events.dirty_count(), 8);

        events.on_player_connect(&player);
        assert!(events
            .get("minecraft:loaded")
            .expect("loaded bossbar")
            .live_players()
            .contains(&player));
        events.on_player_disconnect(&player);
        assert!(events
            .get("minecraft:loaded")
            .expect("loaded bossbar")
            .live_players()
            .is_empty());
        assert!(events
            .get("minecraft:loaded")
            .expect("loaded bossbar")
            .stored_players()
            .contains(&player));

        events.create(
            "00000000-0000-0000-0000-000000000099",
            "minecraft:new",
            Tag::String("New".to_string()),
        );
        assert_eq!(events.dirty_count(), 9);
        assert_eq!(events.pack().len(), 2);
        assert!(events.remove("minecraft:new").is_some());
        assert_eq!(events.dirty_count(), 10);
        assert!(events.remove("minecraft:missing").is_none());
        assert_eq!(events.dirty_count(), 10);
    }
}
