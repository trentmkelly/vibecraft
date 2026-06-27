#![allow(dead_code)]

use crate::chat_component::filter_mask::FilterMask;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredTextModel {
    raw: String,
    mask: FilterMask,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterableModel<T> {
    raw: T,
    filtered: Option<T>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterableCodecShape<T> {
    Full { raw: T, filtered: Option<T> },
    Simple(T),
}

impl FilteredTextModel {
    pub fn pass_through(message: impl Into<String>) -> Self {
        Self {
            raw: message.into(),
            mask: FilterMask::pass_through(),
        }
    }

    pub fn fully_filtered(message: impl Into<String>) -> Self {
        Self {
            raw: message.into(),
            mask: FilterMask::fully_filtered(),
        }
    }

    pub fn new(raw: impl Into<String>, mask: FilterMask) -> Self {
        Self {
            raw: raw.into(),
            mask,
        }
    }

    pub fn empty() -> Self {
        Self::pass_through("")
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn mask(&self) -> &FilterMask {
        &self.mask
    }

    pub fn filtered(&self) -> Option<String> {
        self.mask.apply(&self.raw)
    }

    pub fn filtered_or_empty(&self) -> String {
        self.filtered().into_iter().collect()
    }

    pub fn is_filtered(&self) -> bool {
        !self.mask.is_empty()
    }
}

impl<T> FilterableModel<T> {
    pub fn new(raw: T, filtered: Option<T>) -> Self {
        Self { raw, filtered }
    }

    pub fn pass_through(value: T) -> Self {
        Self {
            raw: value,
            filtered: None,
        }
    }

    pub fn raw(&self) -> &T {
        &self.raw
    }

    pub fn filtered(&self) -> Option<&T> {
        self.filtered.as_ref()
    }

    pub fn get(&self, filter_enabled: bool) -> &T {
        if filter_enabled {
            match &self.filtered {
                Some(filtered) => filtered,
                None => &self.raw,
            }
        } else {
            &self.raw
        }
    }

    pub fn map<U>(&self, mut function: impl FnMut(&T) -> U) -> FilterableModel<U> {
        FilterableModel {
            raw: function(&self.raw),
            filtered: self.filtered.as_ref().map(function),
        }
    }

    pub fn resolve<U>(&self, mut function: impl FnMut(&T) -> Option<U>) -> Option<FilterableModel<U>> {
        let new_raw = function(&self.raw)?;
        if let Some(filtered) = &self.filtered {
            let new_filtered = function(filtered)?;
            Some(FilterableModel::new(new_raw, Some(new_filtered)))
        } else {
            Some(FilterableModel::new(new_raw, None))
        }
    }

    pub fn codec_shape(&self) -> FilterableCodecShape<&T> {
        FilterableCodecShape::Full {
            raw: &self.raw,
            filtered: self.filtered.as_ref(),
        }
    }

    pub fn simple_codec_shape(&self) -> FilterableCodecShape<&T> {
        FilterableCodecShape::Simple(&self.raw)
    }
}

impl FilterableModel<String> {
    pub fn from_filtered_text(text: &FilteredTextModel) -> Self {
        Self {
            raw: text.raw().to_string(),
            filtered: text.is_filtered().then(|| text.filtered_or_empty()),
        }
    }
}

pub fn filterable_from_codec_shape<T>(shape: FilterableCodecShape<T>) -> FilterableModel<T> {
    match shape {
        FilterableCodecShape::Full { raw, filtered } => FilterableModel::new(raw, filtered),
        FilterableCodecShape::Simple(raw) => FilterableModel::pass_through(raw),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtered_text_factories_apply_filter_mask_like_java() {
        assert_eq!(FilteredTextModel::empty(), FilteredTextModel::pass_through(""));

        let pass = FilteredTextModel::pass_through("hello");
        assert_eq!(pass.raw(), "hello");
        assert_eq!(pass.filtered(), Some("hello".to_string()));
        assert_eq!(pass.filtered_or_empty(), "hello");
        assert!(!pass.is_filtered());

        let full = FilteredTextModel::fully_filtered("secret");
        assert_eq!(full.filtered(), None);
        assert_eq!(full.filtered_or_empty(), "");
        assert!(full.is_filtered());

        let mut mask = FilterMask::partially_filtered(5);
        mask.set_filtered(1);
        mask.set_filtered(3);
        let partial = FilteredTextModel::new("abcde", mask);
        assert_eq!(partial.filtered(), Some("a#c#e".to_string()));
        assert_eq!(partial.filtered_or_empty(), "a#c#e");
        assert!(partial.is_filtered());
    }

    #[test]
    fn filterable_codec_shapes_match_java_full_and_simple_alternatives() {
        let full = filterable_from_codec_shape(FilterableCodecShape::Full {
            raw: "raw".to_string(),
            filtered: Some("filtered".to_string()),
        });
        assert_eq!(full.raw(), "raw");
        assert_eq!(full.filtered(), Some(&"filtered".to_string()));

        let simple = filterable_from_codec_shape(FilterableCodecShape::Simple("plain".to_string()));
        assert_eq!(simple, FilterableModel::pass_through("plain".to_string()));
        assert_eq!(
            simple.codec_shape(),
            FilterableCodecShape::Full {
                raw: &"plain".to_string(),
                filtered: None
            }
        );
        assert_eq!(
            simple.simple_codec_shape(),
            FilterableCodecShape::Simple(&"plain".to_string())
        );
    }

    #[test]
    fn filterable_from_filtered_text_uses_filtered_or_empty_only_when_filtered() {
        let pass = FilteredTextModel::pass_through("hello");
        assert_eq!(
            FilterableModel::from_filtered_text(&pass),
            FilterableModel::new("hello".to_string(), None)
        );

        let full = FilteredTextModel::fully_filtered("secret");
        assert_eq!(
            FilterableModel::from_filtered_text(&full),
            FilterableModel::new("secret".to_string(), Some(String::new()))
        );

        let mut mask = FilterMask::partially_filtered(3);
        mask.set_filtered(0);
        let partial = FilteredTextModel::new("abc", mask);
        assert_eq!(
            FilterableModel::from_filtered_text(&partial),
            FilterableModel::new("abc".to_string(), Some("#bc".to_string()))
        );
    }

    #[test]
    fn filterable_get_map_and_resolve_match_java_option_flow() {
        let value = FilterableModel::new("raw".to_string(), Some("filtered".to_string()));
        assert_eq!(value.get(false), "raw");
        assert_eq!(value.get(true), "filtered");

        let pass = FilterableModel::pass_through("raw".to_string());
        assert_eq!(pass.get(true), "raw");

        let mapped = value.map(|text| text.len());
        assert_eq!(mapped, FilterableModel::new(3, Some(8)));

        let resolved = value.resolve(|text| text.strip_prefix('f').map(str::len));
        assert_eq!(resolved, None);

        let resolved = value.resolve(|text| Some(text.len()));
        assert_eq!(resolved, Some(FilterableModel::new(3, Some(8))));

        let pass_resolved = pass.resolve(|text| Some(text.len()));
        assert_eq!(pass_resolved, Some(FilterableModel::new(3, None)));

        let filtered_fails = value.resolve(|text| {
            if text == "filtered" {
                None
            } else {
                Some(text.len())
            }
        });
        assert_eq!(filtered_fails, None);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn filterable_sources_match_java_26_1_2() {
        const FILTERABLE: &str =
            vibecraft_java_source!("/net/minecraft/server/network/Filterable.java");
        const FILTERED_TEXT: &str =
            vibecraft_java_source!("/net/minecraft/server/network/FilteredText.java");

        for sentinel in [
            "public record Filterable<T>(T raw, Optional<T> filtered)",
            "public static <T> Codec<Filterable<T>> codec(final Codec<T> valueCodec)",
            "valueCodec.fieldOf(\"raw\").forGetter(Filterable::raw)",
            "valueCodec.optionalFieldOf(\"filtered\").forGetter(Filterable::filtered)",
            "Codec<Filterable<T>> simpleCodec = valueCodec.xmap(Filterable::passThrough, Filterable::raw);",
            "return Codec.withAlternative(fullCodec, simpleCodec);",
            "public static <B extends ByteBuf, T> StreamCodec<B, Filterable<T>> streamCodec(final StreamCodec<B, T> valueCodec)",
            "valueCodec.apply(ByteBufCodecs::optional)",
            "public static <T> Filterable<T> passThrough(final T value)",
            "return new Filterable<>(value, Optional.empty());",
            "public static Filterable<String> from(final FilteredText text)",
            "text.isFiltered() ? Optional.of(text.filteredOrEmpty()) : Optional.empty()",
            "public T get(final boolean filterEnabled)",
            "return filterEnabled ? this.filtered.orElse(this.raw) : this.raw;",
            "public <U> Filterable<U> map(final Function<T, U> function)",
            "return new Filterable<>(function.apply(this.raw), this.filtered.map(function));",
            "public <U> Optional<Filterable<U>> resolve(final Function<T, Optional<U>> function)",
            "Optional<U> newRaw = function.apply(this.raw);",
            "if (newRaw.isEmpty())",
            "Optional<U> newFiltered = function.apply(this.filtered.get());",
            "return newFiltered.isEmpty() ? Optional.empty() : Optional.of(new Filterable<>(newRaw.get(), newFiltered));",
        ] {
            assert!(
                FILTERABLE.contains(sentinel),
                "Filterable.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public record FilteredText(String raw, FilterMask mask)",
            "public static final FilteredText EMPTY = passThrough(\"\");",
            "public static FilteredText passThrough(final String message)",
            "return new FilteredText(message, FilterMask.PASS_THROUGH);",
            "public static FilteredText fullyFiltered(final String message)",
            "return new FilteredText(message, FilterMask.FULLY_FILTERED);",
            "public @Nullable String filtered()",
            "return this.mask.apply(this.raw);",
            "public String filteredOrEmpty()",
            "return Objects.requireNonNullElse(this.filtered(), \"\");",
            "public boolean isFiltered()",
            "return !this.mask.isEmpty();",
        ] {
            assert!(
                FILTERED_TEXT.contains(sentinel),
                "FilteredText.java is missing sentinel: {sentinel}"
            );
        }
    }
}
