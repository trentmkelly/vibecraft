#![allow(dead_code)]

use crate::block_behavior::InteractionResult;

pub const DEMO_DAYS: i32 = 5;
pub const TOTAL_PLAY_TICKS: i64 = 120_500;

#[derive(Debug, Clone, PartialEq)]
pub enum DemoModeEffect {
    GameEvent { event: DemoGameEvent, param: f32 },
    SystemMessage { translation_key: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoGameEvent {
    DemoEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoActionDecision {
    ForwardToServerPlayerGameMode,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoInteractionDecision {
    ForwardToServerPlayerGameMode,
    Blocked(InteractionResult),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DemoModeModel {
    displayed_intro: bool,
    demo_has_ended: bool,
    demo_ended_reminder: i32,
    game_mode_ticks: i32,
}

impl DemoModeModel {
    pub fn tick(&mut self, game_time: i64) -> Vec<DemoModeEffect> {
        self.game_mode_ticks += 1;
        let mut effects = Vec::new();
        let day = game_time / 24_000 + 1;

        if !self.displayed_intro && self.game_mode_ticks > 20 {
            self.displayed_intro = true;
            effects.push(demo_event(0.0));
        }

        self.demo_has_ended = game_time > TOTAL_PLAY_TICKS;
        if self.demo_has_ended {
            self.demo_ended_reminder += 1;
        }

        if game_time % 24_000 == 500 {
            if day <= 6 {
                if day == 6 {
                    effects.push(demo_event(104.0));
                } else {
                    effects.push(system_message(format!("demo.day.{day}")));
                }
            }
        } else if day == 1 {
            if game_time == 100 {
                effects.push(demo_event(101.0));
            } else if game_time == 175 {
                effects.push(demo_event(102.0));
            } else if game_time == 250 {
                effects.push(demo_event(103.0));
            }
        } else if day == 5 && game_time % 24_000 == 22_000 {
            effects.push(system_message("demo.day.warning"));
        }

        effects
    }

    pub fn handle_block_break_action(&mut self) -> (DemoActionDecision, Vec<DemoModeEffect>) {
        if self.demo_has_ended {
            (
                DemoActionDecision::Blocked,
                self.output_demo_reminder(),
            )
        } else {
            (
                DemoActionDecision::ForwardToServerPlayerGameMode,
                Vec::new(),
            )
        }
    }

    pub fn use_item(&mut self) -> (DemoInteractionDecision, Vec<DemoModeEffect>) {
        if self.demo_has_ended {
            (
                DemoInteractionDecision::Blocked(InteractionResult::Pass),
                self.output_demo_reminder(),
            )
        } else {
            (
                DemoInteractionDecision::ForwardToServerPlayerGameMode,
                Vec::new(),
            )
        }
    }

    pub fn use_item_on(&mut self) -> (DemoInteractionDecision, Vec<DemoModeEffect>) {
        self.use_item()
    }

    pub fn demo_has_ended(&self) -> bool {
        self.demo_has_ended
    }

    pub fn demo_ended_reminder(&self) -> i32 {
        self.demo_ended_reminder
    }

    pub fn game_mode_ticks(&self) -> i32 {
        self.game_mode_ticks
    }

    fn output_demo_reminder(&mut self) -> Vec<DemoModeEffect> {
        if self.demo_ended_reminder > 100 {
            self.demo_ended_reminder = 0;
            vec![system_message("demo.reminder")]
        } else {
            Vec::new()
        }
    }
}

fn demo_event(param: f32) -> DemoModeEffect {
    DemoModeEffect::GameEvent {
        event: DemoGameEvent::DemoEvent,
        param,
    }
}

fn system_message(translation_key: impl Into<String>) -> DemoModeEffect {
    DemoModeEffect::SystemMessage {
        translation_key: translation_key.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_constants_match_java() {
        assert_eq!(DEMO_DAYS, 5);
        assert_eq!(TOTAL_PLAY_TICKS, 120_500);
    }

    #[test]
    fn tick_sends_intro_once_after_twenty_game_mode_ticks() {
        let mut demo = DemoModeModel::default();

        for time in 0..20 {
            assert!(demo.tick(time).is_empty());
        }

        assert_eq!(demo.tick(20), vec![demo_event(0.0)]);
        assert!(demo.tick(21).is_empty());
        assert_eq!(demo.game_mode_ticks(), 22);
    }

    #[test]
    fn tick_sends_java_demo_tutorial_events_and_day_messages() {
        let mut demo = DemoModeModel {
            displayed_intro: true,
            ..DemoModeModel::default()
        };

        assert_eq!(demo.tick(100), vec![demo_event(101.0)]);
        assert_eq!(demo.tick(175), vec![demo_event(102.0)]);
        assert_eq!(demo.tick(250), vec![demo_event(103.0)]);
        assert_eq!(demo.tick(500), vec![system_message("demo.day.1")]);
        assert_eq!(demo.tick(24_500), vec![system_message("demo.day.2")]);
        assert_eq!(demo.tick(48_500), vec![system_message("demo.day.3")]);
        assert_eq!(demo.tick(72_500), vec![system_message("demo.day.4")]);
        assert_eq!(demo.tick(96_500), vec![system_message("demo.day.5")]);
        assert_eq!(demo.tick(120_500), vec![demo_event(104.0)]);
        assert_eq!(demo.tick(118_000), vec![system_message("demo.day.warning")]);
    }

    #[test]
    fn demo_end_and_reminder_counter_match_java_strict_thresholds() {
        let mut demo = DemoModeModel {
            displayed_intro: true,
            ..DemoModeModel::default()
        };

        assert!(demo.tick(TOTAL_PLAY_TICKS).contains(&demo_event(104.0)));
        assert!(!demo.demo_has_ended());
        assert_eq!(demo.demo_ended_reminder(), 0);

        assert!(demo.tick(TOTAL_PLAY_TICKS + 1).is_empty());
        assert!(demo.demo_has_ended());
        assert_eq!(demo.demo_ended_reminder(), 1);

        let (decision, effects) = demo.handle_block_break_action();
        assert_eq!(decision, DemoActionDecision::Blocked);
        assert!(effects.is_empty());

        for time in TOTAL_PLAY_TICKS + 2..=TOTAL_PLAY_TICKS + 101 {
            assert!(demo.tick(time).is_empty());
        }

        let (result, effects) = demo.use_item();
        assert_eq!(result, DemoInteractionDecision::Blocked(InteractionResult::Pass));
        assert_eq!(effects, vec![system_message("demo.reminder")]);
        assert_eq!(demo.demo_ended_reminder(), 0);
    }

    #[test]
    fn interactions_forward_before_end_and_pass_after_end() {
        let mut demo = DemoModeModel::default();

        assert_eq!(
            demo.handle_block_break_action(),
            (
                DemoActionDecision::ForwardToServerPlayerGameMode,
                Vec::new()
            )
        );
        assert_eq!(
            demo.use_item(),
            (
                DemoInteractionDecision::ForwardToServerPlayerGameMode,
                Vec::new()
            )
        );
        assert_eq!(
            demo.use_item_on(),
            (
                DemoInteractionDecision::ForwardToServerPlayerGameMode,
                Vec::new()
            )
        );

        demo.tick(TOTAL_PLAY_TICKS + 1);

        assert_eq!(
            demo.handle_block_break_action(),
            (DemoActionDecision::Blocked, Vec::new())
        );
        assert_eq!(
            demo.use_item(),
            (
                DemoInteractionDecision::Blocked(InteractionResult::Pass),
                Vec::new()
            )
        );
        assert_eq!(
            demo.use_item_on(),
            (
                DemoInteractionDecision::Blocked(InteractionResult::Pass),
                Vec::new()
            )
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn demo_mode_source_matches_java_26_1_2() {
        const DEMO_MODE: &str =
            vibecraft_java_source!("/net/minecraft/server/level/DemoMode.java");

        for sentinel in [
            "public class DemoMode extends ServerPlayerGameMode",
            "public static final int DEMO_DAYS = 5;",
            "public static final int TOTAL_PLAY_TICKS = 120500;",
            "private boolean displayedIntro;",
            "private boolean demoHasEnded;",
            "private int demoEndedReminder;",
            "private int gameModeTicks;",
            "this.gameModeTicks++;",
            "long day = time / 24000L + 1L;",
            "if (!this.displayedIntro && this.gameModeTicks > 20)",
            "new ClientboundGameEventPacket(ClientboundGameEventPacket.DEMO_EVENT, 0.0F)",
            "this.demoHasEnded = time > 120500L;",
            "this.demoEndedReminder++;",
            "if (time % 24000L == 500L)",
            "this.player.connection.send(new ClientboundGameEventPacket(ClientboundGameEventPacket.DEMO_EVENT, 104.0F));",
            "this.player.sendSystemMessage(Component.translatable(\"demo.day.\" + day));",
            "if (time == 100L)",
            "ClientboundGameEventPacket.DEMO_EVENT, 101.0F",
            "time == 175L",
            "ClientboundGameEventPacket.DEMO_EVENT, 102.0F",
            "time == 250L",
            "ClientboundGameEventPacket.DEMO_EVENT, 103.0F",
            "day == 5L && time % 24000L == 22000L",
            "Component.translatable(\"demo.day.warning\")",
            "if (this.demoEndedReminder > 100)",
            "Component.translatable(\"demo.reminder\")",
            "this.demoEndedReminder = 0;",
            "if (this.demoHasEnded)",
            "return InteractionResult.PASS;",
        ] {
            assert!(
                DEMO_MODE.contains(sentinel),
                "DemoMode.java is missing sentinel: {sentinel}"
            );
        }
    }
}
