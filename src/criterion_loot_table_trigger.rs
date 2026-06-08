use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootTableTriggerInstanceModel {
    pub player_predicate_present: bool,
    pub loot_table: ResourceKeyModel,
}

impl LootTableTriggerInstanceModel {
    pub fn new(player_predicate_present: bool, loot_table: ResourceKeyModel) -> Self {
        Self {
            player_predicate_present,
            loot_table,
        }
    }

    pub fn loot_table_used(loot_table: ResourceKeyModel) -> LootTableCriterionModel {
        LootTableCriterionModel {
            trigger_id: trigger_id(),
            instance: Self::new(false, loot_table),
        }
    }

    pub fn matches(&self, loot_table: &ResourceKeyModel) -> bool {
        &self.loot_table == loot_table
    }

    pub fn codec_field_names() -> [&'static str; 2] {
        ["player", "loot_table"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootTableCriterionModel {
    pub trigger_id: Identifier,
    pub instance: LootTableTriggerInstanceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LootTableTriggerModel {
    listeners: Vec<LootTableTriggerInstanceModel>,
}

impl LootTableTriggerModel {
    pub fn new(listeners: Vec<LootTableTriggerInstanceModel>) -> Self {
        Self { listeners }
    }

    pub fn trigger(&self, loot_table: &ResourceKeyModel) -> Vec<&LootTableTriggerInstanceModel> {
        self.listeners
            .iter()
            .filter(|instance| instance.matches(loot_table))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceKeyModel {
    registry: Identifier,
    location: Identifier,
}

impl ResourceKeyModel {
    pub fn loot_table(location: Identifier) -> Self {
        Self {
            registry: id("minecraft:loot_table"),
            location,
        }
    }
}

fn trigger_id() -> Identifier {
    id("minecraft:player_generates_container_loot")
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loot_table(path: &str) -> ResourceKeyModel {
        ResourceKeyModel::loot_table(id(path))
    }

    #[test]
    fn matches_only_the_same_loot_table_resource_key() {
        let simple_dungeon = loot_table("minecraft:chests/simple_dungeon");
        let instance = LootTableTriggerInstanceModel::new(true, simple_dungeon.clone());

        assert!(instance.matches(&simple_dungeon));
        assert!(!instance.matches(&loot_table("minecraft:gameplay/fishing")));
    }

    #[test]
    fn trigger_forwards_generated_loot_table_key_to_instances() {
        let matching =
            LootTableTriggerInstanceModel::new(true, loot_table("minecraft:chests/simple_dungeon"));
        let non_matching =
            LootTableTriggerInstanceModel::new(true, loot_table("minecraft:gameplay/fishing"));
        let trigger = LootTableTriggerModel::new(vec![matching.clone(), non_matching]);

        assert_eq!(
            trigger.trigger(&loot_table("minecraft:chests/simple_dungeon")),
            vec![&matching]
        );
        assert!(trigger
            .trigger(&loot_table("minecraft:chests/abandoned_mineshaft"))
            .is_empty());
    }

    #[test]
    fn loot_table_used_factory_matches_java_trigger_shape() {
        let table = loot_table("minecraft:chests/trial_chambers/reward");
        let criterion = LootTableTriggerInstanceModel::loot_table_used(table.clone());

        assert_eq!(
            criterion.trigger_id,
            id("minecraft:player_generates_container_loot")
        );
        assert!(!criterion.instance.player_predicate_present);
        assert_eq!(criterion.instance.loot_table, table);
    }

    #[test]
    fn codec_field_names_match_java_record_codec() {
        assert_eq!(
            LootTableTriggerInstanceModel::codec_field_names(),
            ["player", "loot_table"]
        );
    }
}
