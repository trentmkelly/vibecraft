#![allow(
    dead_code,
    reason = "Java SynchedEntityData is a reusable entity-metadata model; some parity APIs are currently exercised by tests before all entity runtime callers migrate"
)]

use crate::network::play::EntityDataValue;
use std::hash::{Hash, Hasher};

pub const MAX_ENTITY_DATA_ID: u8 = 254;

#[derive(Debug, Clone, Copy)]
pub struct EntityDataAccessor {
    pub id: u8,
    pub serializer_id: i32,
}

impl EntityDataAccessor {
    pub fn new(id: u8, serializer_id: i32) -> Result<Self, String> {
        if id > MAX_ENTITY_DATA_ID {
            return Err(format!(
                "Data value id is too big with {id}! (Max is {MAX_ENTITY_DATA_ID})"
            ));
        }
        Ok(Self { id, serializer_id })
    }

    pub fn data_value(&self, encoded_payload: Vec<u8>) -> EntityDataValue {
        EntityDataValue::raw(self.id, self.serializer_id, encoded_payload)
    }
}

impl PartialEq for EntityDataAccessor {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for EntityDataAccessor {}

impl Hash for EntityDataAccessor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::fmt::Display for EntityDataAccessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<entity data: {}>", self.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchedDataEvent {
    pub accessor_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncedDataHolderEvents {
    pub updated_accessors: Vec<SynchedDataEvent>,
    pub updated_batches: Vec<Vec<u8>>,
}

impl SyncedDataHolderEvents {
    fn on_synced_data_updated(&mut self, accessor: EntityDataAccessor) {
        self.updated_accessors.push(SynchedDataEvent {
            accessor_id: accessor.id,
        });
    }

