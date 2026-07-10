#![allow(dead_code)]

use crate::chat_component::{Component, ComponentArgument};
use crate::command::BlockPos;
use crate::storage::nbt::{nbt_utils, Tag};

pub const ERROR_NOT_A_BLOCK_ENTITY: &str = "commands.data.block.invalid";

pub trait DataAccessor {
    fn set_data(&mut self, tag: Tag) -> Result<(), String>;
    fn get_data(&self) -> Result<Tag, String>;
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

    fn get_data(&self) -> Result<Tag, String> {
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
    }
}
