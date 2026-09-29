//! What loot predicates and functions can see of the world.
//!
//! Java hands loot evaluation the live `Entity`, `DamageSource`, `BlockEntity` and
//! `ServerLevel` objects. The loot model instead receives immutable snapshots of the
//! facts the vanilla predicates read, taken by the caller when it builds the
//! [`LootContext`](super::LootContext):
//!
//! * [`LootEntity`] for each entity context parameter ([`LootEntityTarget`]),
//! * [`LootDamageSource`] for `LootContextParams.DAMAGE_SOURCE`,
//! * [`LootLevel`] for the `ServerLevel` queries `LocationPredicate` makes,
//! * [`RegistryTags`] for resolved registry tags (`HolderSet` tag references).

use std::collections::{BTreeMap, HashMap};

use super::{LootParamKey, LootStack};

/// `LootContext.EntityTarget`: the entity context parameters a loot condition, function
/// or predicate can name, by serialized name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LootEntityTarget {
    This,
    Attacker,
    DirectAttacker,
    AttackingPlayer,
    TargetEntity,
    InteractingEntity,
}

impl LootEntityTarget {
    /// Every target, in Java declaration order.
    pub const ALL: [Self; 6] = [
        Self::This,
        Self::Attacker,
        Self::DirectAttacker,
        Self::AttackingPlayer,
        Self::TargetEntity,
        Self::InteractingEntity,
    ];

    /// `EntityTarget.getSerializedName`.
    pub fn name(self) -> &'static str {
        match self {
            Self::This => "this",
            Self::Attacker => "attacker",
            Self::DirectAttacker => "direct_attacker",
            Self::AttackingPlayer => "attacking_player",
            Self::TargetEntity => "target_entity",
            Self::InteractingEntity => "interacting_entity",
        }
    }

    /// `EntityTarget.CODEC.byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|target| target.name() == name)
    }

    /// `EntityTarget.contextParam`.
    pub fn param(self) -> LootParamKey {
        match self {
            Self::This => LootParamKey::ThisEntity,
            Self::Attacker => LootParamKey::AttackingEntity,
            Self::DirectAttacker => LootParamKey::DirectAttackingEntity,
            Self::AttackingPlayer => LootParamKey::LastDamagePlayer,
            Self::TargetEntity => LootParamKey::TargetEntity,
            Self::InteractingEntity => LootParamKey::InteractingEntity,
        }
    }
}

/// A snapshot of an `Entity` as loot predicates observe it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LootEntity {
    /// `Entity.getType()` registry id.
    pub entity_type: String,
    /// The entity's data components (`Entity.get(type)`) by component id, in the
    /// canonical text form of [`canonical_component_value`].
    pub components: HashMap<String, String>,
    /// Whether the entity is a `LivingEntity` (equipment predicates need one).
    pub living: bool,
    /// `LivingEntity.getItemBySlot` per `EquipmentSlot` serialized name; absent slots
    /// hold the empty stack.
    pub equipment: HashMap<String, LootStack>,
    /// `Entity` boolean state read by `EntityFlagsPredicate`.
    pub flags: EntityFlags,
    /// The entity's vehicle (`Entity.getVehicle`).
    pub vehicle: Option<Box<LootEntity>>,
    /// `Sheep.isSheared`; `Some` exactly for sheep.
    pub sheared: Option<bool>,
    /// `Slime.getSize`; `Some` exactly for slimes and magma cubes.
    pub slime_size: Option<i32>,
    /// `Raider.hasRaid` / `isCaptain`; `Some` exactly for raiders.
    pub raider: Option<RaiderState>,
    /// `FishingHook.isOpenWaterFishing`; `Some` exactly for fishing hooks.
    pub open_water_fishing: Option<bool>,
}

/// The `Entity` state `EntityFlagsPredicate` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EntityFlags {
    pub on_ground: bool,
    pub on_fire: bool,
    pub crouching: bool,
    pub sprinting: bool,
    pub swimming: bool,
    /// `isFallFlying` or a flying player (`EntityFlagsPredicate` `is_flying`).
    pub flying: bool,
    pub in_water: bool,
    /// `LivingEntity.isFallFlying`.
    pub fall_flying: bool,
    /// `LivingEntity.isBaby`.
    pub baby: bool,
}

/// `Raider` state read by `RaiderPredicate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RaiderState {
    pub has_raid: bool,
    pub is_captain: bool,
}

impl LootEntity {
    /// A non-living entity of `entity_type`.
    pub fn new(entity_type: impl Into<String>) -> Self {
        Self {
            entity_type: entity_type.into(),
            ..Self::default()
        }
    }

    /// Marks the entity as a `LivingEntity`.
    pub fn living(mut self) -> Self {
        self.living = true;
        self
    }

    /// Sets a data component from its JSON form.
    pub fn with_component(mut self, component: &str, value: &serde_json::Value) -> Self {
        self.components.insert(
            component.to_string(),
            canonical_component_value(component, value),
        );
        self
    }

    /// Sets the stack in an equipment slot (`head`, `chest`, `legs`, `feet`, `body`,
    /// `mainhand`, `offhand`).
    pub fn with_equipment(mut self, slot: &str, stack: LootStack) -> Self {
        self.equipment.insert(slot.to_string(), stack);
        self
    }

