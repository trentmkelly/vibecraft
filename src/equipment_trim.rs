#![allow(dead_code)]

pub const DYE_COLORS: &[&str] = &[
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];

pub const BASE_EQUIPMENT_ASSETS: &[&str] = &[
    "minecraft:leather",
    "minecraft:copper",
    "minecraft:chainmail",
    "minecraft:iron",
    "minecraft:gold",
    "minecraft:diamond",
    "minecraft:turtle_scute",
    "minecraft:netherite",
    "minecraft:armadillo_scute",
    "minecraft:elytra",
    "minecraft:saddle",
    "minecraft:trader_llama",
    "minecraft:trader_llama_baby",
];

pub const TRIM_MATERIAL_ORDER: &[&str] = &[
    "minecraft:quartz",
    "minecraft:iron",
    "minecraft:netherite",
    "minecraft:redstone",
    "minecraft:copper",
    "minecraft:gold",
    "minecraft:emerald",
    "minecraft:diamond",
    "minecraft:lapis",
    "minecraft:amethyst",
    "minecraft:resin",
];

pub const TRIM_PATTERN_ORDER: &[&str] = &[
    "minecraft:sentry",
    "minecraft:dune",
    "minecraft:coast",
    "minecraft:wild",
    "minecraft:ward",
    "minecraft:eye",
    "minecraft:vex",
    "minecraft:tide",
    "minecraft:snout",
    "minecraft:rib",
    "minecraft:spire",
    "minecraft:wayfinder",
    "minecraft:shaper",
    "minecraft:silence",
    "minecraft:raiser",
    "minecraft:host",
    "minecraft:flow",
    "minecraft:bolt",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimMaterialDef {
    pub id: &'static str,
    pub color: u32,
    pub asset_group: &'static str,
}

pub const TRIM_MATERIALS: &[TrimMaterialDef] = &[
    TrimMaterialDef {
        id: "minecraft:quartz",
        color: 14931140,
        asset_group: "quartz",
    },
    TrimMaterialDef {
        id: "minecraft:iron",
        color: 15527148,
        asset_group: "iron",
    },
    TrimMaterialDef {
        id: "minecraft:netherite",
        color: 6445145,
        asset_group: "netherite",
    },
    TrimMaterialDef {
        id: "minecraft:redstone",
        color: 9901575,
        asset_group: "redstone",
    },
    TrimMaterialDef {
        id: "minecraft:copper",
        color: 11823181,
        asset_group: "copper",
    },
    TrimMaterialDef {
        id: "minecraft:gold",
        color: 14594349,
        asset_group: "gold",
    },
    TrimMaterialDef {
        id: "minecraft:emerald",
        color: 1155126,
        asset_group: "emerald",
    },
    TrimMaterialDef {
        id: "minecraft:diamond",
        color: 7269586,
        asset_group: "diamond",
    },
    TrimMaterialDef {
        id: "minecraft:lapis",
        color: 4288151,
        asset_group: "lapis",
    },
    TrimMaterialDef {
        id: "minecraft:amethyst",
        color: 10116294,
        asset_group: "amethyst",
    },
    TrimMaterialDef {
        id: "minecraft:resin",
        color: 16545810,
        asset_group: "resin",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimPatternDef {
    pub id: &'static str,
    pub asset_id: &'static str,
    pub decal: bool,
    pub template_item: &'static str,
}

pub const TRIM_PATTERNS: &[TrimPatternDef] = &[
    trim("sentry"),
    trim("dune"),
    trim("coast"),
    trim("wild"),
    trim("ward"),
    trim("eye"),
    trim("vex"),
    trim("tide"),
    trim("snout"),
    trim("rib"),
    trim("spire"),
    trim("wayfinder"),
    trim("shaper"),
    trim("silence"),
    trim("raiser"),
    trim("host"),
    trim("flow"),
    trim("bolt"),
];

const fn trim(name: &'static str) -> TrimPatternDef {
    TrimPatternDef {
        id: name,
        asset_id: name,
        decal: false,
        template_item: name,
    }
}

pub fn equipment_asset_ids() -> Vec<String> {
    let mut ids = BASE_EQUIPMENT_ASSETS
        .iter()
        .map(|id| (*id).to_string())
        .collect::<Vec<_>>();
    ids.extend(
        DYE_COLORS
            .iter()
            .map(|color| format!("minecraft:{color}_carpet")),
    );
    ids.extend(
        DYE_COLORS
            .iter()
            .map(|color| format!("minecraft:{color}_harness")),
    );
    ids
}

pub fn trim_material(id: &str) -> Option<&'static TrimMaterialDef> {
    let id = normalize(id);
    TRIM_MATERIALS.iter().find(|material| material.id == id)
}

pub fn trim_pattern(id: &str) -> Option<&'static TrimPatternDef> {
    let id = id.strip_prefix("minecraft:").unwrap_or(id);
    TRIM_PATTERNS.iter().find(|pattern| pattern.id == id)
}

pub fn trim_display_name(pattern: &TrimPatternDef, material: &TrimMaterialDef) -> String {
    format!(
        "trim_pattern.minecraft.{} & trim_material.minecraft.{}",
        pattern.id, material.asset_group
    )
}

pub fn spawn_armor_trims_count(
    pattern_count: usize,
    material_count: usize,
    armor_piece_count: usize,
) -> usize {
    pattern_count * material_count * armor_piece_count
}

pub fn smithing_trim_recipe_id(template_item: &str) -> String {
    format!("minecraft:{template_item}_armor_trim_smithing_template_smithing_trim")
}

fn normalize(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equipment_assets_include_base_carpet_and_harness_assets() {
        let ids = equipment_asset_ids();
        assert_eq!(BASE_EQUIPMENT_ASSETS.len(), 13);
        assert_eq!(DYE_COLORS.len(), 16);
        assert_eq!(ids.len(), 45);
        assert!(ids.contains(&"minecraft:diamond".to_string()));
        assert!(ids.contains(&"minecraft:red_carpet".to_string()));
        assert!(ids.contains(&"minecraft:black_harness".to_string()));
    }

    #[test]
    fn trim_materials_match_bootstrap_order_colors_and_asset_groups() {
        assert_eq!(TRIM_MATERIALS.len(), 11);
        assert_eq!(
            TRIM_MATERIAL_ORDER,
            TRIM_MATERIALS.iter().map(|m| m.id).collect::<Vec<_>>()
        );
        assert_eq!(trim_material("quartz").unwrap().color, 14931140);
        assert_eq!(
            trim_material("minecraft:resin").unwrap().asset_group,
            "resin"
        );
    }

    #[test]
    fn trim_patterns_match_spawn_command_order_and_recipe_templates() {
        assert_eq!(TRIM_PATTERNS.len(), 18);
        assert_eq!(TRIM_PATTERN_ORDER[0], "minecraft:sentry");
        assert_eq!(TRIM_PATTERN_ORDER[17], "minecraft:bolt");
        assert_eq!(trim_pattern("minecraft:flow").unwrap().asset_id, "flow");
        assert!(!trim_pattern("bolt").unwrap().decal);
        assert_eq!(
            smithing_trim_recipe_id("bolt"),
            "minecraft:bolt_armor_trim_smithing_template_smithing_trim"
        );
    }

    #[test]
    fn armor_trim_components_and_spawn_grid_follow_vanilla_shapes() {
        let material = trim_material("copper").unwrap();
        let pattern = trim_pattern("flow").unwrap();
        assert_eq!(
            trim_display_name(pattern, material),
            "trim_pattern.minecraft.flow & trim_material.minecraft.copper"
        );
        assert_eq!(
            spawn_armor_trims_count(TRIM_PATTERNS.len(), TRIM_MATERIALS.len(), 25),
            18 * 11 * 25
        );
        assert_eq!(
            spawn_armor_trims_count(1, TRIM_MATERIALS.len(), 25),
            11 * 25
        );
    }
}
