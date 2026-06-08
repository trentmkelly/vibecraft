use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionProviderModel {
    pub online_players: Vec<String>,
}

impl SuggestionProviderModel {
    pub fn get_custom_tab_suggestions(&self) -> Vec<String> {
        self.online_players.clone()
    }

    pub fn get_selected_entities(&self) -> Vec<String> {
        Vec::new()
    }

    pub fn get_relevant_coordinates(&self) -> Vec<TextCoordinates> {
        vec![TextCoordinates::default_global()]
    }

    pub fn get_absolute_coordinates(&self) -> Vec<TextCoordinates> {
        vec![TextCoordinates::default_global()]
    }

    pub fn suggest_registry_elements(
        &self,
        registry: &RegistryLookupModel,
        elements: ElementSuggestionType,
        builder: &mut SuggestionsBuilderModel,
    ) {
        if elements.should_suggest_tags() {
            suggest_resource_with_prefix(registry.tags.iter().cloned(), builder, "#");
        }
        if elements.should_suggest_elements() {
            suggest_resource(registry.elements.iter().cloned(), builder);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLookupModel {
    pub tags: Vec<Identifier>,
    pub elements: Vec<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionsBuilderModel {
    remaining: String,
    suggestions: Vec<SuggestionModel>,
}

impl SuggestionsBuilderModel {
    pub fn new(remaining: impl Into<String>) -> Self {
        Self {
            remaining: remaining.into(),
            suggestions: Vec::new(),
        }
    }

    pub fn remaining(&self) -> &str {
        &self.remaining
    }

    pub fn suggest(&mut self, value: impl Into<String>) {
        self.suggestions.push(SuggestionModel {
            value: value.into(),
            tooltip: None,
        });
    }

    pub fn suggest_with_tooltip(&mut self, value: impl Into<String>, tooltip: impl Into<String>) {
        self.suggestions.push(SuggestionModel {
            value: value.into(),
            tooltip: Some(tooltip.into()),
        });
    }

    pub fn build(self) -> Vec<SuggestionModel> {
        self.suggestions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuggestionModel {
    pub value: String,
    pub tooltip: Option<String>,
}

pub fn filter_resources<T: Clone>(
    values: impl IntoIterator<Item = T>,
    contents: &str,
    converter: impl Fn(&T) -> Identifier,
) -> Vec<T> {
    let has_namespace = contents.contains(':');
    let mut accepted = Vec::new();
    for value in values {
        let id = converter(&value);
        if has_namespace {
            if matches_sub_str(contents, &id.to_string()) {
                accepted.push(value);
            }
        } else if matches_sub_str(contents, id.namespace()) || matches_sub_str(contents, id.path())
        {
            accepted.push(value);
        }
    }
    accepted
}

pub fn filter_resources_with_prefix<T: Clone>(
    values: impl IntoIterator<Item = T>,
    contents: &str,
    prefix: &str,
    converter: impl Fn(&T) -> Identifier,
) -> Vec<T> {
    let values: Vec<T> = values.into_iter().collect();
    if contents.is_empty() {
        return values;
    }
    let common_prefix = common_prefix(contents, prefix);
    if common_prefix.is_empty() {
        Vec::new()
    } else {
        filter_resources(values, &contents[common_prefix.len()..], converter)
    }
}

pub fn suggest_resource(
    values: impl IntoIterator<Item = Identifier>,
    builder: &mut SuggestionsBuilderModel,
) {
    let contents = builder.remaining().to_lowercase();
    for value in filter_resources(values, &contents, |id| id.clone()) {
        builder.suggest(value.to_string());
    }
}

pub fn suggest_resource_with_prefix(
    values: impl IntoIterator<Item = Identifier>,
    builder: &mut SuggestionsBuilderModel,
    prefix: &str,
) {
    let contents = builder.remaining().to_lowercase();
    for value in filter_resources_with_prefix(values, &contents, prefix, |id| id.clone()) {
        builder.suggest(format!("{prefix}{value}"));
    }
}

pub fn suggest_resource_with_tooltip<T: Clone>(
    values: impl IntoIterator<Item = T>,
    builder: &mut SuggestionsBuilderModel,
    id: impl Fn(&T) -> Identifier,
    tooltip: impl Fn(&T) -> String,
) {
    let contents = builder.remaining().to_lowercase();
    for value in filter_resources(values, &contents, |value| id(value)) {
        builder.suggest_with_tooltip(id(&value).to_string(), tooltip(&value));
    }
}

pub fn suggest_coordinates(
    current_input: &str,
    all_suggestions: &[TextCoordinates],
    builder: &mut SuggestionsBuilderModel,
    validator: impl Fn(&str) -> bool,
) {
    let mut result = Vec::new();
    if current_input.is_empty() {
        for coordinate in all_suggestions {
            let full_value = format!("{} {} {}", coordinate.x, coordinate.y, coordinate.z);
            if validator(&full_value) {
                result.push(coordinate.x.clone());
                result.push(format!("{} {}", coordinate.x, coordinate.y));
                result.push(full_value);
            }
        }
    } else {
        let fields = java_space_split(current_input);
        if fields.len() == 1 {
            for coordinate in all_suggestions {
                let full_value = format!("{} {} {}", fields[0], coordinate.y, coordinate.z);
                if validator(&full_value) {
                    result.push(format!("{} {}", fields[0], coordinate.y));
                    result.push(full_value);
                }
            }
        } else if fields.len() == 2 {
            for coordinate in all_suggestions {
                let full_value = format!("{} {} {}", fields[0], fields[1], coordinate.z);
                if validator(&full_value) {
                    result.push(full_value);
                }
            }
        }
    }
    suggest(result, builder);
}

pub fn suggest_2d_coordinates(
    current_input: &str,
    all_suggestions: &[TextCoordinates],
    builder: &mut SuggestionsBuilderModel,
    validator: impl Fn(&str) -> bool,
) {
    let mut result = Vec::new();
    if current_input.is_empty() {
        for coordinate in all_suggestions {
            let full_value = format!("{} {}", coordinate.x, coordinate.z);
            if validator(&full_value) {
                result.push(coordinate.x.clone());
                result.push(full_value);
            }
        }
    } else {
        let fields = java_space_split(current_input);
        if fields.len() == 1 {
            for coordinate in all_suggestions {
                let full_value = format!("{} {}", fields[0], coordinate.z);
                if validator(&full_value) {
                    result.push(full_value);
                }
            }
        }
    }
    suggest(result, builder);
}

pub fn suggest(values: impl IntoIterator<Item = String>, builder: &mut SuggestionsBuilderModel) {
    let lower_prefix = builder.remaining().to_lowercase();
    for name in values {
        if matches_sub_str(&lower_prefix, &name.to_lowercase()) {
            builder.suggest(name);
        }
    }
}

pub fn suggest_with_tooltip<T>(
    values: impl IntoIterator<Item = T>,
    builder: &mut SuggestionsBuilderModel,
    to_string: impl Fn(&T) -> String,
    tooltip: impl Fn(&T) -> String,
) {
    let lower_prefix = builder.remaining().to_lowercase();
    for value in values {
        let name = to_string(&value);
        if matches_sub_str(&lower_prefix, &name.to_lowercase()) {
            builder.suggest_with_tooltip(name, tooltip(&value));
        }
    }
}

pub fn matches_sub_str(pattern: &str, input: &str) -> bool {
    let mut index = 0;
    while !input[index..].starts_with(pattern) {
        let Some(relative) = input[index..].find(['.', '_', '/']) else {
            return false;
        };
        index += relative + 1;
        if index > input.len() {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementSuggestionType {
    Tags,
    Elements,
    All,
}

impl ElementSuggestionType {
    pub fn should_suggest_tags(self) -> bool {
        matches!(self, Self::Tags | Self::All)
    }

    pub fn should_suggest_elements(self) -> bool {
        matches!(self, Self::Elements | Self::All)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCoordinates {
    pub x: String,
    pub y: String,
    pub z: String,
}

impl TextCoordinates {
    pub fn new(x: impl Into<String>, y: impl Into<String>, z: impl Into<String>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub fn default_local() -> Self {
        Self::new("^", "^", "^")
    }

    pub fn default_global() -> Self {
        Self::new("~", "~", "~")
    }
}

fn common_prefix(left: &str, right: &str) -> String {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .map(|(left, _right)| left)
        .collect()
}

fn java_space_split(value: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = value.split(' ').collect();
    while parts.last() == Some(&"") {
        parts.pop();
    }
    parts
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> Vec<Identifier> {
        vec![
            id("minecraft:oak_log"),
            id("minecraft:spruce_log"),
            id("custom:oak_planks"),
            id("minecraft:stone"),
        ]
    }

    #[test]
    fn provider_defaults_match_java_interface_defaults() {
        let provider = SuggestionProviderModel {
            online_players: vec!["Alex".to_string(), "Steve".to_string()],
        };

        assert_eq!(
            provider.get_custom_tab_suggestions(),
            ["Alex".to_string(), "Steve".to_string()]
        );
        assert!(provider.get_selected_entities().is_empty());
        assert_eq!(
            provider.get_relevant_coordinates(),
            [TextCoordinates::default_global()]
        );
        assert_eq!(
            provider.get_absolute_coordinates(),
            [TextCoordinates::default_global()]
        );
    }

    #[test]
    fn element_suggestion_type_flags_match_java_enum() {
        assert!(ElementSuggestionType::Tags.should_suggest_tags());
        assert!(!ElementSuggestionType::Tags.should_suggest_elements());
        assert!(!ElementSuggestionType::Elements.should_suggest_tags());
        assert!(ElementSuggestionType::Elements.should_suggest_elements());
        assert!(ElementSuggestionType::All.should_suggest_tags());
        assert!(ElementSuggestionType::All.should_suggest_elements());
    }

    #[test]
    fn matches_sub_string_checks_start_and_splitter_boundaries() {
        assert!(matches_sub_str("oak", "oak_log"));
        assert!(matches_sub_str("log", "oak_log"));
        assert!(matches_sub_str("planks", "blocks/oak_planks"));
        assert!(!matches_sub_str("ak", "oak_log"));
    }

    #[test]
    fn filter_resources_uses_full_id_when_namespace_is_present() {
        let filtered = filter_resources(values(), "custom:oak", |id| id.clone());

        assert_eq!(filtered, [id("custom:oak_planks")]);
    }

    #[test]
    fn filter_resources_without_namespace_matches_namespace_or_path() {
        let filtered = filter_resources(values(), "spruce", |id| id.clone());

        assert_eq!(filtered, [id("minecraft:spruce_log")]);
        assert_eq!(
            filter_resources(values(), "custom", |id| id.clone()),
            [id("custom:oak_planks")]
        );
    }

    #[test]
    fn prefix_filter_accepts_all_for_empty_contents_and_strips_common_prefix() {
        assert_eq!(
            filter_resources_with_prefix(values(), "", "#", |id| id.clone()).len(),
            4
        );
        assert_eq!(
            filter_resources_with_prefix(values(), "#oak", "#", |id| id.clone()),
            [id("minecraft:oak_log"), id("custom:oak_planks")]
        );
        assert!(filter_resources_with_prefix(values(), "oak", "#", |id| id.clone()).is_empty());
    }

    #[test]
    fn suggest_resource_adds_matching_identifier_strings() {
        let mut builder = SuggestionsBuilderModel::new("oak");

        suggest_resource(values(), &mut builder);

        assert_eq!(
            builder.build(),
            [
                SuggestionModel {
                    value: "minecraft:oak_log".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "custom:oak_planks".to_string(),
                    tooltip: None,
                },
            ]
        );
    }

    #[test]
    fn suggest_resource_with_prefix_adds_prefix_to_suggestions() {
        let mut builder = SuggestionsBuilderModel::new("#oak");

        suggest_resource_with_prefix(values(), &mut builder, "#");

        assert_eq!(
            builder.build(),
            [
                SuggestionModel {
                    value: "#minecraft:oak_log".to_string(),
                    tooltip: None,
                },
                SuggestionModel {
                    value: "#custom:oak_planks".to_string(),
                    tooltip: None,
                },
            ]
        );
    }

    #[test]
    fn suggest_resource_with_tooltip_preserves_tooltip() {
        let mut builder = SuggestionsBuilderModel::new("stone");

        suggest_resource_with_tooltip(
            values(),
            &mut builder,
            |id| id.clone(),
            |id| format!("tooltip:{id}"),
        );

        assert_eq!(
            builder.build(),
            [SuggestionModel {
                value: "minecraft:stone".to_string(),
                tooltip: Some("tooltip:minecraft:stone".to_string()),
            }]
        );
    }

    #[test]
    fn suggest_coordinates_builds_incremental_3d_suggestions() {
        let coordinates = [TextCoordinates::new("~", "64", "~")];
        let mut empty = SuggestionsBuilderModel::new("");
        let mut one_field = SuggestionsBuilderModel::new("1");
        let mut two_fields = SuggestionsBuilderModel::new("1 2");

        suggest_coordinates("", &coordinates, &mut empty, |_| true);
        suggest_coordinates("1", &coordinates, &mut one_field, |_| true);
        suggest_coordinates("1 2", &coordinates, &mut two_fields, |_| true);

        assert_eq!(
            empty
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["~", "~ 64", "~ 64 ~"]
        );
        assert_eq!(
            one_field
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["1 64", "1 64 ~"]
        );
        assert_eq!(
            two_fields
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["1 2 ~"]
        );
    }

    #[test]
    fn coordinate_suggestions_respect_validator() {
        let coordinates = [TextCoordinates::new("1", "2", "3")];
        let mut builder = SuggestionsBuilderModel::new("");

        suggest_coordinates("", &coordinates, &mut builder, |value| value == "1 2 3");

        assert_eq!(
            builder
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["1", "1 2", "1 2 3"]
        );
    }

    #[test]
    fn suggest_2d_coordinates_builds_incremental_suggestions() {
        let coordinates = [TextCoordinates::new("~", "64", "~")];
        let mut empty = SuggestionsBuilderModel::new("");
        let mut one_field = SuggestionsBuilderModel::new("1");

        suggest_2d_coordinates("", &coordinates, &mut empty, |_| true);
        suggest_2d_coordinates("1", &coordinates, &mut one_field, |_| true);

        assert_eq!(
            empty
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["~", "~ ~"]
        );
        assert_eq!(
            one_field
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["1 ~"]
        );
    }

    #[test]
    fn suggest_string_values_matches_case_insensitively_at_splitters() {
        let mut builder = SuggestionsBuilderModel::new("bar");

        suggest(
            [
                "foo_bar".to_string(),
                "baz".to_string(),
                "qux/bar".to_string(),
            ],
            &mut builder,
        );

        assert_eq!(
            builder
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["foo_bar", "qux/bar"]
        );
    }

    #[test]
    fn suggest_with_tooltip_uses_string_converter_and_tooltip_converter() {
        let mut builder = SuggestionsBuilderModel::new("two");

        suggest_with_tooltip(
            [1, 2],
            &mut builder,
            |value| {
                if *value == 1 {
                    "one".to_string()
                } else {
                    "two".to_string()
                }
            },
            |value| format!("number:{value}"),
        );

        assert_eq!(
            builder.build(),
            [SuggestionModel {
                value: "two".to_string(),
                tooltip: Some("number:2".to_string()),
            }]
        );
    }

    #[test]
    fn provider_suggest_registry_elements_includes_tags_and_elements_by_type() {
        let provider = SuggestionProviderModel {
            online_players: Vec::new(),
        };
        let registry = RegistryLookupModel {
            tags: vec![id("minecraft:logs")],
            elements: vec![id("minecraft:oak_log")],
        };
        let mut tags = SuggestionsBuilderModel::new("#log");
        let mut elements = SuggestionsBuilderModel::new("oak");
        let mut all = SuggestionsBuilderModel::new("");

        provider.suggest_registry_elements(&registry, ElementSuggestionType::Tags, &mut tags);
        provider.suggest_registry_elements(
            &registry,
            ElementSuggestionType::Elements,
            &mut elements,
        );
        provider.suggest_registry_elements(&registry, ElementSuggestionType::All, &mut all);

        assert_eq!(
            tags.build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["#minecraft:logs"]
        );
        assert_eq!(
            elements
                .build()
                .into_iter()
                .map(|s| s.value)
                .collect::<Vec<_>>(),
            ["minecraft:oak_log"]
        );
        assert_eq!(
            all.build().into_iter().map(|s| s.value).collect::<Vec<_>>(),
            ["#minecraft:logs", "minecraft:oak_log"]
        );
    }

    #[test]
    fn text_coordinate_defaults_match_java_constants() {
        assert_eq!(
            TextCoordinates::default_local(),
            TextCoordinates::new("^", "^", "^")
        );
        assert_eq!(
            TextCoordinates::default_global(),
            TextCoordinates::new("~", "~", "~")
        );
    }
}
