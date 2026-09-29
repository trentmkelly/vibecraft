//! Guards `docs/client_only_assets.md`: the dedicated server reads exactly two asset files
//! (`lang/en_us.json`, `lang/deprecated.json`) plus the `.mcassetsroot` pack-root probe, so every
//! other asset category (models, textures, fonts, sounds, shaders, ...) is a documented deferral.

use std::path::{Path, PathBuf};

/// The only asset paths the Java dedicated server references (relative to `assets/`).
const SERVER_READ_ASSETS: [&str; 3] = [
    "minecraft/lang/en_us.json",
    "minecraft/lang/deprecated.json",
    ".mcassetsroot",
];

fn collect_files(dir: &Path, root: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_files(&path, root, out);
        } else {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

/// The vendored asset tree holds only what the server reads; a new file means a new category
/// that must be classified in the deferral document.
#[test]
fn vendored_assets_contain_only_server_read_files() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/assets");
    let mut files = Vec::new();
    collect_files(&root, &root, &mut files);
    files.sort();
    let mut expected: Vec<String> = SERVER_READ_ASSETS.iter().map(|s| s.to_string()).collect();
    expected.sort();
    assert_eq!(files, expected);
}

/// The deferral document must exist and name every client-only category it defers.
#[test]
fn deferral_document_lists_client_only_categories() {
    let doc = include_str!("../docs/client_only_assets.md");
    for category in [
        "models", "textures", "font", "shaders", "sounds", "particles", "atlases", "blockstates",
        "items", "equipment", "post_effect", "texts", "waypoint_style", "gpu_warnlist.json",
        "regional_compliancies.json", "sounds.json", "resourcepacks",
    ] {
        assert!(doc.contains(category), "docs/client_only_assets.md missing {category}");
    }
}

/// With the optional decompilation present: every `assets/` path literal in the Java server
/// code is one of the server-read files, so no client-only category is referenced.
#[cfg(vibecraft_has_decompiled_sources)]
#[test]
fn java_server_references_only_server_read_asset_paths() {
    let root = PathBuf::from(env!("VIBECRAFT_DECOMPILED_SOURCE_ROOT")).join("net");
    let mut stack = vec![root];
    let mut referenced = std::collections::BTreeSet::new();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "java") {
                let text = std::fs::read_to_string(&path).unwrap_or_default();
                for line in text.lines().filter(|l| l.contains("\"/assets/") || l.contains("\"assets/")) {
                    referenced.insert(line.trim().to_string());
                }
            }
        }
    }
    for line in &referenced {
        assert!(
            SERVER_READ_ASSETS
                .iter()
                .any(|a| line.contains(&format!("/assets/{a}"))),
            "Java server references an undeferred asset path: {line}"
        );
    }
    assert!(referenced.iter().any(|l| l.contains("en_us.json")));
    assert!(referenced.iter().any(|l| l.contains("deprecated.json")));
}