    /// Marks the entity as a sheep with the given `Sheep.isSheared`.
    pub fn with_sheared(mut self, sheared: bool) -> Self {
        self.sheared = Some(sheared);
        self
    }

    /// Sets the entity's flags.
    pub fn with_flags(mut self, flags: EntityFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the entity's vehicle.
    pub fn with_vehicle(mut self, vehicle: LootEntity) -> Self {
        self.vehicle = Some(Box::new(vehicle));
        self
    }

    /// Marks the entity as a slime of `size`.
    pub fn with_slime_size(mut self, size: i32) -> Self {
        self.slime_size = Some(size);
        self
    }

    /// Marks the entity as a raider.
    pub fn with_raider(mut self, raider: RaiderState) -> Self {
        self.raider = Some(raider);
        self
    }

    /// Marks the entity as a fishing hook.
    pub fn with_open_water_fishing(mut self, open_water: bool) -> Self {
        self.open_water_fishing = Some(open_water);
        self
    }
}

/// The canonical text of a data component value, used to compare an entity's
/// component with a `DataComponentExactPredicate` value. Strings compare as written,
/// except registry-holder components (the `*/variant` types) whose value is an
/// identifier and so gain the default `minecraft:` namespace; every other value
/// compares as its compact JSON.
pub fn canonical_component_value(component: &str, value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) if component.ends_with("/variant") => {
            if text.contains(':') {
                text.clone()
            } else {
                format!("minecraft:{text}")
            }
        }
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// A snapshot of a `DamageSource` as `DamageSourcePredicate` observes it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LootDamageSource {
    /// `DamageSource.type()` registry id (`minecraft:lightning_bolt`).
    pub damage_type: String,
    /// `DamageSource.getDirectEntity`.
    pub direct_entity: Option<LootEntity>,
    /// `DamageSource.getEntity` (the causing entity).
    pub source_entity: Option<LootEntity>,
    /// `DamageSource.isDirect`.
    pub is_direct: bool,
}

/// The block state at a position, as `BlockPredicate` observes it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LootBlockState {
    /// The block's registry id.
    pub block: String,
    /// The state's property values by property name.
    pub properties: BTreeMap<String, String>,
}

/// The `ServerLevel` queries `LocationPredicate` makes.
pub trait LootLevel: Send + Sync {
    /// `Level.dimension().identifier()`.
    fn dimension(&self) -> String;
    /// `Level.isLoaded(pos)`.
    fn is_loaded(&self, x: i32, y: i32, z: i32) -> bool;
    /// `Level.getBiome(pos)`'s registry id.
    fn biome(&self, x: i32, y: i32, z: i32) -> String;
    /// `Level.getBlockState(pos)`.
    fn block_state(&self, x: i32, y: i32, z: i32) -> LootBlockState;
}

/// Resolved registry tags: `registry path -> tag id -> member ids` in tag order (the
/// order `HolderSet.Named` iterates, which random selection depends on).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryTags {
    by_registry: HashMap<String, HashMap<String, Vec<String>>>,
}

impl RegistryTags {
    /// Adds (or replaces) the tag `tag` of `registry` (`"item"`, `"enchantment"`,
    /// `"entity_type"`, `"damage_type"`, `"worldgen/biome"`, ...).
    pub fn insert(&mut self, registry: &str, tag: &str, members: Vec<String>) {
        self.by_registry
            .entry(registry.to_string())
            .or_default()
            .insert(tag.to_string(), members);
    }

    /// The members of `tag` in `registry`.
    pub fn get(&self, registry: &str, tag: &str) -> Option<&[String]> {
        self.by_registry
            .get(registry)?
            .get(tag)
            .map(Vec::as_slice)
    }
}

/// The tag registries loot `HolderSet`s reference: the tag directories under
/// `tags/` of the item, enchantment, instrument, block, entity type, damage type and
/// biome registries.
const LOOT_TAG_REGISTRIES: [&str; 7] = [
    "item",
    "enchantment",
    "instrument",
    "block",
    "entity_type",
    "damage_type",
    "worldgen/biome",
];

impl RegistryTags {
    /// `TagLoader.loadTagsForRegistry` for every registry in [`LOOT_TAG_REGISTRIES`]:
    /// the merged, nested-tag-resolved tags of every pack in `manager`. Element
    /// existence is not checked here (the registries themselves live elsewhere), so an
    /// entry naming an unknown element is kept, like Java's `required: false`.
    pub fn load(manager: &crate::registry_pipeline::resources::ResourceManager) -> Self {
        let mut tags = Self::default();
        for registry in LOOT_TAG_REGISTRIES {
            let mut known = |_: &crate::registry::Identifier, _: bool| true;
            let mut problems = Vec::new();
            let loaded = crate::registry_pipeline::tags::load_tags_for_registry(
                manager,
                registry,
                &mut known,
                &mut problems,
            );
            for (tag, members) in loaded {
                tags.insert(
                    registry,
                    &tag.to_string(),
                    members.iter().map(ToString::to_string).collect(),
                );
            }
        }
        tags
    }
}
