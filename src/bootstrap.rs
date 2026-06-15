#![allow(dead_code)]

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapStreamWrapper {
    Logged,
    DebugLogged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapException {
    pub message: String,
    pub suppressed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapValidationReport {
    pub missing_translation_logs: Vec<String>,
    pub commands_validated: bool,
    pub default_attributes_validated: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BootstrapTranslationInputs<'a> {
    pub attributes: &'a [&'a str],
    pub entity_types: &'a [&'a str],
    pub mob_effects: &'a [&'a str],
    pub items: &'a [&'a str],
    pub blocks: &'a [&'a str],
    pub custom_stats: &'a [&'a str],
    pub gamerules: &'a [(&'a str, &'a str)],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapModel {
    bootstrapped: bool,
    bootstrap_duration_ms: i64,
    debug_logging: bool,
    registry_empty: bool,
    player_entity_registered: bool,
    running_in_ide: bool,
    wrapped_streams: Option<BootstrapStreamWrapper>,
    init_calls: Vec<&'static str>,
    attributes: Vec<String>,
    entity_types: Vec<String>,
    mob_effects: Vec<String>,
    items: Vec<String>,
    blocks: Vec<String>,
    custom_stats: Vec<String>,
    gamerules: Vec<(String, String)>,
    translations: BTreeSet<String>,
}

impl Default for BootstrapModel {
    fn default() -> Self {
        Self {
            bootstrapped: false,
            bootstrap_duration_ms: -1,
            debug_logging: false,
            registry_empty: false,
            player_entity_registered: true,
            running_in_ide: false,
            wrapped_streams: None,
            init_calls: Vec::new(),
            attributes: Vec::new(),
            entity_types: Vec::new(),
            mob_effects: Vec::new(),
            items: Vec::new(),
            blocks: Vec::new(),
            custom_stats: Vec::new(),
            gamerules: Vec::new(),
            translations: BTreeSet::new(),
        }
    }
}

