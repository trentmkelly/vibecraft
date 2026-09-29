//! Data pack resource access: `PackResources`, `ResourceManager`,
//! `FileToIdConverter`, and the built-in vanilla pack.
//!
//! Java's `MultiPackResourceManager` stacks packs from lowest to highest priority;
//! [`ResourceManager::list_matching_resources`] returns the top-most resource for
//! each id (used for registry elements) and
//! [`ResourceManager::list_matching_resource_stacks`] returns every pack's copy in
//! priority order (used for tags, which merge across packs).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::network::configuration::KnownPack;
use crate::registry::Identifier;
use crate::registry_pipeline::zip_pack::SharedZip;
use crate::resources::ResourceFilter;

/// Data directory inside a pack (`PackType.SERVER_DATA.getDirectory()`).
const DATA_DIRECTORY: &str = "data";

/// Pack id of the built-in vanilla pack (`VanillaPackResources`).
pub const VANILLA_PACK_ID: &str = "vanilla";

/// `FileToIdConverter`: maps resource-file identifiers to element identifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileToIdConverter {
    prefix: String,
    extension: String,
}

impl FileToIdConverter {
    /// `FileToIdConverter.json(prefix)`.
    pub fn json(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            extension: ".json".to_string(),
        }
    }

    /// The directory that is scanned inside each namespace.
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// `fileToId`.
    pub fn file_to_id(&self, file: &Identifier) -> Result<Identifier, String> {
        let path = file.path();
        let inner = path
            .strip_prefix(self.prefix.as_str())
            .and_then(|rest| rest.strip_prefix('/'))
            .and_then(|rest| rest.strip_suffix(self.extension.as_str()))
            .ok_or_else(|| {
                format!(
                    "{file} is not a {}/*{} resource",
                    self.prefix, self.extension
                )
            })?;
        file.with_path(inner)
    }

    fn matches_file_name(&self, name: &str) -> bool {
        name.ends_with(&self.extension)
    }
}

/// Where a listed file's bytes come from.
#[derive(Debug, Clone)]
enum FileHandle {
    /// A file on disk.
    Path(PathBuf),
    /// An entry of a shared zip archive.
    ZipEntry { archive: SharedZip, name: String },
}

/// A file found in a pack.
#[derive(Debug, Clone)]
pub struct PackFile {
    /// Resource-file id including directory prefix and extension.
    pub location: Identifier,
    handle: FileHandle,
}

impl PackFile {
    /// A file on disk.
    fn on_disk(location: Identifier, path: PathBuf) -> Self {
        Self {
            location,
            handle: FileHandle::Path(path),
        }
    }

    /// A zip archive entry.
    pub(crate) fn zip_entry(location: Identifier, archive: SharedZip, name: String) -> Self {
        Self {
            location,
            handle: FileHandle::ZipEntry { archive, name },
        }
    }

    /// Reads the file as UTF-8 text.
    pub fn read_to_string(&self) -> io::Result<String> {
        match &self.handle {
            FileHandle::Path(path) => fs::read_to_string(path),
            FileHandle::ZipEntry { archive, name } => archive.read_entry(name),
        }
    }
}

/// A source of data resources (`PackResources`).
pub trait PackResources: Send + Sync {
    /// `PackResources.packId()`.
    fn pack_id(&self) -> &str;

    /// The known-pack identity clients may already have (`PackLocationInfo.knownPackInfo`).
    fn known_pack(&self) -> Option<&KnownPack>;

    /// `PackResources.getNamespaces(SERVER_DATA)`.
    fn namespaces(&self) -> BTreeSet<String>;

    /// Lists every file matching `converter` in this pack.
    fn list_resources(&self, converter: &FileToIdConverter) -> Vec<PackFile>;

    /// The pack's `filter` metadata section, hiding resources of lower-priority
    /// packs (`ResourceFilterSection`).
    fn filter(&self) -> Option<&ResourceFilter> {
        None
    }
}

