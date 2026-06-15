#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadCommandOutput {
    pub success_count: i32,
    pub feedback_key: &'static str,
    pub broadcast_to_admins: bool,
    pub selected_packs: Vec<String>,
}

pub fn discover_new_packs(
    available_packs: &[String],
    current_packs: &[String],
    disabled_packs: &[String],
) -> Vec<String> {
    let mut selected = current_packs.to_vec();
    for pack in available_packs {
        if !disabled_packs.contains(pack) && !selected.contains(pack) {
            selected.push(pack.clone());
        }
    }
    selected
}

pub fn execute_reload_command(
    available_packs: &[String],
    current_packs: &[String],
    disabled_packs: &[String],
) -> ReloadCommandOutput {
    ReloadCommandOutput {
        success_count: 0,
        feedback_key: "commands.reload.success",
        broadcast_to_admins: true,
        selected_packs: discover_new_packs(available_packs, current_packs, disabled_packs),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const RELOAD_COMMAND_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/commands/ReloadCommand.java");

    #[test]
    fn reload_discovers_new_available_non_disabled_packs_after_current_selection() {
        let available = strings(&["vanilla", "kept", "new_pack", "disabled_pack"]);
        let current = strings(&["vanilla", "kept"]);
        let disabled = strings(&["disabled_pack"]);

        assert_eq!(
            discover_new_packs(&available, &current, &disabled),
            strings(&["vanilla", "kept", "new_pack"])
        );
        assert_eq!(
            execute_reload_command(&available, &current, &disabled),
            ReloadCommandOutput {
                success_count: 0,
                feedback_key: "commands.reload.success",
                broadcast_to_admins: true,
                selected_packs: strings(&["vanilla", "kept", "new_pack"]),
            }
        );
    }

    #[test]
    fn reload_does_not_duplicate_existing_selected_packs() {
        let available = strings(&["vanilla", "kept", "kept", "new_pack"]);
        let current = strings(&["vanilla", "kept"]);
        let disabled = Vec::new();

        assert_eq!(
            discover_new_packs(&available, &current, &disabled),
            strings(&["vanilla", "kept", "new_pack"])
        );
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn reload_command_source_matches_java_26_1_2() {
        for sentinel in [
            "Commands.literal(\"reload\")",
            ".requires(Commands.hasPermission(Commands.LEVEL_GAMEMASTERS))",
            "packRepository.reload();",
            "Collection<String> selected = Lists.newArrayList(currentPacks);",
            "worldData.getDataConfiguration().dataPacks().getDisabled()",
            "for (String pack : packRepository.getAvailableIds())",
            "if (!disabled.contains(pack) && !selected.contains(pack))",
            "selected.add(pack);",
            "source.sendSuccess(() -> Component.translatable(\"commands.reload.success\"), true)",
            "reloadPacks(newSelectedPacks, source);",
            "source.getServer().reloadResources(selectedPacks).exceptionally",
            "source.sendFailure(Component.translatable(\"commands.reload.failure\"))",
            "return 0;",
        ] {
            assert!(
                RELOAD_COMMAND_JAVA.contains(sentinel),
                "ReloadCommand.java is missing sentinel: {sentinel}"
            );
        }
    }
}
