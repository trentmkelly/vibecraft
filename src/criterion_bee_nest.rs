use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeeNestDestroyedTriggerInput {
    pub block_state: Identifier,
    pub item_stack: ItemStackModel,
    pub num_bees_inside: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    pub item: Identifier,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeeNestDestroyedTriggerInstance {
    pub player_predicate_present: bool,
    pub block: Option<Identifier>,
    pub item: Option<ItemPredicateModel>,
    pub bees_inside: IntBoundsModel,
}

impl BeeNestDestroyedTriggerInstance {
    pub fn new(
        block: Option<Identifier>,
        item: Option<ItemPredicateModel>,
        bees_inside: IntBoundsModel,
    ) -> Self {
        Self {
            player_predicate_present: false,
            block,
            item,
            bees_inside,
        }
    }

    pub fn destroyed_bee_nest(
        block: Identifier,
        item_predicate: ItemPredicateModel,
        num_bees_inside: IntBoundsModel,
    ) -> BeeNestDestroyedCriterion {
        BeeNestDestroyedCriterion {
            trigger_id: Identifier::parse("minecraft:bee_nest_destroyed").unwrap(),
            instance: Self::new(Some(block), Some(item_predicate), num_bees_inside),
        }
    }

    pub fn matches(&self, input: &BeeNestDestroyedTriggerInput) -> bool {
        if self
            .block
            .as_ref()
            .is_some_and(|block| block != &input.block_state)
        {
            return false;
        }

        if self
            .item
            .as_ref()
            .is_some_and(|item| !item.test(&input.item_stack))
        {
            return false;
        }

        self.bees_inside.matches(input.num_bees_inside)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeeNestDestroyedCriterion {
    pub trigger_id: Identifier,
    pub instance: BeeNestDestroyedTriggerInstance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    pub items: Option<Vec<Identifier>>,
    pub count: IntBoundsModel,
}

impl ItemPredicateModel {
    pub fn any() -> Self {
        Self {
            items: None,
            count: IntBoundsModel::any(),
        }
    }

    pub fn item(item: Identifier) -> Self {
        Self {
            items: Some(vec![item]),
            count: IntBoundsModel::any(),
        }
    }

    pub fn with_count(mut self, count: IntBoundsModel) -> Self {
        self.count = count;
        self
    }

    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        self.items
            .as_ref()
            .is_none_or(|items| items.contains(&item_stack.item))
            && self.count.matches(item_stack.count)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntBoundsModel {
    pub min: Option<i32>,
    pub max: Option<i32>,
}

impl IntBoundsModel {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
        }
    }

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

    pub fn at_least(value: i32) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    pub fn matches(&self, value: i32) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn input(block: &str, item: &str, count: i32, bees: i32) -> BeeNestDestroyedTriggerInput {
        BeeNestDestroyedTriggerInput {
            block_state: id(block),
            item_stack: ItemStackModel {
                item: id(item),
                count,
            },
            num_bees_inside: bees,
        }
    }

    #[test]
    fn bee_nest_destroyed_matches_optional_block_item_and_bee_bounds_like_java() {
        let instance = BeeNestDestroyedTriggerInstance::new(
            Some(id("minecraft:bee_nest")),
            Some(
                ItemPredicateModel::item(id("minecraft:shears"))
                    .with_count(IntBoundsModel::exactly(1)),
            ),
            IntBoundsModel::between(2, 3),
        );

        assert!(instance.matches(&input("minecraft:bee_nest", "minecraft:shears", 1, 2)));
        assert!(instance.matches(&input("minecraft:bee_nest", "minecraft:shears", 1, 3)));
        assert!(!instance.matches(&input("minecraft:beehive", "minecraft:shears", 1, 2)));
        assert!(!instance.matches(&input("minecraft:bee_nest", "minecraft:stick", 1, 2)));
        assert!(!instance.matches(&input("minecraft:bee_nest", "minecraft:shears", 2, 2)));
        assert!(!instance.matches(&input("minecraft:bee_nest", "minecraft:shears", 1, 4)));
    }

    #[test]
    fn bee_nest_destroyed_omitted_predicates_default_to_any_like_java_codec() {
        let instance = BeeNestDestroyedTriggerInstance::new(None, None, IntBoundsModel::any());

        assert!(instance.matches(&input("minecraft:beehive", "minecraft:stick", 64, 0)));
        assert!(instance.matches(&input("minecraft:bee_nest", "minecraft:diamond", 1, 12)));

        let explicit_any_item = BeeNestDestroyedTriggerInstance::new(
            None,
            Some(ItemPredicateModel::any()),
            IntBoundsModel::any(),
        );
        assert!(explicit_any_item.matches(&input("minecraft:bee_nest", "minecraft:diamond", 1, 0,)));
    }

    #[test]
    fn bee_nest_destroyed_factory_uses_java_trigger_id_and_fields() {
        let criterion = BeeNestDestroyedTriggerInstance::destroyed_bee_nest(
            id("minecraft:bee_nest"),
            ItemPredicateModel::item(id("minecraft:shears")),
            IntBoundsModel::at_least(3),
        );

        assert_eq!(
            criterion.trigger_id,
            Identifier::parse("minecraft:bee_nest_destroyed").unwrap()
        );
        assert_eq!(criterion.instance.block, Some(id("minecraft:bee_nest")));
        assert!(criterion
            .instance
            .matches(&input("minecraft:bee_nest", "minecraft:shears", 1, 3)));
        assert!(!criterion.instance.matches(&input(
            "minecraft:bee_nest",
            "minecraft:shears",
            1,
            2
        )));
    }
}