/// A pack backed by a directory with a `data/` tree.
#[derive(Debug, Clone)]
pub struct DirectoryPack {
    id: String,
    root: PathBuf,
    known_pack: Option<KnownPack>,
    filter: Option<ResourceFilter>,
}

impl DirectoryPack {
    /// Creates a pack rooted at `root` (the directory containing `data/`).
    pub fn new(id: &str, root: impl Into<PathBuf>, known_pack: Option<KnownPack>) -> Self {
        Self {
            id: id.to_string(),
            root: root.into(),
            known_pack,
            filter: None,
        }
    }

    /// Attaches the pack's `filter` metadata section.
    pub fn with_filter(mut self, filter: Option<ResourceFilter>) -> Self {
        self.filter = filter;
        self
    }

    /// The built-in vanilla data pack bundled under `vanilla-data/`. Clients that
    /// select `minecraft:core` already hold its contents, so registry entries that
    /// come from it need not be re-sent.
    pub fn vanilla() -> Self {
        Self::new(
            VANILLA_PACK_ID,
            Path::new(env!("CARGO_MANIFEST_DIR")).join("vanilla-data"),
            Some(vanilla_known_pack()),
        )
    }

    fn collect(
        directory: &Path,
        relative: &str,
        converter: &FileToIdConverter,
        namespace: &str,
        out: &mut Vec<PackFile>,
    ) {
        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = entry.path();
            let relative_path = format!("{relative}/{name}");
            if path.is_dir() {
                Self::collect(&path, &relative_path, converter, namespace, out);
            } else if converter.matches_file_name(&name) {
                // Java skips (and logs) files whose path is not a valid identifier.
                if let Ok(location) = Identifier::new(namespace, &relative_path) {
                    out.push(PackFile::on_disk(location, path));
                }
            }
        }
    }
}

impl PackResources for DirectoryPack {
    fn pack_id(&self) -> &str {
        &self.id
    }

    fn known_pack(&self) -> Option<&KnownPack> {
        self.known_pack.as_ref()
    }

    fn namespaces(&self) -> BTreeSet<String> {
        let Ok(entries) = fs::read_dir(self.root.join(DATA_DIRECTORY)) else {
            return BTreeSet::new();
        };
        entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| Identifier::is_valid_namespace(name))
            .collect()
    }

    fn filter(&self) -> Option<&ResourceFilter> {
        self.filter.as_ref()
    }

    fn list_resources(&self, converter: &FileToIdConverter) -> Vec<PackFile> {
        let data = self.root.join(DATA_DIRECTORY);
        let Ok(namespaces) = fs::read_dir(&data) else {
            return Vec::new();
        };
        let mut namespaces: Vec<_> = namespaces.flatten().collect();
        namespaces.sort_by_key(|entry| entry.file_name());
        let mut files = Vec::new();
        for namespace in namespaces {
            let name = namespace.file_name().to_string_lossy().into_owned();
            if !namespace.path().is_dir() || !Identifier::is_valid_namespace(&name) {
                continue;
            }
            Self::collect(
                &namespace.path().join(converter.prefix()),
                converter.prefix(),
                converter,
                &name,
                &mut files,
            );
        }
        files
    }
}

/// `KnownPack.vanilla("core")` at the current version.
pub fn vanilla_known_pack() -> KnownPack {
    KnownPack::vanilla("core")
}

/// A resource together with the pack it came from (`Resource`).
#[derive(Clone)]
pub struct Resource<'a> {
    pack: &'a dyn PackResources,
    file: PackFile,
}

impl Resource<'_> {
    /// `Resource.sourcePackId()`.
    pub fn source_pack_id(&self) -> &str {
        self.pack.pack_id()
    }

    /// `Resource.knownPackInfo()`.
    pub fn known_pack_info(&self) -> Option<&KnownPack> {
        self.pack.known_pack()
    }

    /// Reads the resource as UTF-8 text.
    pub fn read_to_string(&self) -> io::Result<String> {
        self.file.read_to_string()
    }
}

