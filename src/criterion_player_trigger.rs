use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTriggerInstanceModel {
    pub player: Option<ContextAwarePredicateModel>,
}

impl PlayerTriggerInstanceModel {
    pub fn new(player: Option<ContextAwarePredicateModel>) -> Self {
        Self { player }
    }

    pub fn located(location: LocationPredicateBuilderModel) -> PlayerCriterionModel {
        Self::located_player(EntityPredicateBuilderModel::entity().located(location))
    }

    pub fn located_player(player: EntityPredicateBuilderModel) -> PlayerCriterionModel {
        PlayerCriterionModel {
            trigger_id: id("minecraft:location"),
            instance: Self::new(Some(ContextAwarePredicateModel::wrap(player.build()))),
        }
    }

    pub fn located_optional(player: Option<EntityPredicateModel>) -> PlayerCriterionModel {
        PlayerCriterionModel {
            trigger_id: id("minecraft:location"),
            instance: Self::new(EntityPredicateModel::wrap(player)),
        }
    }

    pub fn slept_in_bed() -> PlayerCriterionModel {
        Self::empty(id("minecraft:slept_in_bed"))
    }

    pub fn raid_won() -> PlayerCriterionModel {
        Self::empty(id("minecraft:hero_of_the_village"))
    }

    pub fn avoid_vibration() -> PlayerCriterionModel {
        Self::empty(id("minecraft:avoid_vibration"))
    }

    pub fn tick() -> PlayerCriterionModel {
        Self::empty(id("minecraft:tick"))
    }

    pub fn walk_on_block_with_equipment(
        step_on_block: Identifier,
        required_equipment: Identifier,
    ) -> PlayerCriterionModel {
        Self::located_player(
            EntityPredicateBuilderModel::entity()
                .equipment(
                    EntityEquipmentPredicateBuilderModel::equipment()
                        .feet(ItemPredicateBuilderModel::item().of(required_equipment)),
                )
                .stepping_on(
                    LocationPredicateBuilderModel::location()
                        .set_block(BlockPredicateBuilderModel::block().of(step_on_block)),
                ),
        )
    }

    fn empty(trigger_id: Identifier) -> PlayerCriterionModel {
        PlayerCriterionModel {
            trigger_id,
            instance: Self::new(None),
        }
    }

