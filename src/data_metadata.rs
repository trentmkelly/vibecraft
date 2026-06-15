use std::collections::BTreeMap;

const PACK_METADATA_GENERATOR_JAVA: &str = vibecraft_java_source!("/net/minecraft/data/metadata/PackMetadataGenerator.java");
const DATA_METADATA_PACKAGE_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/metadata/package-info.java");
const DATA_METADATA_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackMetadataGeneratorModel {
    output_folder: &'static str,
    elements: BTreeMap<&'static str, &'static str>,
}

impl PackMetadataGeneratorModel {
    fn new(output_folder: &'static str) -> Self {
        Self {
            output_folder,
            elements: BTreeMap::new(),
        }
    }

    fn add(mut self, section_name: &'static str, encoded_json: &'static str) -> Self {
        self.elements.insert(section_name, encoded_json);
        self
    }

    fn run(&self) -> (&'static str, BTreeMap<&'static str, &'static str>) {
        ("pack.mcmeta", self.elements.clone())
    }

    fn save_path(&self) -> String {
        format!("{}/pack.mcmeta", self.output_folder)
    }

    fn get_name(&self) -> &'static str {
        "Pack Metadata"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FeaturePackMetadataSummary {
    pack_section: &'static str,
    description: &'static str,
    pack_version_source: &'static str,
    feature_flags_section: Option<&'static str>,
}

fn feature_pack_metadata(description: &'static str) -> FeaturePackMetadataSummary {
    FeaturePackMetadataSummary {
        pack_section: "pack",
        description,
        pack_version_source:
            "DetectedVersion.BUILT_IN.packVersion(PackType.SERVER_DATA).minorRange()",
        feature_flags_section: None,
    }
}

fn feature_pack_metadata_with_flags(description: &'static str) -> FeaturePackMetadataSummary {
    FeaturePackMetadataSummary {
        feature_flags_section: Some("features"),
        ..feature_pack_metadata(description)
    }
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, expected: &[&str]) {
    for sentinel in expected {
        assert!(
            source.contains(sentinel),
            "missing Java sentinel: {sentinel}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_metadata_generator_adds_sections_by_type_name_and_overwrites_existing_keys() {
        let generator = PackMetadataGeneratorModel::new("generated")
            .add("pack", "{\"description\":\"one\"}")
            .add("features", "{\"enabled\":[\"bundle\"]}")
            .add("pack", "{\"description\":\"two\"}");
        let (_, elements) = generator.run();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements["pack"], "{\"description\":\"two\"}");
        assert_eq!(elements["features"], "{\"enabled\":[\"bundle\"]}");
    }

    #[test]
    fn pack_metadata_generator_run_and_name_match_java_provider_contract() {
        let generator = PackMetadataGeneratorModel::new("generated");
        let (file_name, elements) = generator.run();
        assert_eq!(file_name, "pack.mcmeta");
        assert!(elements.is_empty());
        assert_eq!(generator.save_path(), "generated/pack.mcmeta");
        assert_eq!(generator.get_name(), "Pack Metadata");
    }

    #[test]
    fn feature_pack_helpers_match_java_pack_and_feature_flag_sections() {
        assert_eq!(
            feature_pack_metadata("test pack"),
            FeaturePackMetadataSummary {
                pack_section: "pack",
                description: "test pack",
                pack_version_source:
                    "DetectedVersion.BUILT_IN.packVersion(PackType.SERVER_DATA).minorRange()",
                feature_flags_section: None,
            }
        );
        assert_eq!(
            feature_pack_metadata_with_flags("test pack").feature_flags_section,
            Some("features")
        );
    }

    #[test]
    fn pack_metadata_generator_java_source_counts_match_authoritative_file() {
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "new HashMap<>()"),
            1
        );
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "this.elements"),
            2
        );
        assert_eq!(count_occurrences(PACK_METADATA_GENERATOR_JAVA, "add("), 4);
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "DataProvider.saveStable"),
            1
        );
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "pack.mcmeta"),
            1
        );
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "forFeaturePack"),
            3
        );
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "FeatureFlagsMetadataSection"),
            3
        );
        assert_eq!(
            count_occurrences(PACK_METADATA_GENERATOR_JAVA, "PackMetadataSection"),
            3
        );
    }

    #[test]
    fn pack_metadata_generator_java_source_sentinels_match_authoritative_file() {
        assert_source_contains_all(
            PACK_METADATA_GENERATOR_JAVA,
            &[
                "public class PackMetadataGenerator implements DataProvider",
                "private final PackOutput output;",
                "private final Map<String, Supplier<JsonElement>> elements = new HashMap<>();",
                "this.elements\n         .put(type.name(), () -> ((JsonElement)type.codec().encodeStart(JsonOps.INSTANCE, value).getOrThrow(IllegalArgumentException::new)).getAsJsonObject());",
                "JsonObject result = new JsonObject();",
                "this.elements.forEach((id, data) -> result.add(id, data.get()));",
                "DataProvider.saveStable(cache, result, this.output.getOutputFolder().resolve(\"pack.mcmeta\"))",
                "return \"Pack Metadata\";",
                "PackMetadataSection.SERVER_TYPE",
                "new PackMetadataSection(description, DetectedVersion.BUILT_IN.packVersion(PackType.SERVER_DATA).minorRange())",
                "FeatureFlagsMetadataSection.TYPE",
                "new FeatureFlagsMetadataSection(flags)",
            ],
        );
    }

    #[test]
    fn data_metadata_package_info_matches_java_null_marked_metadata() {
        const { assert!(DATA_METADATA_PACKAGE_NULL_MARKED) };
        assert_eq!(
            count_occurrences(DATA_METADATA_PACKAGE_INFO_JAVA, "@NullMarked"),
            1
        );
        assert!(DATA_METADATA_PACKAGE_INFO_JAVA.contains("package net.minecraft.data.metadata;"));
        assert!(
            DATA_METADATA_PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;")
        );
    }
}
