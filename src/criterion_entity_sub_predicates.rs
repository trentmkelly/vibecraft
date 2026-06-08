use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySubPredicateInstanceModel {
    codec: EntitySubPredicateTypeModel,
    required_entity_kind: Identifier,
}

impl EntitySubPredicateInstanceModel {
    pub fn new(codec: EntitySubPredicateTypeModel, required_entity_kind: Identifier) -> Self {
        Self {
            codec,
            required_entity_kind,
        }
    }

    pub fn codec(&self) -> EntitySubPredicateTypeModel {
        self.codec
    }

    pub fn matches(
        &self,
        entity: &EntityContextModel,
        _level: &LevelContextModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        entity.entity_kind == self.required_entity_kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntitySubPredicateTypeModel {
    Lightning,
    FishingHook,
    Player,
    Slime,
    Raider,
    Sheep,
}

impl EntitySubPredicateTypeModel {
    pub fn id(self) -> Identifier {
        Identifier::parse(match self {
            Self::Lightning => "minecraft:lightning",
            Self::FishingHook => "minecraft:fishing_hook",
            Self::Player => "minecraft:player",
            Self::Slime => "minecraft:slime",
            Self::Raider => "minecraft:raider",
            Self::Sheep => "minecraft:sheep",
        })
        .unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySubPredicateRegistryModel {
    entries: BTreeMap<Identifier, EntitySubPredicateTypeModel>,
}

impl EntitySubPredicateRegistryModel {
    pub fn bootstrap() -> (Self, EntitySubPredicateTypeModel) {
        let mut registry = Self {
            entries: BTreeMap::new(),
        };

        let lightning = registry.register("lightning", EntitySubPredicateTypeModel::Lightning);
        registry.register("fishing_hook", EntitySubPredicateTypeModel::FishingHook);
        registry.register("player", EntitySubPredicateTypeModel::Player);
        registry.register("slime", EntitySubPredicateTypeModel::Slime);
        registry.register("raider", EntitySubPredicateTypeModel::Raider);
        registry.register("sheep", EntitySubPredicateTypeModel::Sheep);

        (registry, lightning)
    }

    fn register(
        &mut self,
        id: &str,
        value: EntitySubPredicateTypeModel,
    ) -> EntitySubPredicateTypeModel {
        self.entries.insert(Identifier::parse(id).unwrap(), value);
        value
    }

    pub fn get(&self, id: &Identifier) -> Option<EntitySubPredicateTypeModel> {
        self.entries.get(id).copied()
    }

    pub fn ids(&self) -> Vec<Identifier> {
        self.entries.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityContextModel {
    entity_kind: Identifier,
}

impl EntityContextModel {
    pub fn new(entity_kind: Identifier) -> Self {
        Self { entity_kind }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LevelContextModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3Model {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3Model {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    #[test]
    fn entity_sub_predicate_exposes_codec_and_routes_matches() {
        let predicate = EntitySubPredicateInstanceModel::new(
            EntitySubPredicateTypeModel::Slime,
            id("minecraft:slime"),
        );

        assert_eq!(predicate.codec(), EntitySubPredicateTypeModel::Slime);
        assert!(predicate.matches(
            &EntityContextModel::new(id("minecraft:slime")),
            &LevelContextModel,
            Some(Vec3Model::new(0, 64, 0))
        ));
        assert!(!predicate.matches(
            &EntityContextModel::new(id("minecraft:zombie")),
            &LevelContextModel,
            None
        ));
    }

    #[test]
    fn sub_predicate_type_ids_match_java_registry_names() {
        assert_eq!(
            EntitySubPredicateTypeModel::Lightning.id(),
            id("minecraft:lightning")
        );
        assert_eq!(
            EntitySubPredicateTypeModel::FishingHook.id(),
            id("minecraft:fishing_hook")
        );
        assert_eq!(
            EntitySubPredicateTypeModel::Player.id(),
            id("minecraft:player")
        );
        assert_eq!(
            EntitySubPredicateTypeModel::Slime.id(),
            id("minecraft:slime")
        );
        assert_eq!(
            EntitySubPredicateTypeModel::Raider.id(),
            id("minecraft:raider")
        );
        assert_eq!(
            EntitySubPredicateTypeModel::Sheep.id(),
            id("minecraft:sheep")
        );
    }

    #[test]
    fn registry_bootstrap_registers_all_java_sub_predicate_types() {
        let (registry, bootstrap_value) = EntitySubPredicateRegistryModel::bootstrap();

        assert_eq!(bootstrap_value, EntitySubPredicateTypeModel::Lightning);
        assert_eq!(
            registry.get(&id("minecraft:lightning")),
            Some(EntitySubPredicateTypeModel::Lightning)
        );
        assert_eq!(
            registry.get(&id("minecraft:fishing_hook")),
            Some(EntitySubPredicateTypeModel::FishingHook)
        );
        assert_eq!(
            registry.get(&id("minecraft:player")),
            Some(EntitySubPredicateTypeModel::Player)
        );
        assert_eq!(
            registry.get(&id("minecraft:slime")),
            Some(EntitySubPredicateTypeModel::Slime)
        );
        assert_eq!(
            registry.get(&id("minecraft:raider")),
            Some(EntitySubPredicateTypeModel::Raider)
        );
        assert_eq!(
            registry.get(&id("minecraft:sheep")),
            Some(EntitySubPredicateTypeModel::Sheep)
        );
    }

    #[test]
    fn registry_ids_are_namespaced_like_identifier_codec_entries() {
        let (registry, _) = EntitySubPredicateRegistryModel::bootstrap();

        assert_eq!(
            registry.ids(),
            vec![
                id("minecraft:fishing_hook"),
                id("minecraft:lightning"),
                id("minecraft:player"),
                id("minecraft:raider"),
                id("minecraft:sheep"),
                id("minecraft:slime"),
            ]
        );
    }
}
