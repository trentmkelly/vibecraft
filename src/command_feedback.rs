#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument, Style, TextColor};
use crate::command::CommandResult;
use crate::localization_keys::{assert_known_emitted_key, MessageKeyFamily};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFeedbackContext {
    pub source_name: String,
    pub source_accepts_feedback: bool,
    pub source_silent: bool,
    pub source_informs_admins: bool,
    pub source_is_server: bool,
    pub send_command_feedback_rule: bool,
    pub log_admin_commands_rule: bool,
    pub admins: Vec<String>,
}

impl CommandFeedbackContext {
    pub fn console() -> Self {
        Self {
            source_name: "Server".to_string(),
            source_accepts_feedback: true,
            source_silent: false,
            source_informs_admins: true,
            source_is_server: true,
            send_command_feedback_rule: true,
            log_admin_commands_rule: true,
            admins: Vec::new(),
        }
    }

    pub fn with_console_broadcast(mut self, enabled: bool) -> Self {
        self.source_name = "Server".to_string();
        self.source_is_server = true;
        self.source_informs_admins = enabled;
        self
    }

    pub fn with_rcon_broadcast(mut self, enabled: bool) -> Self {
        self.source_name = "Rcon".to_string();
        self.source_is_server = false;
        self.source_informs_admins = enabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackArgument {
    pub name: &'static str,
    pub value: ComponentArgument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackTarget {
    Source,
    Admin(String),
    Log,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFeedbackPacket {
    pub target: FeedbackTarget,
    pub overlay: bool,
    pub component: Component,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFeedbackPlan {
    pub success_count: i32,
    pub packets: Vec<CommandFeedbackPacket>,
}

pub fn format_command_feedback(
    result: &CommandResult,
    args: &[FeedbackArgument],
) -> Result<Component, String> {
    let key = assert_known_emitted_key(result.feedback_key)?;
    if key.family != MessageKeyFamily::Command {
        return Err(format!(
            "feedback key is not a command key: {}",
            result.feedback_key
        ));
    }
    let ordered_args = key
        .args
        .iter()
        .map(|name| {
            args.iter()
                .find(|arg| &arg.name == name)
                .map(|arg| arg.value.clone())
                .unwrap_or_else(|| ComponentArgument::String(format!("<{name}>")))
        })
        .collect();
    Ok(Component::translatable(result.feedback_key, ordered_args))
}

pub fn route_command_feedback(
    result: &CommandResult,
    args: &[FeedbackArgument],
    context: &CommandFeedbackContext,
) -> Result<CommandFeedbackPlan, String> {
    let component = format_command_feedback(result, args)?;
    let mut packets = Vec::new();
    if context.source_accepts_feedback && !context.source_silent {
        packets.push(CommandFeedbackPacket {
            target: FeedbackTarget::Source,
            overlay: false,
            component: component.clone(),
        });
    }
    if result.broadcast_to_admins && context.source_informs_admins && !context.source_silent {
        let admin_component = admin_broadcast_component(&context.source_name, component.clone());
        if context.send_command_feedback_rule {
            for admin in &context.admins {
                if admin != &context.source_name {
                    packets.push(CommandFeedbackPacket {
                        target: FeedbackTarget::Admin(admin.clone()),
                        overlay: false,
                        component: admin_component.clone(),
                    });
                }
            }
        }
        if !context.source_is_server && context.log_admin_commands_rule {
            packets.push(CommandFeedbackPacket {
                target: FeedbackTarget::Log,
                overlay: false,
                component: admin_component,
            });
        }
    }
    Ok(CommandFeedbackPlan {
        success_count: result.success_count,
        packets,
    })
}

pub fn arg_string(name: &'static str, value: impl Into<String>) -> FeedbackArgument {
    FeedbackArgument {
        name,
        value: ComponentArgument::String(value.into()),
    }
}

pub fn arg_number(name: &'static str, value: i32) -> FeedbackArgument {
    FeedbackArgument {
        name,
        value: ComponentArgument::Number(value),
    }
}

pub fn arg_component(name: &'static str, value: Component) -> FeedbackArgument {
    FeedbackArgument {
        name,
        value: ComponentArgument::Component(Box::new(value)),
    }
}

pub fn feedback_style(success: bool) -> Style {
    let color_name = if success { "gray" } else { "red" };
    let Some(color) = TextColor::parse(color_name) else {
        unreachable!("built-in feedback color {color_name} must be a known legacy color");
    };
    Style::empty().with_color(color)
}

fn admin_broadcast_component(source_name: &str, component: Component) -> Component {
    Component::translatable(
        "chat.type.admin",
        vec![
            ComponentArgument::String(source_name.to_string()),
            ComponentArgument::Component(Box::new(component)),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_component::{ResolutionContext, TranslationTable};

    fn result(key: &'static str, broadcast_to_admins: bool) -> CommandResult {
        CommandResult {
            success_count: 1,
            feedback_key: key,
            broadcast_to_admins,
        }
    }

    fn translations() -> TranslationTable {
        TranslationTable::default()
            .with("commands.kick.success", "Kicked %s: %s")
            .with(
                "commands.list.players",
                "There are %s of a max of %s players online: %s",
            )
            .with("commands.save.success", "Saved the game")
            .with("chat.type.admin", "[%s: %s]")
    }

    #[test]
    fn formats_command_feedback_components_with_vanilla_argument_order() {
        let component = format_command_feedback(
            &result("commands.kick.success", true),
            &[
                arg_string("reason", "Done"),
                arg_component("player", Component::literal("Steve")),
            ],
        )
        .unwrap();
        assert_eq!(
            component.render_plain(&translations(), &ResolutionContext::default()),
            "Kicked Steve: Done"
        );
        assert_eq!(
            component.to_json(),
            "{\"translate\":\"commands.kick.success\",\"with\":[{\"text\":\"Steve\"},\"Done\"]}"
        );
    }

    #[test]
    fn missing_feedback_args_are_visible_placeholders_not_silent_reorders() {
        let component = format_command_feedback(
            &result("commands.list.players", false),
            &[arg_number("count", 2)],
        )
        .unwrap();
        assert_eq!(
            component.render_plain(&translations(), &ResolutionContext::default()),
            "There are 2 of a max of <max> players online: <players>"
        );
    }

    #[test]
    fn routes_feedback_to_source_admins_and_log_when_vanilla_broadcast_flag_is_set() {
        let context = CommandFeedbackContext {
            source_name: "Rcon".to_string(),
            source_accepts_feedback: true,
            source_silent: false,
            source_informs_admins: true,
            source_is_server: false,
            send_command_feedback_rule: true,
            log_admin_commands_rule: true,
            admins: vec!["Alex".to_string(), "Rcon".to_string()],
        };
        let plan =
            route_command_feedback(&result("commands.save.success", true), &[], &context).unwrap();
        assert_eq!(plan.success_count, 1);
        assert_eq!(plan.packets.len(), 3);
        assert_eq!(plan.packets[0].target, FeedbackTarget::Source);
        assert_eq!(
            plan.packets[1].target,
            FeedbackTarget::Admin("Alex".to_string())
        );
        assert_eq!(plan.packets[2].target, FeedbackTarget::Log);
        assert_eq!(
            plan.packets[1]
                .component
                .render_plain(&translations(), &ResolutionContext::default()),
            "[Rcon: Saved the game]"
        );
    }

    #[test]
    fn gamerules_and_silent_sources_suppress_the_same_routes_as_vanilla() {
        let mut context = CommandFeedbackContext::console().with_rcon_broadcast(true);
        context.send_command_feedback_rule = false;
        context.admins = vec!["Alex".to_string()];
        let plan =
            route_command_feedback(&result("commands.save.success", true), &[], &context).unwrap();
        assert_eq!(
            plan.packets
                .iter()
                .map(|packet| &packet.target)
                .collect::<Vec<_>>(),
            vec![&FeedbackTarget::Source, &FeedbackTarget::Log]
        );

        context.source_silent = true;
        let silent =
            route_command_feedback(&result("commands.save.success", true), &[], &context).unwrap();
        assert!(silent.packets.is_empty());
    }

    #[test]
    fn console_and_rcon_broadcast_properties_gate_admin_notifications_like_vanilla() {
        let mut console = CommandFeedbackContext::console().with_console_broadcast(false);
        console.admins = vec!["Alex".to_string()];
        let console_plan =
            route_command_feedback(&result("commands.save.success", true), &[], &console).unwrap();
        assert_eq!(
            console_plan
                .packets
                .iter()
                .map(|packet| &packet.target)
                .collect::<Vec<_>>(),
            vec![&FeedbackTarget::Source]
        );

        let mut rcon = CommandFeedbackContext::console().with_rcon_broadcast(false);
        rcon.admins = vec!["Alex".to_string()];
        let rcon_plan =
            route_command_feedback(&result("commands.save.success", true), &[], &rcon).unwrap();
        assert_eq!(
            rcon_plan
                .packets
                .iter()
                .map(|packet| &packet.target)
                .collect::<Vec<_>>(),
            vec![&FeedbackTarget::Source]
        );

        rcon = rcon.with_rcon_broadcast(true);
        let broadcast_plan =
            route_command_feedback(&result("commands.save.success", true), &[], &rcon).unwrap();
        assert_eq!(
            broadcast_plan
                .packets
                .iter()
                .map(|packet| &packet.target)
                .collect::<Vec<_>>(),
            vec![
                &FeedbackTarget::Source,
                &FeedbackTarget::Admin("Alex".to_string()),
                &FeedbackTarget::Log
            ]
        );
    }

    #[test]
    fn rejects_non_command_or_unknown_feedback_keys() {
        assert!(format_command_feedback(&result("death.attack.generic", false), &[]).is_err());
        assert!(format_command_feedback(&result("commands.missing.key", false), &[]).is_err());
    }

    #[test]
    fn feedback_styles_match_vanilla_gray_success_and_red_failure_convention() {
        assert!(Component::literal("ok")
            .styled(feedback_style(true))
            .to_json()
            .contains("\"color\":\"gray\""));
        assert!(Component::literal("bad")
            .styled(feedback_style(false))
            .to_json()
            .contains("\"color\":\"red\""));
    }
}