/// `CompositePackResources`: a primary pack with overlay directories layered on top;
/// later overlays win over earlier ones and all win over the primary.
pub struct CompositePack {
    primary: Box<dyn PackResources>,
    /// Overlays in declaration order.
    overlays: Vec<Box<dyn PackResources>>,
}

impl CompositePack {
    /// Layers `overlays` (declaration order) over `primary`.
    pub fn new(primary: Box<dyn PackResources>, overlays: Vec<Box<dyn PackResources>>) -> Self {
        Self { primary, overlays }
    }
}

impl PackResources for CompositePack {
    fn pack_id(&self) -> &str {
        self.primary.pack_id()
    }

    fn known_pack(&self) -> Option<&KnownPack> {
        self.primary.known_pack()
    }

    fn namespaces(&self) -> BTreeSet<String> {
        self.overlays
            .iter()
            .flat_map(|overlay| overlay.namespaces())
            .chain(self.primary.namespaces())
            .collect()
    }

    fn filter(&self) -> Option<&ResourceFilter> {
        self.primary.filter()
    }

    fn list_resources(&self, converter: &FileToIdConverter) -> Vec<PackFile> {
        // `result::putIfAbsent` over the stack [reversed overlays..., primary].
        let mut result: BTreeMap<Identifier, PackFile> = BTreeMap::new();
        for layer in self
            .overlays
            .iter()
            .rev()
            .chain(std::iter::once(&self.primary))
        {
            for file in layer.list_resources(converter) {
                result.entry(file.location.clone()).or_insert(file);
            }
        }
        result.into_values().collect()
    }
}

/// `MultiPackResourceManager`: packs ordered from lowest to highest priority.
#[derive(Default)]
pub struct ResourceManager {
    packs: Vec<Box<dyn PackResources>>,
}

impl ResourceManager {
    /// A manager over `packs` (lowest priority first).
    pub fn new(packs: Vec<Box<dyn PackResources>>) -> Self {
        Self { packs }
    }

    /// A manager holding only the built-in vanilla pack.
    pub fn vanilla() -> Self {
        Self::new(vec![Box::new(DirectoryPack::vanilla())])
    }

    /// `listPacks().knownPackInfo()`: the packs a client may already have.
    pub fn known_packs(&self) -> Vec<KnownPack> {
        self.packs
            .iter()
            .filter_map(|pack| pack.known_pack().cloned())
            .collect()
    }

    /// `FallbackResourceManager.listResourceStacks`: every pack's copy of each id,
    /// lowest priority first. A pack's `filter` first hides matching copies from the
    /// packs below it, in the namespaces it provides or filters.
    pub fn list_matching_resource_stacks(
        &self,
        converter: &FileToIdConverter,
    ) -> BTreeMap<Identifier, Vec<Resource<'_>>> {
        let mut stacks: BTreeMap<Identifier, Vec<Resource<'_>>> = BTreeMap::new();
        for pack in &self.packs {
            if let Some(filter) = pack.filter() {
                let provided = pack.namespaces();
                stacks.retain(|id, _| {
                    let applies = provided.contains(id.namespace())
                        || filter.is_namespace_filtered(id.namespace());
                    !(applies && filter.is_path_filtered(id.path()))
                });
            }
            for file in pack.list_resources(converter) {
                stacks
                    .entry(file.location.clone())
                    .or_default()
                    .push(Resource {
                        pack: pack.as_ref(),
                        file,
                    });
            }
        }
        stacks
    }

    /// `listMatchingResources`: the highest-priority copy of each id.
    pub fn list_matching_resources(
        &self,
        converter: &FileToIdConverter,
    ) -> BTreeMap<Identifier, Resource<'_>> {
        self.list_matching_resource_stacks(converter)
            .into_iter()
            .filter_map(|(id, mut stack)| stack.pop().map(|top| (id, top)))
            .collect()
    }
}
