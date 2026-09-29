//! Pack discovery and opening: `FolderRepositorySource` / `PackDetector` for the
//! world `datapacks` folder, `BuiltInPackSource` for the bundled feature packs,
//! and `Pack.ResourcesSupplier` (`PackContent::open`).

#[cfg(test)]
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::network::configuration::KnownPack;
use crate::registry_pipeline::resources::{
    CompositePack, DirectoryPack, PackResources, VANILLA_PACK_ID,
};
use crate::registry_pipeline::zip_pack::{SharedZip, ZipPack};
use crate::storage::validation::DirectoryValidator;

#[cfg(test)]
use super::WorldDataPack;
use super::{parse_filter_section, parse_pack_metadata, DataPack, DataPackMetadata, PackSource};

/// Where the resources of a [`DataPack`] live (`Pack.ResourcesSupplier`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackContent {
    /// A pack described by metadata only (no resources to open).
    Detached,
    /// The built-in `vanilla` pack.
    Vanilla,
    /// A bundled feature pack (`data/minecraft/datapacks/<id>`).
    BundledFeature(String),
    /// A pack directory containing `pack.mcmeta` and `data/`.
    Folder(PathBuf),
    /// A `.zip` archive containing `pack.mcmeta` and `data/`.
    Zip(PathBuf),
}

/// The bundled data root (`vanilla-data/data/minecraft/datapacks`).
fn bundled_feature_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data/data/minecraft/datapacks")
}

impl PackContent {
    /// `Pack.open` = `ResourcesSupplier.openFull`: the pack's resources with its
    /// applicable overlays layered on top.
    pub fn open(
        &self,
        pack_id: &str,
        metadata: &DataPackMetadata,
    ) -> Result<Box<dyn PackResources>, String> {
        match self {
            Self::Detached => Err(format!("Data pack {pack_id} has no resources")),
            Self::Vanilla => Ok(Box::new(DirectoryPack::vanilla())),
            Self::BundledFeature(id) => Ok(Self::open_directory(
                pack_id,
                &bundled_feature_root().join(id),
                Some(KnownPack::vanilla(id)),
                metadata,
            )),
            Self::Folder(path) => Ok(Self::open_directory(pack_id, path, None, metadata)),
            Self::Zip(path) => Self::open_zip(pack_id, path, metadata),
        }
    }

    fn open_directory(
        pack_id: &str,
        root: &Path,
        known_pack: Option<KnownPack>,
        metadata: &DataPackMetadata,
    ) -> Box<dyn PackResources> {
        let filter = fs::read_to_string(root.join("pack.mcmeta"))
            .ok()
            .and_then(|text| parse_filter_section(&text));
        let primary = DirectoryPack::new(pack_id, root, known_pack.clone()).with_filter(filter);
        if metadata.overlays.is_empty() {
            return Box::new(primary);
        }
        let overlays = metadata
            .overlays
            .iter()
            .map(|overlay| {
                Box::new(DirectoryPack::new(
                    pack_id,
                    root.join(overlay),
                    known_pack.clone(),
                )) as Box<dyn PackResources>
            })
            .collect();
        Box::new(CompositePack::new(Box::new(primary), overlays))
    }

    fn open_zip(
        pack_id: &str,
        path: &Path,
        metadata: &DataPackMetadata,
    ) -> Result<Box<dyn PackResources>, String> {
        let zip = SharedZip::open(path)
            .map_err(|err| format!("Failed to open pack {}: {err}", path.display()))?;
        let filter = ZipPack::new(pack_id, zip.clone(), "", None)
            .read_root("pack.mcmeta")
            .and_then(|text| parse_filter_section(&text));
        let primary = ZipPack::new(pack_id, zip.clone(), "", None).with_filter(filter);
        if metadata.overlays.is_empty() {
            return Ok(Box::new(primary));
        }
        let overlays = metadata
            .overlays
            .iter()
            .map(|overlay| {
                Box::new(ZipPack::new(pack_id, zip.clone(), overlay, None))
                    as Box<dyn PackResources>
            })
            .collect();
        Ok(Box::new(CompositePack::new(Box::new(primary), overlays)))
    }
}

