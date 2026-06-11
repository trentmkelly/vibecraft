#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapItemColor {
    pub rgb: i32,
}

impl MapItemColor {
    pub const DEFAULT: Self = Self { rgb: 4_603_950 };

    pub fn new(rgb: i32) -> Self {
        Self { rgb }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_properties::MapPostProcessing;

    const MAP_ITEM_COLOR_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/MapItemColor.java"
    );
    const MAP_POST_PROCESSING_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/world/item/component/MapPostProcessing.java"
    );

    #[test]
    fn map_item_color_default_and_codecs_match_java() {
        for sentinel in [
            "public record MapItemColor(int rgb)",
            "Codec.INT.xmap(MapItemColor::new, MapItemColor::rgb)",
            "ByteBufCodecs.INT.map(MapItemColor::new, MapItemColor::rgb)",
            "public static final MapItemColor DEFAULT = new MapItemColor(4603950);",
        ] {
            assert!(
                MAP_ITEM_COLOR_JAVA.contains(sentinel),
                "missing MapItemColor sentinel {sentinel}"
            );
        }

        assert_eq!(MapItemColor::DEFAULT, MapItemColor { rgb: 4_603_950 });
        assert_eq!(MapItemColor::new(0x123456).rgb, 0x123456);
    }

    #[test]
    fn map_post_processing_ids_zero_fallback_and_stream_codec_match_java() {
        for sentinel in [
            "LOCK(0)",
            "SCALE(1)",
            "ByIdMap.OutOfBoundsStrategy.ZERO",
            "ByteBufCodecs.idMapper(ID_MAP, MapPostProcessing::id)",
        ] {
            assert!(
                MAP_POST_PROCESSING_JAVA.contains(sentinel),
                "missing MapPostProcessing sentinel {sentinel}"
            );
        }

        assert_eq!(MapPostProcessing::Lock.id(), 0);
        assert_eq!(MapPostProcessing::Scale.id(), 1);
        assert_eq!(MapPostProcessing::by_id(0), MapPostProcessing::Lock);
        assert_eq!(MapPostProcessing::by_id(1), MapPostProcessing::Scale);
        assert_eq!(MapPostProcessing::by_id(-1), MapPostProcessing::Lock);
        assert_eq!(MapPostProcessing::by_id(99), MapPostProcessing::Lock);
    }
}
