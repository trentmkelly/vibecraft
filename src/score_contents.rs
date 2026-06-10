use crate::chat_component::{Component, ResolutionContext};

pub const SCORE_CONTENTS_INNER_CODEC_FIELDS: [&str; 2] = ["name", "objective"];
pub const SCORE_CONTENTS_MAP_CODEC_FIELD: &str = "score";
pub const WILDCARD_SCORE_HOLDER: &str = "*";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreTargetModel {
    Selector { source: String },
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreContentsModel {
    name: ScoreTargetModel,
    objective: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreContentsError {
    NotSingleEntity,
}

impl ScoreContentsModel {
    pub fn new(name: ScoreTargetModel, objective: impl Into<String>) -> Self {
        Self {
            name,
            objective: objective.into(),
        }
    }

    pub fn name(&self) -> &ScoreTargetModel {
        &self.name
    }

    pub fn objective(&self) -> &str {
        &self.objective
    }

    pub fn codec_field(&self) -> &'static str {
        SCORE_CONTENTS_MAP_CODEC_FIELD
    }

    pub fn resolve(&self, context: &ResolutionContext) -> Result<Component, ScoreContentsError> {
        if context.source.is_none() {
            return Ok(Component::empty());
        }

        let score_holder = self.find_target_name(context)?;
        let score_name = if score_holder == WILDCARD_SCORE_HOLDER {
            context
                .default_scoreboard_entity
                .as_deref()
                .unwrap_or(score_holder.as_str())
        } else {
            score_holder.as_str()
        };

        Ok(context
            .score(score_name, &self.objective)
            .map(|value| Component::literal(value.to_string()))
            .unwrap_or_else(Component::empty))
    }

    fn find_target_name(&self, context: &ResolutionContext) -> Result<String, ScoreContentsError> {
        match &self.name {
            ScoreTargetModel::Selector { source } => match context.selector(source) {
                Some(entities) if entities.len() == 1 => Ok(entities[0].clone()),
                Some(entities) if entities.is_empty() => Ok(source.clone()),
                Some(_) => Err(ScoreContentsError::NotSingleEntity),
                None => Ok(source.clone()),
            },
            ScoreTargetModel::Name(name) => Ok(name.clone()),
        }
    }
}

impl std::fmt::Display for ScoreContentsModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "score{{name='{}', objective='{}'}}",
            self.name, self.objective
        )
    }
}

