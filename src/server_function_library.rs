#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::command::{
    load_command_function_tags_from_resources, load_command_functions_from_resources,
    CommandFunctionDefinition, CommandFunctionTag,
};
use crate::registry::Identifier;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerFunctionLibrary {
    functions: BTreeMap<Identifier, CommandFunctionDefinition>,
    tags: BTreeMap<Identifier, Vec<CommandFunctionDefinition>>,
    errors: Vec<String>,
}

impl ServerFunctionLibrary {
    pub const TYPE_KEY: &'static str = "minecraft:function";
    pub const FUNCTION_EXTENSION: &'static str = ".mcfunction";
    pub const FUNCTION_DIRECTORY: &'static str = "function";
    pub const TAG_DIRECTORY: &'static str = "tags/function";

    pub fn reload_from_resources<'a>(
        function_resources: impl IntoIterator<Item = (&'a str, &'a str)>,
        tag_resources: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        let mut errors = Vec::new();
        let mut functions = BTreeMap::new();

        match load_command_functions_from_resources(function_resources) {
            Ok(loaded_functions) => {
                for function in loaded_functions {
                    match Identifier::parse(&function.id) {
                        Ok(id) => {
                            functions.insert(id, function);
                        }
                        Err(err) => errors.push(format!(
                            "Failed to load function {}: invalid id: {err}",
                            function.id
                        )),
                    }
                }
            }
            Err(err) => errors.push(err),
        }

        let raw_tags = match load_command_function_tags_from_resources(tag_resources) {
            Ok(tags) => tags,
            Err(err) => {
                errors.push(err);
                Vec::new()
            }
        };

        let mut tags = BTreeMap::new();
        for tag in &raw_tags {
            if let Ok(id) = Identifier::parse(&tag.id) {
                let mut resolving = BTreeSet::new();
                let resolved = resolve_tag_functions(&id, &raw_tags, &functions, &mut resolving, &mut errors);
                tags.insert(id, resolved);
            }
        }

        Self {
            functions,
            tags,
            errors,
        }
    }

    pub fn get_function(&self, id: &Identifier) -> Option<&CommandFunctionDefinition> {
        self.functions.get(id)
    }

    pub fn get_functions(&self) -> &BTreeMap<Identifier, CommandFunctionDefinition> {
        &self.functions
    }

    pub fn get_tag(&self, tag: &Identifier) -> &[CommandFunctionDefinition] {
        self.tags.get(tag).map_or(&[], Vec::as_slice)
    }

    pub fn get_available_tags(&self) -> impl Iterator<Item = &Identifier> {
        self.tags.keys()
    }

    pub fn errors(&self) -> &[String] {
        &self.errors
    }
}

