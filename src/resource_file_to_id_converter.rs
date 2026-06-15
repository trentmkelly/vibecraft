#![allow(dead_code)]

use crate::registry::{Identifier, ResourceKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileToIdConverter {
    prefix: String,
    extension: String,
}

impl FileToIdConverter {
    pub fn new(prefix: impl Into<String>, extension: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            extension: extension.into(),
        }
    }

    pub fn json(prefix: impl Into<String>) -> Self {
        Self::new(prefix, ".json")
    }

    pub fn registry<T>(registry: &ResourceKey<T>) -> Self {
        Self::json(registry.location().path())
    }

    pub fn id_to_file(&self, id: &Identifier) -> Result<Identifier, String> {
        Identifier::new(
            id.namespace(),
            &format!("{}/{}{}", self.prefix, id.path(), self.extension),
        )
    }

    pub fn file_to_id(&self, file: &Identifier) -> Result<Identifier, String> {
        let path = file.path();
        let start = self.prefix.len() + 1;
        let end = path
            .len()
            .checked_sub(self.extension.len())
            .ok_or_else(|| "file path is shorter than extension".to_string())?;
        if start > end {
            return Err("file path is shorter than prefix and extension".to_string());
        }
        Identifier::new(file.namespace(), &path[start..end])
    }

    pub fn extension_matches(&self, id: &Identifier) -> bool {
        id.path().ends_with(&self.extension)
    }

    pub fn matching_resources<'a, R>(
        &self,
        resources: impl IntoIterator<Item = (&'a Identifier, R)>,
    ) -> Vec<(&'a Identifier, R)> {
        resources
            .into_iter()
            .filter(|(id, _)| id.path().starts_with(&self.prefix) && self.extension_matches(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_TO_ID_CONVERTER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/resources/FileToIdConverter.java");

    #[test]
    fn json_converter_maps_ids_to_prefixed_files_and_back_like_java() {
        assert_java_contains(
            FILE_TO_ID_CONVERTER_JAVA,
            &[
                "public record FileToIdConverter(String prefix, String extension)",
                "return new FileToIdConverter(prefix, \".json\");",
                "return id.withPath(this.prefix + \"/\" + id.getPath() + this.extension);",
                "return file.withPath(path.substring(this.prefix.length() + 1, path.length() - this.extension.length()));",
                "return id.getPath().endsWith(this.extension);",
            ],
        );

        let converter = FileToIdConverter::json("worldgen/biome");
        let plains = Identifier::parse("minecraft:plains").unwrap();
        let file = converter.id_to_file(&plains).unwrap();
        assert_eq!(file.to_string(), "minecraft:worldgen/biome/plains.json");
        assert_eq!(converter.file_to_id(&file).unwrap(), plains);
        assert!(converter.extension_matches(&file));
        assert!(!converter.extension_matches(
            &Identifier::parse("minecraft:worldgen/biome/plains.txt").unwrap()
        ));
    }

    #[test]
    fn registry_converter_uses_registry_element_directory_path() {
        assert_java_contains(
            FILE_TO_ID_CONVERTER_JAVA,
            &[
                "public static FileToIdConverter registry(final ResourceKey<? extends Registry<?>> registry)",
                "return json(Registries.elementsDirPath(registry));",
            ],
        );
        let registry = ResourceKey::<()>::new(
            Identifier::parse("minecraft:root").unwrap(),
            Identifier::parse("minecraft:worldgen/biome").unwrap(),
        );
        let converter = FileToIdConverter::registry(&registry);
        assert_eq!(
            converter
                .id_to_file(&Identifier::parse("custom:sky/islands").unwrap())
                .unwrap()
                .to_string(),
            "custom:worldgen/biome/sky/islands.json"
        );
    }

    #[test]
    fn matching_resources_uses_prefix_and_extension_predicate_like_resource_manager_call() {
        assert_java_contains(
            FILE_TO_ID_CONVERTER_JAVA,
            &[
                "return manager.listResources(this.prefix, this::extensionMatches);",
                "return manager.listResourceStacks(this.prefix, this::extensionMatches);",
            ],
        );
        let converter = FileToIdConverter::json("loot_table");
        let loot = Identifier::parse("minecraft:loot_table/chests/simple_dungeon.json").unwrap();
        let wrong_extension =
            Identifier::parse("minecraft:loot_table/chests/simple_dungeon.mcmeta").unwrap();
        let wrong_prefix = Identifier::parse("minecraft:tags/block/mineable/pickaxe.json").unwrap();

        let matches = converter.matching_resources([
            (&loot, "loot"),
            (&wrong_extension, "metadata"),
            (&wrong_prefix, "tag"),
        ]);
        assert_eq!(matches, vec![(&loot, "loot")]);
    }

    fn assert_java_contains(source: &str, sentinels: &[&str]) {
        if source.is_empty() {
            return;
        }
        for sentinel in sentinels {
            assert!(
                source.contains(sentinel),
                "missing Java source sentinel {sentinel}"
            );
        }
    }
}
