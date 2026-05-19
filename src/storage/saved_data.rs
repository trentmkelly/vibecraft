use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::storage::datafix::{require_current_tag_data_version, TARGET_DATA_VERSION};
use crate::storage::nbt::{read_gzip_named_tag, read_named_tag, write_gzip_named_tag, Tag};

#[derive(Debug, Clone, PartialEq)]
pub struct SavedDataEntry {
    pub data: Tag,
    dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SavedDataStorage {
    data_folder: PathBuf,
    cache: HashMap<ResourceLocation, Option<SavedDataEntry>>,
    closed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandStorage {
    saved_data: SavedDataStorage,
    namespaces: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceLocation {
    pub namespace: String,
    pub path: String,
}

impl SavedDataEntry {
    pub fn new(data: Tag) -> Self {
        Self { data, dirty: true }
    }

    pub fn clean(data: Tag) -> Self {
        Self { data, dirty: false }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }
}

impl SavedDataStorage {
    pub fn new(data_folder: impl Into<PathBuf>) -> Self {
        Self {
            data_folder: data_folder.into(),
            cache: HashMap::new(),
            closed: false,
        }
    }

    pub fn data_file(&self, id: &ResourceLocation) -> io::Result<PathBuf> {
        let path = self
            .data_folder
            .join(&id.namespace)
            .join(format!("{}.dat", id.path));
        let data_root = absolutize_for_check(&self.data_folder)?;
        let file_path = data_root
            .join(&id.namespace)
            .join(format!("{}.dat", id.path));
        if !file_path.starts_with(&data_root) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "saved data path escapes data directory",
            ));
        }
        Ok(path)
    }

    pub fn get(&mut self, id: &ResourceLocation) -> io::Result<Option<&Tag>> {
        if !self.cache.contains_key(id) {
            let loaded = self.read_saved_data(id)?;
            self.cache
                .insert(id.clone(), loaded.map(SavedDataEntry::clean));
        }
        Ok(self
            .cache
            .get(id)
            .and_then(Option::as_ref)
            .map(|entry| &entry.data))
    }

    pub fn compute_if_absent<F>(
        &mut self,
        id: ResourceLocation,
        constructor: F,
    ) -> io::Result<&mut Tag>
    where
        F: FnOnce() -> Tag,
    {
        if !self.cache.contains_key(&id) {
            let loaded = self.read_saved_data(&id)?;
            self.cache
                .insert(id.clone(), loaded.map(SavedDataEntry::clean));
        }
        if self.cache.get(&id).and_then(Option::as_ref).is_none() {
            self.cache
                .insert(id.clone(), Some(SavedDataEntry::new(constructor())));
        }
        Ok(&mut self.cache.get_mut(&id).unwrap().as_mut().unwrap().data)
    }

    pub fn set(&mut self, id: ResourceLocation, data: Tag) {
        self.cache.insert(id, Some(SavedDataEntry::new(data)));
    }

    pub fn schedule_save(&mut self) -> io::Result<Vec<PathBuf>> {
        if self.closed {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Trying to schedule save when SavedDataStorage is already closed",
            ));
        }

        let mut saved = Vec::new();
        let mut ids = self.cache.keys().cloned().collect::<Vec<_>>();
        ids.sort();
        for id in ids {
            let path = self.data_file(&id)?;
            let Some(Some(entry)) = self.cache.get_mut(&id) else {
                continue;
            };
            if !entry.is_dirty() {
                continue;
            }
            let tag = wrap_saved_data(entry.data.clone());
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            write_gzip_named_tag(fs::File::create(&path)?, "", &tag)?;
            entry.set_dirty(false);
            saved.push(path);
        }
        Ok(saved)
    }

    pub fn save_and_join(&mut self) -> io::Result<Vec<PathBuf>> {
        self.schedule_save()
    }

    pub fn close(&mut self) -> io::Result<Vec<PathBuf>> {
        if self.closed {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Trying to close SavedDataStorage when it is already closed",
            ));
        }
        let saved = self.save_and_join()?;
        self.closed = true;
        Ok(saved)
    }

    fn read_saved_data(&self, id: &ResourceLocation) -> io::Result<Option<Tag>> {
        let path = self.data_file(id)?;
        if !path.exists() {
            return Ok(None);
        }
        let (_name, tag) = read_saved_data_file(&path)?;
        require_current_tag_data_version(&id.to_string(), &tag)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        Ok(match tag {
            Tag::Compound(mut values) => values
                .drain(..)
                .find(|(name, _)| name == "data")
                .map(|(_, data)| data),
            other => Some(other),
        })
    }
}

