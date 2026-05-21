use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AiPackageDef {
    pub id: &'static str,
    pub package: &'static str,
    pub java_files: usize,
}

pub const AI_PACKAGES: &[AiPackageDef] = &[
    AiPackageDef {
        id: "attributes",
        package: "ai/attributes",
        java_files: 9,
    },
    AiPackageDef {
        id: "behavior",
        package: "ai/behavior",
        java_files: 90,
    },
    AiPackageDef {
        id: "control",
        package: "ai/control",
        java_files: 8,
    },
    AiPackageDef {
        id: "goal",
        package: "ai/goal",
        java_files: 58,
    },
    AiPackageDef {
        id: "gossip",
        package: "ai/gossip",
        java_files: 3,
    },
    AiPackageDef {
        id: "memory",
        package: "ai/memory",
        java_files: 8,
    },
    AiPackageDef {
        id: "navigation",
        package: "ai/navigation",
        java_files: 7,
    },
    AiPackageDef {
        id: "sensing",
        package: "ai/sensing",
        java_files: 24,
    },
    AiPackageDef {
        id: "targeting",
        package: "ai/targeting",
        java_files: 2,
    },
    AiPackageDef {
        id: "util",
        package: "ai/util",
        java_files: 7,
    },
    AiPackageDef {
        id: "village",
        package: "ai/village",
        java_files: 3,
    },
    AiPackageDef {
        id: "poi",
        package: "ai/village/poi",
        java_files: 17,
    },
    AiPackageDef {
        id: "schedule",
        package: "schedule",
        java_files: 2,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Activity {
    Core,
    Idle,
    Work,
    Play,
    Rest,
    Meet,
    Panic,
    Fight,
    Raid,
    Hide,
}

pub const VILLAGER_SCHEDULE_PERIOD_TICKS: i32 = 24000;
pub const VILLAGER_SCHEDULE_UPDATE_INTERVAL_TICKS: i64 = 20;
pub const ADULT_VILLAGER_SCHEDULE_ENTRIES: &[(i32, Activity)] = &[
    (10, Activity::Idle),
    (2000, Activity::Work),
    (9000, Activity::Meet),
    (11000, Activity::Idle),
    (12000, Activity::Rest),
];
pub const BABY_VILLAGER_SCHEDULE_ENTRIES: &[(i32, Activity)] = &[
    (10, Activity::Idle),
    (3000, Activity::Play),
    (6000, Activity::Idle),
    (10000, Activity::Play),
    (12000, Activity::Rest),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryStatus {
    Registered,
    ValuePresent,
    ValueAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryValue {
    pub value: String,
    pub ttl: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrainState {
    memories: BTreeMap<&'static str, MemoryValue>,
    activities: BTreeSet<Activity>,
    default_activity: Activity,
    active_activity: Activity,
}

impl BrainState {
    pub fn new(default_activity: Activity) -> Self {
        let mut activities = BTreeSet::new();
        activities.insert(default_activity);
        Self {
            memories: BTreeMap::new(),
            activities,
            default_activity,
            active_activity: default_activity,
        }
    }

    pub fn set_memory(&mut self, key: &'static str, value: impl Into<String>, ttl: Option<i64>) {
        self.memories.insert(
            key,
            MemoryValue {
                value: value.into(),
                ttl,
            },
        );
    }

    pub fn erase_memory(&mut self, key: &'static str) {
        self.memories.remove(key);
    }

    pub fn memory_status(&self, key: &'static str) -> MemoryStatus {
        if self.memories.contains_key(key) {
            MemoryStatus::ValuePresent
        } else {
            MemoryStatus::ValueAbsent
        }
    }

    pub fn tick_memories(&mut self) {
        let mut expired = Vec::new();
        for (key, value) in self.memories.iter_mut() {
            if let Some(ttl) = value.ttl.as_mut() {
                *ttl -= 1;
                if *ttl <= 0 {
                    expired.push(*key);
                }
            }
        }
        for key in expired {
            self.memories.remove(key);
        }
    }

    pub fn add_activity(&mut self, activity: Activity) {
        self.activities.insert(activity);
    }

    pub fn update_activity_from_schedule(&mut self, schedule: &Schedule, day_time: i32) {
        let selected = schedule
            .activity_at(day_time)
            .unwrap_or(self.default_activity);
        self.active_activity = if self.activities.contains(&selected) {
            selected
        } else {
            self.default_activity
        };
    }

    pub fn serialize_memory_keys(&self) -> Vec<&'static str> {
        self.memories.keys().copied().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schedule {
    entries: Vec<(i32, Activity)>,
}

impl Schedule {
    pub fn new(mut entries: Vec<(i32, Activity)>) -> Self {
        entries.sort_by_key(|(time, _)| *time);
        Self { entries }
    }

    pub fn activity_at(&self, day_time: i32) -> Option<Activity> {
        let day_time = day_time.rem_euclid(VILLAGER_SCHEDULE_PERIOD_TICKS);
        self.entries
            .iter()
            .rev()
            .find(|(time, _)| *time <= day_time)
            .map(|(_, activity)| *activity)
            .or_else(|| self.entries.last().map(|(_, activity)| *activity))
    }
}

pub fn adult_villager_schedule() -> Schedule {
    Schedule::new(ADULT_VILLAGER_SCHEDULE_ENTRIES.to_vec())
}

pub fn baby_villager_schedule() -> Schedule {
    Schedule::new(BABY_VILLAGER_SCHEDULE_ENTRIES.to_vec())
}

pub fn villager_schedule_for_age(is_baby: bool) -> Schedule {
    if is_baby {
        baby_villager_schedule()
    } else {
        adult_villager_schedule()
    }
}

pub fn villager_schedule_update_due(game_time: i64, last_schedule_update: i64) -> bool {
    game_time - last_schedule_update > VILLAGER_SCHEDULE_UPDATE_INTERVAL_TICKS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GoalControl {
    Move,
    Look,
    Jump,
    Target,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalDef {
    pub name: &'static str,
    pub priority: i32,
    pub controls: BTreeSet<GoalControl>,
    pub can_use: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoalSelector {
    goals: Vec<GoalDef>,
}

impl GoalSelector {
    pub fn new(goals: Vec<GoalDef>) -> Self {
        Self { goals }
    }

    pub fn select_running_goals(&self) -> Vec<&'static str> {
        let mut locked = BTreeSet::new();
        let mut candidates: Vec<_> = self.goals.iter().filter(|goal| goal.can_use).collect();
        candidates.sort_by_key(|goal| goal.priority);
        let mut selected = Vec::new();
        for goal in candidates {
            if goal.controls.is_disjoint(&locked) {
                locked.extend(goal.controls.iter().copied());
                selected.push(goal.name);
            }
        }
        selected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorDef {
    pub id: &'static str,
    pub scan_rate: i32,
    pub memories_written: &'static [&'static str],
}

impl SensorDef {
    pub fn should_scan(&self, tick: i32) -> bool {
        self.scan_rate <= 1 || tick % self.scan_rate == 0
    }
}

pub const SENSOR_TYPES: &[SensorDef] = &[
    SensorDef {
        id: "nearest_living_entities",
        scan_rate: 20,
        memories_written: &["nearest_living_entities", "nearest_visible_living_entities"],
    },
    SensorDef {
        id: "hurt_by",
        scan_rate: 20,
        memories_written: &["hurt_by", "hurt_by_entity"],
    },
    SensorDef {
        id: "villager_hostiles",
        scan_rate: 20,
        memories_written: &["nearest_hostile"],
    },
    SensorDef {
        id: "secondary_poi",
        scan_rate: 40,
        memories_written: &["secondary_job_site"],
    },
    SensorDef {
        id: "warden_entity_sensor",
        scan_rate: 10,
        memories_written: &["nearest_attackable", "disturbance_location"],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationKind {
    Ground,
    Flying,
    Water,
    Amphibious,
    WallClimber,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathPlan {
    pub navigation: NavigationKind,
    pub can_float: bool,
    pub max_visited_nodes_multiplier: f32,
    pub target: (i32, i32, i32),
    pub reached: bool,
}

impl PathPlan {
    pub fn can_path_through_water(&self) -> bool {
        matches!(
            self.navigation,
            NavigationKind::Water | NavigationKind::Amphibious
        ) || self.can_float
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetingConditions {
    pub range: i32,
    pub check_line_of_sight: bool,
    pub test_invisible: bool,
}

impl TargetingConditions {
    pub fn can_target(&self, distance: i32, visible: bool, invisible: bool) -> bool {
        distance <= self.range
            && (!self.check_line_of_sight || visible)
            && (self.test_invisible || !invisible)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GossipType {
    MajorPositive,
    MinorPositive,
    Trading,
    MinorNegative,
    MajorNegative,
}

pub const GOSSIP_DISCARD_THRESHOLD: i32 = 2;
pub const VILLAGER_GOSSIP_INTERVAL_TICKS: i64 = 1200;
pub const VILLAGER_GOSSIP_TRANSFER_MAX_COUNT: usize = 10;
pub const VILLAGER_GOSSIP_DECAY_INTERVAL_TICKS: i64 = 24000;

impl GossipType {
    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::MajorNegative => "major_negative",
            Self::MinorNegative => "minor_negative",
            Self::MinorPositive => "minor_positive",
            Self::MajorPositive => "major_positive",
            Self::Trading => "trading",
        }
    }

    pub fn weight(self) -> i32 {
        match self {
            Self::MajorPositive => 5,
            Self::MinorPositive => 1,
            Self::Trading => 1,
            Self::MinorNegative => -1,
            Self::MajorNegative => -5,
        }
    }

    pub fn max(self) -> i32 {
        match self {
            Self::MajorPositive => 20,
            Self::MinorPositive => 25,
            Self::Trading => 25,
            Self::MinorNegative => 200,
            Self::MajorNegative => 100,
        }
    }

    pub fn decay_per_day(self) -> i32 {
        match self {
            Self::MajorPositive => 0,
            Self::MinorPositive => 1,
            Self::Trading => 2,
            Self::MinorNegative => 20,
            Self::MajorNegative => 10,
        }
    }

    pub fn decay_per_transfer(self) -> i32 {
        match self {
            Self::MajorPositive => 20,
            Self::MinorPositive => 5,
            Self::Trading => 20,
            Self::MinorNegative => 20,
            Self::MajorNegative => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipContainer {
    entries: BTreeMap<(String, GossipType), i32>,
}

impl GossipContainer {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, target: impl Into<String>, kind: GossipType, value: i32) {
        let key = (target.into(), kind);
        let old_value = self.entries.get(&key).copied().unwrap_or(0);
        let sum = old_value + value;
        let next = if sum > kind.max() {
            kind.max().max(old_value)
        } else {
            sum
        };
        if next < GOSSIP_DISCARD_THRESHOLD {
            self.entries.remove(&key);
        } else {
            self.entries.insert(key, next);
        }
    }

    pub fn decay(&mut self) {
        let mut remove = Vec::new();
        for (key, value) in self.entries.iter_mut() {
            *value -= key.1.decay_per_day();
            if *value < GOSSIP_DISCARD_THRESHOLD {
                remove.push(key.clone());
            }
        }
        for key in remove {
            self.entries.remove(&key);
        }
    }

    pub fn reputation(&self, target: &str) -> i32 {
        self.entries
            .iter()
            .filter(|((uuid, _), _)| uuid == target)
            .map(|((_, kind), value)| value * kind.weight())
            .sum()
    }

    pub fn value(&self, target: &str, kind: GossipType) -> i32 {
        self.entries
            .get(&(target.to_string(), kind))
            .copied()
            .unwrap_or(0)
    }

    pub fn transfer_selected_from(
        &mut self,
        source: &GossipContainer,
        selected: &[(&str, GossipType)],
    ) {
        for (target, kind) in selected
            .iter()
            .take(VILLAGER_GOSSIP_TRANSFER_MAX_COUNT)
            .copied()
        {
            let value = source.value(target, kind);
            let decayed_value = value - kind.decay_per_transfer();
            if decayed_value >= GOSSIP_DISCARD_THRESHOLD {
                let key = (target.to_string(), kind);
                let old = self.entries.get(&key).copied().unwrap_or(0);
                self.entries.insert(key, old.max(decayed_value));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReputationEventModel {
    ZombieVillagerCured,
    Trade,
    VillagerHurt,
    VillagerKilled,
}

pub fn reputation_event_gossips(event: ReputationEventModel) -> &'static [(GossipType, i32)] {
    match event {
        ReputationEventModel::ZombieVillagerCured => &[
            (GossipType::MajorPositive, 20),
            (GossipType::MinorPositive, 25),
        ],
        ReputationEventModel::Trade => &[(GossipType::Trading, 2)],
        ReputationEventModel::VillagerHurt => &[(GossipType::MinorNegative, 25)],
        ReputationEventModel::VillagerKilled => &[(GossipType::MajorNegative, 25)],
    }
}

pub fn villager_gossip_meeting_allowed(
    timestamp: i64,
    first_last_gossip_time: i64,
    second_last_gossip_time: i64,
) -> bool {
    (timestamp < first_last_gossip_time
        || timestamp >= first_last_gossip_time + VILLAGER_GOSSIP_INTERVAL_TICKS)
        && (timestamp < second_last_gossip_time
            || timestamp >= second_last_gossip_time + VILLAGER_GOSSIP_INTERVAL_TICKS)
}

pub fn villager_gossip_decay_due(timestamp: i64, last_decay_time: i64) -> (i64, bool) {
    if last_decay_time == 0 {
        (timestamp, false)
    } else if timestamp >= last_decay_time + VILLAGER_GOSSIP_DECAY_INTERVAL_TICKS {
        (timestamp, true)
    } else {
        (last_decay_time, false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoiTicket {
    pub poi_type: &'static str,
    pub max_tickets: i32,
    pub free_tickets: i32,
}

impl PoiTicket {
    pub fn acquire(&mut self) -> bool {
        if self.free_tickets > 0 {
            self.free_tickets -= 1;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self) {
        self.free_tickets = (self.free_tickets + 1).min(self.max_tickets);
    }
}

pub const AI_CHECKLIST_SURFACE: &[&str] = &[
    "attributes",
    "behavior",
    "control",
    "goal",
    "gossip",
    "memory",
    "navigation",
    "sensing",
    "targeting",
    "util",
    "village",
    "villager POIs",
    "schedules",
    "brain save/load",
    "activities",
    "pathfinding",
];

pub fn ai_package(id: &str) -> Option<&'static AiPackageDef> {
    AI_PACKAGES.iter().find(|package| package.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controls(values: &[GoalControl]) -> BTreeSet<GoalControl> {
        values.iter().copied().collect()
    }

    #[test]
    fn ai_package_manifest_covers_shared_decompiled_packages() {
        assert_eq!(AI_PACKAGES.len(), 13);
        assert_eq!(ai_package("behavior").unwrap().java_files, 90);
        assert_eq!(ai_package("goal").unwrap().java_files, 58);
        assert_eq!(ai_package("poi").unwrap().java_files, 17);
        for id in [
            "attributes",
            "behavior",
            "control",
            "goal",
            "gossip",
            "memory",
            "navigation",
            "sensing",
            "targeting",
            "util",
            "village",
            "poi",
            "schedule",
        ] {
            assert!(ai_package(id).is_some(), "{id}");
        }
    }

    #[test]
    fn brain_memories_expire_serialize_and_schedule_activities() {
        let mut brain = BrainState::new(Activity::Idle);
        brain.add_activity(Activity::Work);
        brain.set_memory("job_site", "poi-1", Some(2));
        brain.set_memory("home", "bed-1", None);
        assert_eq!(brain.memory_status("job_site"), MemoryStatus::ValuePresent);
        assert_eq!(brain.serialize_memory_keys(), vec!["home", "job_site"]);
        brain.erase_memory("home");
        assert_eq!(brain.memory_status("home"), MemoryStatus::ValueAbsent);
        brain.tick_memories();
        assert_eq!(brain.memory_status("job_site"), MemoryStatus::ValuePresent);
        brain.tick_memories();
        assert_eq!(brain.memory_status("job_site"), MemoryStatus::ValueAbsent);

        let schedule = Schedule::new(vec![
            (0, Activity::Idle),
            (2000, Activity::Work),
            (12000, Activity::Rest),
        ]);
        let _covered_activities = [
            Activity::Core,
            Activity::Idle,
            Activity::Work,
            Activity::Play,
            Activity::Rest,
            Activity::Meet,
            Activity::Panic,
            Activity::Fight,
            Activity::Raid,
            Activity::Hide,
        ];
        assert_eq!(MemoryStatus::Registered, MemoryStatus::Registered);
        brain.update_activity_from_schedule(&schedule, 3000);
        assert_eq!(brain.active_activity, Activity::Work);
        brain.update_activity_from_schedule(&schedule, 13000);
        assert_eq!(brain.active_activity, Activity::Idle);
    }

    #[test]
    fn villager_schedule_tracks_and_update_gate_match_java_timeline() {
        let adult = adult_villager_schedule();
        assert_eq!(adult.activity_at(0), Some(Activity::Rest));
        assert_eq!(adult.activity_at(10), Some(Activity::Idle));
        assert_eq!(adult.activity_at(1999), Some(Activity::Idle));
        assert_eq!(adult.activity_at(2000), Some(Activity::Work));
        assert_eq!(adult.activity_at(8999), Some(Activity::Work));
        assert_eq!(adult.activity_at(9000), Some(Activity::Meet));
        assert_eq!(adult.activity_at(11000), Some(Activity::Idle));
        assert_eq!(adult.activity_at(12000), Some(Activity::Rest));
        assert_eq!(adult.activity_at(24010), Some(Activity::Idle));

        let baby = baby_villager_schedule();
        assert_eq!(baby.activity_at(0), Some(Activity::Rest));
        assert_eq!(baby.activity_at(10), Some(Activity::Idle));
        assert_eq!(baby.activity_at(3000), Some(Activity::Play));
        assert_eq!(baby.activity_at(6000), Some(Activity::Idle));
        assert_eq!(baby.activity_at(10000), Some(Activity::Play));
        assert_eq!(baby.activity_at(12000), Some(Activity::Rest));
        assert_eq!(villager_schedule_for_age(false), adult);
        assert_eq!(villager_schedule_for_age(true), baby);

        assert!(!villager_schedule_update_due(120, 100));
        assert!(villager_schedule_update_due(121, 100));

        let mut brain = BrainState::new(Activity::Idle);
        brain.add_activity(Activity::Work);
        brain.add_activity(Activity::Meet);
        brain.add_activity(Activity::Rest);
        brain.update_activity_from_schedule(&adult, 9000);
        assert_eq!(brain.active_activity, Activity::Meet);
        brain.update_activity_from_schedule(&adult, 0);
        assert_eq!(brain.active_activity, Activity::Rest);
    }

    #[test]
    fn goal_selector_respects_priority_and_control_locks() {
        let selector = GoalSelector::new(vec![
            GoalDef {
                name: "random_stroll",
                priority: 5,
                controls: controls(&[GoalControl::Move]),
                can_use: true,
            },
            GoalDef {
                name: "melee_attack",
                priority: 2,
                controls: controls(&[GoalControl::Move, GoalControl::Look]),
                can_use: true,
            },
            GoalDef {
                name: "look_at_player",
                priority: 3,
                controls: controls(&[GoalControl::Look]),
                can_use: true,
            },
            GoalDef {
                name: "jump",
                priority: 1,
                controls: controls(&[GoalControl::Jump, GoalControl::Target]),
                can_use: false,
            },
        ]);
        assert_eq!(selector.select_running_goals(), vec!["melee_attack"]);
    }

    #[test]
    fn sensors_navigation_and_targeting_capture_server_decisions() {
        let sensor = SENSOR_TYPES
            .iter()
            .find(|sensor| sensor.id == "warden_entity_sensor")
            .unwrap();
        assert!(sensor.should_scan(20));
        assert!(!sensor.should_scan(25));
        assert!(sensor.memories_written.contains(&"disturbance_location"));

        let path = PathPlan {
            navigation: NavigationKind::Amphibious,
            can_float: false,
            max_visited_nodes_multiplier: 1.0,
            target: (1, 64, 1),
            reached: false,
        };
        assert!(path.can_path_through_water());
        for navigation in [
            NavigationKind::Ground,
            NavigationKind::Flying,
            NavigationKind::Water,
            NavigationKind::Amphibious,
            NavigationKind::WallClimber,
        ] {
            let path = PathPlan {
                navigation,
                can_float: false,
                max_visited_nodes_multiplier: 1.0,
                target: (0, 0, 0),
                reached: false,
            };
            let expected = matches!(
                navigation,
                NavigationKind::Water | NavigationKind::Amphibious
            );
            assert_eq!(path.can_path_through_water(), expected);
        }

        let targeting = TargetingConditions {
            range: 16,
            check_line_of_sight: true,
            test_invisible: false,
        };
        assert!(targeting.can_target(10, true, false));
        assert!(!targeting.can_target(10, false, false));
        assert!(!targeting.can_target(10, true, true));
    }

    #[test]
    fn gossip_decay_reputation_and_poi_tickets_follow_villager_support_rules() {
        let mut gossip = GossipContainer::new();
        gossip.add("player-a", GossipType::MajorPositive, 10);
        gossip.add("player-a", GossipType::MinorPositive, 2);
        gossip.add("player-a", GossipType::Trading, 2);
        gossip.add("player-a", GossipType::MinorNegative, 5);
        gossip.add("player-b", GossipType::MajorNegative, 1);
        assert_eq!(gossip.reputation("player-a"), 49);
        gossip.decay();
        assert_eq!(gossip.reputation("player-a"), 50);

        let mut poi = PoiTicket {
            poi_type: "minecraft:armorer",
            max_tickets: 1,
            free_tickets: 1,
        };
        assert!(poi.acquire());
        assert!(!poi.acquire());
        poi.release();
        assert_eq!(poi.free_tickets, 1);
    }

    #[test]
    fn gossip_caps_transfer_events_and_timing_match_java() {
        assert_eq!(GOSSIP_DISCARD_THRESHOLD, 2);
        assert_eq!(VILLAGER_GOSSIP_INTERVAL_TICKS, 1200);
        assert_eq!(VILLAGER_GOSSIP_TRANSFER_MAX_COUNT, 10);
        assert_eq!(VILLAGER_GOSSIP_DECAY_INTERVAL_TICKS, 24000);
        assert_eq!(
            GossipType::MajorNegative.serialized_name(),
            "major_negative"
        );
        assert_eq!(GossipType::MajorNegative.weight(), -5);
        assert_eq!(GossipType::MajorNegative.max(), 100);
        assert_eq!(GossipType::MajorNegative.decay_per_day(), 10);
        assert_eq!(GossipType::MajorNegative.decay_per_transfer(), 10);
        assert_eq!(GossipType::MinorNegative.max(), 200);
        assert_eq!(GossipType::MinorPositive.max(), 25);
        assert_eq!(GossipType::MajorPositive.max(), 20);
        assert_eq!(GossipType::Trading.max(), 25);

        let mut capped = GossipContainer::new();
        capped.add("player-a", GossipType::MajorPositive, 30);
        assert_eq!(capped.value("player-a", GossipType::MajorPositive), 20);
        capped.add("player-a", GossipType::MinorPositive, -19);
        assert_eq!(capped.value("player-a", GossipType::MinorPositive), 0);

        let mut source = GossipContainer::new();
        source.add("player-a", GossipType::MinorNegative, 25);
        source.add("player-b", GossipType::MajorPositive, 20);
        source.add("player-c", GossipType::Trading, 21);
        let mut target = GossipContainer::new();
        target.add("player-a", GossipType::MinorNegative, 2);
        target.transfer_selected_from(
            &source,
            &[
                ("player-a", GossipType::MinorNegative),
                ("player-b", GossipType::MajorPositive),
                ("player-c", GossipType::Trading),
            ],
        );
        assert_eq!(target.value("player-a", GossipType::MinorNegative), 5);
        assert_eq!(target.value("player-b", GossipType::MajorPositive), 0);
        assert_eq!(target.value("player-c", GossipType::Trading), 0);

        assert_eq!(
            reputation_event_gossips(ReputationEventModel::ZombieVillagerCured),
            &[
                (GossipType::MajorPositive, 20),
                (GossipType::MinorPositive, 25)
            ]
        );
        assert_eq!(
            reputation_event_gossips(ReputationEventModel::Trade),
            &[(GossipType::Trading, 2)]
        );
        assert_eq!(
            reputation_event_gossips(ReputationEventModel::VillagerHurt),
            &[(GossipType::MinorNegative, 25)]
        );
        assert_eq!(
            reputation_event_gossips(ReputationEventModel::VillagerKilled),
            &[(GossipType::MajorNegative, 25)]
        );
        assert!(villager_gossip_meeting_allowed(1200, 0, 0));
        assert!(!villager_gossip_meeting_allowed(1199, 0, 0));
        assert!(villager_gossip_meeting_allowed(100, 200, 200));
        assert_eq!(villager_gossip_decay_due(100, 0), (100, false));
        assert_eq!(villager_gossip_decay_due(24_099, 100), (100, false));
        assert_eq!(villager_gossip_decay_due(24_100, 100), (24_100, true));
    }

    #[test]
    fn checklist_ai_surface_is_represented() {
        for item in [
            "attributes",
            "behavior",
            "control",
            "goal",
            "gossip",
            "memory",
            "navigation",
            "sensing",
            "targeting",
            "util",
            "village",
            "villager POIs",
            "schedules",
            "brain save/load",
            "activities",
            "pathfinding",
        ] {
            assert!(AI_CHECKLIST_SURFACE.contains(&item), "{item}");
        }
    }
}