/// `Pack.readMetaAndCreate`: builds a pack from `pack.mcmeta` text, or `None` when
/// the metadata is missing/invalid (Java logs a warning and drops the pack).
fn read_meta_and_create(
    id: &str,
    source: PackSource,
    mcmeta: Option<String>,
    content: PackContent,
) -> Option<DataPack> {
    let Some(text) = mcmeta else {
        crate::log::log_warn(&format!("Missing metadata in pack {id}"));
        return None;
    };
    match parse_pack_metadata(&text) {
        Ok(metadata) => Some(
            DataPack::new(id, source)
                .with_metadata(metadata)
                .with_content(content),
        ),
        Err(error) => {
            crate::log::log_warn(&format!("Failed to read pack {id} metadata: {error}"));
            None
        }
    }
}

/// `FolderRepositorySource.loadPacks` + `PackDetector`: every directory pack (with a
/// `pack.mcmeta` file) and `.zip` archive in `folder`, identified as
/// `file/<file name>`. The folder is created when missing
/// (`FileUtil.createDirectoriesSafe`).
pub fn discover_pack_folder(folder: &Path, source: PackSource) -> Vec<DataPack> {
    if let Err(error) = fs::create_dir_all(folder) {
        crate::log::log_warn(&format!(
            "Failed to list packs in {}: {error}",
            folder.display()
        ));
        return Vec::new();
    }
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    let validator = DirectoryValidator::deny_all();
    let mut packs = Vec::new();
    for path in paths {
        match detect_pack(&path, &validator) {
            Ok(Some((content, mcmeta))) => {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let id = format!("file/{name}");
                packs.extend(read_meta_and_create(&id, source, mcmeta, content));
            }
            Ok(None) => crate::log::log_info(&format!(
                "Found non-pack entry '{}', ignoring",
                path.display()
            )),
            Err(message) => crate::log::log_warn(&message),
        }
    }
    packs
}

/// `PackDetector.detectPackResources`: the content of a pack candidate plus its
/// `pack.mcmeta` text, `Ok(None)` for non-pack entries.
fn detect_pack(
    path: &Path,
    validator: &DirectoryValidator<fn(&Path) -> bool>,
) -> Result<Option<(PackContent, Option<String>)>, String> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(None);
    };
    let mut target = path.to_path_buf();
    let mut is_dir = metadata.is_dir();
    let mut is_file = metadata.is_file();
    if metadata.is_symlink() {
        let issues = validator.validate_symlink(path).map_err(|err| {
            format!(
                "Failed to read properties of '{}', ignoring: {err}",
                path.display()
            )
        })?;
        if !issues.is_empty() {
            return Err(forbidden_symlink_message(path));
        }
        target = fs::read_link(path).unwrap_or_else(|_| path.to_path_buf());
        let resolved = fs::symlink_metadata(&target).map_err(|err| {
            format!(
                "Failed to read properties of '{}', ignoring: {err}",
                path.display()
            )
        })?;
        is_dir = resolved.is_dir();
        is_file = resolved.is_file();
    }
    if is_dir {
        let mut issues = Vec::new();
        validator
            .validate_known_directory(&target, &mut issues)
            .map_err(|err| {
                format!(
                    "Failed to read properties of '{}', ignoring: {err}",
                    path.display()
                )
            })?;
        if !issues.is_empty() {
            return Err(forbidden_symlink_message(path));
        }
        let mcmeta = target.join("pack.mcmeta");
        if !mcmeta.is_file() {
            return Ok(None);
        }
        let text = fs::read_to_string(&mcmeta).ok();
        return Ok(Some((PackContent::Folder(target), text)));
    }
    let is_zip = is_file && target.extension().is_some_and(|ext| ext == "zip");
    if !is_zip {
        return Ok(None);
    }
    let text = SharedZip::open(&target)
        .ok()
        .and_then(|zip| ZipPack::new("", zip, "", None).read_root("pack.mcmeta"));
    Ok(Some((PackContent::Zip(target), text)))
}

