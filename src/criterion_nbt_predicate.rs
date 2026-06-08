use crate::storage::nbt::{parse_snbt, Tag};

pub const SELECTED_ITEM_TAG: &str = "SelectedItem";

#[derive(Debug, Clone, PartialEq)]
pub struct NbtPredicateModel {
    tag: Tag,
}

impl NbtPredicateModel {
    pub fn new(tag: Tag) -> Self {
        Self { tag }
    }

    pub fn from_snbt(input: &str) -> std::io::Result<Self> {
        parse_snbt(input).map(Self::new)
    }

    pub fn matches_components(&self, components: &DataComponentGetterModel) -> bool {
        self.matches_tag(Some(components.custom_data.tag()))
    }

    pub fn matches_entity(&self, entity: &EntityNbtModel) -> bool {
        let tag = get_entity_tag_to_compare(entity);
        self.matches_tag(Some(&tag))
    }

    pub fn matches_tag(&self, tag: Option<&Tag>) -> bool {
        tag.is_some_and(|tag| compare_nbt(&self.tag, tag, true))
    }

    pub fn stream_round_trip(&self) -> Self {
        Self::new(self.tag.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataComponentGetterModel {
    custom_data: CustomDataModel,
}

impl DataComponentGetterModel {
    pub fn new(custom_data: Option<CustomDataModel>) -> Self {
        Self {
            custom_data: custom_data.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomDataModel {
    tag: Tag,
}

impl CustomDataModel {
    pub fn new(tag: Tag) -> Self {
        Self { tag }
    }

    fn tag(&self) -> &Tag {
        &self.tag
    }
}

impl Default for CustomDataModel {
    fn default() -> Self {
        Self {
            tag: Tag::Compound(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityNbtModel {
    saved_without_id: Tag,
    player_selected_item: Option<Tag>,
}

impl EntityNbtModel {
    pub fn entity(saved_without_id: Tag) -> Self {
        Self {
            saved_without_id,
            player_selected_item: None,
        }
    }

    pub fn player(saved_without_id: Tag, selected_item: Option<Tag>) -> Self {
        Self {
            saved_without_id,
            player_selected_item: selected_item,
        }
    }
}

pub fn get_entity_tag_to_compare(entity: &EntityNbtModel) -> Tag {
    let mut tag = entity.saved_without_id.clone();
    if let Some(selected_item) = &entity.player_selected_item {
        set_compound_field(&mut tag, SELECTED_ITEM_TAG, selected_item.clone());
    }
    tag
}

fn compare_nbt(expected: &Tag, actual: &Tag, partial_list_matches: bool) -> bool {
    if std::ptr::eq(expected, actual) {
        return true;
    }

    if std::mem::discriminant(expected) != std::mem::discriminant(actual) {
        return false;
    }

    match (expected, actual) {
        (Tag::Compound(expected_values), Tag::Compound(actual_values)) => {
            if actual_values.len() < expected_values.len() {
                return false;
            }

            expected_values.iter().all(|(name, expected_child)| {
                compound_field(actual_values, name).is_some_and(|actual_child| {
                    compare_nbt(expected_child, actual_child, partial_list_matches)
                })
            })
        }
        (Tag::List(expected_values), Tag::List(actual_values)) if partial_list_matches => {
            if expected_values.is_empty() {
                return actual_values.is_empty();
            }

            actual_values.len() >= expected_values.len()
                && expected_values.iter().all(|expected_child| {
                    actual_values.iter().any(|actual_child| {
                        compare_nbt(expected_child, actual_child, partial_list_matches)
                    })
                })
        }
        _ => expected == actual,
    }
}

fn compound_field<'a>(values: &'a [(String, Tag)], name: &str) -> Option<&'a Tag> {
    values
        .iter()
        .find_map(|(field_name, value)| (field_name == name).then_some(value))
}

fn set_compound_field(tag: &mut Tag, name: &str, value: Tag) {
    let Tag::Compound(values) = tag else {
        return;
    };

    if let Some((_, existing)) = values.iter_mut().find(|(field_name, _)| field_name == name) {
        *existing = value;
    } else {
        values.push((name.to_string(), value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(input: &str) -> Tag {
        parse_snbt(input).unwrap()
    }

    #[test]
    fn parsed_codec_and_stream_codec_preserve_compound_tag() {
        let predicate = NbtPredicateModel::from_snbt("{Health:20,Tags:[\"a\"]}").unwrap();

        assert_eq!(predicate.stream_round_trip(), predicate);
        assert!(predicate.matches_tag(Some(&tag("{Health:20,Tags:[\"a\"]}"))));
    }

    #[test]
    fn matches_tag_rejects_null_and_uses_partial_compound_matching() {
        let predicate = NbtPredicateModel::new(tag("{Health:20,CustomName:\"Alex\"}"));

        assert!(!predicate.matches_tag(None));
        assert!(predicate.matches_tag(Some(&tag("{Health:20,CustomName:\"Alex\",Air:300}"))));
        assert!(!predicate.matches_tag(Some(&tag("{Health:20}"))));
        assert!(!predicate.matches_tag(Some(&tag("{Health:19,CustomName:\"Alex\"}"))));
    }

    #[test]
    fn compare_nbt_uses_java_partial_unordered_list_matching() {
        let predicate =
            NbtPredicateModel::new(tag("{Inventory:[{Slot:1b,id:\"minecraft:stone\"}]}"));

        assert!(predicate.matches_tag(Some(&tag(
            "{Inventory:[{Slot:2b,id:\"minecraft:dirt\"},{Slot:1b,id:\"minecraft:stone\",Count:64b}]}"
        ))));
        assert!(!predicate.matches_tag(Some(&tag("{Inventory:[{Slot:2b,id:\"minecraft:dirt\"}]}"))));
        assert!(
            NbtPredicateModel::new(tag("{Inventory:[]}")).matches_tag(Some(&tag("{Inventory:[]}")))
        );
        assert!(!NbtPredicateModel::new(tag("{Inventory:[]}"))
            .matches_tag(Some(&tag("{Inventory:[{Slot:0b}]}"))));
    }

    #[test]
    fn matches_components_reads_custom_data_or_empty_default() {
        let empty_predicate = NbtPredicateModel::new(tag("{}"));
        let named_predicate = NbtPredicateModel::new(tag("{display:{Name:\"Sword\"}}"));
        let components = DataComponentGetterModel::new(Some(CustomDataModel::new(tag(
            "{display:{Name:\"Sword\",Lore:[\"Sharp\"]}}",
        ))));

        assert!(empty_predicate.matches_components(&DataComponentGetterModel::new(None)));
        assert!(named_predicate.matches_components(&components));
        assert!(!named_predicate.matches_components(&DataComponentGetterModel::new(None)));
    }

    #[test]
    fn entity_tag_to_compare_uses_save_without_id_data() {
        let predicate = NbtPredicateModel::new(tag("{Health:20f,Air:300s}"));
        let entity = EntityNbtModel::entity(tag("{Health:20f,Air:300s,id:\"minecraft:zombie\"}"));

        assert!(predicate.matches_entity(&entity));
    }

    #[test]
    fn player_entity_tag_includes_non_empty_selected_item() {
        let predicate =
            NbtPredicateModel::new(tag("{SelectedItem:{id:\"minecraft:diamond_sword\"}}"));
        let player = EntityNbtModel::player(
            tag("{Health:20f}"),
            Some(tag("{id:\"minecraft:diamond_sword\",Count:1b}")),
        );
        let empty_hand = EntityNbtModel::player(tag("{Health:20f}"), None);

        assert!(predicate.matches_entity(&player));
        assert!(!predicate.matches_entity(&empty_hand));
        assert_eq!(
            compound_field(
                match &get_entity_tag_to_compare(&player) {
                    Tag::Compound(values) => values,
                    _ => unreachable!(),
                },
                SELECTED_ITEM_TAG,
            ),
            Some(&tag("{id:\"minecraft:diamond_sword\",Count:1b}"))
        );
    }
}
