use crate::chat_component::{ResolutionContext, Style};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentContentsModel {
    codec: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutableComponentModel {
    contents: ComponentContentsModel,
}

impl ComponentContentsModel {
    pub fn new(codec: &'static str) -> Self {
        Self { codec }
    }

    pub fn visit<T>(&self, _output: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        None
    }

    pub fn visit_styled<T>(
        &self,
        _output: &mut impl FnMut(&Style, &str) -> Option<T>,
        _current_style: &Style,
    ) -> Option<T> {
        None
    }

    pub fn resolve(
        &self,
        _context: &ResolutionContext,
        _recursion_depth: i32,
    ) -> MutableComponentModel {
        MutableComponentModel::create(self.clone())
    }

    pub fn codec(&self) -> &'static str {
        self.codec
    }
}

impl MutableComponentModel {
    pub fn create(contents: ComponentContentsModel) -> Self {
        Self { contents }
    }

    pub fn contents(&self) -> &ComponentContentsModel {
        &self.contents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPONENT_CONTENTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/ComponentContents.java"
    );

    #[test]
    fn component_contents_default_methods_match_java_interface() {
        for sentinel in [
            "public interface ComponentContents",
            "default <T> Optional<T> visit(final FormattedText.StyledContentConsumer<T> output, final Style currentStyle)",
            "return Optional.empty();",
            "default <T> Optional<T> visit(final FormattedText.ContentConsumer<T> output)",
            "default MutableComponent resolve(final ResolutionContext context, final int recursionDepth) throws CommandSyntaxException",
            "return MutableComponent.create(this);",
            "MapCodec<? extends ComponentContents> codec();",
        ] {
            assert!(
                COMPONENT_CONTENTS_JAVA.contains(sentinel),
                "missing ComponentContents Java sentinel: {sentinel}"
            );
        }

        let contents = ComponentContentsModel::new("minecraft:test");
        let mut plain_visited = false;
        assert_eq!(
            contents.visit::<()>(&mut |_text| {
                plain_visited = true;
                Some(())
            }),
            None
        );
        assert!(!plain_visited);

        let style = Style::empty();
        let mut styled_visited = false;
        assert_eq!(
            contents.visit_styled::<()>(
                &mut |_style, _text| {
                    styled_visited = true;
                    Some(())
                },
                &style,
            ),
            None
        );
        assert!(!styled_visited);

        let resolved = contents.resolve(&ResolutionContext::default(), 99);
        assert_eq!(resolved.contents(), &contents);
        assert_eq!(contents.codec(), "minecraft:test");
    }
}