fn forbidden_symlink_message(path: &Path) -> String {
    format!(
        "Ignoring potential pack entry: Failed to validate '{}'. Found forbidden symlinks",
        path.display()
    )
}

/// `ServerPacksSource.loadPacks`: the built-in `vanilla` pack and the bundled
/// feature packs (`BuiltInPackSource.listBundledPacks`).
pub fn builtin_packs() -> Vec<DataPack> {
    let mut packs = Vec::new();
    let vanilla_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data");
    let vanilla_meta = fs::read_to_string(vanilla_root.join("pack.mcmeta")).ok();
    packs.extend(read_meta_and_create(
        VANILLA_PACK_ID,
        PackSource::BuiltIn,
        Some(vanilla_meta.unwrap_or_else(|| super::VANILLA_PACK_MCMETA.to_string())),
        PackContent::Vanilla,
    ));
    if let Ok(entries) = fs::read_dir(bundled_feature_root()) {
        let mut ids: Vec<String> = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect();
        ids.sort();
        for id in ids {
            let mcmeta =
                fs::read_to_string(bundled_feature_root().join(&id).join("pack.mcmeta")).ok();
            packs.extend(read_meta_and_create(
                &id,
                PackSource::Feature,
                mcmeta,
                PackContent::BundledFeature(id.clone()),
            ));
        }
    }
    packs
}

/// Every pack of the world `datapacks` folder together with its text resources.
///
/// Unlike Java (which opens packs lazily) this eagerly reads UTF-8 files and is kept
/// for callers that inspect pack contents directly; the server itself opens packs
/// through [`PackContent::open`].
#[cfg(test)]
pub fn load_world_data_packs(datapack_dir: &Path) -> Result<Vec<WorldDataPack>, String> {
    if !datapack_dir.exists() {
        return Ok(Vec::new());
    }
    discover_pack_folder(datapack_dir, PackSource::World)
        .into_iter()
        .map(|pack| {
            let resources = match &pack.content {
                PackContent::Folder(root) => read_directory_resources(root)?,
                _ => BTreeMap::new(),
            };
            Ok(WorldDataPack { pack, resources })
        })
        .collect()
}

#[cfg(test)]
fn read_directory_resources(root: &Path) -> Result<BTreeMap<String, String>, String> {
    fn walk(root: &Path, current: &Path, out: &mut BTreeMap<String, String>) -> Result<(), String> {
        let entries = fs::read_dir(current).map_err(|err| {
            format!(
                "Failed to read datapack directory '{}': {err}",
                current.display()
            )
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if let (Ok(relative), Ok(text)) =
                (path.strip_prefix(root), fs::read_to_string(&path))
            {
                out.insert(relative.to_string_lossy().replace('\\', "/"), text);
            }
        }
        Ok(())
    }
    let mut resources = BTreeMap::new();
    walk(root, root, &mut resources)?;
    Ok(resources)
}

/// `DataPackCommand.createPack`: creates `pack_dir`, its `data` directory and a
/// `pack.mcmeta` with the current server data format range (`minorRange()` =
/// `min_format [major, minor]`, `max_format major`), pretty-printed like Gson with
/// a two-space indent.
pub fn write_pack_skeleton(pack_dir: &Path, description: &str) -> std::io::Result<()> {
    use super::PackFormat;
    fs::create_dir(pack_dir)?;
    fs::create_dir(pack_dir.join("data"))?;
    let current = PackFormat::current_server_data();
    let description = serde_json::to_string(description)?;
    let mcmeta = format!(
        "{{\n  \"pack\": {{\n    \"description\": {description},\n    \"min_format\": [\n      {},\n      {}\n    ],\n    \"max_format\": {}\n  }}\n}}",
        current.major, current.minor, current.major
    );
    fs::write(pack_dir.join("pack.mcmeta"), mcmeta)
}
