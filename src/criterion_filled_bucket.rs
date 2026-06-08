use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilledBucketTriggerInstance {
    pub player_predicate_present: bool,
    pub item: Option<ItemPredicateModel>,
}

impl FilledBucketTriggerInstance {
    pub fn new(item: Option<ItemPredicateModel>) -> Self {
        Self {
            player_predicate_present: false,
            item,
        }
    }

    pub fn matches(&self, item: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|predicate| predicate.test(item))
    }

    pub fn filled_bucket(item: ItemPredicateBuilder) -> FilledBucketCriterion {
        FilledBucketCriterion {
            trigger_id: trigger_id(),
            instance: Self::new(Some(item.build())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilledBucketCriterion {
    pub trigger_id: Identifier,
    pub instance: FilledBucketTriggerInstance,
}

fn trigger_id() -> Identifier {
    Identifier::parse("minecraft:filled_bucket").unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
}

impl ItemStackModel {
    pub fn new(item: Identifier) -> Self {
        Self { item }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    item: Option<Identifier>,
}

impl ItemPredicateModel {
    pub fn test(&self, item_stack: &ItemStackModel) -> bool {
        self.item
            .as_ref()
            .is_none_or(|item| item == &item_stack.item)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemPredicateBuilder {
    item: Option<Identifier>,
}

impl ItemPredicateBuilder {
    pub fn item() -> Self {
        Self::default()
    }

    pub fn of(mut self, item: Identifier) -> Self {
        self.item = Some(item);
        self
    }

    pub fn build(self) -> ItemPredicateModel {
        ItemPredicateModel { item: self.item }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn stack(item: &str) -> ItemStackModel {
        ItemStackModel::new(id(item))
    }

    #[test]
    fn omitted_item_predicate_matches_any_filled_bucket_event() {
        let instance = FilledBucketTriggerInstance::new(None);

        assert!(instance.matches(&stack("minecraft:water_bucket")));
        assert!(instance.matches(&stack("minecraft:lava_bucket")));
    }

    #[test]
    fn item_predicate_filters_filled_bucket_stack() {
        let instance = FilledBucketTriggerInstance::new(Some(
            ItemPredicateBuilder::item()
                .of(id("minecraft:water_bucket"))
                .build(),
        ));

        assert!(instance.matches(&stack("minecraft:water_bucket")));
        assert!(!instance.matches(&stack("minecraft:lava_bucket")));
    }

    #[test]
    fn factory_uses_java_trigger_id_and_builds_item_predicate() {
        let criterion = FilledBucketTriggerInstance::filled_bucket(
            ItemPredicateBuilder::item().of(id("minecraft:powder_snow_bucket")),
        );

        assert_eq!(criterion.trigger_id, id("minecraft:filled_bucket"));
        assert!(!criterion.instance.player_predicate_present);
        assert!(criterion
            .instance
            .matches(&stack("minecraft:powder_snow_bucket")));
        assert!(!criterion.instance.matches(&stack("minecraft:water_bucket")));
    }
}
