//! The advancement predicates loot conditions embed: `EntityPredicate`,
//! `DamageSourcePredicate` and `LocationPredicate`.
//!
//! Java shares these records with advancement criteria
//! (`net.minecraft.advancements.criterion`). The JSON documents are validated in full
//! by the advancement condition schema; the loot model evaluates the fields vanilla
//! loot tables use and keeps a condition `Unmodeled` when a predicate carries a field
//! outside that set (see `json_codec/predicates.rs`), so nothing is evaluated
//! approximately.

use super::{
    EntityFlags, HolderSet, LootContext, LootDamageSource, LootEntity, LootParamKey,
    LootParamValue, ToolPredicate,
};

/// `EntityPredicate.matches` for the modeled fields.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EntityPredicate {
    /// `type`: an `EntityTypePredicate` holder set.
    pub entity_type: Option<HolderSet>,
    /// `components`: `DataComponentExactPredicate` entries (component id and the
    /// canonical value text, see `canonical_component_value`).
    pub components: Vec<(String, String)>,
    /// `type_specific`.
    pub type_specific: Option<TypeSpecificPredicate>,
    /// `equipment`: an `ItemPredicate` per `EquipmentSlot` serialized name.
    pub equipment: Vec<(String, ToolPredicate)>,
    /// `flags`.
    pub flags: Option<EntityFlagsPredicate>,
    /// `vehicle`: an entity predicate the entity's vehicle must satisfy.
    pub vehicle: Option<Box<EntityPredicate>>,
}

/// `EntityFlagsPredicate`: each present flag must equal the entity's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EntityFlagsPredicate {
    pub is_on_ground: Option<bool>,
    pub is_on_fire: Option<bool>,
    pub is_crouching: Option<bool>,
    pub is_sprinting: Option<bool>,
    pub is_swimming: Option<bool>,
    pub is_flying: Option<bool>,
    pub is_baby: Option<bool>,
    pub is_in_water: Option<bool>,
    pub is_fall_flying: Option<bool>,
}

impl EntityFlagsPredicate {
    /// `EntityFlagsPredicate.matches`. `is_baby` and `is_fall_flying` only constrain
    /// living entities.
    pub fn matches(&self, entity: &LootEntity) -> bool {
        let flags: EntityFlags = entity.flags;
        let equal = |expected: Option<bool>, actual: bool| expected.is_none_or(|e| e == actual);
        equal(self.is_on_ground, flags.on_ground)
            && equal(self.is_on_fire, flags.on_fire)
            && equal(self.is_crouching, flags.crouching)
            && equal(self.is_sprinting, flags.sprinting)
            && equal(self.is_swimming, flags.swimming)
            && equal(self.is_flying, flags.flying)
            && equal(self.is_in_water, flags.in_water)
            && (!entity.living || equal(self.is_fall_flying, flags.fall_flying))
            && (!entity.living || equal(self.is_baby, flags.baby))
    }
}

/// `MinMaxBounds.Ints`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntBounds {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl IntBounds {
    /// `MinMaxBounds.Ints.matches`.
    pub fn matches(self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

/// The `EntitySubPredicate` types vanilla loot tables use.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSpecificPredicate {
    /// `SheepPredicate`.
    Sheep { sheared: Option<bool> },
    /// `SlimePredicate`.
    Slime { size: IntBounds },
    /// `RaiderPredicate`.
    Raider { has_raid: bool, is_captain: bool },
    /// `FishingHookPredicate`.
    FishingHook { in_open_water: Option<bool> },
}

impl EntityPredicate {
    /// `EntityPredicate.matches`: false for an absent entity.
    pub fn matches(&self, entity: Option<&LootEntity>, context: &LootContext) -> bool {
        let Some(entity) = entity else {
            return false;
        };
        if let Some(types) = &self.entity_type {
            if !types.contains_in("entity_type", &entity.entity_type, context) {
                return false;
            }
        }
        if let Some(type_specific) = &self.type_specific {
            if !type_specific.matches(entity) {
                return false;
            }
        }
        if !self.equipment.is_empty() && !self.equipment_matches(entity, context) {
            return false;
        }
        if self
            .flags
            .as_ref()
            .is_some_and(|flags| !flags.matches(entity))
        {
            return false;
        }
        if let Some(vehicle) = &self.vehicle {
            if !vehicle.matches(entity.vehicle.as_deref(), context) {
                return false;
            }
        }
        // `DataComponentMatchers.test`: every exact entry must equal the entity's value.
        self.components
            .iter()
            .all(|(component, value)| entity.components.get(component) == Some(value))
    }

    /// `EntityEquipmentPredicate.matches`: only living entities have equipment slots.
    fn equipment_matches(&self, entity: &LootEntity, context: &LootContext) -> bool {
        if !entity.living {
            return false;
        }
        let empty = super::LootStack::new("minecraft:air", 0);
        self.equipment.iter().all(|(slot, predicate)| {
            predicate.matches_stack(entity.equipment.get(slot).unwrap_or(&empty), context)
        })
    }
}

