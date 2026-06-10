#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::biome::BUILTIN_BIOMES;
use crate::block_catalog::TOP_LEVEL_BLOCK_CLASSES;
use crate::command_tree::{vanilla_like_tree, CommandNodeKind};
use crate::item_catalog::ITEM_SOURCE_SURFACE;
use crate::storage::region::ChunkPos;
use crate::worldgen::generate_overworld_chunk_for_preset;
use crate::worldgen_comparison::build_vibecraft_worldgen_report;

pub fn generate_reports(root: impl AsRef<Path>) -> Result<PathBuf, String> {
    let report_dir = root.as_ref().join("reports");
    let tag_dir = root.as_ref().join("data").join("minecraft").join("tags");
    fs::create_dir_all(&report_dir)
        .map_err(|err| format!("failed to create '{}': {err}", report_dir.display()))?;
    fs::create_dir_all(tag_dir.join("block"))
        .map_err(|err| format!("failed to create generated block tags: {err}"))?;
    fs::create_dir_all(tag_dir.join("item"))
        .map_err(|err| format!("failed to create generated item tags: {err}"))?;

    write_json(report_dir.join("registries.json"), &registries_report())?;
    write_json(report_dir.join("commands.json"), &commands_report())?;
    write_json(report_dir.join("biomes.json"), &biomes_report())?;
    write_json(report_dir.join("blocks.json"), &blocks_report())?;
    write_json(report_dir.join("items.json"), &items_report())?;
    write_json(
        report_dir.join("worldgen_chunks.json"),
        &worldgen_chunks_report()?,
    )?;
    write_json(tag_dir.join("block").join("mineable.json"), &empty_tag())?;
    write_json(tag_dir.join("item").join("tools.json"), &empty_tag())?;
    Ok(report_dir)
}

fn registries_report() -> Value {
    json!({
        "minecraft:worldgen/biome": {
            "protocol_id": 0,
            "default": "minecraft:plains",
            "entries": ids_with_protocol(BUILTIN_BIOMES.iter().map(|biome| biome.id)),
        },
        "minecraft:block": {
            "entries": ids_with_protocol(TOP_LEVEL_BLOCK_CLASSES.iter().map(|entry| entry.class_name)),
        },
        "minecraft:item": {
            "entries": ids_with_protocol(ITEM_SOURCE_SURFACE.iter().map(|entry| entry.java_name)),
        },
    })
}

fn commands_report() -> Value {
    let tree = vanilla_like_tree();
    let nodes = tree
        .nodes
        .iter()
        .map(|node| {
            json!({
                "id": node.id,
                "type": command_node_type(&node.kind),
                "name": command_node_name(&node.kind),
                "executable": node.executable,
                "requirement_level": node.requirement_level,
                "children": node.children,
                "redirect": node.redirect,
                "fork": node.fork,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "type": "root",
        "root": tree.root,
        "nodes": nodes,
    })
}

fn biomes_report() -> Value {
    json!({
        "count": BUILTIN_BIOMES.len(),
        "values": BUILTIN_BIOMES.iter().map(|biome| biome.id).collect::<Vec<_>>(),
    })
}

fn blocks_report() -> Value {
    json!({
        "count": TOP_LEVEL_BLOCK_CLASSES.len(),
        "values": TOP_LEVEL_BLOCK_CLASSES.iter().map(|entry| entry.class_name).collect::<Vec<_>>(),
    })
}

fn items_report() -> Value {
    json!({
        "count": ITEM_SOURCE_SURFACE.len(),
        "values": ITEM_SOURCE_SURFACE.iter().map(|entry| entry.java_name).collect::<Vec<_>>(),
    })
}

fn worldgen_chunks_report() -> Result<Value, String> {
    let chunks = [
        generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "flat")?,
        generate_overworld_chunk_for_preset(ChunkPos { x: 1, z: 0 }, "flat")?,
        generate_overworld_chunk_for_preset(ChunkPos { x: 0, z: 0 }, "normal")?,
    ];
    Ok(build_vibecraft_worldgen_report(
        chunks.iter().map(|chunk| ("overworld", chunk)),
    ))
}

fn ids_with_protocol<'a>(ids: impl Iterator<Item = &'a str>) -> BTreeMap<&'a str, Value> {
    ids.enumerate()
        .map(|(protocol_id, id)| (id, json!({ "protocol_id": protocol_id })))
        .collect()
}

fn command_node_type(kind: &CommandNodeKind) -> &'static str {
    match kind {
        CommandNodeKind::Root => "root",
        CommandNodeKind::Literal(_) => "literal",
        CommandNodeKind::Argument { .. } => "argument",
    }
}

fn command_node_name(kind: &CommandNodeKind) -> Option<&'static str> {
    match kind {
        CommandNodeKind::Root => None,
        CommandNodeKind::Literal(name) => Some(name),
        CommandNodeKind::Argument { name, .. } => Some(name),
    }
}

fn empty_tag() -> Value {
    json!({
        "replace": false,
        "values": [],
    })
}

fn write_json(path: PathBuf, value: &Value) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(value)
        .map_err(|err| format!("failed to encode '{}': {err}", path.display()))?;
    fs::write(&path, format!("{raw}\n"))
        .map_err(|err| format!("failed to write '{}': {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::generate_reports;

    #[test]
    fn report_generator_writes_vanilla_named_outputs() {
        let root = std::env::temp_dir().join(format!(
            "vibecraft-generated-reports-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);

        let report_dir = generate_reports(&root).unwrap();
        assert_eq!(report_dir, root.join("reports"));

        for file in [
            "registries.json",
            "commands.json",
            "biomes.json",
            "blocks.json",
            "items.json",
            "worldgen_chunks.json",
        ] {
            let path = report_dir.join(file);
            assert!(path.is_file(), "missing {}", path.display());
            let value: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            assert!(
                value.is_object(),
                "{} should contain a JSON object",
                path.display()
            );
        }
        assert!(root
            .join("data/minecraft/tags/block/mineable.json")
            .is_file());
        assert!(root.join("data/minecraft/tags/item/tools.json").is_file());

        let registries: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(report_dir.join("registries.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            registries["minecraft:worldgen/biome"]["entries"]
                .as_object()
                .unwrap()
                .len(),
            65
        );
        let worldgen: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(report_dir.join("worldgen_chunks.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            worldgen["format"].as_str(),
            Some("vibecraft-worldgen-signatures-v1")
        );
        assert_eq!(worldgen["chunks"].as_array().unwrap().len(), 3);

        let _ = std::fs::remove_dir_all(&root);
    }
}
