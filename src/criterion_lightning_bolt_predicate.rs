use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningBoltPredicateModel {
    blocks_set_on_fire: IntBoundsModel,
    entity_struck: Option<EntityPredicateModel>,
}

impl LightningBoltPredicateModel {
    pub fn new(
        blocks_set_on_fire: IntBoundsModel,
        entity_struck: Option<EntityPredicateModel>,
    ) -> Self {
        Self {
            blocks_set_on_fire,
            entity_struck,
        }
    }

    pub fn block_set_on_fire(count: IntBoundsModel) -> Self {
        Self::new(count, None)
    }

    pub fn codec(&self) -> EntitySubPredicateTypeModel {
        EntitySubPredicateTypeModel::Lightning
    }

    pub fn matches(
        &self,
        entity: &EntityModel,
        level: &ServerLevelModel,
        position: Option<Vec3Model>,
    ) -> bool {
        let EntityModel::LightningBolt(bolt) = entity else {
            return false;
        };

        self.blocks_set_on_fire.matches(bolt.blocks_set_on_fire)
            && self.entity_struck.as_ref().is_none_or(|predicate| {
                bolt.hit_entities
                    .iter()
                    .any(|entity| predicate.matches(level, position, entity))
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySubPredicateTypeModel {
    Lightning,
}

impl EntitySubPredicateTypeModel {
    pub fn id(self) -> &'static str {
        match self {
            Self::Lightning => "minecraft:lightning",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityModel {
    Generic { entity_type: Identifier },
    LightningBolt(LightningBoltModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningBoltModel {
    blocks_set_on_fire: i32,
    hit_entities: Vec<HitEntityModel>,
}

impl LightningBoltModel {
    pub fn new(blocks_set_on_fire: i32, hit_entities: Vec<HitEntityModel>) -> Self {
        Self {
            blocks_set_on_fire,
            hit_entities,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HitEntityModel {
    entity_type: Identifier,
    level_id: Identifier,
    position: Option<Vec3Model>,
}

impl HitEntityModel {
    pub fn new(entity_type: Identifier, level_id: Identifier, position: Option<Vec3Model>) -> Self {
        Self {
            entity_type,
            level_id,
            position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLevelModel {
    id: Identifier,
}

impl ServerLevelModel {
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    entity_type: Option<Identifier>,
    level_id: Option<Identifier>,
    position: Option<Vec3Model>,
}

impl EntityPredicateModel {
    pub fn any() -> Self {
        Self {
            entity_type: None,
            level_id: None,
            position: None,
        }
    }

    pub fn entity_type(entity_type: Identifier) -> Self {
        Self {
            entity_type: Some(entity_type),
            ..Self::any()
        }
    }

    pub fn in_level(level_id: Identifier) -> Self {
        Self {
            level_id: Some(level_id),
            ..Self::any()
        }
    }

    pub fn at_position(position: Vec3Model) -> Self {
        Self {
            position: Some(position),
            ..Self::any()
        }
    }

    fn matches(
        &self,
        level: &ServerLevelModel,
        position: Option<Vec3Model>,
        entity: &HitEntityModel,
    ) -> bool {
        self.entity_type
            .as_ref()
            .is_none_or(|entity_type| entity_type == &entity.entity_type)
            && self
                .level_id
                .as_ref()
                .is_none_or(|level_id| level_id == &level.id && level_id == &entity.level_id)
            && self.position.is_none_or(|expected| {
                position == Some(expected) && entity.position == Some(expected)
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBoundsModel {
    pub const ANY: Self = Self {
        min: None,
        max: None,
    };

    pub fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    pub fn between(min: i32, max: i32) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| min <= value) && self.max.is_none_or(|max| max >= value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn level() -> ServerLevelModel {
        ServerLevelModel::new(id("minecraft:overworld"))
    }

    fn hit(entity_type: &str) -> HitEntityModel {
        HitEntityModel::new(
            id(entity_type),
            id("minecraft:overworld"),
            Some(Vec3Model::new(1, 2, 3)),
        )
    }

    fn lightning(blocks_set_on_fire: i32, hit_entities: Vec<HitEntityModel>) -> EntityModel {
        EntityModel::LightningBolt(LightningBoltModel::new(blocks_set_on_fire, hit_entities))
    }

    #[test]
    fn non_lightning_entities_never_match() {
        let predicate = LightningBoltPredicateModel::new(IntBoundsModel::ANY, None);

        assert!(!predicate.matches(
            &EntityModel::Generic {
                entity_type: id("minecraft:zombie")
            },
            &level(),
            None,
        ));
    }

    #[test]
    fn blocks_set_on_fire_bounds_are_checked_for_lightning_bolts() {
        let predicate =
            LightningBoltPredicateModel::block_set_on_fire(IntBoundsModel::between(2, 4));

        assert!(predicate.matches(&lightning(3, Vec::new()), &level(), None));
        assert!(!predicate.matches(&lightning(1, Vec::new()), &level(), None));
    }

    #[test]
    fn omitted_entity_struck_predicate_matches_after_fire_count() {
        let predicate = LightningBoltPredicateModel::new(IntBoundsModel::exactly(0), None);

        assert!(predicate.matches(&lightning(0, Vec::new()), &level(), None));
    }

    #[test]
    fn entity_struck_predicate_matches_any_hit_entity_with_level_and_position_context() {
        let predicate = LightningBoltPredicateModel::new(
            IntBoundsModel::ANY,
            Some(EntityPredicateModel {
                entity_type: Some(id("minecraft:creeper")),
                level_id: Some(id("minecraft:overworld")),
                position: Some(Vec3Model::new(1, 2, 3)),
            }),
        );

        assert!(predicate.matches(
            &lightning(0, vec![hit("minecraft:pig"), hit("minecraft:creeper")]),
            &level(),
            Some(Vec3Model::new(1, 2, 3)),
        ));
        assert!(!predicate.matches(
            &lightning(0, vec![hit("minecraft:pig")]),
            &level(),
            Some(Vec3Model::new(1, 2, 3)),
        ));
        assert!(!predicate.matches(
            &lightning(0, vec![hit("minecraft:creeper")]),
            &level(),
            Some(Vec3Model::new(9, 2, 3)),
        ));
    }

    #[test]
    fn codec_returns_registered_lightning_sub_predicate_type() {
        let predicate = LightningBoltPredicateModel::new(IntBoundsModel::ANY, None);

        assert_eq!(predicate.codec(), EntitySubPredicateTypeModel::Lightning);
        assert_eq!(predicate.codec().id(), "minecraft:lightning");
    }

    #[test]
    fn entity_predicate_helpers_preserve_expected_fields() {
        assert!(
            EntityPredicateModel::entity_type(id("minecraft:pig")).matches(
                &level(),
                None,
                &HitEntityModel::new(id("minecraft:pig"), id("minecraft:overworld"), None),
            )
        );
        assert!(
            EntityPredicateModel::in_level(id("minecraft:overworld")).matches(
                &level(),
                None,
                &HitEntityModel::new(id("minecraft:pig"), id("minecraft:overworld"), None),
            )
        );
        assert!(
            EntityPredicateModel::at_position(Vec3Model::new(1, 2, 3)).matches(
                &level(),
                Some(Vec3Model::new(1, 2, 3)),
                &hit("minecraft:pig"),
            )
        );
    }
}
