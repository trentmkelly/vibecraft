//! Java `SignApplicator` item behavior shared by ink-sac style sign uses.
//!
//! Live sign block interaction still needs to route through this module from
//! the sign block use path; see `TODO(live-sign-applicator-use)`.

#![allow(
    dead_code,
    reason = "live sign block interaction is deferred behind TODO(live-sign-applicator-use)"
)]

use crate::block_entity::{SignBlockEntityModel, SignText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignApplicatorKind {
    GlowInkSac,
    InkSac,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignApplicatorResult {
    pub changed: bool,
    pub sound: Option<&'static str>,
}

/// Java `SignApplicator.canApplyToSign` default: the targeted text must have
/// a visible message for the player's current text-filtering setting.
pub fn can_apply_to_sign(text: &SignText, player_filters_text: bool) -> bool {
    text.has_message(player_filters_text)
}

/// Java `GlowInkSacItem.tryApplyToSign` and `InkSacItem.tryApplyToSign`.
pub fn try_apply_to_sign(
    sign: &mut SignBlockEntityModel,
    front_text: bool,
    kind: SignApplicatorKind,
) -> SignApplicatorResult {
    let (glowing, sound) = match kind {
        SignApplicatorKind::GlowInkSac => (true, "minecraft:item.glow_ink_sac.use"),
        SignApplicatorKind::InkSac => (false, "minecraft:item.ink_sac.use"),
    };
    let changed = sign.update_text(front_text, |mut text| {
        text.has_glowing_text = glowing;
        text
    });
    SignApplicatorResult {
        changed,
        sound: changed.then_some(sound),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_entity::{SignBlockEntityModel, SignLine};

    #[test]
    fn sign_applicator_default_gate_uses_filtered_or_raw_visible_message() {
        let mut sign = SignBlockEntityModel::default();
        assert!(!can_apply_to_sign(&sign.front_text, false));
        assert!(!can_apply_to_sign(&sign.front_text, true));

        sign.front_text.lines[0] = SignLine::new("raw message", "");
        assert!(can_apply_to_sign(&sign.front_text, false));
        assert!(!can_apply_to_sign(&sign.front_text, true));

        sign.front_text.lines[0] = SignLine::new("", "filtered message");
        assert!(!can_apply_to_sign(&sign.front_text, false));
        assert!(can_apply_to_sign(&sign.front_text, true));
    }

    #[test]
    fn glow_ink_sac_sets_glowing_text_and_reports_java_sound_only_on_change() {
        let mut sign = SignBlockEntityModel::default();
        sign.front_text.lines[0] = SignLine::new("hello", "hello");

        let applied =
            try_apply_to_sign(&mut sign, true, SignApplicatorKind::GlowInkSac);
        assert!(applied.changed);
        assert_eq!(applied.sound, Some("minecraft:item.glow_ink_sac.use"));
        assert!(sign.front_text.has_glowing_text);

        let unchanged =
            try_apply_to_sign(&mut sign, true, SignApplicatorKind::GlowInkSac);
        assert!(!unchanged.changed);
        assert_eq!(unchanged.sound, None);
    }

    #[test]
    fn ink_sac_clears_glowing_text_on_the_targeted_side_only() {
        let mut sign = SignBlockEntityModel::default();
        sign.front_text.has_glowing_text = true;
        sign.back_text.has_glowing_text = true;

        let applied = try_apply_to_sign(&mut sign, false, SignApplicatorKind::InkSac);
        assert!(applied.changed);
        assert_eq!(applied.sound, Some("minecraft:item.ink_sac.use"));
        assert!(sign.front_text.has_glowing_text);
        assert!(!sign.back_text.has_glowing_text);

        let unchanged = try_apply_to_sign(&mut sign, false, SignApplicatorKind::InkSac);
        assert!(!unchanged.changed);
        assert_eq!(unchanged.sound, None);
    }
}
