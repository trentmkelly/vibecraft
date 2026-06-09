const NBT_TO_SNBT_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/structures/NbtToSnbt.java");
const SNBT_TO_NBT_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/structures/SnbtToNbt.java");
const SNBT_DATAFIXER_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/structures/SnbtDatafixer.java");
const STRUCTURE_UPDATER_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/structures/StructureUpdater.java"
);
const DATA_STRUCTURES_PACKAGE_JAVA: &str =
    include_str!("../../decompiled-server-26.1.2/net/minecraft/data/structures/package-info.java");

const NBT_TO_SNBT_SENTINELS: &[&str] = &[
    "public class NbtToSnbt implements DataProvider",
    "private final Iterable<Path> inputFolders;",
    "private final PackOutput output;",
    "Files.walk(input)",
    "walk.filter(path -> path.toString().endsWith(\".nbt\"))",
    "CompletableFuture.runAsync(() -> convertStructure(cache, path, getName(input, path), output), Util.ioPool())",
    "LOGGER.error(\"Failed to read structure input directory\", e);",
    "return \"NBT -> SNBT\";",
    "root.relativize(path).toString().replaceAll(\"\\\\\\\\\", \"/\")",
    "name.substring(0, name.length() - \".nbt\".length())",
    "Path resultPath = output.resolve(name + \".snbt\");",
    "writeSnbt(cache, resultPath, NbtUtils.structureToSnbt(NbtIo.readCompressed(input, NbtAccounter.unlimitedHeap())));",
    "LOGGER.info(\"Converted {} from NBT to SNBT\", name);",
    "LOGGER.error(\"Couldn't convert {} from NBT to SNBT at {}\", new Object[]{name, path, e});",
    "hashedBytes.write(text.getBytes(StandardCharsets.UTF_8));",
    "hashedBytes.write(10);",
    "cache.writeIfNeeded(destination, bytes.toByteArray(), hashedBytes.hash());",
];

const SNBT_TO_NBT_SENTINELS: &[&str] = &[
    "public class SnbtToNbt implements DataProvider",
    "private final List<SnbtToNbt.Filter> filters = Lists.newArrayList();",
    "public SnbtToNbt(final PackOutput output, final Path inputFolder)",
    "public SnbtToNbt addFilter(final SnbtToNbt.Filter filter)",
    "result = filter.apply(name, result);",
    "files.filter(path -> path.toString().endsWith(\".snbt\"))",
    "Util.backgroundExecutor().forName(\"SnbtToNbt\")",
    "throw new RuntimeException(\"Failed to read structure input directory, aborting\", e);",
    "return Util.sequenceFailFast(tasks);",
    "return \"SNBT -> NBT\";",
    "name.substring(0, name.length() - \".snbt\".length())",
    "String input = IOUtils.toString(reader);",
    "CompoundTag updated = this.applyFilters(name, NbtUtils.snbtToStructure(input));",
    "NbtIo.writeCompressed(updated, hos);",
    "return new SnbtToNbt.TaskResult(name, bytes, hash);",
    "Path destination = output.resolve(task.name + \".nbt\");",
    "cache.writeIfNeeded(destination, task.payload, task.hash);",
    "LOGGER.error(\"Couldn't write structure {} at {}\", new Object[]{task.name, destination, e});",
    "public interface Filter",
    "private static class StructureConversionException extends RuntimeException",
    "private record TaskResult(String name, byte[] payload, HashCode hash)",
];

const DATAFIXER_SENTINELS: &[&str] = &[
    "SharedConstants.setVersion(DetectedVersion.BUILT_IN);",
    "Bootstrap.bootStrap();",
    "for (String dir : args) {\n         updateInDirectory(dir);\n      }",
    "Files.walk(Paths.get(structureDir))",
    "walk.filter(path -> path.toString().endsWith(\".snbt\"))",
    "String snbt = Files.readString(path);",
    "CompoundTag readSnbt = NbtUtils.snbtToStructure(snbt);",
    "CompoundTag updatedTag = StructureUpdater.update(path.toString(), readSnbt);",
    "NbtToSnbt.writeSnbt(CachedOutput.NO_CACHE, path, NbtUtils.structureToSnbt(updatedTag));",
    "catch (CommandSyntaxException | IOException e)",
];