impl CommandStorage {
    pub fn new(saved_data: SavedDataStorage) -> Self {
        Self {
            saved_data,
            namespaces: BTreeSet::new(),
        }
    }

    pub fn get(&mut self, id: &ResourceLocation) -> io::Result<Tag> {
        self.namespaces.insert(id.namespace.clone());
        let storage_id = command_storage_id(&id.namespace);
        Ok(command_storage_contents(self.saved_data.get(&storage_id)?)?
            .get(&id.path)
            .cloned()
            .unwrap_or_else(|| Tag::Compound(Vec::new())))
    }

    pub fn set(&mut self, id: ResourceLocation, contents: Tag) -> io::Result<()> {
        let storage_id = command_storage_id(&id.namespace);
        self.namespaces.insert(id.namespace.clone());
        let container = self
            .saved_data
            .compute_if_absent(storage_id, empty_command_storage_container)?;
        let mut contents_map = command_storage_contents_mut(container)?;
        if is_empty_compound(&contents) {
            contents_map.remove(&id.path);
        } else {
            contents_map.insert(id.path, contents);
        }
        Ok(())
    }

    pub fn data_get_storage(&mut self, id: &ResourceLocation) -> io::Result<Tag> {
        self.get(id)
    }

    pub fn data_merge_storage(&mut self, id: ResourceLocation, nbt: Tag) -> io::Result<bool> {
        if nbt_depth_exceeds(&nbt, 512) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "NBT path data is too deep",
            ));
        }
        let old = self.get(&id)?;
        let merged = merge_compound_tags(old.clone(), nbt)?;
        if old == merged {
            return Ok(false);
        }
        self.set(id, merged)?;
        Ok(true)
    }

    pub fn keys(&mut self) -> io::Result<Vec<ResourceLocation>> {
        let mut keys = Vec::new();
        for namespace in self.namespaces.clone() {
            let storage_id = command_storage_id(&namespace);
            for path in command_storage_contents(self.saved_data.get(&storage_id)?)?.keys() {
                keys.push(ResourceLocation {
                    namespace: namespace.clone(),
                    path: path.clone(),
                });
            }
        }
        keys.sort();
        Ok(keys)
    }

    pub fn save_and_join(&mut self) -> io::Result<Vec<PathBuf>> {
        self.saved_data.save_and_join()
    }
}

impl ResourceLocation {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> io::Result<Self> {
        let namespace = namespace.into();
        let path = path.into();
        validate_namespace(&namespace)?;
        validate_path(&path)?;
        Ok(Self { namespace, path })
    }

    pub fn parse(value: &str) -> io::Result<Self> {
        let (namespace, path) = value.split_once(':').unwrap_or(("minecraft", value));
        Self::new(namespace, path)
    }
}

impl std::fmt::Display for ResourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.path)
    }
}

fn command_storage_id(namespace: &str) -> ResourceLocation {
    ResourceLocation {
        namespace: namespace.to_string(),
        path: "command_storage".to_string(),
    }
}

fn empty_command_storage_container() -> Tag {
    Tag::Compound(vec![("contents".to_string(), Tag::Compound(Vec::new()))])
}

fn command_storage_contents(tag: Option<&Tag>) -> io::Result<BTreeMap<String, Tag>> {
    let Some(Tag::Compound(fields)) = tag else {
        return Ok(BTreeMap::new());
    };
    let Some((_, Tag::Compound(contents))) = fields.iter().find(|(name, _)| name == "contents")
    else {
        return Ok(BTreeMap::new());
    };
    Ok(contents.iter().cloned().collect())
}

fn command_storage_contents_mut(tag: &mut Tag) -> io::Result<BTreeMapProxy<'_>> {
    let Tag::Compound(fields) = tag else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "command storage container must be a compound",
        ));
    };
    if !fields.iter().any(|(name, _)| name == "contents") {
        fields.push(("contents".to_string(), Tag::Compound(Vec::new())));
    }
    let Some((_, Tag::Compound(contents))) = fields.iter_mut().find(|(name, _)| name == "contents")
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "command storage contents must be a compound",
        ));
    };
    Ok(BTreeMapProxy { contents })
}