    fn on_synced_data_updated_batch(&mut self, updated_items: &[EntityDataValue]) {
        self.updated_batches
            .push(updated_items.iter().map(|item| item.index).collect());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DataItem {
    accessor: EntityDataAccessor,
    value: EntityDataValue,
    initial_value: EntityDataValue,
    dirty: bool,
}

impl DataItem {
    fn new(accessor: EntityDataAccessor, encoded_payload: Vec<u8>) -> Self {
        let value = accessor.data_value(encoded_payload);
        Self {
            accessor,
            initial_value: value.clone(),
            value,
            dirty: false,
        }
    }

    fn is_set_to_default(&self) -> bool {
        self.value == self.initial_value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchedEntityData {
    items_by_id: Vec<DataItem>,
    dirty: bool,
    events: SyncedDataHolderEvents,
}

impl SynchedEntityData {
    pub fn builder(expected_item_count: usize) -> SynchedEntityDataBuilder {
        SynchedEntityDataBuilder {
            items_by_id: vec![None; expected_item_count],
        }
    }

    pub fn get(&self, accessor: EntityDataAccessor) -> Option<&EntityDataValue> {
        self.items_by_id
            .get(accessor.id as usize)
            .map(|item| &item.value)
    }

    pub fn set(
        &mut self,
        accessor: EntityDataAccessor,
        encoded_payload: Vec<u8>,
        force_dirty: bool,
    ) -> Result<(), String> {
        let item = self.item_mut(accessor)?;
        let next = accessor.data_value(encoded_payload);
        if force_dirty || next != item.value {
            item.value = next;
            item.dirty = true;
            self.dirty = true;
            self.events.on_synced_data_updated(accessor);
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn events(&self) -> &SyncedDataHolderEvents {
        &self.events
    }

    pub fn pack_dirty(&mut self) -> Option<Vec<EntityDataValue>> {
        if !self.dirty {
            return None;
        }
        self.dirty = false;
        let mut result = Vec::new();
        for item in &mut self.items_by_id {
            if item.dirty {
                item.dirty = false;
                result.push(item.value.clone());
            }
        }
        Some(result)
    }

    pub fn get_non_default_values(&self) -> Option<Vec<EntityDataValue>> {
        let result = self
            .items_by_id
            .iter()
            .filter(|item| !item.is_set_to_default())
            .map(|item| item.value.clone())
            .collect::<Vec<_>>();
        (!result.is_empty()).then_some(result)
    }

    pub fn assign_values(&mut self, items: Vec<EntityDataValue>) -> Result<(), String> {
        for item in &items {
            let data_item = self
                .items_by_id
                .get_mut(item.index as usize)
                .ok_or_else(|| format!("Unknown entity data item {}", item.index))?;
            if data_item.accessor.serializer_id != item.serializer_id {
                return Err(format!(
                    "Invalid entity data item type for field {}",
                    data_item.accessor.id
                ));
            }
            data_item.value = item.clone();
            self.events.on_synced_data_updated(data_item.accessor);
        }
        self.events.on_synced_data_updated_batch(&items);
        Ok(())
    }

    fn item_mut(&mut self, accessor: EntityDataAccessor) -> Result<&mut DataItem, String> {
        let item = self
            .items_by_id
            .get_mut(accessor.id as usize)
            .ok_or_else(|| format!("Unknown entity data item {}", accessor.id))?;
        if item.accessor.serializer_id != accessor.serializer_id {
            return Err(format!(
                "Invalid entity data item type for field {}",
                accessor.id
            ));
        }
        Ok(item)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchedEntityDataBuilder {
    items_by_id: Vec<Option<DataItem>>,
}

impl SynchedEntityDataBuilder {
    pub fn define(
        mut self,
        accessor: EntityDataAccessor,
        encoded_payload: Vec<u8>,
    ) -> Result<Self, String> {
        let id = accessor.id as usize;
        if id >= self.items_by_id.len() {
            return Err(format!(
                "Data value id is too big with {}! (Max is {})",
                accessor.id,
                self.items_by_id.len()
            ));
        }
        if self.items_by_id[id].is_some() {
            return Err(format!("Duplicate id value for {}!", accessor.id));
        }
        self.items_by_id[id] = Some(DataItem::new(accessor, encoded_payload));
        Ok(self)
    }

    pub fn build(self) -> Result<SynchedEntityData, String> {
        let mut items_by_id = Vec::with_capacity(self.items_by_id.len());
        for (id, item) in self.items_by_id.into_iter().enumerate() {
            let item = item.ok_or_else(|| {
                format!("Entity has not defined synched data value {id}")
            })?;
            items_by_id.push(item);
        }
        Ok(SynchedEntityData {
            items_by_id,
            dirty: false,
            events: SyncedDataHolderEvents::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::play::EntityMetadataValue;

    #[test]
    fn entity_data_accessor_equality_hash_and_display_match_java() {
        const ACCESSOR_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/syncher/EntityDataAccessor.java"
        );

        assert!(ACCESSOR_JAVA.contains("return this.id == that.id;"));
        assert!(ACCESSOR_JAVA.contains("return this.id;"));
        assert!(ACCESSOR_JAVA.contains("return \"<entity data: \" + this.id + \">\";"));

        let byte = EntityDataAccessor::new(2, 0).unwrap();
        let int_same_id = EntityDataAccessor::new(2, 1).unwrap();
        let other = EntityDataAccessor::new(3, 0).unwrap();
        assert_eq!(byte, int_same_id);
        assert_ne!(byte, other);
        assert_eq!(byte.to_string(), "<entity data: 2>");
    }

    #[test]
    fn synched_entity_data_dirty_default_and_assign_semantics_match_java() {
        const HOLDER_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/syncher/SyncedDataHolder.java"
        );
        const DATA_JAVA: &str = include_str!(
            "../../decompiled-server-26.1.2/net/minecraft/network/syncher/SynchedEntityData.java"
        );

        for sentinel in [
            "void onSyncedDataUpdated(EntityDataAccessor<?> accessor);",
            "void onSyncedDataUpdated(List<SynchedEntityData.DataValue<?>> updatedItems);",
        ] {
            assert!(HOLDER_JAVA.contains(sentinel));
        }
        for sentinel in [
            "private static final int MAX_ID_VALUE = 254;",
            "if (forceDirty || ObjectUtils.notEqual(value, dataItem.getValue()))",
            "public @Nullable List<SynchedEntityData.DataValue<?>> packDirty()",
            "public @Nullable List<SynchedEntityData.DataValue<?>> getNonDefaultValues()",
            "this.entity.onSyncedDataUpdated(items);",
            "if (!Objects.equals(item.serializer(), dataItem.accessor.serializer()))",
            "EntityDataSerializers.getSerializedId(this.serializer)",
            "throw new DecoderException(\"Unknown serializer type \" + type)",
        ] {
            assert!(DATA_JAVA.contains(sentinel), "missing Java sentinel {sentinel}");
        }

        let byte = EntityDataAccessor::new(0, 0).unwrap();
        let var_int = EntityDataAccessor::new(1, 1).unwrap();
        let mut data = SynchedEntityData::builder(2)
            .define(byte, vec![0])
            .unwrap()
            .define(var_int, vec![0])
            .unwrap()
            .build()
            .unwrap();

        assert!(!data.is_dirty());
        assert_eq!(data.pack_dirty(), None);
        assert_eq!(data.get_non_default_values(), None);

        data.set(byte, vec![0], false).unwrap();
        assert!(!data.is_dirty());
        assert!(data.events().updated_accessors.is_empty());

        data.set(byte, vec![1], false).unwrap();
        assert!(data.is_dirty());
        assert_eq!(data.events().updated_accessors[0].accessor_id, 0);
        assert_eq!(data.get_non_default_values().unwrap()[0], byte.data_value(vec![1]));
        assert_eq!(data.pack_dirty().unwrap(), vec![byte.data_value(vec![1])]);
        assert!(!data.is_dirty());
        assert_eq!(data.pack_dirty(), None);

        data.set(byte, vec![1], true).unwrap();
        assert_eq!(data.pack_dirty().unwrap(), vec![byte.data_value(vec![1])]);

        let assigned = EntityMetadataValue::VarInt(300);
        let assigned_value = EntityDataValue::typed(1, assigned).unwrap();
        data.assign_values(vec![assigned_value.clone()]).unwrap();
        assert_eq!(data.get(var_int), Some(&assigned_value));
        assert_eq!(data.events().updated_batches.last(), Some(&vec![1]));

        let mismatch = EntityDataValue::raw(1, 0, vec![3]);
        assert!(data.assign_values(vec![mismatch]).unwrap_err().contains(
            "Invalid entity data item type for field 1"
        ));
    }

    #[test]
    fn synched_entity_data_builder_rejects_duplicate_missing_and_oversized_ids() {
        assert!(EntityDataAccessor::new(255, 0).is_err());
        let byte = EntityDataAccessor::new(0, 0).unwrap();
        let duplicate = SynchedEntityData::builder(1)
            .define(byte, vec![0])
            .unwrap()
            .define(byte, vec![1])
            .unwrap_err();
        assert!(duplicate.contains("Duplicate id value for 0"));

        let missing = SynchedEntityData::builder(2)
            .define(byte, vec![0])
            .unwrap()
            .build()
            .unwrap_err();
        assert!(missing.contains("has not defined synched data value 1"));

        let too_large = SynchedEntityData::builder(1)
            .define(EntityDataAccessor::new(1, 0).unwrap(), vec![0])
            .unwrap_err();
        assert!(too_large.contains("Data value id is too big with 1"));
    }
}