fn resolve_tag_functions(
    tag_id: &Identifier,
    raw_tags: &[CommandFunctionTag],
    functions: &BTreeMap<Identifier, CommandFunctionDefinition>,
    resolving: &mut BTreeSet<Identifier>,
    errors: &mut Vec<String>,
) -> Vec<CommandFunctionDefinition> {
    if !resolving.insert(tag_id.clone()) {
        errors.push(format!("Function tag {tag_id} contains a cycle"));
        return Vec::new();
    }

    let Some(tag) = raw_tags.iter().find(|tag| tag.id == tag_id.to_string()) else {
        errors.push(format!("Missing function tag {tag_id}"));
        resolving.remove(tag_id);
        return Vec::new();
    };

    let mut resolved = Vec::new();
    for entry in &tag.functions {
        if let Some(nested_tag) = entry.strip_prefix('#') {
            match Identifier::parse(nested_tag) {
                Ok(nested_id) => {
                    resolved.extend(resolve_tag_functions(
                        &nested_id, raw_tags, functions, resolving, errors,
                    ));
                }
                Err(err) => errors.push(format!("Invalid function tag reference {entry}: {err}")),
            }
            continue;
        }

        match Identifier::parse(entry) {
            Ok(function_id) => {
                if let Some(function) = functions.get(&function_id) {
                    resolved.push(function.clone());
                } else {
                    errors.push(format!("Missing function {function_id} for tag {tag_id}"));
                }
            }
            Err(err) => errors.push(format!("Invalid function reference {entry}: {err}")),
        }
    }

    resolving.remove(tag_id);
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/ServerFunctionLibrary.java");

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap_or_else(|err| panic!("{err}"))
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_function_library_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public class ServerFunctionLibrary implements PreparableReloadListener"));
        assert!(JAVA_SOURCE.contains("ResourceKey.createRegistryKey("));
        assert!(JAVA_SOURCE.contains("Identifier.withDefaultNamespace(\"function\")"));
        assert!(JAVA_SOURCE.contains("new FileToIdConverter(Registries.elementsDirPath(TYPE_KEY), \".mcfunction\")"));
        assert!(JAVA_SOURCE.contains("private volatile Map<Identifier, CommandFunction<CommandSourceStack>> functions = ImmutableMap.of();"));
        assert!(JAVA_SOURCE.contains("private final TagLoader<CommandFunction<CommandSourceStack>> tagsLoader"));
        assert!(JAVA_SOURCE.contains("private volatile Map<Identifier, List<CommandFunction<CommandSourceStack>>> tags = Map.of();"));
        assert!(JAVA_SOURCE.contains("public Optional<CommandFunction<CommandSourceStack>> getFunction(final Identifier id)"));
        assert!(JAVA_SOURCE.contains("public Map<Identifier, CommandFunction<CommandSourceStack>> getFunctions()"));
        assert!(JAVA_SOURCE.contains("public List<CommandFunction<CommandSourceStack>> getTag(final Identifier tag)"));
        assert!(JAVA_SOURCE.contains("public Iterable<Identifier> getAvailableTags()"));
        assert!(JAVA_SOURCE.contains("CommandFunction.fromLines(id, this.dispatcher, compilationContext, lines)"));
        assert!(JAVA_SOURCE.contains("LOGGER.error(\"Failed to load function {}\", id, throwable);"));
        assert!(JAVA_SOURCE.contains("this.functions = newFunctions.build();"));
        assert!(JAVA_SOURCE.contains("this.tags = this.tagsLoader.build("));
    }

    #[test]
    fn server_function_library_loads_functions_and_resolves_tags() {
        let library = ServerFunctionLibrary::reload_from_resources(
            [
                ("data/minecraft/function/tick.mcfunction", "say tick"),
                ("data/custom/function/load.mcfunction", "# comment\nsay load"),
            ],
            [(
                "data/minecraft/tags/function/tick.json",
                r#"{"values":["minecraft:tick","custom:load"]}"#,
            )],
        );

        assert!(library.errors().is_empty());
        assert_eq!(
            library
                .get_function(&id("minecraft:tick"))
                .map(|function| function.commands.clone()),
            Some(vec!["say tick".to_string()])
        );
        assert_eq!(library.get_functions().len(), 2);
        assert_eq!(
            library
                .get_tag(&id("minecraft:tick"))
                .iter()
                .map(|function| function.id.as_str())
                .collect::<Vec<_>>(),
            vec!["minecraft:tick", "custom:load"]
        );
        assert_eq!(
            library
                .get_available_tags()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["minecraft:tick"]
        );
    }

    #[test]
    fn server_function_library_replaces_previous_state_and_records_failures() {
        let library = ServerFunctionLibrary::reload_from_resources(
            [("data/minecraft/function/valid.mcfunction", "say valid")],
            [(
                "data/minecraft/tags/function/tick.json",
                r#"{"values":["minecraft:valid","minecraft:missing"]}"#,
            )],
        );

        assert_eq!(library.get_functions().len(), 1);
        assert_eq!(library.get_tag(&id("minecraft:tick")).len(), 1);
        assert_eq!(
            library.errors(),
            &["Missing function minecraft:missing for tag minecraft:tick".to_string()]
        );

        let reloaded =
            ServerFunctionLibrary::reload_from_resources([("data/minecraft/function/new.mcfunction", "say new")], []);
        assert!(reloaded.get_function(&id("minecraft:valid")).is_none());
        assert!(reloaded.get_function(&id("minecraft:new")).is_some());
        assert!(reloaded.get_available_tags().next().is_none());
    }

    #[test]
    fn server_function_library_resolves_nested_tags_in_order() {
        let library = ServerFunctionLibrary::reload_from_resources(
            [
                ("data/minecraft/function/a.mcfunction", "say a"),
                ("data/minecraft/function/b.mcfunction", "say b"),
            ],
            [
                (
                    "data/minecraft/tags/function/tick.json",
                    r##"{"values":["#minecraft:nested","minecraft:b"]}"##,
                ),
                (
                    "data/minecraft/tags/function/nested.json",
                    r#"{"values":["minecraft:a"]}"#,
                ),
            ],
        );

        assert!(library.errors().is_empty());
        assert_eq!(
            library
                .get_tag(&id("minecraft:tick"))
                .iter()
                .map(|function| function.id.as_str())
                .collect::<Vec<_>>(),
            vec!["minecraft:a", "minecraft:b"]
        );
    }
}
