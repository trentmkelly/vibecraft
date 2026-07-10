#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument};
use crate::command::BlockPos;
use crate::storage::nbt::{nbt_utils, Tag};
use crate::storage::saved_data::{CommandStorage, ResourceLocation};

pub const ERROR_NOT_A_BLOCK_ENTITY: &str = "commands.data.block.invalid";
pub const ERROR_NO_PLAYERS: &str = "commands.data.entity.invalid";

pub trait DataAccessor {
    fn set_data(&mut self, tag: Tag) -> Result<(), String>;
    fn get_data(&mut self) -> Result<Tag, String>;
    fn get_modified_success(&self) -> Component;
    fn get_print_success(&self, data: &Tag) -> Component;
    fn get_scaled_print_success(&self, path: &str, scale: f64, value: i32) -> Component;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockUpdateEvent {
    pub position: BlockPos,
    pub old_state: String,
    pub new_state: String,
    pub flags: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBlockEntityModel {
    pub block_state: String,
    pub full_metadata: Tag,
    pub problem_path: String,
    pub changed: bool,
    pub updates: Vec<BlockUpdateEvent>,
}

impl CommandBlockEntityModel {
    pub fn new(block_state: impl Into<String>, full_metadata: Tag, problem_path: impl Into<String>) -> Self {
        Self {
            block_state: block_state.into(),
            full_metadata,
            problem_path: problem_path.into(),
            changed: false,
            updates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockDataAccessor {
    pub entity: CommandBlockEntityModel,
    pub position: BlockPos,
}

impl BlockDataAccessor {
    pub const fn new(entity: CommandBlockEntityModel, position: BlockPos) -> Self {
        Self { entity, position }
    }
}

impl DataAccessor for BlockDataAccessor {
    fn set_data(&mut self, tag: Tag) -> Result<(), String> {
        let Tag::Compound(_) = tag else {
            return Err("BlockDataAccessor requires a compound tag".to_string());
        };
        let state = self.entity.block_state.clone();
        self.entity.full_metadata = tag;
        self.entity.changed = true;
        self.entity.updates.push(BlockUpdateEvent {
            position: self.position,
            old_state: state.clone(),
            new_state: state,
            flags: 3,
        });
        Ok(())
    }

    fn get_data(&mut self) -> Result<Tag, String> {
        Ok(self.entity.full_metadata.clone())
    }

    fn get_modified_success(&self) -> Component {
        position_component("commands.data.block.modified", self.position, Vec::new())
    }

    fn get_print_success(&self, data: &Tag) -> Component {
        position_component(
            "commands.data.block.query",
            self.position,
            vec![ComponentArgument::Component(Box::new(
                nbt_utils::to_pretty_component(data),
            ))],
        )
    }

    fn get_scaled_print_success(&self, path: &str, scale: f64, value: i32) -> Component {
        Component::translatable(
            "commands.data.block.get",
            vec![
                ComponentArgument::String(path.to_string()),
                ComponentArgument::Number(self.position.x),
                ComponentArgument::Number(self.position.y),
                ComponentArgument::Number(self.position.z),
                ComponentArgument::String(format!("{scale:.2}")),
                ComponentArgument::Number(value),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandEntityModel {
    pub uuid: String,
    pub display_name: Component,
    pub is_player: bool,
    pub comparison_tag: Tag,
    pub problem_path: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityDataAccessor {
    pub entity: CommandEntityModel,
}

impl EntityDataAccessor {
    pub const fn new(entity: CommandEntityModel) -> Self {
        Self { entity }
    }
}

impl DataAccessor for EntityDataAccessor {
    fn set_data(&mut self, tag: Tag) -> Result<(), String> {
        if self.entity.is_player {
            return Err(ERROR_NO_PLAYERS.to_string());
        }
        let Tag::Compound(_) = tag else {
            return Err("EntityDataAccessor requires a compound tag".to_string());
        };
        let uuid = self.entity.uuid.clone();
        self.entity.comparison_tag = tag;
        self.entity.uuid = uuid;
        Ok(())
    }

    fn get_data(&mut self) -> Result<Tag, String> {
        Ok(self.entity.comparison_tag.clone())
    }

    fn get_modified_success(&self) -> Component {
        Component::translatable(
            "commands.data.entity.modified",
            vec![ComponentArgument::Component(Box::new(
                self.entity.display_name.clone(),
            ))],
        )
    }

    fn get_print_success(&self, data: &Tag) -> Component {
        Component::translatable(
            "commands.data.entity.query",
            vec![
                ComponentArgument::Component(Box::new(self.entity.display_name.clone())),
                ComponentArgument::Component(Box::new(nbt_utils::to_pretty_component(data))),
            ],
        )
    }

    fn get_scaled_print_success(&self, path: &str, scale: f64, value: i32) -> Component {
        Component::translatable(
            "commands.data.entity.get",
            vec![
                ComponentArgument::String(path.to_string()),
                ComponentArgument::Component(Box::new(self.entity.display_name.clone())),
                ComponentArgument::String(format!("{scale:.2}")),
                ComponentArgument::Number(value),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityDataProviderModel {
    pub argument_name: String,
}

impl EntityDataProviderModel {
    pub fn new(argument_name: impl Into<String>) -> Self {
        Self {
            argument_name: argument_name.into(),
        }
    }

    pub fn access(&self, entity: CommandEntityModel) -> EntityDataAccessor {
        EntityDataAccessor::new(entity)
    }

    pub fn wrapped_argument_shape(&self) -> String {
        format!("entity <{}>", self.argument_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageDataAccessor {
    pub storage: CommandStorage,
    pub id: ResourceLocation,
}

impl StorageDataAccessor {
    pub const fn new(storage: CommandStorage, id: ResourceLocation) -> Self {
        Self { storage, id }
    }

    pub fn suggestions(&mut self) -> Result<Vec<ResourceLocation>, String> {
        self.storage.keys().map_err(|error| error.to_string())
    }
}

impl DataAccessor for StorageDataAccessor {
    fn set_data(&mut self, tag: Tag) -> Result<(), String> {
        let Tag::Compound(_) = tag else {
            return Err("StorageDataAccessor requires a compound tag".to_string());
        };
        self.storage
            .set(self.id.clone(), tag)
            .map_err(|error| error.to_string())
    }

    fn get_data(&mut self) -> Result<Tag, String> {
        self.storage
            .get(&self.id)
            .map_err(|error| error.to_string())
    }

    fn get_modified_success(&self) -> Component {
        Component::translatable(
            "commands.data.storage.modified",
            vec![ComponentArgument::String(self.id.to_string())],
        )
    }

    fn get_print_success(&self, data: &Tag) -> Component {
        Component::translatable(
            "commands.data.storage.query",
            vec![
                ComponentArgument::String(self.id.to_string()),
                ComponentArgument::Component(Box::new(nbt_utils::to_pretty_component(data))),
            ],
        )
    }

    fn get_scaled_print_success(&self, path: &str, scale: f64, value: i32) -> Component {
        Component::translatable(
            "commands.data.storage.get",
            vec![
                ComponentArgument::String(path.to_string()),
                ComponentArgument::String(self.id.to_string()),
                ComponentArgument::String(format!("{scale:.2}")),
                ComponentArgument::Number(value),
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageDataProviderModel {
    pub argument_name: String,
}

impl StorageDataProviderModel {
    pub fn new(argument_name: impl Into<String>) -> Self {
        Self {
            argument_name: argument_name.into(),
        }
    }

    pub fn access(&self, storage: CommandStorage, id: ResourceLocation) -> StorageDataAccessor {
        StorageDataAccessor::new(storage, id)
    }

    pub fn wrapped_argument_shape(&self) -> String {
        format!("storage <{}> [suggest command-storage keys]", self.argument_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDataProviderModel {
    pub argument_prefix: String,
}

impl BlockDataProviderModel {
    pub fn new(argument_prefix: impl Into<String>) -> Self {
        Self {
            argument_prefix: argument_prefix.into(),
        }
    }

    pub fn position_argument(&self) -> String {
        format!("{}Pos", self.argument_prefix)
    }

    pub fn access(
        &self,
        entity: Option<CommandBlockEntityModel>,
        position: BlockPos,
    ) -> Result<BlockDataAccessor, &'static str> {
        entity
            .map(|entity| BlockDataAccessor::new(entity, position))
            .ok_or(ERROR_NOT_A_BLOCK_ENTITY)
    }

    pub fn wrapped_argument_shape(&self) -> String {
        format!("block <{}>", self.position_argument())
    }
}

fn position_component(
    key: &str,
    position: BlockPos,
    mut trailing: Vec<ComponentArgument>,
) -> Component {
    let mut args = vec![
        ComponentArgument::Number(position.x),
        ComponentArgument::Number(position.y),
        ComponentArgument::Number(position.z),
    ];
    args.append(&mut trailing);
    Component::translatable(key, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_accessor_matches_load_update_and_feedback_behavior() {
        let position = BlockPos { x: 1, y: 64, z: -2 };
        let replacement = Tag::Compound(vec![(
            "id".to_string(),
            Tag::String("minecraft:chest".to_string()),
        )]);
        let entity = CommandBlockEntityModel::new(
            "minecraft:chest",
            Tag::Compound(Vec::new()),
            "chest at 1,64,-2",
        );
        let mut accessor = BlockDataAccessor::new(entity, position);
        accessor.set_data(replacement.clone()).unwrap();
        assert_eq!(accessor.get_data(), Ok(replacement.clone()));
        assert!(accessor.entity.changed);
        assert_eq!(
            accessor.entity.updates,
            vec![BlockUpdateEvent {
                position,
                old_state: "minecraft:chest".to_string(),
                new_state: "minecraft:chest".to_string(),
                flags: 3,
            }]
        );
        assert_eq!(
            accessor.get_modified_success(),
            position_component("commands.data.block.modified", position, Vec::new())
        );
        assert_eq!(
            accessor.get_print_success(&replacement),
            position_component(
                "commands.data.block.query",
                position,
                vec![ComponentArgument::Component(Box::new(
                    nbt_utils::to_pretty_component(&replacement)
                ))]
            )
        );
        assert_eq!(
            accessor.get_scaled_print_success("Items[0].Count", 0.5, 3),
            Component::translatable(
                "commands.data.block.get",
                vec![
                    ComponentArgument::String("Items[0].Count".to_string()),
                    ComponentArgument::Number(1),
                    ComponentArgument::Number(64),
                    ComponentArgument::Number(-2),
                    ComponentArgument::String("0.50".to_string()),
                    ComponentArgument::Number(3),
                ]
            )
        );
    }

    #[test]
    fn provider_uses_prefixed_loaded_position_and_rejects_missing_entity() {
        let provider = BlockDataProviderModel::new("target");
        let position = BlockPos { x: 0, y: 0, z: 0 };
        assert_eq!(provider.position_argument(), "targetPos");
        assert_eq!(provider.wrapped_argument_shape(), "block <targetPos>");
        assert_eq!(provider.access(None, position), Err(ERROR_NOT_A_BLOCK_ENTITY));
    }

    #[test]
    fn entity_accessor_rejects_players_and_restores_non_player_uuid() {
        let tag = Tag::Compound(vec![(
            "UUID".to_string(),
            Tag::String("replacement-uuid".to_string()),
        )]);
        let mut player = EntityDataAccessor::new(entity(true));
        assert_eq!(
            player.set_data(tag.clone()),
            Err(ERROR_NO_PLAYERS.to_string())
        );

        let mut entity_accessor = EntityDataAccessor::new(entity(false));
        entity_accessor.set_data(tag.clone()).unwrap();
        assert_eq!(entity_accessor.entity.uuid, "original-uuid");
        assert_eq!(entity_accessor.get_data(), Ok(tag.clone()));
        assert_eq!(
            entity_accessor.get_modified_success(),
            Component::translatable(
                "commands.data.entity.modified",
                vec![ComponentArgument::Component(Box::new(Component::literal("Pig")))]
            )
        );
        assert_eq!(
            entity_accessor.get_scaled_print_success("Health", 2.0, 40),
            Component::translatable(
                "commands.data.entity.get",
                vec![
                    ComponentArgument::String("Health".to_string()),
                    ComponentArgument::Component(Box::new(Component::literal("Pig"))),
                    ComponentArgument::String("2.00".to_string()),
                    ComponentArgument::Number(40),
                ]
            )
        );
        let provider = EntityDataProviderModel::new("target");
        assert_eq!(provider.wrapped_argument_shape(), "entity <target>");
        assert_eq!(provider.access(entity(false)).entity.uuid, "original-uuid");
    }

    #[test]
    fn storage_accessor_uses_real_command_storage_and_suggestions() {
        use crate::storage::saved_data::SavedDataStorage;

        let data_dir = std::env::temp_dir().join(format!(
            "vibecraft-storage-accessor-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&data_dir);
        let storage = CommandStorage::new(SavedDataStorage::new(&data_dir));
        let id = ResourceLocation::parse("example:state").unwrap();
        let provider = StorageDataProviderModel::new("target");
        let mut accessor = provider.access(storage, id.clone());
        let value = Tag::Compound(vec![("value".to_string(), Tag::Int(7))]);

        accessor.set_data(value.clone()).unwrap();
        assert_eq!(accessor.get_data(), Ok(value.clone()));
        assert_eq!(accessor.suggestions(), Ok(vec![id.clone()]));
        assert_eq!(
            accessor.get_modified_success(),
            Component::translatable(
                "commands.data.storage.modified",
                vec![ComponentArgument::String("example:state".to_string())]
            )
        );
        assert_eq!(
            accessor.get_print_success(&value),
            Component::translatable(
                "commands.data.storage.query",
                vec![
                    ComponentArgument::String("example:state".to_string()),
                    ComponentArgument::Component(Box::new(nbt_utils::to_pretty_component(&value))),
                ]
            )
        );
        assert_eq!(
            accessor.get_scaled_print_success("value", 1.25, 8),
            Component::translatable(
                "commands.data.storage.get",
                vec![
                    ComponentArgument::String("value".to_string()),
                    ComponentArgument::String("example:state".to_string()),
                    ComponentArgument::String("1.25".to_string()),
                    ComponentArgument::Number(8),
                ]
            )
        );
        accessor.set_data(Tag::Compound(Vec::new())).unwrap();
        assert_eq!(accessor.get_data(), Ok(Tag::Compound(Vec::new())));
        assert!(accessor.suggestions().unwrap().is_empty());
        assert_eq!(
            provider.wrapped_argument_shape(),
            "storage <target> [suggest command-storage keys]"
        );
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn data_accessor_and_block_accessor_sources_match_java_26_1_2() {
        const INTERFACE: &str = vibecraft_java_source!(
            "/net/minecraft/server/commands/data/DataAccessor.java"
        );
        for sentinel in [
            "void setData(CompoundTag tag) throws CommandSyntaxException;",
            "CompoundTag getData() throws CommandSyntaxException;",
            "Component getModifiedSuccess();",
            "Component getPrintSuccess(Tag data);",
            "Component getPrintSuccess(NbtPathArgument.NbtPath path, double scale, int value);",
        ] {
            assert!(INTERFACE.contains(sentinel), "DataAccessor.java missing: {sentinel}");
        }
        const BLOCK: &str = vibecraft_java_source!(
            "/net/minecraft/server/commands/data/BlockDataAccessor.java"
        );
        for sentinel in [
            "commands.data.block.invalid",
            "BlockPosArgument.getLoadedBlockPos(context, argPrefix + \"Pos\")",
            "return new BlockDataAccessor(entity, pos);",
            "Commands.literal(\"block\")",
            "this.entity.loadWithComponents(TagValueInput.create(reporter, this.entity.getLevel().registryAccess(), tag));",
            "this.entity.setChanged();",
            "this.entity.getLevel().sendBlockUpdated(this.pos, state, state, 3);",
            "return this.entity.saveWithFullMetadata(this.entity.getLevel().registryAccess());",
            "commands.data.block.modified",
            "commands.data.block.query",
            "String.format(Locale.ROOT, \"%.2f\", scale)",
        ] {
            assert!(BLOCK.contains(sentinel), "BlockDataAccessor.java missing: {sentinel}");
        }


        const ENTITY: &str = vibecraft_java_source!(
            "/net/minecraft/server/commands/data/EntityDataAccessor.java"
        );
        for sentinel in [
            "commands.data.entity.invalid",
            "EntityArgument.getEntity(context, arg)",
            "Commands.literal(\"entity\")",
            "if (this.entity instanceof Player)",
            "UUID uuid = this.entity.getUUID();",
            "this.entity.load(TagValueInput.create(reporter, this.entity.registryAccess(), tag));",
            "this.entity.setUUID(uuid);",
            "return NbtPredicate.getEntityTagToCompare(this.entity);",
            "commands.data.entity.modified",
            "commands.data.entity.query",
            "String.format(Locale.ROOT, \"%.2f\", scale)",
        ] {
            assert!(ENTITY.contains(sentinel), "EntityDataAccessor.java missing: {sentinel}");
        }


        const STORAGE: &str = vibecraft_java_source!(
            "/net/minecraft/server/commands/data/StorageDataAccessor.java"
        );
        for sentinel in [
            "SharedSuggestionProvider.suggestResource(getGlobalTags(c).keys(), p)",
            "new StorageDataAccessor(StorageDataAccessor.getGlobalTags(context), IdentifierArgument.getId(context, arg))",
            "Commands.literal(\"storage\")",
            "getServer().getCommandStorage();",
            "this.storage.set(this.id, tag);",
            "return this.storage.get(this.id);",
            "commands.data.storage.modified",
            "commands.data.storage.query",
            "commands.data.storage.get",
            "String.format(Locale.ROOT, \"%.2f\", scale)",
        ] {
            assert!(STORAGE.contains(sentinel), "StorageDataAccessor.java missing: {sentinel}");
        }
    }

    fn entity(is_player: bool) -> CommandEntityModel {
        CommandEntityModel {
            uuid: "original-uuid".to_string(),
            display_name: Component::literal("Pig"),
            is_player,
            comparison_tag: Tag::Compound(Vec::new()),
            problem_path: "Pig".to_string(),
        }
    }
}
