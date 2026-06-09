#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DyeColorModel {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

fn wool_item_by_dye() -> Vec<(DyeColorModel, &'static str)> {
    vec![
        (DyeColorModel::White, "minecraft:white_wool"),
        (DyeColorModel::Orange, "minecraft:orange_wool"),
        (DyeColorModel::Magenta, "minecraft:magenta_wool"),
        (DyeColorModel::LightBlue, "minecraft:light_blue_wool"),
        (DyeColorModel::Yellow, "minecraft:yellow_wool"),
        (DyeColorModel::Lime, "minecraft:lime_wool"),
        (DyeColorModel::Pink, "minecraft:pink_wool"),
        (DyeColorModel::Gray, "minecraft:gray_wool"),
        (DyeColorModel::LightGray, "minecraft:light_gray_wool"),
        (DyeColorModel::Cyan, "minecraft:cyan_wool"),
        (DyeColorModel::Purple, "minecraft:purple_wool"),
        (DyeColorModel::Blue, "minecraft:blue_wool"),
        (DyeColorModel::Brown, "minecraft:brown_wool"),
        (DyeColorModel::Green, "minecraft:green_wool"),
        (DyeColorModel::Red, "minecraft:red_wool"),
        (DyeColorModel::Black, "minecraft:black_wool"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loot_data_wool_item_by_dye_matches_java_enum_map() {
        let mapping = wool_item_by_dye();
        assert_eq!(mapping.len(), 16);
        assert_eq!(
            mapping,
            vec![
                (DyeColorModel::White, "minecraft:white_wool"),
                (DyeColorModel::Orange, "minecraft:orange_wool"),
                (DyeColorModel::Magenta, "minecraft:magenta_wool"),
                (DyeColorModel::LightBlue, "minecraft:light_blue_wool"),
                (DyeColorModel::Yellow, "minecraft:yellow_wool"),
                (DyeColorModel::Lime, "minecraft:lime_wool"),
                (DyeColorModel::Pink, "minecraft:pink_wool"),
                (DyeColorModel::Gray, "minecraft:gray_wool"),
                (DyeColorModel::LightGray, "minecraft:light_gray_wool"),
                (DyeColorModel::Cyan, "minecraft:cyan_wool"),
                (DyeColorModel::Purple, "minecraft:purple_wool"),
                (DyeColorModel::Blue, "minecraft:blue_wool"),
                (DyeColorModel::Brown, "minecraft:brown_wool"),
                (DyeColorModel::Green, "minecraft:green_wool"),
                (DyeColorModel::Red, "minecraft:red_wool"),
                (DyeColorModel::Black, "minecraft:black_wool"),
            ]
        );
    }
}
