use crate::chat_component::{Component, ObjectContent, ResolutionContext, Style};

pub const OBJECT_CONTENTS_PLACEHOLDER: &str = "\u{fffc}";
pub const OBJECT_CONTENTS_CODEC_FIELDS: [&str; 2] = ["object", "fallback"];
pub const OBJECT_INFO_CODEC_TYPES: [&str; 2] = ["atlas", "player"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectContentsModel {
    contents: ObjectContent,
    fallback: Option<Component>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectContentsResolution {
    Valid(ObjectContentsModel),
    Fallback(Component),
}

impl ObjectContentsModel {
    pub fn new(contents: ObjectContent, fallback: Option<Component>) -> Self {
        Self { contents, fallback }
    }

    pub fn contents(&self) -> &ObjectContent {
        &self.contents
    }

    pub fn fallback(&self) -> Option<&Component> {
        self.fallback.as_ref()
    }

    pub fn codec_fields(&self) -> [&'static str; 2] {
        OBJECT_CONTENTS_CODEC_FIELDS
    }

    pub fn object_info_codec_types(&self) -> [&'static str; 2] {
        OBJECT_INFO_CODEC_TYPES
    }

    pub fn resolve(
        &self,
        context: &ResolutionContext,
        _recursion_depth: i32,
    ) -> ObjectContentsResolution {
        let resolved_fallback = self.fallback.clone();
        match context.validate(&self.contents) {
            Some(validated) => {
                ObjectContentsResolution::Valid(Self::new(validated.clone(), resolved_fallback))
            }
            None => ObjectContentsResolution::Fallback(
                resolved_fallback
                    .unwrap_or_else(|| Component::literal(self.contents.default_fallback())),
            ),
        }
    }

    pub fn visit<T>(&self, output: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        match &self.fallback {
            Some(fallback) => output(&fallback.get_string()),
            None => output(&self.contents.default_fallback()),
        }
    }

    pub fn visit_styled<T>(
        &self,
        output: &mut impl FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        output(
            &current_style
                .clone()
                .with_font(self.contents.font_description()),
            OBJECT_CONTENTS_PLACEHOLDER,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::resolution_context::ObjectInfoValidatorModel;
    use crate::chat_component::FontDescription;

    const OBJECT_CONTENTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/ObjectContents.java"
    );
    const OBJECT_INFO_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/ObjectInfo.java"
    );
    const OBJECT_INFOS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/ObjectInfos.java"
    );
    const ATLAS_SPRITE_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/AtlasSprite.java"
    );
    const PLAYER_SPRITE_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/objects/PlayerSprite.java"
    );

    fn atlas(atlas: &str, sprite: &str) -> ObjectContent {
        ObjectContent::AtlasSprite {
            atlas: atlas.to_string(),
            sprite: sprite.to_string(),
        }
    }

    fn player(profile: &str, hat: bool) -> ObjectContent {
        ObjectContent::PlayerSprite {
            profile: profile.to_string(),
            hat,
        }
    }

    #[test]
    fn object_contents_java_source_contract_is_tracked() {
        for sentinel in [
            "public record ObjectContents(ObjectInfo contents, Optional<Component> fallback) implements ComponentContents",
            "private static final String PLACEHOLDER = Character.toString",
            "ObjectInfos.CODEC.forGetter(ObjectContents::contents)",
            "ComponentSerialization.CODEC.optionalFieldOf(\"fallback\").forGetter(ObjectContents::fallback)",
            "Optional<MutableComponent> fallback = ComponentUtils.resolve(context, this.fallback, recursionDepth);",
            "ObjectInfo validatedContents = context.validate(this.contents);",
            "fallback.orElseGet(() -> Component.literal(this.contents.defaultFallback()))",
            "MutableComponent.create(new ObjectContents(validatedContents, fallback.map(o -> (Component)o)))",
            "return this.fallback.isPresent() ? this.fallback.get().visit(output) : output.accept(this.contents.defaultFallback());",
            "return output.accept(currentStyle.withFont(this.contents.fontDescription()), PLACEHOLDER);",
        ] {
            assert!(
                OBJECT_CONTENTS_JAVA.contains(sentinel),
                "missing ObjectContents Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "FontDescription fontDescription();",
            "String defaultFallback();",
            "MapCodec<? extends ObjectInfo> codec();",
        ] {
            assert!(
                OBJECT_INFO_JAVA.contains(sentinel),
                "missing ObjectInfo Java sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "ComponentSerialization.createLegacyComponentMatcher(ID_MAPPER, ObjectInfo::codec, \"object\")",
            "ID_MAPPER.put(\"atlas\", AtlasSprite.MAP_CODEC);",
            "ID_MAPPER.put(\"player\", PlayerSprite.MAP_CODEC);",
        ] {
            assert!(
                OBJECT_INFOS_JAVA.contains(sentinel),
                "missing ObjectInfos Java sentinel: {sentinel}"
            );
        }
    }

    #[test]
    fn object_info_default_fallbacks_and_fonts_match_java() {
        assert!(ATLAS_SPRITE_JAVA
            .contains("public static final Identifier DEFAULT_ATLAS = AtlasIds.BLOCKS;"));
        assert!(ATLAS_SPRITE_JAVA.contains(
            "return id.getNamespace().equals(\"minecraft\") ? id.getPath() : id.toString();"
        ));
        assert!(
            PLAYER_SPRITE_JAVA.contains("return this.player.name().map(name -> \"[\" + name + \" head]\").orElse(\"[unknown player head]\");")
        );

        let default_atlas = atlas("minecraft:blocks", "minecraft:stone");
        assert_eq!(default_atlas.default_fallback(), "[stone]");
        assert_eq!(
            default_atlas.font_description(),
            FontDescription::AtlasSprite {
                atlas: "minecraft:blocks".to_string(),
                sprite: "minecraft:stone".to_string(),
            }
        );

        let custom_atlas = atlas("mod:icons", "minecraft:copper_golem");
        assert_eq!(custom_atlas.default_fallback(), "[copper_golem@mod:icons]");

        let custom_sprite = atlas("minecraft:blocks", "mod:gear");
        assert_eq!(custom_sprite.default_fallback(), "[mod:gear]");

        assert_eq!(player("Steve", true).default_fallback(), "[Steve head]");
        assert_eq!(player("", true).default_fallback(), "[unknown player head]");
        assert_eq!(
            player("Alex", false).font_description(),
            FontDescription::PlayerSprite {
                profile: "Alex".to_string(),
                hat: false,
            }
        );
    }

    #[test]
    fn object_contents_resolve_validates_or_uses_fallback_like_java() {
        let contents =
            ObjectContentsModel::new(atlas("minecraft:blocks", "minecraft:oak_log"), None);
        let allow_all = ResolutionContext::default();
        assert_eq!(
            contents.resolve(&allow_all, 0),
            ObjectContentsResolution::Valid(contents.clone())
        );

        let deny_all = ResolutionContext::builder()
            .with_object_info_validator(ObjectInfoValidatorModel::DenyAll)
            .build();
        assert_eq!(
            contents.resolve(&deny_all, 0),
            ObjectContentsResolution::Fallback(Component::literal("[oak_log]"))
        );

        let with_fallback = ObjectContentsModel::new(
            player("Steve", true),
            Some(Component::literal("Steve icon")),
        );
        assert_eq!(
            with_fallback.resolve(&deny_all, 0),
            ObjectContentsResolution::Fallback(Component::literal("Steve icon"))
        );

        let allow_one = ResolutionContext::builder()
            .with_object_info_validator(ObjectInfoValidatorModel::AllowObjects(vec![player(
                "Steve", true,
            )]))
            .build();
        assert_eq!(
            with_fallback.resolve(&allow_one, 0),
            ObjectContentsResolution::Valid(with_fallback.clone())
        );
    }

    #[test]
    fn object_contents_visitors_codec_and_accessors_match_java_surface() {
        let style = Style::empty();
        let contents =
            ObjectContentsModel::new(atlas("minecraft:blocks", "minecraft:diamond"), None);

        assert_eq!(contents.codec_fields(), ["object", "fallback"]);
        assert_eq!(contents.object_info_codec_types(), ["atlas", "player"]);
        assert_eq!(
            contents.contents(),
            &atlas("minecraft:blocks", "minecraft:diamond")
        );
        assert_eq!(contents.fallback(), None);
        assert_eq!(
            contents.visit(&mut |visited| Some(visited.to_string())),
            Some("[diamond]".to_string())
        );
        assert_eq!(
            contents.visit_styled(
                &mut |visited_style, text| Some((visited_style.clone(), text.to_string())),
                &style,
            ),
            Some((
                style.with_font(FontDescription::AtlasSprite {
                    atlas: "minecraft:blocks".to_string(),
                    sprite: "minecraft:diamond".to_string(),
                }),
                OBJECT_CONTENTS_PLACEHOLDER.to_string(),
            ))
        );

        let fallback =
            ObjectContentsModel::new(player("Alex", true), Some(Component::literal("Alex icon")));
        assert_eq!(
            fallback.visit(&mut |visited| Some(visited.to_string())),
            Some("Alex icon".to_string())
        );
    }
}