struct BTreeMapProxy<'a> {
    contents: &'a mut Vec<(String, Tag)>,
}

impl BTreeMapProxy<'_> {
    fn insert(&mut self, key: String, value: Tag) {
        if let Some((_, existing)) = self.contents.iter_mut().find(|(name, _)| name == &key) {
            *existing = value;
        } else {
            self.contents.push((key, value));
            self.contents
                .sort_by(|(left, _), (right, _)| left.cmp(right));
        }
    }

    fn remove(&mut self, key: &str) {
        self.contents.retain(|(name, _)| name != key);
    }
}

fn wrap_saved_data(data: Tag) -> Tag {
    Tag::Compound(vec![
        ("data".to_string(), data),
        ("DataVersion".to_string(), Tag::Int(TARGET_DATA_VERSION)),
    ])
}

fn read_saved_data_file(path: &Path) -> io::Result<(String, Tag)> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(&[0x1f, 0x8b]) {
        read_gzip_named_tag(bytes.as_slice())
    } else {
        read_named_tag(&mut bytes.as_slice())
    }
}

fn is_empty_compound(tag: &Tag) -> bool {
    matches!(tag, Tag::Compound(values) if values.is_empty())
}

fn merge_compound_tags(mut old: Tag, incoming: Tag) -> io::Result<Tag> {
    let (Tag::Compound(old_values), Tag::Compound(incoming_values)) = (&mut old, incoming) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "data merge storage requires compound NBT",
        ));
    };
    merge_compound_values(old_values, incoming_values);
    Ok(old)
}

fn merge_compound_values(old: &mut Vec<(String, Tag)>, incoming: Vec<(String, Tag)>) {
    for (name, value) in incoming {
        match (
            old.iter_mut().find(|(old_name, _)| old_name == &name),
            value,
        ) {
            (Some((_, Tag::Compound(old_child))), Tag::Compound(incoming_child)) => {
                merge_compound_values(old_child, incoming_child);
            }
            (Some((_, old_value)), incoming_value) => {
                *old_value = incoming_value;
            }
            (None, incoming_value) => old.push((name, incoming_value)),
        }
    }
}

fn nbt_depth_exceeds(tag: &Tag, max_depth: usize) -> bool {
    fn walk(tag: &Tag, depth: usize, max_depth: usize) -> bool {
        if depth > max_depth {
            return true;
        }
        match tag {
            Tag::List(values) => values.iter().any(|child| walk(child, depth + 1, max_depth)),
            Tag::Compound(values) => values
                .iter()
                .any(|(_, child)| walk(child, depth + 1, max_depth)),
            _ => false,
        }
    }
    walk(tag, 0, max_depth)
}

fn absolutize_for_check(path: &Path) -> io::Result<PathBuf> {
    if path.exists() {
        path.canonicalize()
    } else if let Some(parent) = path.parent() {
        Ok(parent
            .canonicalize()?
            .join(path.file_name().unwrap_or_default()))
    } else {
        Ok(path.to_path_buf())
    }
}

fn validate_namespace(namespace: &str) -> io::Result<()> {
    if !namespace.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'_'
                || byte == b'-'
                || byte == b'.'
        })
    {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid resource namespace",
        ))
    }
}

