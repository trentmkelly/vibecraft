use std::collections::{BTreeMap, BTreeSet};

use crate::registry::{feature_flags, FeatureFlagSet, Identifier};

pub const COMMAND_PREFIX: &str = "/";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandsModel {
    pub registrations: Vec<CommandRegistration>,
    pub consumer: &'static str,
}

impl CommandsModel {
    pub fn new(selection: CommandSelection, flags: RegistrationFlags) -> Self {
        let mut registrations = Vec::new();
        register_core_commands(&mut registrations, selection);

        if flags.jvm_profiler_available {
            registrations.push(CommandRegistration::plain("jfr"));
        }
        if flags.debug_chase_command {
            registrations.push(CommandRegistration::plain("chase"));
        }
        if flags.debug_dev_commands || flags.running_in_ide {
            registrations.extend([
                CommandRegistration::with_context("raid"),
                CommandRegistration::plain("debugpath"),
                CommandRegistration::plain("debugmobspawning"),
                CommandRegistration::plain("warden_spawn_tracker"),
                CommandRegistration::plain("spawn_armor_trims"),
                CommandRegistration::plain("serverpack"),
            ]);
            if selection.include_dedicated() {
                registrations.push(CommandRegistration::with_context("debugconfig"));
            }
        }
        if selection.include_dedicated() {
            registrations.extend([
                CommandRegistration::plain("ban-ip"),
                CommandRegistration::plain("banlist"),
                CommandRegistration::plain("ban"),
                CommandRegistration::plain("deop"),
                CommandRegistration::plain("op"),
                CommandRegistration::plain("pardon"),
                CommandRegistration::plain("pardon-ip"),
                CommandRegistration::plain("perf"),
                CommandRegistration::plain("save-all"),
                CommandRegistration::plain("save-off"),
                CommandRegistration::plain("save-on"),
                CommandRegistration::plain("setidletimeout"),
                CommandRegistration::plain("stop"),
                CommandRegistration::plain("transfer"),
                CommandRegistration::plain("whitelist"),
            ]);
        }
        if selection.include_integrated() {
            registrations.push(CommandRegistration::plain("publish"));
        }

        Self {
            registrations,
            consumer: "ExecutionCommandSource.resultConsumer",
        }
    }

