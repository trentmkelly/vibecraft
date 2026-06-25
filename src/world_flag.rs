#![allow(dead_code)]

use crate::registry::Identifier;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

pub const FILTERED_REGISTRIES: &[&str] = &[
    "minecraft:item",
    "minecraft:block",
    "minecraft:entity_type",
    "minecraft:game_rule",
    "minecraft:menu",
    "minecraft:potion",
    "minecraft:mob_effect",
];

pub trait FeatureElement {
    fn required_features(&self) -> FeatureFlagSet;

    fn is_enabled(&self, enabled_features: &FeatureFlagSet) -> bool {
        self.required_features().is_subset_of(enabled_features)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureFlagUniverse {
    id: String,
}

impl FeatureFlagUniverse {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl fmt::Display for FeatureFlagUniverse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureFlag {
    universe: Rc<FeatureFlagUniverse>,
    mask: u64,
}

impl FeatureFlag {
    fn new(universe: Rc<FeatureFlagUniverse>, bit: usize) -> Self {
        Self {
            universe,
            mask: 1_u64 << bit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureFlagSet {
    universe: Option<Rc<FeatureFlagUniverse>>,
    mask: u64,
}

impl FeatureFlagSet {
    pub const MAX_CONTAINER_SIZE: usize = 64;

    pub fn empty() -> Self {
        Self {
            universe: None,
            mask: 0,
        }
    }

    pub fn create(
        universe: Rc<FeatureFlagUniverse>,
        flags: impl IntoIterator<Item = FeatureFlag>,
    ) -> Result<Self, String> {
        let mut mask = 0_u64;
        let mut saw_flag = false;
        for flag in flags {
            saw_flag = true;
            mask = compute_mask(&universe, mask, &flag)?;
        }
        if saw_flag {
            Ok(Self {
                universe: Some(universe),
                mask,
            })
        } else {
            Ok(Self::empty())
        }
    }

    pub fn of() -> Self {
        Self::empty()
    }

    pub fn of_one(flag: &FeatureFlag) -> Self {
        Self {
            universe: Some(Rc::clone(&flag.universe)),
            mask: flag.mask,
        }
    }

    pub fn of_many(first: &FeatureFlag, rest: &[FeatureFlag]) -> Result<Self, String> {
        let mut mask = first.mask;
        for flag in rest {
            mask = compute_mask(&first.universe, mask, flag)?;
        }
        Ok(Self {
            universe: Some(Rc::clone(&first.universe)),
            mask,
        })
    }

    pub fn contains(&self, flag: &FeatureFlag) -> bool {
        match &self.universe {
            Some(universe) if Rc::ptr_eq(universe, &flag.universe) => self.mask & flag.mask != 0,
            _ => false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.universe.is_none() && self.mask == 0
    }

    pub fn is_subset_of(&self, set: &Self) -> bool {
        match &self.universe {
            None => true,
            Some(universe) => match &set.universe {
                Some(other_universe) if Rc::ptr_eq(universe, other_universe) => {
                    self.mask & !set.mask == 0
                }
                _ => false,
            },
        }
    }

    pub fn intersects(&self, set: &Self) -> bool {
        match (&self.universe, &set.universe) {
            (Some(universe), Some(other_universe)) if Rc::ptr_eq(universe, other_universe) => {
                self.mask & set.mask != 0
            }
            _ => false,
        }
    }

    pub fn join(&self, other: &Self) -> Result<Self, String> {
        match (&self.universe, &other.universe) {
            (None, _) => Ok(other.clone()),
            (_, None) => Ok(self.clone()),
            (Some(universe), Some(other_universe)) if Rc::ptr_eq(universe, other_universe) => {
                Ok(Self {
                    universe: Some(Rc::clone(universe)),
                    mask: self.mask | other.mask,
                })
            }
            (Some(universe), Some(other_universe)) => Err(format!(
                "Mismatched set elements: '{}' != '{}'",
                universe, other_universe
            )),
        }
    }

    pub fn subtract(&self, other: &Self) -> Result<Self, String> {
        match (&self.universe, &other.universe) {
            (None, _) | (_, None) => Ok(self.clone()),
            (Some(universe), Some(other_universe)) if Rc::ptr_eq(universe, other_universe) => {
                let mask = self.mask & !other.mask;
                if mask == 0 {
                    Ok(Self::empty())
                } else {
                    Ok(Self {
                        universe: Some(Rc::clone(universe)),
                        mask,
                    })
                }
            }
            (Some(universe), Some(other_universe)) => Err(format!(
                "Mismatched set elements: '{}' != '{}'",
                universe, other_universe
            )),
        }
    }
}

fn compute_mask(
    universe: &Rc<FeatureFlagUniverse>,
    mask: u64,
    flag: &FeatureFlag,
) -> Result<u64, String> {
    if !Rc::ptr_eq(universe, &flag.universe) {
        return Err(format!(
            "Mismatched feature universe, expected '{}', but got '{}'",
            universe, flag.universe
        ));
    }
    Ok(mask | flag.mask)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureFlagRegistry {
    universe: Rc<FeatureFlagUniverse>,
    names: BTreeMap<Identifier, FeatureFlag>,
    all_flags: FeatureFlagSet,
}

impl FeatureFlagRegistry {
    pub fn builder(universe_id: &str) -> FeatureFlagRegistryBuilder {
        FeatureFlagRegistryBuilder::new(universe_id)
    }

    pub fn main_26_1_2() -> Result<Self, String> {
        let mut builder = Self::builder("main");
        builder.create_vanilla("vanilla")?;
        builder.create_vanilla("trade_rebalance")?;
        builder.create_vanilla("redstone_experiments")?;
        builder.create_vanilla("minecart_improvements")?;
        builder.build()
    }

    pub fn is_subset(&self, set: &FeatureFlagSet) -> bool {
        set.is_subset_of(&self.all_flags)
    }

    pub fn all_flags(&self) -> &FeatureFlagSet {
        &self.all_flags
    }

    pub fn subset(&self, flags: &[FeatureFlag]) -> Result<FeatureFlagSet, String> {
        FeatureFlagSet::create(Rc::clone(&self.universe), flags.iter().cloned())
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_names(
        &self,
        flag_ids: impl IntoIterator<Item = Identifier>,
    ) -> (FeatureFlagSet, Vec<Identifier>) {
        let mut flags = Vec::new();
        let mut unknown = Vec::new();
        for flag_id in flag_ids {
            if let Some(flag) = self.names.get(&flag_id) {
                flags.push(flag.clone());
            } else {
                unknown.push(flag_id);
            }
        }
        let set = FeatureFlagSet::create(Rc::clone(&self.universe), flags)
            .unwrap_or_else(|err| panic!("{err}"));
        (set, unknown)
    }

    pub fn to_names(&self, set: &FeatureFlagSet) -> BTreeSet<Identifier> {
        self.names
            .iter()
            .filter(|(_id, flag)| set.contains(flag))
            .map(|(id, _flag)| id.clone())
            .collect()
    }
}

pub struct FeatureFlagRegistryBuilder {
    universe: Rc<FeatureFlagUniverse>,
    id: usize,
    flags: BTreeMap<Identifier, FeatureFlag>,
}

impl FeatureFlagRegistryBuilder {
    pub fn new(universe_id: &str) -> Self {
        Self {
            universe: Rc::new(FeatureFlagUniverse::new(universe_id)),
            id: 0,
            flags: BTreeMap::new(),
        }
    }

    pub fn create_vanilla(&mut self, name: &str) -> Result<FeatureFlag, String> {
        self.create(Identifier::with_default_namespace(name)?)
    }

    pub fn create(&mut self, name: Identifier) -> Result<FeatureFlag, String> {
        if self.id >= FeatureFlagSet::MAX_CONTAINER_SIZE {
            return Err("Too many feature flags".to_string());
        }
        let flag = FeatureFlag::new(Rc::clone(&self.universe), self.id);
        self.id += 1;
        if self.flags.insert(name.clone(), flag.clone()).is_some() {
            return Err(format!("Duplicate feature flag {name}"));
        }
        Ok(flag)
    }

    pub fn build(self) -> Result<FeatureFlagRegistry, String> {
        let all_flags =
            FeatureFlagSet::create(Rc::clone(&self.universe), self.flags.values().cloned())?;
        Ok(FeatureFlagRegistry {
            universe: self.universe,
            names: self.flags,
            all_flags,
        })
    }
}

pub struct FeatureFlags {
    pub vanilla: FeatureFlag,
    pub trade_rebalance: FeatureFlag,
    pub redstone_experiments: FeatureFlag,
    pub minecart_improvements: FeatureFlag,
    pub registry: FeatureFlagRegistry,
    pub vanilla_set: FeatureFlagSet,
    pub default_flags: FeatureFlagSet,
}

impl FeatureFlags {
    pub fn main_26_1_2() -> Result<Self, String> {
        let mut builder = FeatureFlagRegistry::builder("main");
        let vanilla = builder.create_vanilla("vanilla")?;
        let trade_rebalance = builder.create_vanilla("trade_rebalance")?;
        let redstone_experiments = builder.create_vanilla("redstone_experiments")?;
        let minecart_improvements = builder.create_vanilla("minecart_improvements")?;
        let registry = builder.build()?;
        let vanilla_set = FeatureFlagSet::of_one(&vanilla);
        let default_flags = vanilla_set.clone();
        Ok(Self {
            vanilla,
            trade_rebalance,
            redstone_experiments,
            minecart_improvements,
            registry,
            vanilla_set,
            default_flags,
        })
    }

    pub fn print_missing_flags(
        registry: &FeatureFlagRegistry,
        allowed_flags: &FeatureFlagSet,
        requested_flags: &FeatureFlagSet,
    ) -> String {
        let requested = registry.to_names(requested_flags);
        let allowed = registry.to_names(allowed_flags);
        requested
            .iter()
            .filter(|id| !allowed.contains(*id))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn is_experimental(&self, features: &FeatureFlagSet) -> bool {
        !features.is_subset_of(&self.vanilla_set)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeatureElementModel {
    required_features: FeatureFlagSet,
}

impl FeatureElementModel {
    pub fn new(required_features: FeatureFlagSet) -> Self {
        Self { required_features }
    }
}

impl FeatureElement for FeatureElementModel {
    fn required_features(&self) -> FeatureFlagSet {
        self.required_features.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected Ok(..), got Err({err:?})"),
        }
    }

    fn id(value: &str) -> Identifier {
        must_ok(Identifier::parse(value))
    }

    #[test]
    fn feature_flag_universe_and_masks_match_java_identity_model() {
        let flag_source = vibecraft_java_source!("/net/minecraft/world/flag/FeatureFlag.java");
        let universe_source =
            vibecraft_java_source!("/net/minecraft/world/flag/FeatureFlagUniverse.java");
        assert!(flag_source.contains("this.mask = 1L << bit;"));
        assert!(universe_source.contains("return this.id;"));

        let mut first_builder = FeatureFlagRegistry::builder("main");
        let first = must_ok(first_builder.create_vanilla("vanilla"));
        let mut second_builder = FeatureFlagRegistry::builder("main");
        let second = must_ok(second_builder.create_vanilla("vanilla"));
        assert_eq!(first.mask, 1);
        assert_eq!(second.mask, 1);
        assert!(!Rc::ptr_eq(&first.universe, &second.universe));
        assert_eq!(first.universe.to_string(), "main");
    }

    #[test]
    fn feature_flag_set_preserves_empty_subset_and_universe_errors() {
        let source = vibecraft_java_source!("/net/minecraft/world/flag/FeatureFlagSet.java");
        assert!(source.contains("private static final FeatureFlagSet EMPTY = new FeatureFlagSet(null, 0L);"));
        assert!(source.contains("public static final int MAX_CONTAINER_SIZE = 64;"));
        assert!(source.contains("Mismatched feature universe, expected '"));
        assert!(source.contains("Mismatched set elements: '"));

        let mut builder = FeatureFlagRegistry::builder("main");
        let vanilla = must_ok(builder.create_vanilla("vanilla"));
        let trade = must_ok(builder.create_vanilla("trade_rebalance"));
        let vanilla_set = FeatureFlagSet::of_one(&vanilla);
        let trade_set = FeatureFlagSet::of_one(&trade);
        let joined = must_ok(vanilla_set.join(&trade_set));
        assert!(FeatureFlagSet::empty().is_subset_of(&vanilla_set));
        assert!(vanilla_set.is_subset_of(&joined));
        assert!(joined.intersects(&trade_set));
        assert_eq!(must_ok(joined.subtract(&trade_set)), vanilla_set);
        assert!(must_ok(vanilla_set.subtract(&vanilla_set)).is_empty());

        let mut other_builder = FeatureFlagRegistry::builder("other");
        let other = must_ok(other_builder.create_vanilla("vanilla"));
        assert_eq!(
            FeatureFlagSet::of_many(&vanilla, std::slice::from_ref(&other)),
            Err("Mismatched feature universe, expected 'main', but got 'other'".to_string())
        );
        assert_eq!(
            vanilla_set.join(&FeatureFlagSet::of_one(&other)),
            Err("Mismatched set elements: 'main' != 'other'".to_string())
        );
    }

    #[test]
    fn feature_flag_registry_builder_names_and_unknowns_match_java() {
        let source = vibecraft_java_source!("/net/minecraft/world/flag/FeatureFlagRegistry.java");
        assert!(source.contains("if (this.id >= 64)"));
        assert!(source.contains("Duplicate feature flag "));
        assert!(source.contains("Unknown feature flag: {}"));
        assert!(source.contains("DataResult.error(() -> \"Unknown feature ids: \" + unknownIds, result)"));

        let mut duplicate_builder = FeatureFlagRegistry::builder("main");
        let _duplicate_first = must_ok(duplicate_builder.create_vanilla("vanilla"));
        assert_eq!(
            duplicate_builder.create_vanilla("vanilla"),
            Err("Duplicate feature flag minecraft:vanilla".to_string())
        );

        let mut builder = FeatureFlagRegistry::builder("main");
        let vanilla = must_ok(builder.create_vanilla("vanilla"));
        let trade = must_ok(builder.create_vanilla("trade_rebalance"));
        let registry = must_ok(builder.build());

        let (resolved, unknown) = registry.from_names([id("minecraft:vanilla"), id("custom:missing")]);
        assert!(resolved.contains(&vanilla));
        assert_eq!(unknown, vec![id("custom:missing")]);

        let names = registry.to_names(&must_ok(FeatureFlagSet::of_many(&vanilla, &[trade])));
        assert!(names.contains(&id("minecraft:vanilla")));
        assert!(names.contains(&id("minecraft:trade_rebalance")));
        assert!(registry.is_subset(&resolved));

        let mut full_builder = FeatureFlagRegistry::builder("main");
        for index in 0..64 {
            must_ok(full_builder.create_vanilla(&format!("flag_{index}")));
        }
        assert_eq!(
            full_builder.create_vanilla("too_many"),
            Err("Too many feature flags".to_string())
        );
    }

    #[test]
    fn feature_flags_static_values_and_missing_flags_match_java() {
        let source = vibecraft_java_source!("/net/minecraft/world/flag/FeatureFlags.java");
        assert!(source.contains("new FeatureFlagRegistry.Builder(\"main\")"));
        assert!(source.contains("VANILLA = builder.createVanilla(\"vanilla\")"));
        assert!(source.contains("TRADE_REBALANCE = builder.createVanilla(\"trade_rebalance\")"));
        assert!(source.contains("DEFAULT_FLAGS = VANILLA_SET;"));
        assert!(source.contains("return !features.isSubsetOf(VANILLA_SET);"));

        let flags = must_ok(FeatureFlags::main_26_1_2());
        assert_eq!(
            flags.registry.to_names(&flags.default_flags),
            BTreeSet::from([id("minecraft:vanilla")])
        );
        assert!(!flags.is_experimental(&flags.vanilla_set));
        let requested = must_ok(flags.vanilla_set.join(&FeatureFlagSet::of_one(
            &flags.minecart_improvements,
        )));
        assert!(flags.is_experimental(&requested));
        assert_eq!(
            FeatureFlags::print_missing_flags(&flags.registry, &flags.vanilla_set, &requested),
            "minecraft:minecart_improvements"
        );
    }

    #[test]
    fn feature_element_filtered_registries_and_enabled_check_match_java() {
        let source = vibecraft_java_source!("/net/minecraft/world/flag/FeatureElement.java");
        assert!(source.contains("Registries.ITEM, Registries.BLOCK, Registries.ENTITY_TYPE"));
        assert!(source.contains("return this.requiredFeatures().isSubsetOf(enabledFeatures);"));

        assert_eq!(
            FILTERED_REGISTRIES,
            [
                "minecraft:item",
                "minecraft:block",
                "minecraft:entity_type",
                "minecraft:game_rule",
                "minecraft:menu",
                "minecraft:potion",
                "minecraft:mob_effect",
            ]
        );

        let flags = must_ok(FeatureFlags::main_26_1_2());
        let element = FeatureElementModel::new(FeatureFlagSet::of_one(&flags.redstone_experiments));
        assert!(!element.is_enabled(&flags.default_flags));
        let enabled = must_ok(flags.default_flags.join(&FeatureFlagSet::of_one(
            &flags.redstone_experiments,
        )));
        assert!(element.is_enabled(&enabled));
    }
}
