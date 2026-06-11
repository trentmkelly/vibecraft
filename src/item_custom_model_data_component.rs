#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct CustomModelData {
    pub floats: Vec<f32>,
    pub flags: Vec<bool>,
    pub strings: Vec<String>,
    pub colors: Vec<u32>,
}

impl CustomModelData {
    pub fn empty() -> Self {
        Self {
            floats: Vec::new(),
            flags: Vec::new(),
            strings: Vec::new(),
            colors: Vec::new(),
        }
    }

    pub fn new(
        floats: Vec<f32>,
        flags: Vec<bool>,
        strings: Vec<String>,
        colors: Vec<u32>,
    ) -> Self {
        Self {
            floats,
            flags,
            strings,
            colors: colors.into_iter().map(|color| color & 0x00FF_FFFF).collect(),
        }
    }

    pub fn get_float(&self, index: i32) -> Option<f32> {
        get_safe(&self.floats, index).copied()
    }

    pub fn get_boolean(&self, index: i32) -> Option<bool> {
        get_safe(&self.flags, index).copied()
    }

    pub fn get_string(&self, index: i32) -> Option<&str> {
        get_safe(&self.strings, index).map(String::as_str)
    }

    pub fn get_color(&self, index: i32) -> Option<u32> {
        get_safe(&self.colors, index).copied()
    }
}

fn get_safe<T>(values: &[T], index: i32) -> Option<&T> {
    usize::try_from(index)
        .ok()
        .and_then(|index| values.get(index))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSTOM_MODEL_DATA_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/CustomModelData.java"
    );

    #[test]
    fn custom_model_data_lists_empty_constant_and_safe_getters_match_java() {
        for sentinel in [
            "public record CustomModelData(List<Float> floats, List<Boolean> flags, List<String> strings, List<Integer> colors)",
            "public static final CustomModelData EMPTY = new CustomModelData(List.of(), List.of(), List.of(), List.of());",
            "Codec.FLOAT.listOf().optionalFieldOf(\"floats\", List.of()).forGetter(CustomModelData::floats)",
            "Codec.BOOL.listOf().optionalFieldOf(\"flags\", List.of()).forGetter(CustomModelData::flags)",
            "Codec.STRING.listOf().optionalFieldOf(\"strings\", List.of()).forGetter(CustomModelData::strings)",
            "ExtraCodecs.RGB_COLOR_CODEC.listOf().optionalFieldOf(\"colors\", List.of()).forGetter(CustomModelData::colors)",
            "private static <T> @Nullable T getSafe(final List<T> values, final int index)",
            "return index >= 0 && index < values.size() ? values.get(index) : null;",
        ] {
            assert!(
                CUSTOM_MODEL_DATA_JAVA.contains(sentinel),
                "missing CustomModelData sentinel {sentinel}"
            );
        }

        assert_eq!(CustomModelData::empty(), CustomModelData::new(vec![], vec![], vec![], vec![]));

        let data = CustomModelData::new(
            vec![1.5, 2.25],
            vec![true, false],
            vec!["alpha".to_string(), "beta".to_string()],
            vec![0x123456, 0xFFABCDEF],
        );
        assert_eq!(data.get_float(0), Some(1.5));
        assert_eq!(data.get_float(2), None);
        assert_eq!(data.get_float(-1), None);
        assert_eq!(data.get_boolean(0), Some(true));
        assert_eq!(data.get_boolean(1), Some(false));
        assert_eq!(data.get_string(1), Some("beta"));
        assert_eq!(data.get_string(-1), None);
        assert_eq!(data.get_color(0), Some(0x123456));
        assert_eq!(data.get_color(1), Some(0xABCDEF));
        assert_eq!(data.get_color(9), None);
    }
}