    pub fn command_names(&self) -> Vec<&'static str> {
        self.registrations
            .iter()
            .map(|registration| registration.name)
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistrationFlags {
    pub jvm_profiler_available: bool,
    pub debug_chase_command: bool,
    pub debug_dev_commands: bool,
    pub running_in_ide: bool,
}

impl RegistrationFlags {
    pub fn vanilla() -> Self {
        Self {
            jvm_profiler_available: false,
            debug_chase_command: false,
            debug_dev_commands: false,
            running_in_ide: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandRegistration {
    pub name: &'static str,
    pub uses_context: bool,
    pub enabled: bool,
}

impl CommandRegistration {
    fn plain(name: &'static str) -> Self {
        Self {
            name,
            uses_context: false,
            enabled: true,
        }
    }

    fn with_context(name: &'static str) -> Self {
        Self {
            name,
            uses_context: true,
            enabled: true,
        }
    }

    fn gated(name: &'static str, enabled: bool) -> Self {
        Self {
            name,
            uses_context: false,
            enabled,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSelection {
    All,
    Dedicated,
    Integrated,
}

impl CommandSelection {
    pub fn include_integrated(self) -> bool {
        matches!(self, Self::All | Self::Integrated)
    }

    pub fn include_dedicated(self) -> bool {
        matches!(self, Self::All | Self::Dedicated)
    }
}

fn register_core_commands(
    registrations: &mut Vec<CommandRegistration>,
    selection: CommandSelection,
) {
    registrations.extend([
        CommandRegistration::plain("advancement"),
        CommandRegistration::with_context("attribute"),
        CommandRegistration::with_context("execute"),
        CommandRegistration::with_context("bossbar"),
        CommandRegistration::with_context("clear"),
        CommandRegistration::with_context("clone"),
        CommandRegistration::with_context("damage"),
        CommandRegistration::plain("data"),
        CommandRegistration::with_context("datapack"),
        CommandRegistration::plain("debug"),
        CommandRegistration::plain("defaultgamemode"),
        CommandRegistration::with_context("dialog"),
        CommandRegistration::plain("difficulty"),
        CommandRegistration::with_context("effect"),
        CommandRegistration::plain("me"),
        CommandRegistration::with_context("enchant"),
        CommandRegistration::plain("experience"),
        CommandRegistration::with_context("fill"),
        CommandRegistration::with_context("fillbiome"),
        CommandRegistration::plain("forceload"),
        CommandRegistration::plain("function"),
        CommandRegistration::plain("gamemode"),
        CommandRegistration::with_context("gamerule"),
        CommandRegistration::with_context("give"),
        CommandRegistration::plain("help"),
        CommandRegistration::with_context("item"),
        CommandRegistration::plain("kick"),
        CommandRegistration::plain("kill"),
        CommandRegistration::plain("list"),
        CommandRegistration::with_context("locate"),
        CommandRegistration::with_context("loot"),
        CommandRegistration::plain("msg"),
        CommandRegistration::plain("swing"),
        CommandRegistration::with_context("particle"),
        CommandRegistration::plain("place"),
        CommandRegistration::plain("playsound"),
        CommandRegistration::plain("random"),
        CommandRegistration::plain("reload"),
        CommandRegistration::plain("recipe"),
        CommandRegistration::plain("fetchprofile"),
        CommandRegistration::plain("return"),
        CommandRegistration::plain("ride"),
        CommandRegistration::plain("rotate"),
        CommandRegistration::plain("say"),
        CommandRegistration::plain("schedule"),
        CommandRegistration::with_context("scoreboard"),
        CommandRegistration::gated("seed", selection != CommandSelection::Integrated),
        CommandRegistration::gated("version", selection != CommandSelection::Integrated),
        CommandRegistration::with_context("setblock"),
        CommandRegistration::plain("spawnpoint"),
        CommandRegistration::plain("setworldspawn"),
        CommandRegistration::plain("spectate"),
        CommandRegistration::plain("spreadplayers"),
        CommandRegistration::plain("stopsound"),
        CommandRegistration::plain("stopwatch"),
        CommandRegistration::with_context("summon"),
        CommandRegistration::plain("tag"),
        CommandRegistration::with_context("team"),
        CommandRegistration::plain("teammsg"),
        CommandRegistration::plain("teleport"),
        CommandRegistration::with_context("tellraw"),
        CommandRegistration::with_context("test"),
        CommandRegistration::plain("tick"),
        CommandRegistration::with_context("time"),
        CommandRegistration::with_context("title"),
        CommandRegistration::plain("trigger"),
        CommandRegistration::with_context("waypoint"),
        CommandRegistration::plain("weather"),
        CommandRegistration::plain("worldborder"),
    ]);
}

pub fn trim_optional_prefix(command: &str) -> &str {
    command.strip_prefix(COMMAND_PREFIX).unwrap_or(command)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResultsModel<S> {
    pub source: S,
    pub reader_can_read: bool,
    pub exceptions: Vec<ParseExceptionModel>,
    pub context_range_empty: bool,
}

pub fn map_source<S>(
    parse: ParseResultsModel<S>,
    source_operator: impl FnOnce(S) -> S,
) -> ParseResultsModel<S> {
    let ParseResultsModel {
        source,
        reader_can_read,
        exceptions,
        context_range_empty,
    } = parse;
    ParseResultsModel {
        source: source_operator(source),
        reader_can_read,
        exceptions,
        context_range_empty,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseExceptionModel {
    Single,
    UnknownCommand,
    UnknownArgument,
}

pub fn get_parse_exception<S>(parse: &ParseResultsModel<S>) -> Option<ParseExceptionModel> {
    if !parse.reader_can_read {
        None
    } else if parse.exceptions.len() == 1 {
        Some(parse.exceptions[0])
    } else if parse.context_range_empty {
        Some(ParseExceptionModel::UnknownCommand)
    } else {
        Some(ParseExceptionModel::UnknownArgument)
    }
}

pub fn validate_parse_results<S>(parse: &ParseResultsModel<S>) -> Result<(), ParseExceptionModel> {
    get_parse_exception(parse).map_or(Ok(()), Err)
}

pub fn finish_parsing<S>(
    parse: &ParseResultsModel<S>,
    command: &str,
) -> Result<&'static str, Vec<ComponentModel>> {
    match validate_parse_results(parse) {
        Ok(()) => Ok("context_chain"),
        Err(exception) => {
            let mut failures = vec![ComponentModel::literal(format!("{exception:?}"))];
            if parse.reader_can_read {
                failures.push(parse_error_context(command, command.len().min(12)));
            }
            Err(failures)
        }
    }
}

fn parse_error_context(input: &str, cursor: usize) -> ComponentModel {
    let start = cursor.saturating_sub(10);
    let mut rendered = String::new();
    if cursor > 10 {
        rendered.push_str("...");
    }
    rendered.push_str(&input[start..cursor]);
    if cursor < input.len() {
        rendered.push_str("[red,underline:");
        rendered.push_str(&input[cursor..]);
        rendered.push(']');
    }
    rendered.push_str("<--[red,italic]");
    ComponentModel::literal(rendered)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExecutionContextTracker {
    pub active: bool,
    pub created_contexts: Vec<(i32, i32)>,
    pub configured_existing: usize,
    pub ran_queue: usize,
}

pub fn execute_command_in_context(
    tracker: &mut ExecutionContextTracker,
    max_chain_length: i32,
    max_forks: i32,
    config: impl FnOnce(&mut ExecutionContextTracker),
) {
    if tracker.active {
        tracker.configured_existing += 1;
        config(tracker);
    } else {
        tracker.active = true;
        tracker
            .created_contexts
            .push((max_chain_length.max(1), max_forks));
        config(tracker);
        tracker.ran_queue += 1;
        tracker.active = false;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandNodeModel {
    pub name: &'static str,
    pub can_use: bool,
    pub no_permission_can_use: bool,
    pub executable: bool,
    pub suggestion_id: Option<Identifier>,
    pub redirect: Option<&'static str>,
    pub children: Vec<CommandNodeModel>,
}

impl CommandNodeModel {
    pub fn new(name: &'static str, can_use: bool) -> Self {
        Self {
            name,
            can_use,
            no_permission_can_use: can_use,
            executable: false,
            suggestion_id: None,
            redirect: None,
            children: Vec::new(),
        }
    }
}

pub fn fill_usable_commands(source: &CommandNodeModel) -> CommandNodeModel {
    let mut converted = BTreeSet::new();
    fill_usable_command_node(source, &mut converted)
}

fn fill_usable_command_node(
    source: &CommandNodeModel,
    converted: &mut BTreeSet<&'static str>,
) -> CommandNodeModel {
    converted.insert(source.name);
    let mut target = CommandNodeModel {
        children: Vec::new(),
        ..source.clone()
    };
    for child in &source.children {
        if child.can_use {
            let mut copied = fill_usable_command_node(child, converted);
            if copied
                .redirect
                .is_some_and(|redirect| !converted.contains(redirect))
            {
                copied.redirect = None;
            }
            target.children.push(copied);
        }
    }
    target
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInspectorModel {
    pub no_permission_level: PermissionLevelModel,
}

impl NodeInspectorModel {
    pub fn suggestion_id(&self, node: &CommandNodeModel) -> Option<Identifier> {
        node.suggestion_id.clone()
    }

    pub fn is_executable(&self, node: &CommandNodeModel) -> bool {
        node.executable
    }

    pub fn is_restricted(&self, node: &CommandNodeModel) -> bool {
        !node.no_permission_can_use
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevelModel {
    NoPermissions,
    All,
    Moderators,
    GameMasters,
    Admins,
    Owners,
}

pub fn has_permission(required: PermissionLevelModel) -> impl Fn(PermissionLevelModel) -> bool {
    move |actual| actual >= required
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationContextModel {
    provider: RegistryProviderModel,
}

impl ValidationContextModel {
    pub fn enabled_features(&self) -> FeatureFlagSet {
        FeatureFlagSet::of(&[
            feature_flags::VANILLA,
            feature_flags::TRADE_REBALANCE,
            feature_flags::REDSTONE_EXPERIMENTS,
            feature_flags::MINECART_IMPROVEMENTS,
        ])
    }

    pub fn list_registry_keys(&self) -> Vec<Identifier> {
        self.provider.order.clone()
    }

    pub fn lookup(&self, key: &Identifier) -> Option<ValidationLookupModel> {
        self.provider
            .lookups
            .get(key)
            .cloned()
            .map(ValidationLookupModel)
    }
}

pub fn create_validation_context(provider: RegistryProviderModel) -> ValidationContextModel {
    ValidationContextModel { provider }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistryProviderModel {
    order: Vec<Identifier>,
    lookups: BTreeMap<Identifier, RegistryLookupModel>,
}

impl RegistryProviderModel {
    pub fn new(lookups: impl IntoIterator<Item = RegistryLookupModel>) -> Self {
        let mut provider = Self::default();
        for lookup in lookups {
            provider.order.push(lookup.key.clone());
            provider.lookups.insert(lookup.key.clone(), lookup);
        }
        provider
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLookupModel {
    pub key: Identifier,
    pub tags: BTreeSet<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationLookupModel(pub RegistryLookupModel);

impl ValidationLookupModel {
    pub fn get_tag(&self, tag: &Identifier) -> HolderSetModel {
        if self.0.tags.contains(tag) {
            HolderSetModel::Named(tag.clone())
        } else {
            HolderSetModel::EmptyNamed(tag.clone())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetModel {
    Named(Identifier),
    EmptyNamed(Identifier),
}

pub fn validate_command_tree(used_argument_types: &[ArgumentTypeModel]) -> Result<(), String> {
    let unregistered: Vec<_> = used_argument_types
        .iter()
        .filter(|argument| !argument.registered)
        .map(|argument| argument.name)
        .collect();
    if unregistered.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Unregistered argument types: {}",
            unregistered.join(", ")
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgumentTypeModel {
    pub name: &'static str,
    pub registered: bool,
}

pub fn create_validator(parser: impl Fn(&str) -> Result<(), ()>) -> impl Fn(&str) -> bool {
    move |value| parser(value).is_ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentModel {
    pub text: String,
}

impl ComponentModel {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_selection_flags_match_java_enum() {
        assert!(CommandSelection::All.include_integrated());
        assert!(CommandSelection::All.include_dedicated());
        assert!(!CommandSelection::Dedicated.include_integrated());
        assert!(CommandSelection::Dedicated.include_dedicated());
        assert!(CommandSelection::Integrated.include_integrated());
        assert!(!CommandSelection::Integrated.include_dedicated());
    }

    #[test]
    fn constructor_registers_core_commands_and_sets_result_consumer() {
        let commands = CommandsModel::new(CommandSelection::All, RegistrationFlags::vanilla());
        let names = commands.command_names();

        assert_eq!(names.first(), Some(&"advancement"));
        assert!(names.contains(&"execute"));
        assert!(names.contains(&"me"));
        assert!(!names.contains(&"emote"));
        assert!(names.contains(&"worldborder"));
        assert!(names.contains(&"ban"));
        assert!(names.contains(&"publish"));
        assert_eq!(commands.consumer, "ExecutionCommandSource.resultConsumer");
    }

    #[test]
    fn integrated_selection_keeps_integrated_commands_and_disables_seed_version() {
        let commands =
            CommandsModel::new(CommandSelection::Integrated, RegistrationFlags::vanilla());

        assert!(commands.command_names().contains(&"publish"));
        assert!(!commands.command_names().contains(&"ban"));
        assert_eq!(
            commands
                .registrations
                .iter()
                .find(|registration| registration.name == "seed")
                .map(|registration| registration.enabled),
            Some(false)
        );
        assert_eq!(
            commands
                .registrations
                .iter()
                .find(|registration| registration.name == "version")
                .map(|registration| registration.enabled),
            Some(false)
        );
    }

    #[test]
    fn debug_and_profiler_flags_add_optional_commands_with_dedicated_debug_config() {
        let commands = CommandsModel::new(
            CommandSelection::Dedicated,
            RegistrationFlags {
                jvm_profiler_available: true,
                debug_chase_command: true,
                debug_dev_commands: true,
                running_in_ide: false,
            },
        );
        let names = commands.command_names();

        assert!(names.contains(&"jfr"));
        assert!(names.contains(&"chase"));
        assert!(names.contains(&"raid"));
        assert!(names.contains(&"debugconfig"));
        assert!(!names.contains(&"publish"));
    }

    #[test]
    fn trim_optional_prefix_only_removes_one_leading_slash() {
        assert_eq!(trim_optional_prefix("/say hi"), "say hi");
        assert_eq!(trim_optional_prefix("say hi"), "say hi");
        assert_eq!(trim_optional_prefix("//say hi"), "/say hi");
    }

    #[test]
    fn map_source_replaces_only_parse_source() {
        let parse = ParseResultsModel {
            source: 2,
            reader_can_read: true,
            exceptions: vec![],
            context_range_empty: false,
        };

        assert_eq!(map_source(parse, |source| source + 3).source, 5);
    }

    #[test]
    fn parse_exception_selection_matches_reader_exception_and_context_rules() {
        assert_eq!(
            get_parse_exception(&ParseResultsModel {
                source: (),
                reader_can_read: false,
                exceptions: vec![ParseExceptionModel::Single],
                context_range_empty: true,
            }),
            None
        );
        assert_eq!(
            get_parse_exception(&ParseResultsModel {
                source: (),
                reader_can_read: true,
                exceptions: vec![ParseExceptionModel::Single],
                context_range_empty: true,
            }),
            Some(ParseExceptionModel::Single)
        );
        assert_eq!(
            get_parse_exception(&ParseResultsModel {
                source: (),
                reader_can_read: true,
                exceptions: vec![],
                context_range_empty: true,
            }),
            Some(ParseExceptionModel::UnknownCommand)
        );
        assert_eq!(
            get_parse_exception(&ParseResultsModel {
                source: (),
                reader_can_read: true,
                exceptions: vec![],
                context_range_empty: false,
            }),
            Some(ParseExceptionModel::UnknownArgument)
        );
    }

    #[test]
    fn finish_parsing_returns_context_chain_or_failure_components() {
        let valid = ParseResultsModel {
            source: (),
            reader_can_read: false,
            exceptions: vec![],
            context_range_empty: false,
        };
        let invalid = ParseResultsModel {
            source: (),
            reader_can_read: true,
            exceptions: vec![],
            context_range_empty: false,
        };

        assert_eq!(finish_parsing(&valid, "say hi"), Ok("context_chain"));
        assert!(finish_parsing(&invalid, "0123456789abcdef")
            .unwrap_err()
            .iter()
            .any(|component| component.text.contains("...")));
    }

    #[test]
    fn execute_command_in_context_creates_top_context_and_reuses_nested_context() {
        let mut tracker = ExecutionContextTracker::default();

        execute_command_in_context(&mut tracker, 0, 8, |tracker| {
            execute_command_in_context(tracker, 99, 99, |_| {});
        });

        assert_eq!(tracker.created_contexts, [(1, 8)]);
        assert_eq!(tracker.configured_existing, 1);
        assert_eq!(tracker.ran_queue, 1);
        assert!(!tracker.active);
    }

    #[test]
    fn fill_usable_commands_copies_only_allowed_children_and_keeps_resolved_redirects() {
        let mut root = CommandNodeModel::new("root", true);
        let mut allowed = CommandNodeModel::new("allowed", true);
        allowed.executable = true;
        allowed.redirect = Some("root");
        let denied = CommandNodeModel::new("denied", false);
        root.children = vec![allowed, denied];

        let filtered = fill_usable_commands(&root);

        assert_eq!(filtered.children.len(), 1);
        assert_eq!(filtered.children[0].name, "allowed");
        assert_eq!(filtered.children[0].redirect, Some("root"));
    }

    #[test]
    fn node_inspector_reports_suggestions_executable_and_restricted_state() {
        let inspector = NodeInspectorModel {
            no_permission_level: PermissionLevelModel::NoPermissions,
        };
        let mut node = CommandNodeModel::new("arg", true);
        node.executable = true;
        node.suggestion_id = Some(id("minecraft:ask_server"));
        node.no_permission_can_use = false;
        let unrestricted = CommandNodeModel::new("literal", true);

        assert_eq!(
            inspector.suggestion_id(&node),
            Some(id("minecraft:ask_server"))
        );
        assert!(inspector.is_executable(&node));
        assert!(inspector.is_restricted(&node));
        assert!(!inspector.is_restricted(&unrestricted));
    }

    #[test]
    fn validation_context_exposes_all_features_and_empty_named_missing_tags() {
        let provider = RegistryProviderModel::new([RegistryLookupModel {
            key: id("minecraft:item"),
            tags: BTreeSet::from([id("minecraft:tools")]),
        }]);
        let context = create_validation_context(provider);
        let lookup = context.lookup(&id("minecraft:item")).unwrap();

        assert_eq!(context.list_registry_keys(), [id("minecraft:item")]);
        assert!(context
            .enabled_features()
            .contains(feature_flags::MINECART_IMPROVEMENTS));
        assert_eq!(
            lookup.get_tag(&id("minecraft:tools")),
            HolderSetModel::Named(id("minecraft:tools"))
        );
        assert_eq!(
            lookup.get_tag(&id("minecraft:missing")),
            HolderSetModel::EmptyNamed(id("minecraft:missing"))
        );
    }

    #[test]
    fn validate_reports_unregistered_argument_types() {
        assert_eq!(
            validate_command_tree(&[
                ArgumentTypeModel {
                    name: "entity",
                    registered: true,
                },
                ArgumentTypeModel {
                    name: "custom",
                    registered: false,
                },
            ]),
            Err("Unregistered argument types: custom".to_string())
        );
    }

    #[test]
    fn validator_wraps_parser_success_and_syntax_failure() {
        let validator = create_validator(|value| if value == "ok" { Ok(()) } else { Err(()) });

        assert!(validator("ok"));
        assert!(!validator("bad"));
    }

    #[test]
    fn permission_provider_check_uses_required_level() {
        let admins = has_permission(PermissionLevelModel::Admins);
        let moderators = has_permission(PermissionLevelModel::Moderators);
        let all = has_permission(PermissionLevelModel::All);

        assert!(admins(PermissionLevelModel::Owners));
        assert!(admins(PermissionLevelModel::Admins));
        assert!(!admins(PermissionLevelModel::GameMasters));
        assert!(moderators(PermissionLevelModel::Moderators));
        assert!(!moderators(PermissionLevelModel::All));
        assert!(all(PermissionLevelModel::All));
    }
}
