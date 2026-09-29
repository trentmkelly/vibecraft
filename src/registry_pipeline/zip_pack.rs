//! `FilePackResources`: a data pack stored in a `.zip` archive.
//!
//! `pack.mcmeta` and `data/` sit at the archive root (or, for an overlay, below
//! the overlay directory used as prefix). One archive handle is shared by a pack
//! and its overlays (`SharedZipFileAccess`).

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::{Arc, Mutex};

use zip::ZipArchive;

use crate::network::configuration::KnownPack;
use crate::registry::Identifier;
use crate::registry_pipeline::resources::{FileToIdConverter, PackFile, PackResources};
use crate::resources::ResourceFilter;

/// `PackType.SERVER_DATA.getDirectory()`.
const DATA_DIRECTORY: &str = "data";

/// The opened archive shared between a pack and its overlays.
#[derive(Clone)]
pub struct SharedZip {
    archive: Arc<Mutex<ZipArchive<File>>>,
}

impl std::fmt::Debug for SharedZip {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SharedZip")
    }
}

impl SharedZip {
    /// Opens the archive at `path`.
    pub fn open(path: &Path) -> io::Result<Self> {
        let archive = ZipArchive::new(File::open(path)?).map_err(io::Error::other)?;
        Ok(Self {
            archive: Arc::new(Mutex::new(archive)),
        })
    }

    fn names(&self) -> Vec<String> {
        let archive = self.archive.lock().unwrap_or_else(|e| e.into_inner());
        archive
            .file_names()
            .filter(|name| !name.ends_with('/'))
            .map(str::to_string)
            .collect()
    }

    /// Reads the entry called `name` as UTF-8 text.
    pub fn read_entry(&self, name: &str) -> io::Result<String> {
        let mut archive = self.archive.lock().unwrap_or_else(|e| e.into_inner());
        let mut entry = archive.by_name(name).map_err(io::Error::other)?;
        let mut text = String::new();
        entry.read_to_string(&mut text)?;
        Ok(text)
    }
}

/// A data pack read from a zip archive.
pub struct ZipPack {
    id: String,
    zip: SharedZip,
    /// `FilePackResources.prefix`: empty for the pack itself, the overlay directory
    /// for an overlay.
    prefix: String,
    known_pack: Option<KnownPack>,
    filter: Option<ResourceFilter>,
}

impl ZipPack {
    /// Wraps `zip` (`prefix` is `""` or an overlay directory).
    pub fn new(id: &str, zip: SharedZip, prefix: &str, known_pack: Option<KnownPack>) -> Self {
        Self {
            id: id.to_string(),
            zip,
            prefix: prefix.to_string(),
            known_pack,
            filter: None,
        }
    }

    /// Attaches the pack's `filter` metadata section.
    pub fn with_filter(mut self, filter: Option<ResourceFilter>) -> Self {
        self.filter = filter;
        self
    }

    /// `FilePackResources.addPrefix`.
    fn add_prefix(&self, path: &str) -> String {
        if self.prefix.is_empty() {
            path.to_string()
        } else {
            format!("{}/{path}", self.prefix)
        }
    }

    /// `getRootResource("pack.mcmeta")`: the archive text at `path` below the prefix.
    pub fn read_root(&self, path: &str) -> Option<String> {
        self.zip.read_entry(&self.add_prefix(path)).ok()
    }
}

impl PackResources for ZipPack {
    fn pack_id(&self) -> &str {
        &self.id
    }

    fn known_pack(&self) -> Option<&KnownPack> {
        self.known_pack.as_ref()
    }

    /// `FilePackResources.getNamespaces` (`extractNamespace`).
    fn namespaces(&self) -> BTreeSet<String> {
        let type_prefix = self.add_prefix(&format!("{DATA_DIRECTORY}/"));
        self.zip
            .names()
            .iter()
            .filter_map(|name| name.strip_prefix(type_prefix.as_str()))
            .map(|rest| rest.split('/').next().unwrap_or_default())
            .filter(|namespace| !namespace.is_empty() && Identifier::is_valid_namespace(namespace))
            .map(str::to_string)
            .collect()
    }

    fn filter(&self) -> Option<&ResourceFilter> {
        self.filter.as_ref()
    }

    /// `FilePackResources.listResources` for every namespace, keeping the files
    /// `converter` accepts.
    fn list_resources(&self, converter: &FileToIdConverter) -> Vec<PackFile> {
        let mut names = self.zip.names();
        names.sort();
        let mut files = Vec::new();
        for namespace in self.namespaces() {
            let root = self.add_prefix(&format!("{DATA_DIRECTORY}/{namespace}/"));
            let directory = format!("{root}{}/", converter.prefix());
            for name in names.iter().filter(|name| name.starts_with(&directory)) {
                let Ok(location) = Identifier::new(&namespace, &name[root.len()..]) else {
                    continue;
                };
                if converter.file_to_id(&location).is_ok() {
                    files.push(PackFile::zip_entry(
                        location,
                        self.zip.clone(),
                        name.clone(),
                    ));
                }
            }
        }
        files
    }
}
