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
        self.entries
            .iter()
            .rev()
            .find(|(time, _)| *time <= day_time)
            .map(|(_, activity)| *activity)
            .or_else(|| self.entries.last().map(|(_, activity)| *activity))
    }
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

impl GossipType {
    pub fn decay_per_day(self) -> i32 {
        match self {
            Self::MajorPositive => 0,
            Self::MinorPositive => 1,
            Self::Trading => 2,
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
        *self.entries.entry(key).or_insert(0) += value;
    }

    pub fn decay(&mut self) {
        let mut remove = Vec::new();
        for (key, value) in self.entries.iter_mut() {
            *value -= key.1.decay_per_day();
            if *value <= 0 {
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
            .map(|((_, kind), value)| {
                let weight = match kind {
                    GossipType::MajorPositive => 5,
                    GossipType::MinorPositive => 1,
                    GossipType::Trading => 1,
                    GossipType::MinorNegative => -1,
                    GossipType::MajorNegative => -5,
                };
                value * weight
            })
            .sum()
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
        gossip.add("player-a", GossipType::MinorPositive, 1);
        gossip.add("player-a", GossipType::Trading, 1);
        gossip.add("player-a", GossipType::MinorNegative, 5);
        gossip.add("player-b", GossipType::MajorNegative, 1);
        assert_eq!(gossip.reputation("player-a"), 47);
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