impl std::fmt::Display for ScoreTargetModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Selector { source } => write!(formatter, "Left({source})"),
            Self::Name(name) => write!(formatter, "Right({name})"),
        }
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::resolution_context::ResolutionSourceModel;

    const SCORE_CONTENTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/ScoreContents.java"
    );

    fn source(entity: Option<&str>) -> ResolutionSourceModel {
        ResolutionSourceModel {
            entity: entity.map(ToOwned::to_owned),
        }
    }

    fn context_with_source(entity: Option<&str>) -> ResolutionContext {
        ResolutionContext::create(source(entity))
    }

    #[test]
    fn score_contents_java_source_contract_is_tracked() {
        for sentinel in [
            "public record ScoreContents(Either<CompilableString<EntitySelector>, String> name, String objective) implements ComponentContents",
            "Codec.either(EntitySelector.COMPILABLE_CODEC, Codec.STRING).fieldOf(\"name\")",
            "Codec.STRING.fieldOf(\"objective\")",
            "public static final MapCodec<ScoreContents> MAP_CODEC = INNER_CODEC.fieldOf(\"score\");",
            "return MAP_CODEC;",
            "private ScoreHolder findTargetName(final CommandSourceStack source) throws CommandSyntaxException",
            "if (entities.size() != 1)",
            "throw EntityArgument.ERROR_NOT_SINGLE_ENTITY.create();",
            "return ScoreHolder.forNameOnly(selector.get().source());",
            "return ScoreHolder.forNameOnly((String)this.name.right().orElseThrow());",
            "return scoreInfo.formatValue(objective.numberFormatOrDefault(StyledFormat.NO_STYLE));",
            "return Component.empty();",
            "if (source == null)",
            "ScoreHolder scoreName = entity != null && scoreHolder.equals(ScoreHolder.WILDCARD) ? entity : scoreHolder;",
            "return \"score{name='\" + this.name + \"', objective='\" + this.objective + \"'}\";",
        ] {
            assert!(
                SCORE_CONTENTS_JAVA.contains(sentinel),
                "missing ScoreContents Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn score_contents_resolves_names_wildcard_and_missing_score_like_java() {
        let context = context_with_source(Some("Steve"))
            .with_score("Steve", "kills", 7)
            .with_score("Alex", "kills", 3);

        let direct = ScoreContentsModel::new(ScoreTargetModel::Name("Alex".to_string()), "kills");
        assert_eq!(
            direct
                .resolve(&context)
                .unwrap_or_else(|_| Component::empty())
                .get_string(),
            "3"
        );

        let wildcard = ScoreContentsModel::new(
            ScoreTargetModel::Name(WILDCARD_SCORE_HOLDER.to_string()),
            "kills",
        );
        assert_eq!(
            wildcard
                .resolve(&context)
                .unwrap_or_else(|_| Component::empty())
                .get_string(),
            "7"
        );

        let missing_objective =
            ScoreContentsModel::new(ScoreTargetModel::Name("Alex".to_string()), "deaths");
        assert_eq!(
            missing_objective
                .resolve(&context)
                .unwrap_or_else(|_| Component::literal("unexpected"))
                .get_string(),
            ""
        );
    }

    #[test]
    fn score_contents_source_null_and_selector_target_rules_match_java() {
        let without_source = ResolutionContext::default().with_score("Alex", "kills", 3);
        let direct = ScoreContentsModel::new(ScoreTargetModel::Name("Alex".to_string()), "kills");
        assert_eq!(
            direct
                .resolve(&without_source)
                .unwrap_or_else(|_| Component::literal("unexpected"))
                .get_string(),
            "",
            "Java returns Component.empty when context.source() is null"
        );

        let selector = ScoreContentsModel::new(
            ScoreTargetModel::Selector {
                source: "@p".to_string(),
            },
            "kills",
        );
        let one = context_with_source(None)
            .with_selector("@p", vec!["Alex"])
            .with_score("Alex", "kills", 5);
        assert_eq!(
            selector
                .resolve(&one)
                .unwrap_or_else(|_| Component::empty())
                .get_string(),
            "5"
        );

        let empty = context_with_source(None)
            .with_selector("@p", Vec::new())
            .with_score("@p", "kills", 11);
        assert_eq!(
            selector
                .resolve(&empty)
                .unwrap_or_else(|_| Component::empty())
                .get_string(),
            "11",
            "empty selector results use the selector source as a name-only score holder"
        );

        let multiple = context_with_source(None).with_selector("@p", vec!["Alex", "Steve"]);
        assert_eq!(
            selector.resolve(&multiple),
            Err(ScoreContentsError::NotSingleEntity)
        );
    }

    #[test]
    fn score_contents_accessors_codec_and_to_string_match_java_surface() {
        let score = ScoreContentsModel::new(
            ScoreTargetModel::Selector {
                source: "@s".to_string(),
            },
            "points",
        );

        assert_eq!(
            SCORE_CONTENTS_INNER_CODEC_FIELDS,
            ["name", "objective"],
            "INNER_CODEC stores name and objective fields"
        );
        assert_eq!(score.codec_field(), "score");
        assert_eq!(score.objective(), "points");
        assert_eq!(
            score.name(),
            &ScoreTargetModel::Selector {
                source: "@s".to_string()
            }
        );
        assert_eq!(
            score.to_string(),
            "score{name='Left(@s)', objective='points'}"
        );
    }
}