impl BootstrapModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_debug_logging(&mut self, debug_logging: bool) {
        self.debug_logging = debug_logging;
    }

    pub fn set_registry_empty(&mut self, registry_empty: bool) {
        self.registry_empty = registry_empty;
    }

    pub fn set_player_entity_registered(&mut self, registered: bool) {
        self.player_entity_registered = registered;
    }

    pub fn set_running_in_ide(&mut self, running_in_ide: bool) {
        self.running_in_ide = running_in_ide;
    }

    pub fn add_translation(&mut self, key: impl Into<String>) {
        self.translations.insert(key.into());
    }

    pub fn set_translation_inputs(&mut self, inputs: BootstrapTranslationInputs<'_>) {
        self.attributes = inputs
            .attributes
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        self.entity_types = inputs
            .entity_types
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        self.mob_effects = inputs
            .mob_effects
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        self.items = inputs
            .items
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        self.blocks = inputs
            .blocks
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        self.custom_stats = inputs
            .custom_stats
            .iter()
            .map(|value| (*value).to_string())
            .collect();
        self.gamerules = inputs
            .gamerules
            .iter()
            .map(|(id, description)| ((*id).to_string(), (*description).to_string()))
            .collect();
    }

    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrapped
    }

    pub fn bootstrap_duration_ms(&self) -> i64 {
        self.bootstrap_duration_ms
    }

    pub fn init_calls(&self) -> &[&'static str] {
        &self.init_calls
    }

    pub fn wrapped_streams(&self) -> Option<BootstrapStreamWrapper> {
        self.wrapped_streams
    }

    pub fn boot_strap(&mut self, elapsed_ms: i64) -> Result<(), String> {
        if self.bootstrapped {
            return Ok(());
        }

        self.bootstrapped = true;
        if self.registry_empty {
            return Err("Unable to load registries".to_string());
        }

        self.init_calls.push("FireBlock.bootStrap");
        self.init_calls.push("ComposterBlock.bootStrap");
        if !self.player_entity_registered {
            return Err("Failed loading EntityTypes".to_string());
        }

        self.init_calls.push("EntitySelectorOptions.bootStrap");
        self.init_calls.push("DispenseItemBehavior.bootStrap");
        self.init_calls.push("CauldronInteractions.bootStrap");
        self.init_calls.push("BuiltInRegistries.bootStrap");
        self.init_calls.push("CreativeModeTabs.validate");
        self.wrap_streams();
        self.bootstrap_duration_ms = elapsed_ms.max(0);
        Ok(())
    }

    pub fn check_bootstrap_called<F>(&self, location: F) -> Result<(), BootstrapException>
    where
        F: FnOnce() -> Result<String, String>,
    {
        if self.bootstrapped {
            Ok(())
        } else {
            Err(Self::create_bootstrap_exception(location))
        }
    }

    pub fn validate(&self) -> Result<BootstrapValidationReport, BootstrapException> {
        self.check_bootstrap_called(|| Ok("validate".to_string()))?;
        let mut report = BootstrapValidationReport {
            missing_translation_logs: Vec::new(),
            commands_validated: false,
            default_attributes_validated: true,
        };
        if self.running_in_ide {
            report.missing_translation_logs = self
                .get_missing_translations()
                .into_iter()
                .map(|key| format!("Missing translations: {key}"))
                .collect();
            report.commands_validated = true;
        }
        Ok(report)
    }

    pub fn get_missing_translations(&self) -> BTreeSet<String> {
        let mut missing = BTreeSet::new();
        self.check_translations(&self.attributes, |id| id.to_string(), &mut missing);
        self.check_translations(&self.entity_types, |id| id.to_string(), &mut missing);
        self.check_translations(&self.mob_effects, |id| id.to_string(), &mut missing);
        self.check_translations(&self.items, |id| id.to_string(), &mut missing);
        self.check_translations(&self.blocks, |id| id.to_string(), &mut missing);
        self.check_translations(
            &self.custom_stats,
            |id| format!("stat.{}", id.replace(':', ".")),
            &mut missing,
        );
        for (id, description) in &self.gamerules {
            if !self.translations.contains(description) {
                missing.insert(id.clone());
            }
        }
        missing
    }

    fn check_translations<F>(&self, ids: &[String], description_getter: F, missing: &mut BTreeSet<String>)
    where
        F: Fn(&str) -> String,
    {
        for id in ids {
            let description = description_getter(id);
            if !self.translations.contains(&description) {
                missing.insert(description);
            }
        }
    }

    fn create_bootstrap_exception<F>(location: F) -> BootstrapException
    where
        F: FnOnce() -> Result<String, String>,
    {
        match location() {
            Ok(resolved) => BootstrapException {
                message: format!("Not bootstrapped (called from {resolved})"),
                suppressed: Vec::new(),
            },
            Err(err) => BootstrapException {
                message: "Not bootstrapped (failed to resolve location)".to_string(),
                suppressed: vec![err],
            },
        }
    }

    fn wrap_streams(&mut self) {
        self.wrapped_streams = Some(if self.debug_logging {
            BootstrapStreamWrapper::DebugLogged
        } else {
            BootstrapStreamWrapper::Logged
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/Bootstrap.java");

    #[test]
    fn bootstrap_runs_java_initialization_order_once_and_records_duration() {
        let mut bootstrap = BootstrapModel::new();
        bootstrap.boot_strap(37).unwrap_or_else(|err| panic!("{err}"));
        bootstrap.boot_strap(99).unwrap_or_else(|err| panic!("{err}"));

        assert!(bootstrap.is_bootstrapped());
        assert_eq!(bootstrap.bootstrap_duration_ms(), 37);
        assert_eq!(
            bootstrap.init_calls(),
            &[
                "FireBlock.bootStrap",
                "ComposterBlock.bootStrap",
                "EntitySelectorOptions.bootStrap",
                "DispenseItemBehavior.bootStrap",
                "CauldronInteractions.bootStrap",
                "BuiltInRegistries.bootStrap",
                "CreativeModeTabs.validate",
            ]
        );
        assert_eq!(bootstrap.wrapped_streams(), Some(BootstrapStreamWrapper::Logged));
    }

    #[test]
    fn bootstrap_sets_guard_before_java_failure_checks() {
        let mut empty_registry = BootstrapModel::new();
        empty_registry.set_registry_empty(true);
        assert_eq!(
            empty_registry.boot_strap(1),
            Err("Unable to load registries".to_string())
        );
        assert!(empty_registry.is_bootstrapped());
        assert_eq!(empty_registry.bootstrap_duration_ms(), -1);

        let mut missing_player = BootstrapModel::new();
        missing_player.set_player_entity_registered(false);
        assert_eq!(
            missing_player.boot_strap(1),
            Err("Failed loading EntityTypes".to_string())
        );
        assert!(missing_player.is_bootstrapped());
        assert_eq!(
            missing_player.init_calls(),
            &["FireBlock.bootStrap", "ComposterBlock.bootStrap"]
        );
    }

    #[test]
    fn bootstrap_guard_exception_matches_java_success_and_suppressed_paths() {
        let bootstrap = BootstrapModel::new();
        let resolved = bootstrap
            .check_bootstrap_called(|| Ok("unit-test".to_string()))
            .unwrap_err();
        assert_eq!(
            resolved.message,
            "Not bootstrapped (called from unit-test)"
        );
        assert!(resolved.suppressed.is_empty());

        let failed = bootstrap
            .check_bootstrap_called(|| Err("location exploded".to_string()))
            .unwrap_err();
        assert_eq!(
            failed.message,
            "Not bootstrapped (failed to resolve location)"
        );
        assert_eq!(failed.suppressed, vec!["location exploded"]);
    }

    #[test]
    fn missing_translations_are_sorted_and_use_java_key_shapes() {
        let mut bootstrap = BootstrapModel::new();
        bootstrap.set_translation_inputs(BootstrapTranslationInputs {
            attributes: &["attribute.name.generic.max_health"],
            entity_types: &["entity.minecraft.player"],
            mob_effects: &["effect.minecraft.speed"],
            items: &["item.minecraft.stick"],
            blocks: &["block.minecraft.stone"],
            custom_stats: &["minecraft:jump"],
            gamerules: &[("doDaylightCycle", "gamerule.doDaylightCycle")],
        });
        bootstrap.add_translation("entity.minecraft.player");
        bootstrap.add_translation("item.minecraft.stick");
        bootstrap.add_translation("stat.minecraft.jump");

        assert_eq!(
            bootstrap
                .get_missing_translations()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![
                "attribute.name.generic.max_health",
                "block.minecraft.stone",
                "doDaylightCycle",
                "effect.minecraft.speed",
            ]
        );
    }

    #[test]
    fn validate_checks_default_attributes_always_and_commands_only_in_ide() {
        let mut bootstrap = BootstrapModel::new();
        assert!(bootstrap.validate().is_err());
        bootstrap.boot_strap(0).unwrap_or_else(|err| panic!("{err}"));

        let report = bootstrap.validate().unwrap_or_else(|err| panic!("{err:?}"));
        assert!(report.default_attributes_validated);
        assert!(!report.commands_validated);
        assert!(report.missing_translation_logs.is_empty());

        bootstrap.set_running_in_ide(true);
        bootstrap.set_translation_inputs(BootstrapTranslationInputs {
            attributes: &["attribute.name.generic.attack_damage"],
            ..BootstrapTranslationInputs::default()
        });
        let ide_report = bootstrap.validate().unwrap_or_else(|err| panic!("{err:?}"));
        assert!(ide_report.commands_validated);
        assert_eq!(
            ide_report.missing_translation_logs,
            vec!["Missing translations: attribute.name.generic.attack_damage"]
        );
    }

    #[test]
    fn stream_wrapping_uses_debug_print_stream_only_when_logger_debug_is_enabled() {
        let mut debug = BootstrapModel::new();
        debug.set_debug_logging(true);
        debug.boot_strap(0).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(debug.wrapped_streams(), Some(BootstrapStreamWrapper::DebugLogged));
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn bootstrap_source_shape_matches_java_26_1_2() {
        for sentinel in [
            "public static final PrintStream STDOUT = System.out;",
            "private static volatile boolean isBootstrapped;",
            "public static final AtomicLong bootstrapDuration = new AtomicLong(-1L);",
            "if (BuiltInRegistries.REGISTRY.keySet().isEmpty())",
            "FireBlock.bootStrap();",
            "ComposterBlock.bootStrap();",
            "if (EntityType.getKey(EntityType.PLAYER) == null)",
            "EntitySelectorOptions.bootStrap();",
            "DispenseItemBehavior.bootStrap();",
            "CauldronInteractions.bootStrap();",
            "BuiltInRegistries.bootStrap();",
            "CreativeModeTabs.validate();",
            "bootstrapDuration.set(Duration.between(start, Instant.now()).toMillis());",
            "new TreeSet<>()",
            "id -> \"stat.\" + id.toString().replace(':', '.')",
            "new IllegalArgumentException(\"Not bootstrapped (called from \" + resolvedLocation + \")\")",
            "result.addSuppressed(e);",
            "checkBootstrapCalled(() -> \"validate\");",
            "SharedConstants.IS_RUNNING_IN_IDE",
            "Commands.validate();",
            "DefaultAttributes.validate();",
            "System.setErr(new DebugLoggedPrintStream(\"STDERR\", System.err));",
            "System.setOut(new LoggedPrintStream(\"STDOUT\", STDOUT));",
            "STDOUT.println(string);",
            "STDOUT.close();",
        ] {
            assert!(
                JAVA_SOURCE.contains(sentinel),
                "Bootstrap.java is missing sentinel: {sentinel}"
            );
        }
    }
}