impl TypeSpecificPredicate {
    /// `EntitySubPredicate.matches`.
    fn matches(&self, entity: &LootEntity) -> bool {
        match self {
            // `entity instanceof Sheep sheep ? sheared.isEmpty() || sheep.isSheared() == sheared : false`.
            Self::Sheep { sheared } => entity
                .sheared
                .is_some_and(|actual| sheared.is_none_or(|expected| actual == expected)),
            Self::Slime { size } => entity.slime_size.is_some_and(|actual| size.matches(actual)),
            Self::Raider {
                has_raid,
                is_captain,
            } => entity
                .raider
                .is_some_and(|raider| raider.has_raid == *has_raid && raider.is_captain == *is_captain),
            // An empty `in_open_water` matches any entity; otherwise it must be a hook.
            Self::FishingHook { in_open_water } => match in_open_water {
                None => true,
                Some(expected) => entity.open_water_fishing == Some(*expected),
            },
        }
    }
}

/// `DamageSourcePredicate`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DamageSourcePredicate {
    /// `tags`: `TagPredicate<DamageType>` as (tag id, expected).
    pub tags: Vec<(String, bool)>,
    /// `direct_entity`.
    pub direct_entity: Option<EntityPredicate>,
    /// `source_entity`.
    pub source_entity: Option<EntityPredicate>,
    /// `is_direct`.
    pub is_direct: Option<bool>,
}

impl DamageSourcePredicate {
    /// `DamageSourcePredicate.matches`.
    pub fn matches(&self, source: &LootDamageSource, context: &LootContext) -> bool {
        for (tag, expected) in &self.tags {
            // `TagPredicate.matches`: `holder.is(tag) == expected`.
            let in_tag = context
                .tag_members("damage_type", tag)
                .is_some_and(|members| members.contains(&source.damage_type));
            if in_tag != *expected {
                return false;
            }
        }
        if let Some(direct) = &self.direct_entity {
            if !direct.matches(source.direct_entity.as_ref(), context) {
                return false;
            }
        }
        if let Some(entity) = &self.source_entity {
            if !entity.matches(source.source_entity.as_ref(), context) {
                return false;
            }
        }
        self.is_direct
            .is_none_or(|expected| expected == source.is_direct)
    }
}

/// `MinMaxBounds.Doubles`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DoubleBounds {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl DoubleBounds {
    /// `MinMaxBounds.Doubles.matches`.
    pub fn matches(self, value: f64) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }

    /// `MinMaxBounds.isAny`.
    pub fn is_any(self) -> bool {
        self.min.is_none() && self.max.is_none()
    }
}

/// `BlockPredicate` without an NBT requirement.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlockPredicate {
    /// `blocks`.
    pub blocks: Option<HolderSet>,
    /// `state`: exact `StatePropertiesPredicate` matchers, property to value.
    pub state: Vec<(String, String)>,
}

impl BlockPredicate {
    /// `BlockPredicate.matches(level, pos)`.
    fn matches(&self, context: &LootContext, x: i32, y: i32, z: i32) -> bool {
        let Some(level) = &context.level else {
            return false;
        };
        if !level.is_loaded(x, y, z) {
            return false;
        }
        let state = level.block_state(x, y, z);
        if let Some(blocks) = &self.blocks {
            if !blocks.contains_in("block", &state.block, context) {
                return false;
            }
        }
        // `StatePropertiesPredicate.matches`: the property must exist and equal.
        self.state
            .iter()
            .all(|(property, value)| state.properties.get(property) == Some(value))
    }
}

/// `LocationPredicate` for `position`, `biomes`, `dimension` and `block`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LocationPredicate {
    /// `position`: x, y and z bounds.
    pub position: Option<[DoubleBounds; 3]>,
    /// `biomes`.
    pub biomes: Option<HolderSet>,
    /// `dimension`.
    pub dimension: Option<String>,
    /// `block`.
    pub block: Option<BlockPredicate>,
}

impl LocationPredicate {
    /// `LocationPredicate.matches(level, x, y, z)`.
    pub fn matches(&self, context: &LootContext, x: f64, y: f64, z: f64) -> bool {
        if let Some([bx, by, bz]) = self.position {
            if !(bx.matches(x) && by.matches(y) && bz.matches(z)) {
                return false;
            }
        }
        if let Some(dimension) = &self.dimension {
            match &context.level {
                Some(level) if &level.dimension() == dimension => {}
                _ => return false,
            }
        }
        // `BlockPos.containing`.
        let (bx, by, bz) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if let Some(biomes) = &self.biomes {
            let in_biomes = context.level.as_ref().is_some_and(|level| {
                level.is_loaded(bx, by, bz)
                    && biomes.contains_in("worldgen/biome", &level.biome(bx, by, bz), context)
            });
            if !in_biomes {
                return false;
            }
        }
        self.block
            .as_ref()
            .is_none_or(|block| block.matches(context, bx, by, bz))
    }
}

impl LootContext {
    /// `LootContextParams.ORIGIN`.
    pub fn origin(&self) -> Option<(f64, f64, f64)> {
        match self.params.get(LootParamKey::Origin) {
            Some(LootParamValue::Origin(x, y, z)) => Some((*x, *y, *z)),
            _ => None,
        }
    }
}
