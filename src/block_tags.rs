//! Runtime access to the vanilla block tags (`data/minecraft/tags/block/*.json`,
//! vendored under `vanilla-data/`), with nested `#minecraft:tag` references
//! resolved exactly like Java `TagLoader.build`.
//!
//! 26.1.2 moved most block-support rules into data-driven tags (the
//! `supports_*` family consumed by `canSurvive` overrides), so survival checks
//! and placement logic resolve membership through this module rather than
//! hardcoded block lists.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

static BLOCK_TAGS: LazyLock<HashMap<String, HashSet<String>>> = LazyLock::new(|| {
    let tag_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/tags/block");
    let mut raw: HashMap<String, Vec<String>> = HashMap::new();
    // Tag ids may contain path separators (e.g. `minecraft:mineable/axe`), so
    // walk the directory tree recursively like Java's resource listing does.
    let mut directories = vec![tag_dir.clone()];
    while let Some(directory) = directories.pop() {
        let entries = std::fs::read_dir(&directory)
            .unwrap_or_else(|err| panic!("vendored block tag dir {}: {err}", directory.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|err| panic!("block tag dir entry: {err}"));
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            let Some(name) = path
                .strip_prefix(&tag_dir)
                .ok()
                .and_then(|relative| relative.to_str())
                .and_then(|relative| relative.strip_suffix(".json"))
            else {
                continue;
            };
            let body = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("block tag {}: {err}", path.display()));
            let json: serde_json::Value = serde_json::from_str(&body)
                .unwrap_or_else(|err| panic!("block tag {} JSON: {err}", path.display()));
            let values = json["values"]
                .as_array()
                .unwrap_or_else(|| panic!("block tag {} values", path.display()))
                .iter()
                .map(|value| match value {
                    serde_json::Value::String(id) => id.clone(),
                    // Optional-entry object form: { "id": ..., "required": false }.
                    serde_json::Value::Object(fields) => fields["id"]
                        .as_str()
                        .unwrap_or_else(|| panic!("block tag {} entry id", path.display()))
                        .to_string(),
                    other => panic!("block tag {} unexpected entry {other}", path.display()),
                })
                .collect();
            raw.insert(format!("minecraft:{name}"), values);
        }
    }

    // Resolve nested #tag references to flat block sets.
    fn resolve(
        tag: &str,
        raw: &HashMap<String, Vec<String>>,
        resolved: &mut HashMap<String, HashSet<String>>,
        in_progress: &mut HashSet<String>,
    ) -> HashSet<String> {
        if let Some(existing) = resolved.get(tag) {
            return existing.clone();
        }
        assert!(
            in_progress.insert(tag.to_string()),
            "cyclic block tag reference through {tag}"
        );
        let mut blocks = HashSet::new();
        for value in raw.get(tag).map(Vec::as_slice).unwrap_or_else(|| {
            panic!("block tag {tag} references missing tag");
        }) {
            if let Some(nested) = value.strip_prefix('#') {
                blocks.extend(resolve(nested, raw, resolved, in_progress));
            } else {
                blocks.insert(value.clone());
            }
        }
        in_progress.remove(tag);
        resolved.insert(tag.to_string(), blocks.clone());
        blocks
    }

    let mut resolved = HashMap::new();
    let tags: Vec<String> = raw.keys().cloned().collect();
    for tag in tags {
        resolve(&tag, &raw, &mut resolved, &mut HashSet::new());
    }
    resolved
});

/// Java: `BlockState.is(TagKey)` membership. Both arguments accept bare paths
/// or `minecraft:`-prefixed ids.
pub fn block_tag_contains(tag: &str, block_id: &str) -> bool {
    let tag = if tag.contains(':') {
        tag.to_string()
    } else {
        format!("minecraft:{tag}")
    };
    let block = if block_id.contains(':') {
        block_id.to_string()
    } else {
        format!("minecraft:{block_id}")
    };
    BLOCK_TAGS
        .get(&tag)
        .is_some_and(|blocks| blocks.contains(&block))
}

/// All members of a block tag, if it exists.
pub fn block_tag_members(tag: &str) -> Option<&'static HashSet<String>> {
    let tag = if tag.contains(':') {
        tag.to_string()
    } else {
        format!("minecraft:{tag}")
    };
    BLOCK_TAGS.get(&tag)
}

/// Number of loaded vanilla block tags (for coverage tests).
pub fn block_tag_count() -> usize {
    BLOCK_TAGS.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendored_block_tags_load_and_resolve_nested_references() {
        assert_eq!(block_tag_count(), 248);
        assert!(block_tag_contains("mineable/pickaxe", "minecraft:stone"));
        // supports_bamboo includes direct entries and #minecraft:sand members.
        assert!(block_tag_contains("supports_bamboo", "minecraft:bamboo"));
        assert!(block_tag_contains("minecraft:supports_bamboo", "gravel"));
        assert!(block_tag_contains("supports_bamboo", "minecraft:sand"));
        assert!(block_tag_contains("supports_bamboo", "minecraft:red_sand"));
        assert!(!block_tag_contains("supports_bamboo", "minecraft:stone"));
        // Unknown tags and blocks are simply absent.
        assert!(!block_tag_contains("not_a_tag", "minecraft:stone"));
        assert!(block_tag_members("minecraft:not_a_tag").is_none());

        // Spot checks against vanilla data used by canSurvive rules.
        assert!(block_tag_contains(
            "soul_fire_base_blocks",
            "minecraft:soul_sand"
        ));
        assert!(block_tag_contains("supports_cocoa", "minecraft:jungle_log"));
        // The 26.1.2 dirt tag is only dirt/coarse_dirt/rooted_dirt — grass_block
        // moved to dedicated supports_* tags.
        assert!(block_tag_contains("dirt", "minecraft:dirt"));
        assert!(!block_tag_contains("dirt", "minecraft:grass_block"));
    }
}