    pub fn codec_field_names() -> [&'static str; 1] {
        ["player"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerCriterionModel {
    pub trigger_id: Identifier,
    pub instance: PlayerTriggerInstanceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerTriggerModel {
    listeners: Vec<PlayerTriggerInstanceModel>,
}

impl PlayerTriggerModel {
    pub fn new(listeners: Vec<PlayerTriggerInstanceModel>) -> Self {
        Self { listeners }
    }

    pub fn trigger(&self, _player: &ServerPlayerModel) -> Vec<&PlayerTriggerInstanceModel> {
        self.listeners.iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerModel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextAwarePredicateModel {
    entity: EntityPredicateModel,
}

impl ContextAwarePredicateModel {
    fn wrap(entity: EntityPredicateModel) -> Self {
        Self { entity }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityPredicateModel {
    located: Option<LocationPredicateModel>,
    stepping_on: Option<LocationPredicateModel>,
    equipment: Option<EntityEquipmentPredicateModel>,
}

impl EntityPredicateModel {
    fn wrap(predicate: Option<Self>) -> Option<ContextAwarePredicateModel> {
        predicate.map(ContextAwarePredicateModel::wrap)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityPredicateBuilderModel {
    located: Option<LocationPredicateModel>,
    stepping_on: Option<LocationPredicateModel>,
    equipment: Option<EntityEquipmentPredicateModel>,
}

impl EntityPredicateBuilderModel {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn located(mut self, location: LocationPredicateBuilderModel) -> Self {
        self.located = Some(location.build());
        self
    }

    pub fn stepping_on(mut self, location: LocationPredicateBuilderModel) -> Self {
        self.stepping_on = Some(location.build());
        self
    }

    pub fn equipment(mut self, equipment: EntityEquipmentPredicateBuilderModel) -> Self {
        self.equipment = Some(equipment.build());
        self
    }

    fn build(self) -> EntityPredicateModel {
        EntityPredicateModel {
            located: self.located,
            stepping_on: self.stepping_on,
            equipment: self.equipment,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocationPredicateModel {
    block: Option<BlockPredicateModel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LocationPredicateBuilderModel {
    block: Option<BlockPredicateModel>,
}

impl LocationPredicateBuilderModel {
    pub fn location() -> Self {
        Self::default()
    }

    pub fn set_block(mut self, block: BlockPredicateBuilderModel) -> Self {
        self.block = Some(block.build());
        self
    }

    fn build(self) -> LocationPredicateModel {
        LocationPredicateModel { block: self.block }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicateModel {
    block: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicateBuilderModel {
    block: Option<Identifier>,
}

impl BlockPredicateBuilderModel {
    pub fn block() -> Self {
        Self { block: None }
    }

    pub fn of(mut self, block: Identifier) -> Self {
        self.block = Some(block);
        self
    }

    fn build(self) -> BlockPredicateModel {
        BlockPredicateModel {
            block: self
                .block
                .expect("block predicate test model requires block"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityEquipmentPredicateModel {
    feet: Option<ItemPredicateModel>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityEquipmentPredicateBuilderModel {
    feet: Option<ItemPredicateModel>,
}

impl EntityEquipmentPredicateBuilderModel {
    pub fn equipment() -> Self {
        Self::default()
    }

    pub fn feet(mut self, item: ItemPredicateBuilderModel) -> Self {
        self.feet = Some(item.build());
        self
    }

    fn build(self) -> EntityEquipmentPredicateModel {
        EntityEquipmentPredicateModel { feet: self.feet }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    item: Identifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateBuilderModel {
    item: Option<Identifier>,
}

impl ItemPredicateBuilderModel {
    pub fn item() -> Self {
        Self { item: None }
    }

    pub fn of(mut self, item: Identifier) -> Self {
        self.item = Some(item);
        self
    }

    fn build(self) -> ItemPredicateModel {
        ItemPredicateModel {
            item: self.item.expect("item predicate test model requires item"),
        }
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codec_field_name_matches_java_record_codec() {
        assert_eq!(PlayerTriggerInstanceModel::codec_field_names(), ["player"]);
    }

    #[test]
    fn trigger_invokes_every_listener_with_true_predicate() {
        let first = PlayerTriggerInstanceModel::new(None);
        let second = PlayerTriggerInstanceModel::new(Some(ContextAwarePredicateModel::wrap(
            EntityPredicateBuilderModel::entity().build(),
        )));
        let trigger = PlayerTriggerModel::new(vec![first.clone(), second.clone()]);

        assert_eq!(trigger.trigger(&ServerPlayerModel), vec![&first, &second]);
    }

    #[test]
    fn location_factories_wrap_player_predicates_and_use_location_trigger_id() {
        let location = PlayerTriggerInstanceModel::located(
            LocationPredicateBuilderModel::location()
                .set_block(BlockPredicateBuilderModel::block().of(id("minecraft:lodestone"))),
        );
        let builder =
            PlayerTriggerInstanceModel::located_player(
                EntityPredicateBuilderModel::entity()
                    .located(LocationPredicateBuilderModel::location().set_block(
                        BlockPredicateBuilderModel::block().of(id("minecraft:emerald_block")),
                    ))
                    .stepping_on(LocationPredicateBuilderModel::location().set_block(
                        BlockPredicateBuilderModel::block().of(id("minecraft:gold_block")),
                    )),
            );
        let optional = PlayerTriggerInstanceModel::located_optional(Some(
            EntityPredicateBuilderModel::entity().build(),
        ));
        let empty_optional = PlayerTriggerInstanceModel::located_optional(None);

        assert_eq!(location.trigger_id, id("minecraft:location"));
        assert_eq!(
            location
                .instance
                .player
                .as_ref()
                .and_then(|player| player.entity.located.as_ref())
                .and_then(|located| located.block.as_ref())
                .map(|block| &block.block),
            Some(&id("minecraft:lodestone"))
        );
        assert_eq!(builder.trigger_id, id("minecraft:location"));
        let builder_player = &builder.instance.player.as_ref().unwrap().entity;
        assert_eq!(
            builder_player
                .located
                .as_ref()
                .and_then(|located| located.block.as_ref())
                .map(|block| &block.block),
            Some(&id("minecraft:emerald_block"))
        );
        assert_eq!(
            builder_player
                .stepping_on
                .as_ref()
                .and_then(|located| located.block.as_ref())
                .map(|block| &block.block),
            Some(&id("minecraft:gold_block"))
        );
        assert_eq!(optional.trigger_id, id("minecraft:location"));
        assert!(optional.instance.player.is_some());
        assert_eq!(empty_optional.trigger_id, id("minecraft:location"));
        assert!(empty_optional.instance.player.is_none());
    }

    #[test]
    fn simple_factories_use_java_trigger_ids_and_empty_player_predicates() {
        let slept = PlayerTriggerInstanceModel::slept_in_bed();
        let raid = PlayerTriggerInstanceModel::raid_won();
        let avoid = PlayerTriggerInstanceModel::avoid_vibration();
        let tick = PlayerTriggerInstanceModel::tick();

        assert_eq!(slept.trigger_id, id("minecraft:slept_in_bed"));
        assert_eq!(raid.trigger_id, id("minecraft:hero_of_the_village"));
        assert_eq!(avoid.trigger_id, id("minecraft:avoid_vibration"));
        assert_eq!(tick.trigger_id, id("minecraft:tick"));
        for criterion in [&slept, &raid, &avoid, &tick] {
            assert!(criterion.instance.player.is_none());
        }
    }

    #[test]
    fn walk_on_block_with_equipment_composes_feet_equipment_and_stepping_block_location() {
        let criterion = PlayerTriggerInstanceModel::walk_on_block_with_equipment(
            id("minecraft:powder_snow"),
            id("minecraft:leather_boots"),
        );
        let player = &criterion.instance.player.as_ref().unwrap().entity;

        assert_eq!(criterion.trigger_id, id("minecraft:location"));
        assert_eq!(
            player
                .equipment
                .as_ref()
                .and_then(|equipment| equipment.feet.as_ref())
                .map(|item| &item.item),
            Some(&id("minecraft:leather_boots"))
        );
        assert_eq!(
            player
                .stepping_on
                .as_ref()
                .and_then(|location| location.block.as_ref())
                .map(|block| &block.block),
            Some(&id("minecraft:powder_snow"))
        );
    }
}
