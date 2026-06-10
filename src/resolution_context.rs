use crate::chat_component::{NbtSource, ObjectContent};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionContext {
    pub source: Option<ResolutionSourceModel>,
    pub default_scoreboard_entity: Option<String>,
    pub object_info_validator: ObjectInfoValidatorModel,
    pub depth_limit: i32,
    pub depth_limit_behavior: ResolutionContextLimitBehavior,
    selectors: Vec<(String, Vec<String>)>,
    scores: Vec<(String, String, i32)>,
    keybinds: Vec<(String, String)>,
    nbt_values: Vec<(NbtSource, String, Vec<String>)>,
    player_sprites: Vec<(String, bool, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionSourceModel {
    pub entity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectInfoValidatorModel {
    AllowAll,
    DenyAll,
    AllowObjects(Vec<ObjectContent>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionContextLimitBehavior {
    DiscardRemaining,
    StopProcessingAndCopyRemaining,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionContextBuilder {
    source: Option<ResolutionSourceModel>,
    default_scoreboard_entity: Option<String>,
    object_info_validator: ObjectInfoValidatorModel,
    depth_limit: i32,
    depth_limit_behavior: ResolutionContextLimitBehavior,
}

impl Default for ResolutionContext {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl ResolutionContext {
    pub fn create(source: ResolutionSourceModel) -> Self {
        Self::builder().with_source(source).build()
    }

    pub fn builder() -> ResolutionContextBuilder {
        ResolutionContextBuilder::default()
    }

    pub fn validate<'a>(&self, description: &'a ObjectContent) -> Option<&'a ObjectContent> {
        self.object_info_validator
            .test(description)
            .then_some(description)
    }

    pub fn with_selector(mut self, selector: impl Into<String>, values: Vec<&str>) -> Self {
        self.selectors.push((
            selector.into(),
            values.into_iter().map(ToOwned::to_owned).collect(),
        ));
        self
    }

    pub fn with_score(
        mut self,
        name: impl Into<String>,
        objective: impl Into<String>,
        value: i32,
    ) -> Self {
        self.scores.push((name.into(), objective.into(), value));
        self
    }

    pub fn with_keybind(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.keybinds.push((key.into(), value.into()));
        self
    }

    pub fn with_nbt(
        mut self,
        source: NbtSource,
        path: impl Into<String>,
        values: Vec<&str>,
    ) -> Self {
        self.nbt_values.push((
            source,
            path.into(),
            values.into_iter().map(ToOwned::to_owned).collect(),
        ));
        self
    }

    pub fn with_player_sprite(
        mut self,
        profile: impl Into<String>,
        hat: bool,
        value: impl Into<String>,
    ) -> Self {
        self.player_sprites
            .push((profile.into(), hat, value.into()));
        self
    }

    pub(crate) fn selector(&self, selector: &str) -> Option<Vec<String>> {
        self.selectors
            .iter()
            .find(|(key, _)| key == selector)
            .map(|(_, values)| values.clone())
    }

    pub(crate) fn score(&self, name: &str, objective: &str) -> Option<i32> {
        self.scores
            .iter()
            .find(|(entry_name, entry_objective, _)| {
                entry_name == name && entry_objective == objective
            })
            .map(|(_, _, value)| *value)
    }

    pub(crate) fn keybind(&self, key: &str) -> Option<&str> {
        self.keybinds
            .iter()
            .find(|(entry, _)| entry == key)
            .map(|(_, value)| value.as_str())
    }

    pub(crate) fn nbt(&self, source: &NbtSource, path: &str) -> Vec<String> {
        self.nbt_values
            .iter()
            .find(|(entry_source, entry_path, _)| entry_source == source && entry_path == path)
            .map(|(_, _, values)| values.clone())
            .unwrap_or_default()
    }

    pub(crate) fn player_sprite(&self, profile: &str, hat: bool) -> Option<&str> {
        self.player_sprites
            .iter()
            .find(|(entry_profile, entry_hat, _)| entry_profile == profile && *entry_hat == hat)
            .map(|(_, _, value)| value.as_str())
    }
}

impl Default for ResolutionContextBuilder {
    fn default() -> Self {
        Self {
            source: None,
            default_scoreboard_entity: None,
            object_info_validator: ObjectInfoValidatorModel::AllowAll,
            depth_limit: 100,
            depth_limit_behavior: ResolutionContextLimitBehavior::StopProcessingAndCopyRemaining,
        }
    }
}

impl ResolutionContextBuilder {
    pub fn with_source(mut self, source: ResolutionSourceModel) -> Self {
        self.default_scoreboard_entity = source.entity.clone();
        self.source = Some(source);
        self
    }

    pub fn with_entity_override(
        mut self,
        default_scoreboard_entity: Option<impl Into<String>>,
    ) -> Self {
        self.default_scoreboard_entity = default_scoreboard_entity.map(Into::into);
        self
    }

    pub fn with_object_info_validator(mut self, validator: ObjectInfoValidatorModel) -> Self {
        self.object_info_validator = validator;
        self
    }

    pub fn set_depth_limit(mut self, depth_limit: i32) -> Self {
        self.depth_limit = depth_limit;
        self
    }

    pub fn set_depth_limit_behavior(mut self, behavior: ResolutionContextLimitBehavior) -> Self {
        self.depth_limit_behavior = behavior;
        self
    }

    pub fn build(self) -> ResolutionContext {
        ResolutionContext {
            source: self.source,
            default_scoreboard_entity: self.default_scoreboard_entity,
            object_info_validator: self.object_info_validator,
            depth_limit: self.depth_limit,
            depth_limit_behavior: self.depth_limit_behavior,
            selectors: Vec::new(),
            scores: Vec::new(),
            keybinds: Vec::new(),
            nbt_values: Vec::new(),
            player_sprites: Vec::new(),
        }
    }
}

impl ObjectInfoValidatorModel {
    fn test(&self, description: &ObjectContent) -> bool {
        match self {
            Self::AllowAll => true,
            Self::DenyAll => false,
            Self::AllowObjects(allowed) => allowed.contains(description),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(entity: Option<&str>) -> ResolutionSourceModel {
        ResolutionSourceModel {
            entity: entity.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn resolution_context_builder_defaults_match_java() {
        let context = ResolutionContext::builder().build();

        assert_eq!(context.source, None);
        assert_eq!(context.default_scoreboard_entity, None);
        assert_eq!(
            context.object_info_validator,
            ObjectInfoValidatorModel::AllowAll
        );
        assert_eq!(context.depth_limit, 100);
        assert_eq!(
            context.depth_limit_behavior,
            ResolutionContextLimitBehavior::StopProcessingAndCopyRemaining
        );
        assert!(context
            .validate(&ObjectContent::AtlasSprite {
                atlas: "minecraft:blocks".to_string(),
                sprite: "minecraft:stone".to_string(),
            })
            .is_some());
    }

    #[test]
    fn resolution_context_builder_with_source_sets_default_scoreboard_entity() {
        let context = ResolutionContext::create(source(Some("Steve")));

        assert_eq!(
            context.source,
            Some(ResolutionSourceModel {
                entity: Some("Steve".to_string())
            })
        );
        assert_eq!(context.default_scoreboard_entity, Some("Steve".to_string()));
    }

    #[test]
    fn resolution_context_builder_overrides_validator_depth_and_limit_behavior() {
        let allowed = ObjectContent::PlayerSprite {
            profile: "Steve".to_string(),
            hat: true,
        };
        let denied = ObjectContent::PlayerSprite {
            profile: "Alex".to_string(),
            hat: false,
        };
        let context = ResolutionContext::builder()
            .with_source(source(Some("Steve")))
            .with_entity_override(Some("Alex"))
            .with_object_info_validator(ObjectInfoValidatorModel::AllowObjects(vec![
                allowed.clone()
            ]))
            .set_depth_limit(3)
            .set_depth_limit_behavior(ResolutionContextLimitBehavior::DiscardRemaining)
            .build();

        assert_eq!(context.default_scoreboard_entity, Some("Alex".to_string()));
        assert_eq!(context.depth_limit, 3);
        assert_eq!(
            context.depth_limit_behavior,
            ResolutionContextLimitBehavior::DiscardRemaining
        );
        assert_eq!(context.validate(&allowed), Some(&allowed));
        assert_eq!(context.validate(&denied), None);
    }

    #[test]
    fn resolution_context_preserves_existing_render_lookup_helpers() {
        let context = ResolutionContext::default()
            .with_selector("@a", vec!["Steve", "Alex"])
            .with_score("Steve", "kills", 7)
            .with_keybind("key.jump", "Space")
            .with_nbt(
                NbtSource::Storage("minecraft:test".to_string()),
                "path",
                vec!["one", "two"],
            )
            .with_player_sprite("Steve", true, "steve-hat");

        assert_eq!(
            context.selector("@a"),
            Some(vec!["Steve".to_string(), "Alex".to_string()])
        );
        assert_eq!(context.score("Steve", "kills"), Some(7));
        assert_eq!(context.keybind("key.jump"), Some("Space"));
        assert_eq!(
            context.nbt(&NbtSource::Storage("minecraft:test".to_string()), "path"),
            vec!["one".to_string(), "two".to_string()]
        );
        assert_eq!(context.player_sprite("Steve", true), Some("steve-hat"));
    }
}
