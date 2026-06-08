use std::collections::BTreeMap;

use crate::registry::Identifier;

#[derive(Debug, Clone)]
pub struct CacheableFunctionModel {
    id: Identifier,
    resolved: bool,
    function: Option<CommandFunctionModel>,
}

impl CacheableFunctionModel {
    pub fn new(id: Identifier) -> Self {
        Self {
            id,
            resolved: false,
            function: None,
        }
    }

    pub fn codec_shape() -> CodecShapeModel {
        CodecShapeModel {
            base_codec: "Identifier.CODEC",
            decode: "CacheableFunction::new",
            encode: "CacheableFunction::getId",
        }
    }

    pub fn get(
        &mut self,
        manager: &mut ServerFunctionManagerModel,
    ) -> Option<CommandFunctionModel> {
        if !self.resolved {
            self.function = manager.get(&self.id);
            self.resolved = true;
        }

        self.function.clone()
    }

    pub fn get_id(&self) -> &Identifier {
        &self.id
    }

    pub fn resolved(&self) -> bool {
        self.resolved
    }
}

impl PartialEq for CacheableFunctionModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CacheableFunctionModel {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodecShapeModel {
    pub base_codec: &'static str,
    pub decode: &'static str,
    pub encode: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFunctionModel {
    id: Identifier,
    commands: Vec<String>,
}

impl CommandFunctionModel {
    pub fn new(id: Identifier, commands: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            id,
            commands: commands.into_iter().map(str::to_string).collect(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ServerFunctionManagerModel {
    functions: BTreeMap<Identifier, CommandFunctionModel>,
    lookups: Vec<Identifier>,
}

impl ServerFunctionManagerModel {
    pub fn new(functions: impl IntoIterator<Item = CommandFunctionModel>) -> Self {
        Self {
            functions: functions
                .into_iter()
                .map(|function| (function.id.clone(), function))
                .collect(),
            lookups: Vec::new(),
        }
    }

    fn get(&mut self, id: &Identifier) -> Option<CommandFunctionModel> {
        self.lookups.push(id.clone());
        self.functions.get(id).cloned()
    }

    pub fn lookups(&self) -> &[Identifier] {
        &self.lookups
    }
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn function(value: &str) -> CommandFunctionModel {
        CommandFunctionModel::new(id(value), ["say cached"])
    }

    #[test]
    fn cacheable_function_codec_maps_identifier_to_constructor_and_get_id() {
        let cacheable = CacheableFunctionModel::new(id("minecraft:foo"));

        assert_eq!(
            CacheableFunctionModel::codec_shape(),
            CodecShapeModel {
                base_codec: "Identifier.CODEC",
                decode: "CacheableFunction::new",
                encode: "CacheableFunction::getId",
            }
        );
        assert_eq!(cacheable.get_id(), &id("minecraft:foo"));
    }

    #[test]
    fn new_cacheable_function_starts_unresolved_with_no_cached_function() {
        let mut cacheable = CacheableFunctionModel::new(id("minecraft:missing"));
        let mut manager = ServerFunctionManagerModel::default();

        assert!(!cacheable.resolved());
        assert_eq!(cacheable.get(&mut manager), None);
        assert!(cacheable.resolved());
    }

    #[test]
    fn get_resolves_existing_function_once_and_caches_result() {
        let mut cacheable = CacheableFunctionModel::new(id("minecraft:foo"));
        let expected = function("minecraft:foo");
        let mut manager = ServerFunctionManagerModel::new([expected.clone()]);

        assert_eq!(cacheable.get(&mut manager), Some(expected.clone()));
        assert_eq!(cacheable.get(&mut manager), Some(expected));
        assert_eq!(manager.lookups(), [id("minecraft:foo")]);
    }

    #[test]
    fn get_caches_missing_function_and_does_not_retry() {
        let mut cacheable = CacheableFunctionModel::new(id("minecraft:missing"));
        let mut manager = ServerFunctionManagerModel::new([function("minecraft:other")]);

        assert_eq!(cacheable.get(&mut manager), None);
        assert_eq!(cacheable.get(&mut manager), None);
        assert_eq!(manager.lookups(), [id("minecraft:missing")]);
    }

    #[test]
    fn equality_is_based_on_identifier_only() {
        let mut resolved = CacheableFunctionModel::new(id("minecraft:foo"));
        let unresolved = CacheableFunctionModel::new(id("minecraft:foo"));
        let different = CacheableFunctionModel::new(id("minecraft:bar"));
        let mut manager = ServerFunctionManagerModel::new([function("minecraft:foo")]);

        assert!(resolved.get(&mut manager).is_some());
        assert_eq!(resolved, unresolved);
        assert_ne!(resolved, different);
    }
}