const STRUCTURE_UPDATER_SENTINELS: &[&str] = &[
    "public class StructureUpdater implements SnbtToNbt.Filter",
    "private static final String PREFIX = PackType.SERVER_DATA.getDirectory() + \"/minecraft/structure/\";",
    "return name.startsWith(PREFIX) ? update(name, input) : input;",
    "StructureTemplate structureTemplate = new StructureTemplate();",
    "int fromVersion = NbtUtils.getDataVersion(tag, 500);",
    "int toVersion = 4763;",
    "if (fromVersion < 4763)",
    "LOGGER.warn(\"SNBT Too old, do not forget to update: {} < {}: {}\", new Object[]{fromVersion, 4763, name});",
    "DataFixTypes.STRUCTURE.updateToCurrentVersion(DataFixers.getDataFixer(), tag, fromVersion);",
    "structureTemplate.load(BuiltInRegistries.BLOCK, updated);",
    "return structureTemplate.save(new CompoundTag());",
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructureUpdateDecision {
    should_update: bool,
    warn_too_old: bool,
    from_version: i32,
}

fn provider_name_without_extension(root: &str, path: &str, extension: &str) -> String {
    let relative = path
        .strip_prefix(root)
        .and_then(|rest| rest.strip_prefix('/'))
        .unwrap_or(path)
        .replace('\\', "/");
    relative
        .strip_suffix(extension)
        .unwrap_or(&relative)
        .to_string()
}

fn snbt_write_payload(text: &str) -> Vec<u8> {
    let mut bytes = text.as_bytes().to_vec();
    bytes.push(10);
    bytes
}

fn apply_filter_chain(mut tag: String, filters: &[fn(String) -> String]) -> String {
    for filter in filters {
        tag = filter(tag);
    }
    tag
}

fn structure_update_decision(name: &str, data_version: Option<i32>) -> StructureUpdateDecision {
    let from_version = data_version.unwrap_or(500);
    StructureUpdateDecision {
        should_update: name.starts_with("data/minecraft/structure/"),
        warn_too_old: from_version < 4763,
        from_version,
    }
}

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
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
    fn data_structures_nbt_to_snbt_contract_matches_java() {
        assert_source_contains_all(NBT_TO_SNBT_JAVA, NBT_TO_SNBT_SENTINELS);
        assert_eq!(NBT_TO_SNBT_JAVA.lines().count(), 100);
        assert_eq!(count_occurrences(NBT_TO_SNBT_JAVA, "CompletableFuture"), 11);
        assert_eq!(count_occurrences(NBT_TO_SNBT_JAVA, "endsWith(\".nbt\")"), 1);
        assert_eq!(
            provider_name_without_extension(
                "/input/root",
                "/input/root/village\\plains\\house.nbt",
                ".nbt"
            ),
            "village/plains/house"
        );
        assert_eq!(snbt_write_payload("{foo:1b}"), b"{foo:1b}\n".to_vec());
    }

    #[test]
    fn data_structures_snbt_to_nbt_contract_matches_java() {
        assert_source_contains_all(SNBT_TO_NBT_JAVA, SNBT_TO_NBT_SENTINELS);
        assert_eq!(SNBT_TO_NBT_JAVA.lines().count(), 126);
        assert_eq!(count_occurrences(SNBT_TO_NBT_JAVA, "SnbtToNbt.Filter"), 3);
        assert_eq!(
            count_occurrences(SNBT_TO_NBT_JAVA, "endsWith(\".snbt\")"),
            1
        );
        assert_eq!(
            provider_name_without_extension("/input", "/input/trial/start.snbt", ".snbt"),
            "trial/start"
        );

        fn add_a(tag: String) -> String {
            format!("{tag}|a")
        }
        fn add_b(tag: String) -> String {
            format!("{tag}|b")
        }
        assert_eq!(
            apply_filter_chain("root".to_string(), &[add_a, add_b]),
            "root|a|b"
        );
    }

    #[test]
    fn data_structures_snbt_datafixer_contract_matches_java() {
        assert_source_contains_all(SNBT_DATAFIXER_JAVA, DATAFIXER_SENTINELS);
        assert_eq!(SNBT_DATAFIXER_JAVA.lines().count(), 40);
        assert_eq!(
            count_occurrences(SNBT_DATAFIXER_JAVA, "StructureUpdater.update"),
            1
        );
        assert_eq!(
            count_occurrences(SNBT_DATAFIXER_JAVA, "CachedOutput.NO_CACHE"),
            1
        );
    }

    #[test]
    fn data_structures_structure_updater_contract_matches_java() {
        assert_source_contains_all(STRUCTURE_UPDATER_JAVA, STRUCTURE_UPDATER_SENTINELS);
        assert_eq!(STRUCTURE_UPDATER_JAVA.lines().count(), 34);
        assert_eq!(count_occurrences(STRUCTURE_UPDATER_JAVA, "4763"), 3);

        assert_eq!(
            structure_update_decision("data/minecraft/structure/trial/start", Some(3700)),
            StructureUpdateDecision {
                should_update: true,
                warn_too_old: true,
                from_version: 3700,
            }
        );
        assert_eq!(
            structure_update_decision("assets/minecraft/structure/ignored", None),
            StructureUpdateDecision {
                should_update: false,
                warn_too_old: true,
                from_version: 500,
            }
        );
    }

    #[test]
    fn data_structures_package_is_null_marked() {
        assert!(DATA_STRUCTURES_PACKAGE_JAVA.contains("@NullMarked"));
        assert!(DATA_STRUCTURES_PACKAGE_JAVA.contains("package net.minecraft.data.structures;"));
        assert!(
            DATA_STRUCTURES_PACKAGE_JAVA.contains("import org.jspecify.annotations.NullMarked;")
        );
        assert_eq!(
            count_occurrences(DATA_STRUCTURES_PACKAGE_JAVA, "@NullMarked"),
            1
        );
    }
}
