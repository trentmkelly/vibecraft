use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnchantmentPredicateModel {
    pub enchantments: Option<Vec<Identifier>>,
    pub level: IntBoundsModel,
}

impl EnchantmentPredicateModel {
    pub fn new(enchantments: Option<Vec<Identifier>>, level: IntBoundsModel) -> Self {
        Self {
            enchantments,
            level,
        }
    }

    pub fn enchantment(enchantment: Identifier, level: IntBoundsModel) -> Self {
        Self::new(Some(vec![enchantment]), level)
    }

    pub fn enchantment_set(enchantments: Vec<Identifier>, level: IntBoundsModel) -> Self {
        Self::new(Some(enchantments), level)
    }

    pub fn contained_in(&self, item_enchantments: &ItemEnchantmentsModel) -> bool {
        if let Some(enchantments) = &self.enchantments {
            for enchantment in enchantments {
                if self.matches_enchantment(item_enchantments, enchantment) {
                    return true;
                }
            }

            return false;
        }

        if !self.level.is_any() {
            for level in item_enchantments.enchantments.values() {
                if self.level.matches(*level) {
                    return true;
                }
            }

            return false;
        }

        !item_enchantments.is_empty()
    }

    fn matches_enchantment(
        &self,
        item_enchantments: &ItemEnchantmentsModel,
        enchantment: &Identifier,
    ) -> bool {
        let level = item_enchantments.get_level(enchantment);
        if level == 0 {
            return false;
        }

        self.level.is_any() || self.level.matches(level)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemEnchantmentsModel {
    enchantments: BTreeMap<Identifier, i32>,
}

impl ItemEnchantmentsModel {
    pub fn with(mut self, enchantment: Identifier, level: i32) -> Self {
        self.enchantments.insert(enchantment, level);
        self
    }

    pub fn get_level(&self, enchantment: &Identifier) -> i32 {
        self.enchantments.get(enchantment).copied().unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.enchantments.is_empty()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct IntBoundsModel {
    min: Option<i32>,
    max: Option<i32>,
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

    pub fn is_any(&self) -> bool {
        self.min.is_none() && self.max.is_none()
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

    fn enchantments() -> ItemEnchantmentsModel {
        ItemEnchantmentsModel::default()
            .with(id("minecraft:sharpness"), 3)
            .with(id("minecraft:unbreaking"), 1)
    }

    #[test]
    fn specific_enchantment_set_matches_any_listed_enchantment_with_required_level() {
        let predicate = EnchantmentPredicateModel::enchantment_set(
            vec![id("minecraft:smite"), id("minecraft:sharpness")],
            IntBoundsModel::between(2, 4),
        );

        assert!(predicate.contained_in(&enchantments()));
    }

    #[test]
    fn specific_enchantment_set_fails_when_missing_or_level_out_of_range() {
        let missing = EnchantmentPredicateModel::enchantment(
            id("minecraft:efficiency"),
            IntBoundsModel::any(),
        );
        let wrong_level = EnchantmentPredicateModel::enchantment(
            id("minecraft:sharpness"),
            IntBoundsModel::exactly(5),
        );

        assert!(!missing.contained_in(&enchantments()));
        assert!(!wrong_level.contained_in(&enchantments()));
    }

    #[test]
    fn level_only_predicate_scans_all_item_enchantments() {
        let predicate = EnchantmentPredicateModel::new(None, IntBoundsModel::at_least(3));

        assert!(predicate.contained_in(&enchantments()));
        assert!(!predicate
            .contained_in(&ItemEnchantmentsModel::default().with(id("minecraft:unbreaking"), 1,)));
    }

    #[test]
    fn omitted_enchantments_and_any_level_require_at_least_one_enchantment() {
        let predicate = EnchantmentPredicateModel::new(None, IntBoundsModel::any());

        assert!(predicate.contained_in(&enchantments()));
        assert!(!predicate.contained_in(&ItemEnchantmentsModel::default()));
    }

    #[test]
    fn zero_level_counts_as_absent_for_specific_enchantment_matches() {
        let item_enchantments = ItemEnchantmentsModel::default().with(id("minecraft:sharpness"), 0);
        let predicate = EnchantmentPredicateModel::enchantment(
            id("minecraft:sharpness"),
            IntBoundsModel::any(),
        );

        assert!(!predicate.contained_in(&item_enchantments));
    }

    #[test]
    fn constructor_shapes_preserve_java_fields() {
        let direct = EnchantmentPredicateModel::enchantment(
            id("minecraft:fortune"),
            IntBoundsModel::exactly(2),
        );
        assert_eq!(direct.enchantments, Some(vec![id("minecraft:fortune")]));
        assert_eq!(direct.level, IntBoundsModel::exactly(2));

        let set = EnchantmentPredicateModel::enchantment_set(
            vec![id("minecraft:fortune"), id("minecraft:silk_touch")],
            IntBoundsModel::any(),
        );
        assert_eq!(
            set.enchantments,
            Some(vec![id("minecraft:fortune"), id("minecraft:silk_touch")])
        );
    }
}