fn validate_path(path: &str) -> io::Result<()> {
    if !path.is_empty()
        && !path.contains("..")
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'_' | b'-' | b'.' | b'/')
        })
    {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid resource path",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_data_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("rustcraft-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn saved_data_storage_loads_computes_caches_dirty_and_saves_data_wrappers() {
        let data_dir = temp_data_dir("saved-data-storage");
        let id = ResourceLocation::parse("minecraft:test_data").unwrap();
        let mut storage = SavedDataStorage::new(&data_dir);

        assert_eq!(storage.get(&id).unwrap(), None);
        assert!(storage.schedule_save().unwrap().is_empty());
        let computed = storage
            .compute_if_absent(id.clone(), || {
                Tag::Compound(vec![("value".to_string(), Tag::Int(7))])
            })
            .unwrap();
        assert_eq!(
            computed,
            &Tag::Compound(vec![("value".to_string(), Tag::Int(7))])
        );
        let saved = storage.schedule_save().unwrap();
        assert_eq!(
            saved,
            vec![data_dir.join("minecraft").join("test_data.dat")]
        );
        assert!(storage.schedule_save().unwrap().is_empty());

        let (_name, disk) = read_gzip_named_tag(fs::File::open(&saved[0]).unwrap()).unwrap();
        assert_eq!(
            disk,
            Tag::Compound(vec![
                (
                    "data".to_string(),
                    Tag::Compound(vec![("value".to_string(), Tag::Int(7))])
                ),
                ("DataVersion".to_string(), Tag::Int(TARGET_DATA_VERSION)),
            ])
        );

        let mut reloaded = SavedDataStorage::new(&data_dir);
        assert_eq!(
            reloaded.get(&id).unwrap(),
            Some(&Tag::Compound(vec![("value".to_string(), Tag::Int(7))]))
        );
        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn command_storage_uses_namespaced_saved_data_and_removes_empty_compounds() {
        let data_dir = temp_data_dir("command-storage");
        let mut storage = CommandStorage::new(SavedDataStorage::new(&data_dir));
        let first = ResourceLocation::parse("minecraft:foo/bar").unwrap();
        let second = ResourceLocation::parse("example:value").unwrap();

        assert_eq!(storage.get(&first).unwrap(), Tag::Compound(Vec::new()));
        storage
            .set(
                first.clone(),
                Tag::Compound(vec![("answer".to_string(), Tag::Int(42))]),
            )
            .unwrap();
        storage
            .set(
                second.clone(),
                Tag::Compound(vec![("name".to_string(), Tag::String("value".to_string()))]),
            )
            .unwrap();
        assert_eq!(storage.keys().unwrap(), vec![second.clone(), first.clone()]);
        storage
            .set(first.clone(), Tag::Compound(Vec::new()))
            .unwrap();
        assert_eq!(storage.get(&first).unwrap(), Tag::Compound(Vec::new()));
        assert_eq!(storage.keys().unwrap(), vec![second]);

        let saved = storage.save_and_join().unwrap();
        assert_eq!(
            saved,
            vec![
                data_dir.join("example").join("command_storage.dat"),
                data_dir.join("minecraft").join("command_storage.dat"),
            ]
        );
        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn command_storage_data_merge_then_get_matches_java_storage_accessor() {
        let data_dir = temp_data_dir("command-storage-data-commands");
        let mut storage = CommandStorage::new(SavedDataStorage::new(&data_dir));
        let id = ResourceLocation::parse("minecraft:test").unwrap();

        let changed = storage
            .data_merge_storage(
                id.clone(),
                Tag::Compound(vec![
                    (
                        "outer".to_string(),
                        Tag::Compound(vec![("a".to_string(), Tag::Int(1))]),
                    ),
                    ("name".to_string(), Tag::String("first".to_string())),
                ]),
            )
            .unwrap();
        assert!(changed);
        assert_eq!(
            storage.data_get_storage(&id).unwrap(),
            Tag::Compound(vec![
                (
                    "outer".to_string(),
                    Tag::Compound(vec![("a".to_string(), Tag::Int(1))])
                ),
                ("name".to_string(), Tag::String("first".to_string())),
            ])
        );

        let changed = storage
            .data_merge_storage(
                id.clone(),
                Tag::Compound(vec![
                    (
                        "outer".to_string(),
                        Tag::Compound(vec![("b".to_string(), Tag::Int(2))]),
                    ),
                    ("name".to_string(), Tag::String("second".to_string())),
                ]),
            )
            .unwrap();
        assert!(changed);
        assert_eq!(
            storage.data_get_storage(&id).unwrap(),
            Tag::Compound(vec![
                (
                    "outer".to_string(),
                    Tag::Compound(vec![
                        ("a".to_string(), Tag::Int(1)),
                        ("b".to_string(), Tag::Int(2)),
                    ])
                ),
                ("name".to_string(), Tag::String("second".to_string())),
            ])
        );

        assert!(!storage
            .data_merge_storage(
                id.clone(),
                Tag::Compound(vec![(
                    "outer".to_string(),
                    Tag::Compound(vec![("b".to_string(), Tag::Int(2))])
                )]),
            )
            .unwrap());
        assert!(storage.data_merge_storage(id, Tag::Int(3)).is_err());
        let _ = fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn saved_data_rejects_resource_locations_that_escape_data_folder() {
        assert!(ResourceLocation::parse("minecraft:../escape").is_err());
        assert!(ResourceLocation::parse("bad namespace:path").is_err());
    }
}
