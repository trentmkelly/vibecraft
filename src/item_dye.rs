//! Java `DyeItem`: dye components plus sign and sheep recolor behavior.
//!
//! The model covers the Java item decisions. Live sign block routing and live
//! mob interaction still need to call this module; see
//! `TODO(live-sign-applicator-use)` and `TODO(live-dye-sheep-interaction)`.

#![allow(
    dead_code,
    reason = "live sign/mob item dispatch is deferred behind TODO(live-sign-applicator-use) and TODO(live-dye-sheep-interaction)"
)]

use crate::block_entity::SignBlockEntityModel;
use crate::map_state::DyeColor;
use crate::mob_interaction::DyeColorModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DyeSignResult {
    pub changed: bool,
    pub sound: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DyeSheepResult {
    Pass,
    Success { color: DyeColorModel, consumes: u32 },
}

/// Java `ItemStack.get(DataComponents.DYE)` for vanilla dye items.
pub fn dye_color_for_item(item_id: &str) -> Option<DyeColor> {
    Some(match item_id {
        "minecraft:white_dye" | "minecraft:bone_meal" => DyeColor::White,
        "minecraft:orange_dye" => DyeColor::Orange,
        "minecraft:magenta_dye" => DyeColor::Magenta,
        "minecraft:light_blue_dye" => DyeColor::LightBlue,
        "minecraft:yellow_dye" => DyeColor::Yellow,
        "minecraft:lime_dye" => DyeColor::Lime,
        "minecraft:pink_dye" => DyeColor::Pink,
        "minecraft:gray_dye" => DyeColor::Gray,
        "minecraft:light_gray_dye" => DyeColor::LightGray,
        "minecraft:cyan_dye" => DyeColor::Cyan,
        "minecraft:purple_dye" => DyeColor::Purple,
        "minecraft:blue_dye" | "minecraft:lapis_lazuli" => DyeColor::Blue,
        "minecraft:brown_dye" | "minecraft:cocoa_beans" => DyeColor::Brown,
        "minecraft:green_dye" => DyeColor::Green,
        "minecraft:red_dye" => DyeColor::Red,
        "minecraft:black_dye" | "minecraft:ink_sac" => DyeColor::Black,
        _ => return None,
    })
}

pub fn sheep_dye_color_for_item(item_id: &str) -> Option<DyeColorModel> {
    Some(match dye_color_for_item(item_id)? {
        DyeColor::White => DyeColorModel::White,
        DyeColor::Orange => DyeColorModel::Orange,
        DyeColor::Magenta => DyeColorModel::Magenta,
        DyeColor::LightBlue => DyeColorModel::LightBlue,
        DyeColor::Yellow => DyeColorModel::Yellow,
        DyeColor::Lime => DyeColorModel::Lime,
        DyeColor::Pink => DyeColorModel::Pink,
        DyeColor::Gray => DyeColorModel::Gray,
        DyeColor::LightGray => DyeColorModel::LightGray,
        DyeColor::Cyan => DyeColorModel::Cyan,
        DyeColor::Purple => DyeColorModel::Purple,
        DyeColor::Blue => DyeColorModel::Blue,
        DyeColor::Brown => DyeColorModel::Brown,
        DyeColor::Green => DyeColorModel::Green,
        DyeColor::Red => DyeColorModel::Red,
        DyeColor::Black => DyeColorModel::Black,
    })
}

/// Java `DyeItem.tryApplyToSign`: set the targeted face color to the item's
/// DYE component and play `SoundEvents.DYE_USE` only when the color changed.
pub fn try_apply_to_sign(
    sign: &mut SignBlockEntityModel,
    front_text: bool,
    item_id: &str,
) -> DyeSignResult {
    let Some(color) = dye_color_for_item(item_id) else {
        return DyeSignResult {
            changed: false,
            sound: None,
        };
    };
    let changed = sign.update_text(front_text, |mut text| {
        text.color = color;
        text
    });
    DyeSignResult {
        changed,
        sound: changed.then_some("minecraft:item.dye.use"),
    }
}

/// Java `DyeItem.interactLivingEntity` sheep gate.
pub fn interact_sheep(
    item_id: &str,
    sheep_color: DyeColorModel,
    sheep_alive: bool,
    sheep_sheared: bool,
) -> DyeSheepResult {
    if !sheep_alive || sheep_sheared {
        return DyeSheepResult::Pass;
    }
    let Some(color) = sheep_dye_color_for_item(item_id) else {
        return DyeSheepResult::Pass;
    };
    if color == sheep_color {
        return DyeSheepResult::Pass;
    }
    DyeSheepResult::Success { color, consumes: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_entity::{SignBlockEntityModel, SignLine};

    #[test]
    fn dye_color_for_item_matches_java_dye_components_and_legacy_aliases() {
        assert_eq!(dye_color_for_item("minecraft:white_dye"), Some(DyeColor::White));
        assert_eq!(dye_color_for_item("minecraft:bone_meal"), Some(DyeColor::White));
        assert_eq!(dye_color_for_item("minecraft:blue_dye"), Some(DyeColor::Blue));
        assert_eq!(dye_color_for_item("minecraft:lapis_lazuli"), Some(DyeColor::Blue));
        assert_eq!(dye_color_for_item("minecraft:brown_dye"), Some(DyeColor::Brown));
        assert_eq!(dye_color_for_item("minecraft:cocoa_beans"), Some(DyeColor::Brown));
        assert_eq!(dye_color_for_item("minecraft:black_dye"), Some(DyeColor::Black));
        assert_eq!(dye_color_for_item("minecraft:ink_sac"), Some(DyeColor::Black));
        assert_eq!(dye_color_for_item("minecraft:stick"), None);
    }

    #[test]
    fn dye_item_recolors_targeted_sign_face_and_reports_java_sound_only_on_change() {
        let mut sign = SignBlockEntityModel::default();
        sign.front_text.lines[0] = SignLine::new("front", "front");
        sign.back_text.lines[0] = SignLine::new("back", "back");

        let applied = try_apply_to_sign(&mut sign, true, "minecraft:red_dye");
        assert!(applied.changed);
        assert_eq!(applied.sound, Some("minecraft:item.dye.use"));
        assert_eq!(sign.front_text.color, DyeColor::Red);
        assert_eq!(sign.back_text.color, DyeColor::Black);

        let unchanged = try_apply_to_sign(&mut sign, true, "minecraft:red_dye");
        assert!(!unchanged.changed);
        assert_eq!(unchanged.sound, None);
        assert_eq!(
            try_apply_to_sign(&mut sign, true, "minecraft:stick"),
            DyeSignResult {
                changed: false,
                sound: None,
            }
        );
    }

    #[test]
    fn dye_item_sheep_interaction_requires_alive_unsheared_different_color() {
        assert_eq!(
            interact_sheep("minecraft:blue_dye", DyeColorModel::White, true, false),
            DyeSheepResult::Success {
                color: DyeColorModel::Blue,
                consumes: 1,
            }
        );
        assert_eq!(
            interact_sheep("minecraft:blue_dye", DyeColorModel::Blue, true, false),
            DyeSheepResult::Pass
        );
        assert_eq!(
            interact_sheep("minecraft:blue_dye", DyeColorModel::White, false, false),
            DyeSheepResult::Pass
        );
        assert_eq!(
            interact_sheep("minecraft:blue_dye", DyeColorModel::White, true, true),
            DyeSheepResult::Pass
        );
        assert_eq!(
            interact_sheep("minecraft:stick", DyeColorModel::White, true, false),
            DyeSheepResult::Pass
        );
    }
}
