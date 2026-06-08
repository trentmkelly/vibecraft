use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityEquipmentPredicateModel {
    pub head: Option<ItemPredicateModel>,
    pub chest: Option<ItemPredicateModel>,
    pub legs: Option<ItemPredicateModel>,
    pub feet: Option<ItemPredicateModel>,
    pub body: Option<ItemPredicateModel>,
    pub mainhand: Option<ItemPredicateModel>,
    pub offhand: Option<ItemPredicateModel>,
}

impl EntityEquipmentPredicateModel {
    pub fn builder() -> EntityEquipmentPredicateBuilder {
        EntityEquipmentPredicateBuilder::equipment()
    }

    pub fn captain_predicate() -> Self {
        Self::builder()
            .head(
                ItemPredicateBuilder::item()
                    .of(Identifier::parse("minecraft:white_banner").unwrap())
                    .with_components(DataComponentMatchersModel::requiring_exact([
                        ("minecraft:banner_patterns", "ominous_banner_patterns"),
                        ("minecraft:item_name", "block.minecraft.ominous_banner"),
                    ])),
            )
            .build()
    }

    pub fn matches(&self, entity: Option<&EquipmentEntityModel>) -> bool {
        let Some(entity) = entity else {
            return false;
        };
        if !entity.living {
            return false;
        }

        if self.head.as_ref().is_some_and(|predicate| {
            !predicate.test(&entity.item_by_slot(EquipmentSlotModel::Head))
        }) {
            return false;
        }

        if self.chest.as_ref().is_some_and(|predicate| {
            !predicate.test(&entity.item_by_slot(EquipmentSlotModel::Chest))
        }) {
            return false;
        }

        if self.legs.as_ref().is_some_and(|predicate| {
            !predicate.test(&entity.item_by_slot(EquipmentSlotModel::Legs))
        }) {
            return false;
        }

        if self.feet.as_ref().is_some_and(|predicate| {
            !predicate.test(&entity.item_by_slot(EquipmentSlotModel::Feet))
        }) {
            return false;
        }

        if self.body.as_ref().is_some_and(|predicate| {
            !predicate.test(&entity.item_by_slot(EquipmentSlotModel::Body))
        }) {
            return false;
        }

        if self.mainhand.as_ref().is_some_and(|predicate| {
            !predicate.test(&entity.item_by_slot(EquipmentSlotModel::Mainhand))
        }) {
            return false;
        }

        self.offhand.as_ref().is_none_or(|predicate| {
            predicate.test(&entity.item_by_slot(EquipmentSlotModel::Offhand))
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntityEquipmentPredicateBuilder {
    head: Option<ItemPredicateModel>,
    chest: Option<ItemPredicateModel>,
    legs: Option<ItemPredicateModel>,
    feet: Option<ItemPredicateModel>,
    body: Option<ItemPredicateModel>,
    mainhand: Option<ItemPredicateModel>,
    offhand: Option<ItemPredicateModel>,
}

impl EntityEquipmentPredicateBuilder {
    pub fn equipment() -> Self {
        Self::default()
    }

    pub fn head(mut self, head: ItemPredicateBuilder) -> Self {
        self.head = Some(head.build());
        self
    }

    pub fn chest(mut self, chest: ItemPredicateBuilder) -> Self {
        self.chest = Some(chest.build());
        self
    }

    pub fn legs(mut self, legs: ItemPredicateBuilder) -> Self {
        self.legs = Some(legs.build());
        self
    }

    pub fn feet(mut self, feet: ItemPredicateBuilder) -> Self {
        self.feet = Some(feet.build());
        self
    }

    pub fn body(mut self, body: ItemPredicateBuilder) -> Self {
        self.body = Some(body.build());
        self
    }

    pub fn mainhand(mut self, mainhand: ItemPredicateBuilder) -> Self {
        self.mainhand = Some(mainhand.build());
        self
    }

    pub fn offhand(mut self, offhand: ItemPredicateBuilder) -> Self {
        self.offhand = Some(offhand.build());
        self
    }

    pub fn build(self) -> EntityEquipmentPredicateModel {
        EntityEquipmentPredicateModel {
            head: self.head,
            chest: self.chest,
            legs: self.legs,
            feet: self.feet,
            body: self.body,
            mainhand: self.mainhand,
            offhand: self.offhand,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquipmentSlotModel {
    Head,
    Chest,
    Legs,
    Feet,
    Body,
    Mainhand,
    Offhand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquipmentEntityModel {
    living: bool,
    equipment: BTreeMap<EquipmentSlotModel, ItemStackModel>,
}

impl EquipmentEntityModel {
    pub fn living() -> Self {
        Self {
            living: true,
            equipment: BTreeMap::new(),
        }
    }

    pub fn non_living() -> Self {
        Self {
            living: false,
            equipment: BTreeMap::new(),
        }
    }

    pub fn with_slot(mut self, slot: EquipmentSlotModel, stack: ItemStackModel) -> Self {
        self.equipment.insert(slot, stack);
        self
    }

    pub fn item_by_slot(&self, slot: EquipmentSlotModel) -> ItemStackModel {
        self.equipment
            .get(&slot)
            .cloned()
            .unwrap_or_else(|| ItemStackModel::new(Identifier::parse("minecraft:air").unwrap()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item: Identifier,
    components: BTreeMap<Identifier, String>,
}

impl ItemStackModel {
    pub fn new(item: Identifier) -> Self {
        Self {
            item,
            components: BTreeMap::new(),
        }
    }

    pub fn with_component(mut self, component: Identifier, value: impl Into<String>) -> Self {
        self.components.insert(component, value.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateModel {
    items: Option<Vec<Identifier>>,
    components: DataComponentMatchersModel,
}

impl ItemPredicateModel {
    pub fn test(&self, stack: &ItemStackModel) -> bool {
        if self
            .items
            .as_ref()
            .is_some_and(|items| !items.contains(&stack.item))
        {
            return false;
        }

        self.components.test(&stack.components)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemPredicateBuilder {
    items: Option<Vec<Identifier>>,
    components: DataComponentMatchersModel,
}

impl ItemPredicateBuilder {
    pub fn item() -> Self {
        Self::default()
    }

    pub fn of(mut self, item: Identifier) -> Self {
        self.items = Some(vec![item]);
        self
    }

    pub fn with_components(mut self, components: DataComponentMatchersModel) -> Self {
        self.components = components;
        self
    }

    pub fn build(self) -> ItemPredicateModel {
        ItemPredicateModel {
            items: self.items,
            components: self.components,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentMatchersModel {
    exact: BTreeMap<Identifier, String>,
}

impl DataComponentMatchersModel {
    pub fn requiring_exact<const N: usize>(components: [(&str, &str); N]) -> Self {
        Self {
            exact: components
                .into_iter()
                .map(|(component, value)| {
                    (Identifier::parse(component).unwrap(), value.to_string())
                })
                .collect(),
        }
    }

    pub fn test(&self, components: &BTreeMap<Identifier, String>) -> bool {
        self.exact
            .iter()
            .all(|(component, expected)| components.get(component) == Some(expected))
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

    fn fully_equipped_living() -> EquipmentEntityModel {
        EquipmentEntityModel::living()
            .with_slot(EquipmentSlotModel::Head, stack("minecraft:diamond_helmet"))
            .with_slot(
                EquipmentSlotModel::Chest,
                stack("minecraft:diamond_chestplate"),
            )
            .with_slot(
                EquipmentSlotModel::Legs,
                stack("minecraft:diamond_leggings"),
            )
            .with_slot(EquipmentSlotModel::Feet, stack("minecraft:diamond_boots"))
            .with_slot(EquipmentSlotModel::Body, stack("minecraft:wolf_armor"))
            .with_slot(
                EquipmentSlotModel::Mainhand,
                stack("minecraft:diamond_sword"),
            )
            .with_slot(EquipmentSlotModel::Offhand, stack("minecraft:shield"))
    }

    #[test]
    fn predicate_rejects_null_and_non_living_entities_like_java() {
        let predicate = EntityEquipmentPredicateModel::builder().build();

        assert!(!predicate.matches(None));
        assert!(!predicate.matches(Some(&EquipmentEntityModel::non_living())));
        assert!(predicate.matches(Some(&EquipmentEntityModel::living())));
    }

    #[test]
    fn all_slots_match_when_present() {
        let predicate = EntityEquipmentPredicateModel::builder()
            .head(ItemPredicateBuilder::item().of(id("minecraft:diamond_helmet")))
            .chest(ItemPredicateBuilder::item().of(id("minecraft:diamond_chestplate")))
            .legs(ItemPredicateBuilder::item().of(id("minecraft:diamond_leggings")))
            .feet(ItemPredicateBuilder::item().of(id("minecraft:diamond_boots")))
            .body(ItemPredicateBuilder::item().of(id("minecraft:wolf_armor")))
            .mainhand(ItemPredicateBuilder::item().of(id("minecraft:diamond_sword")))
            .offhand(ItemPredicateBuilder::item().of(id("minecraft:shield")))
            .build();

        assert!(predicate.matches(Some(&fully_equipped_living())));
    }

    #[test]
    fn slots_are_checked_in_java_order() {
        let predicate = EntityEquipmentPredicateModel::builder()
            .head(ItemPredicateBuilder::item().of(id("minecraft:golden_helmet")))
            .chest(ItemPredicateBuilder::item().of(id("minecraft:diamond_chestplate")))
            .build();

        assert!(!predicate.matches(Some(&fully_equipped_living())));
    }

    #[test]
    fn missing_equipment_slot_behaves_like_empty_item_stack() {
        let predicate = EntityEquipmentPredicateModel::builder()
            .mainhand(ItemPredicateBuilder::item().of(id("minecraft:air")))
            .build();

        assert!(predicate.matches(Some(&EquipmentEntityModel::living())));
    }

    #[test]
    fn captain_predicate_requires_ominous_white_banner_on_head() {
        let predicate = EntityEquipmentPredicateModel::captain_predicate();
        let captain = EquipmentEntityModel::living().with_slot(
            EquipmentSlotModel::Head,
            stack("minecraft:white_banner")
                .with_component(id("minecraft:banner_patterns"), "ominous_banner_patterns")
                .with_component(id("minecraft:item_name"), "block.minecraft.ominous_banner")
                .with_component(id("minecraft:rarity"), "uncommon"),
        );
        let plain_banner = EquipmentEntityModel::living()
            .with_slot(EquipmentSlotModel::Head, stack("minecraft:white_banner"));
        let wrong_item = EquipmentEntityModel::living().with_slot(
            EquipmentSlotModel::Head,
            stack("minecraft:black_banner")
                .with_component(id("minecraft:banner_patterns"), "ominous_banner_patterns")
                .with_component(id("minecraft:item_name"), "block.minecraft.ominous_banner"),
        );

        assert!(predicate.matches(Some(&captain)));
        assert!(!predicate.matches(Some(&plain_banner)));
        assert!(!predicate.matches(Some(&wrong_item)));
        assert!(predicate.chest.is_none());
        assert!(predicate.head.is_some());
    }
}
