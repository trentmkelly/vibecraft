#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwingError {
    NoLivingEntity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwingOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub hand: SwingHand,
    pub broadcast_to_admins: bool,
}

pub fn execute_swing_command(
    living_entity_count: i32,
    hand: SwingHand,
) -> Result<SwingOutput, SwingError> {
    if living_entity_count == 0 {
        return Err(SwingError::NoLivingEntity);
    }

    Ok(SwingOutput {
        success_count: living_entity_count,
        feedback_key: if living_entity_count == 1 {
            "commands.swing.success.single"
        } else {
            "commands.swing.success.multiple"
        },
        hand,
        broadcast_to_admins: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const SWING_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/SwingCommand.java");

    #[test]
    fn swing_command_counts_living_entities_and_selects_feedback() {
        assert_eq!(
            execute_swing_command(0, SwingHand::MainHand),
            Err(SwingError::NoLivingEntity)
        );
        assert_eq!(
            execute_swing_command(1, SwingHand::MainHand),
            Ok(SwingOutput {
                success_count: 1,
                feedback_key: "commands.swing.success.single",
                hand: SwingHand::MainHand,
                broadcast_to_admins: true,
            })
        );
        assert_eq!(
            execute_swing_command(2, SwingHand::OffHand),
            Ok(SwingOutput {
                success_count: 2,
                feedback_key: "commands.swing.success.multiple",
                hand: SwingHand::OffHand,
                broadcast_to_admins: true,
            })
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn swing_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"swing\")",
            "Commands.hasPermission(Commands.LEVEL_GAMEMASTERS)",
            "List.of(((CommandSourceStack)c.getSource()).getEntityOrException())",
            "InteractionHand.MAIN_HAND",
            "Commands.argument(\"targets\", EntityArgument.entities())",
            "Commands.literal(\"mainhand\")",
            "Commands.literal(\"offhand\")",
            "InteractionHand.OFF_HAND",
            "if (entity instanceof LivingEntity livingEntity)",
            "livingEntity.swing(hand, true);",
            "livingEntitiesCount++;",
            "if (livingEntitiesCount == 0)",
            "ERROR_NO_LIVING_ENTITY.create()",
            "commands.swing.success.single",
            "commands.swing.success.multiple",
            "return livingEntitiesCount;",
        ] {
            assert!(
                SWING_COMMAND_JAVA.contains(sentinel),
                "SwingCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
