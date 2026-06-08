use crate::command_shared_suggestion_provider::SuggestionsBuilderModel;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const BLOCK_ARGUMENTS_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockDefinitionModel {
    id: String,
    default_properties: BTreeMap<String, String>,
    allowed_values: BTreeMap<String, BTreeSet<String>>,
    has_block_entity: bool,
}

impl BlockDefinitionModel {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            default_properties: BTreeMap::new(),
            allowed_values: BTreeMap::new(),
            has_block_entity: false,
        }
    }

    pub fn with_property(mut self, name: &str, default: &str, values: &[&str]) -> Self {
        self.default_properties
            .insert(name.to_string(), default.to_string());
        self.allowed_values.insert(
            name.to_string(),
            values.iter().map(|value| (*value).to_string()).collect(),
        );
        self
    }

    pub fn with_block_entity(mut self) -> Self {
        self.has_block_entity = true;
        self
    }

    fn default_state(&self) -> BlockStateModel {
        BlockStateModel {
            block_id: self.id.clone(),
            properties: self.default_properties.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockRegistryModel {
    blocks: BTreeMap<String, BlockDefinitionModel>,
    tags: BTreeMap<String, BTreeSet<String>>,
}

impl BlockRegistryModel {
    pub fn with_block(mut self, block: BlockDefinitionModel) -> Self {
        self.blocks.insert(block.id.clone(), block);
        self
    }

    pub fn with_tag(mut self, tag: &str, blocks: &[&str]) -> Self {
        self.tags.insert(
            normalize_id(tag),
            blocks.iter().map(|block| normalize_id(block)).collect(),
        );
        self
    }

    fn block(&self, id: &str) -> Option<&BlockDefinitionModel> {
        self.blocks.get(&normalize_id(id))
    }

    fn tag(&self, id: &str) -> Option<&BTreeSet<String>> {
        self.tags.get(&normalize_id(id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateModel {
    block_id: String,
    properties: BTreeMap<String, String>,
}

impl BlockStateModel {
    pub fn new(block_id: impl Into<String>) -> Self {
        Self {
            block_id: block_id.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn with_property(mut self, name: &str, value: &str) -> Self {
        self.properties.insert(name.to_string(), value.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityModel {
    nbt: BTreeMap<String, String>,
}

impl BlockEntityModel {
    pub fn new(nbt: &[(&str, &str)]) -> Self {
        Self {
            nbt: nbt
                .iter()
                .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
                .collect(),
        }
    }

    fn load_with_components(&mut self, nbt: &BTreeMap<String, String>) -> bool {
        let before = self.nbt.clone();
        self.nbt.extend(nbt.clone());
        self.nbt != before
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInWorldModel {
    state: BlockStateModel,
    entity: Option<BlockEntityModel>,
}

impl BlockInWorldModel {
    pub fn new(state: BlockStateModel, entity: Option<BlockEntityModel>) -> Self {
        Self { state, entity }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerLevelBlockModel {
    blocks: HashMap<(i32, i32, i32), BlockInWorldModel>,
    changed_blocks: BTreeSet<(i32, i32, i32)>,
}

impl ServerLevelBlockModel {
    pub fn with_block(mut self, pos: (i32, i32, i32), block: BlockInWorldModel) -> Self {
        self.blocks.insert(pos, block);
        self
    }

    fn set_block(&mut self, pos: (i32, i32, i32), state: BlockStateModel) -> bool {
        let entry = self
            .blocks
            .entry(pos)
            .or_insert_with(|| BlockInWorldModel::new(BlockStateModel::new("minecraft:air"), None));
        if entry.state == state {
            false
        } else {
            entry.state = state;
            true
        }
    }

    fn block_entity_mut(&mut self, pos: (i32, i32, i32)) -> Option<&mut BlockEntityModel> {
        self.blocks
            .get_mut(&pos)
            .and_then(|block| block.entity.as_mut())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInputModel {
    state: BlockStateModel,
    defined_properties: BTreeSet<String>,
    nbt: Option<BTreeMap<String, String>>,
}

impl BlockInputModel {
    pub fn new(
        state: BlockStateModel,
        defined_properties: BTreeSet<String>,
        nbt: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            state,
            defined_properties,
            nbt,
        }
    }

    pub fn get_state(&self) -> &BlockStateModel {
        &self.state
    }

    pub fn get_defined_properties(&self) -> &BTreeSet<String> {
        &self.defined_properties
    }

    pub fn test(&self, block: &BlockInWorldModel) -> bool {
        if block.state.block_id != self.state.block_id {
            return false;
        }
        for property in &self.defined_properties {
            if block.state.properties.get(property) != self.state.properties.get(property) {
                return false;
            }
        }
        match (&self.nbt, &block.entity) {
            (None, _) => true,
            (Some(expected), Some(entity)) => compare_nbt(expected, &entity.nbt),
            (Some(_), None) => false,
        }
    }

    pub fn place(
        &self,
        level: &mut ServerLevelBlockModel,
        pos: (i32, i32, i32),
        update_flags: u32,
    ) -> bool {
        let mut state = if update_flags & 16 != 0 {
            self.state.clone()
        } else {
            block_update_from_neighbor_shapes(&self.state)
        };
        if state.block_id == "minecraft:air" {
            state = self.state.clone();
        }
        state = self.overwrite_with_defined_properties(state);
        let mut affected = level.set_block(pos, state);
        if let (Some(nbt), Some(entity)) = (&self.nbt, level.block_entity_mut(pos)) {
            if entity.load_with_components(nbt) {
                affected = true;
                level.changed_blocks.insert(pos);
            }
        }
        affected
    }

    fn overwrite_with_defined_properties(&self, mut state: BlockStateModel) -> BlockStateModel {
        for property in &self.defined_properties {
            if let Some(value) = self.state.properties.get(property) {
                state.properties.insert(property.clone(), value.clone());
            }
        }
        state
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockPredicateResultModel {
    Block(BlockInputModel),
    Tag {
        tag_id: String,
        vague_properties: BTreeMap<String, String>,
        nbt: Option<BTreeMap<String, String>>,
    },
}

impl BlockPredicateResultModel {
    pub fn test(&self, registry: &BlockRegistryModel, block: &BlockInWorldModel) -> bool {
        match self {
            Self::Block(input) => input.test(block),
            Self::Tag {
                tag_id,
                vague_properties,
                nbt,
            } => {
                let Some(blocks) = registry.tag(tag_id) else {
                    return false;
                };
                if !blocks.contains(&block.state.block_id) {
                    return false;
                }
                let Some(definition) = registry.block(&block.state.block_id) else {
                    return false;
                };
                for (key, value) in vague_properties {
                    let Some(allowed) = definition.allowed_values.get(key) else {
                        return false;
                    };
                    if !allowed.contains(value) || block.state.properties.get(key) != Some(value) {
                        return false;
                    }
                }
                match (nbt, &block.entity) {
                    (None, _) => true,
                    (Some(expected), Some(entity)) => compare_nbt(expected, &entity.nbt),
                    (Some(_), None) => false,
                }
            }
        }
    }

    pub fn requires_nbt(&self) -> bool {
        match self {
            Self::Block(input) => input.nbt.is_some(),
            Self::Tag { nbt, .. } => nbt.is_some(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockStateArgumentModel;

impl BlockStateArgumentModel {
    pub fn block(_context: &CommandBuildContextModel) -> Self {
        Self
    }

    pub fn parse(
        &self,
        registry: &BlockRegistryModel,
        reader: &mut StringReaderModel,
    ) -> Result<BlockInputModel, BlockArgumentError> {
        let result = parse_for_block(registry, reader, true)?;
        Ok(BlockInputModel::new(
            result.block_state,
            result.properties.keys().cloned().collect(),
            result.nbt,
        ))
    }

    pub fn list_suggestions(
        &self,
        registry: &BlockRegistryModel,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<String> {
        fill_suggestions(registry, builder.remaining(), false, true)
    }

    pub fn examples(&self) -> [&'static str; 4] {
        ["stone", "minecraft:stone", "stone[foo=bar]", "foo{bar=baz}"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockPredicateArgumentModel;

impl BlockPredicateArgumentModel {
    pub fn block_predicate(_context: &CommandBuildContextModel) -> Self {
        Self
    }

    pub fn parse(
        &self,
        registry: &BlockRegistryModel,
        reader: &mut StringReaderModel,
    ) -> Result<BlockPredicateResultModel, BlockArgumentError> {
        parse_for_testing(registry, reader, true)
    }

    pub fn list_suggestions(
        &self,
        registry: &BlockRegistryModel,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<String> {
        fill_suggestions(registry, builder.remaining(), true, true)
    }

    pub fn examples(&self) -> [&'static str; 5] {
        [
            "stone",
            "minecraft:stone",
            "stone[foo=bar]",
            "#stone",
            "#stone[foo=bar]{baz=nbt}",
        ]
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    blocks: HashMap<String, BlockInputModel>,
    predicates: HashMap<String, BlockPredicateResultModel>,
}

impl CommandContextModel {
    pub fn with_block(mut self, name: &str, value: BlockInputModel) -> Self {
        self.blocks.insert(name.to_string(), value);
        self
    }

    pub fn with_predicate(mut self, name: &str, value: BlockPredicateResultModel) -> Self {
        self.predicates.insert(name.to_string(), value);
        self
    }
}

pub fn get_block(context: &CommandContextModel, name: &str) -> Option<BlockInputModel> {
    context.blocks.get(name).cloned()
}

pub fn get_block_predicate(
    context: &CommandContextModel,
    name: &str,
) -> Option<BlockPredicateResultModel> {
    context.predicates.get(name).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockResultModel {
    block_state: BlockStateModel,
    properties: BTreeMap<String, String>,
    nbt: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockArgumentError {
    NoTagsAllowed {
        cursor: usize,
    },
    UnknownBlock {
        cursor: usize,
        id: String,
    },
    UnknownProperty {
        cursor: usize,
        block: String,
        property: String,
    },
    DuplicateProperty {
        cursor: usize,
        block: String,
        property: String,
    },
    InvalidValue {
        cursor: usize,
        block: String,
        property: String,
        value: String,
    },
    ExpectedValue {
        cursor: usize,
        block: String,
        property: String,
    },
    ExpectedEndOfProperties {
        cursor: usize,
    },
    UnknownTag {
        cursor: usize,
        tag: String,
    },
    InvalidIdentifier {
        cursor: usize,
    },
    InvalidNbt {
        cursor: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringReaderModel {
    input: String,
    cursor: usize,
}

impl StringReaderModel {
    pub fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
            cursor: 0,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    fn set_cursor(&mut self, cursor: usize) {
        self.cursor = cursor;
    }

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.can_read() && self.peek().is_ascii_whitespace() {
            self.skip();
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), BlockArgumentError> {
        if self.can_read() && self.peek() == expected {
            self.skip();
            Ok(())
        } else {
            Err(BlockArgumentError::InvalidIdentifier {
                cursor: self.cursor,
            })
        }
    }

    fn read_string(&mut self) -> String {
        let start = self.cursor;
        while self.can_read() && is_string_char(self.peek()) {
            self.skip();
        }
        self.input[start..self.cursor].to_string()
    }

    fn read_identifier(&mut self) -> Result<String, BlockArgumentError> {
        let start = self.cursor;
        while self.can_read() && is_identifier_char(self.peek()) {
            self.skip();
        }
        if start == self.cursor {
            Err(BlockArgumentError::InvalidIdentifier { cursor: start })
        } else {
            Ok(normalize_id(&self.input[start..self.cursor]))
        }
    }
}

fn parse_for_block(
    registry: &BlockRegistryModel,
    reader: &mut StringReaderModel,
    allow_nbt: bool,
) -> Result<BlockResultModel, BlockArgumentError> {
    let cursor = reader.cursor();
    match BlockStateParserModel::new(registry, reader, false, allow_nbt).parse() {
        Ok(ParseResultModel::Block(result)) => Ok(result),
        Ok(ParseResultModel::Tag { .. }) => unreachable!("forTesting=false rejects tags"),
        Err(error) => {
            reader.set_cursor(cursor);
            Err(error)
        }
    }
}

fn parse_for_testing(
    registry: &BlockRegistryModel,
    reader: &mut StringReaderModel,
    allow_nbt: bool,
) -> Result<BlockPredicateResultModel, BlockArgumentError> {
    let cursor = reader.cursor();
    match BlockStateParserModel::new(registry, reader, true, allow_nbt).parse() {
        Ok(ParseResultModel::Block(result)) => {
            Ok(BlockPredicateResultModel::Block(BlockInputModel::new(
                result.block_state,
                result.properties.keys().cloned().collect(),
                result.nbt,
            )))
        }
        Ok(ParseResultModel::Tag {
            tag_id,
            vague_properties,
            nbt,
        }) => Ok(BlockPredicateResultModel::Tag {
            tag_id,
            vague_properties,
            nbt,
        }),
        Err(error) => {
            reader.set_cursor(cursor);
            Err(error)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseResultModel {
    Block(BlockResultModel),
    Tag {
        tag_id: String,
        vague_properties: BTreeMap<String, String>,
        nbt: Option<BTreeMap<String, String>>,
    },
}

struct BlockStateParserModel<'a> {
    registry: &'a BlockRegistryModel,
    reader: &'a mut StringReaderModel,
    for_testing: bool,
    allow_nbt: bool,
    id: String,
    definition: Option<BlockDefinitionModel>,
    state: Option<BlockStateModel>,
    nbt: Option<BTreeMap<String, String>>,
    tag_id: Option<String>,
    properties: BTreeMap<String, String>,
    vague_properties: BTreeMap<String, String>,
}

impl<'a> BlockStateParserModel<'a> {
    fn new(
        registry: &'a BlockRegistryModel,
        reader: &'a mut StringReaderModel,
        for_testing: bool,
        allow_nbt: bool,
    ) -> Self {
        Self {
            registry,
            reader,
            for_testing,
            allow_nbt,
            id: "minecraft:".to_string(),
            definition: None,
            state: None,
            nbt: None,
            tag_id: None,
            properties: BTreeMap::new(),
            vague_properties: BTreeMap::new(),
        }
    }

    fn parse(mut self) -> Result<ParseResultModel, BlockArgumentError> {
        if self.reader.can_read() && self.reader.peek() == '#' {
            self.read_tag()?;
            if self.reader.can_read() && self.reader.peek() == '[' {
                self.read_vague_properties()?;
            }
        } else {
            self.read_block()?;
            if self.reader.can_read() && self.reader.peek() == '[' {
                self.read_properties()?;
            }
        }
        if self.allow_nbt && self.reader.can_read() && self.reader.peek() == '{' {
            self.nbt = Some(read_nbt(self.reader)?);
        }
        if let Some(tag_id) = self.tag_id {
            Ok(ParseResultModel::Tag {
                tag_id,
                vague_properties: self.vague_properties,
                nbt: self.nbt,
            })
        } else {
            Ok(ParseResultModel::Block(BlockResultModel {
                block_state: self.state.expect("block parser set state"),
                properties: self.properties,
                nbt: self.nbt,
            }))
        }
    }

    fn read_block(&mut self) -> Result<(), BlockArgumentError> {
        let start = self.reader.cursor();
        self.id = self.reader.read_identifier()?;
        let Some(block) = self.registry.block(&self.id) else {
            self.reader.set_cursor(start);
            return Err(BlockArgumentError::UnknownBlock {
                cursor: self.reader.cursor(),
                id: self.id.clone(),
            });
        };
        self.definition = Some(block.clone());
        self.state = Some(block.default_state());
        Ok(())
    }

    fn read_tag(&mut self) -> Result<(), BlockArgumentError> {
        if !self.for_testing {
            return Err(BlockArgumentError::NoTagsAllowed {
                cursor: self.reader.cursor(),
            });
        }
        let start = self.reader.cursor();
        self.reader.expect('#')?;
        let id = self.reader.read_identifier()?;
        if self.registry.tag(&id).is_none() {
            self.reader.set_cursor(start);
            return Err(BlockArgumentError::UnknownTag {
                cursor: self.reader.cursor(),
                tag: id,
            });
        }
        self.tag_id = Some(id);
        Ok(())
    }

    fn read_properties(&mut self) -> Result<(), BlockArgumentError> {
        self.reader.skip();
        self.reader.skip_whitespace();
        while self.reader.can_read() && self.reader.peek() != ']' {
            self.reader.skip_whitespace();
            let key_start = self.reader.cursor();
            let key = self.reader.read_string();
            let definition = self.definition.as_ref().expect("definition set");
            let Some(allowed) = definition.allowed_values.get(&key) else {
                self.reader.set_cursor(key_start);
                return Err(BlockArgumentError::UnknownProperty {
                    cursor: self.reader.cursor(),
                    block: self.id.clone(),
                    property: key,
                });
            };
            if self.properties.contains_key(&key) {
                self.reader.set_cursor(key_start);
                return Err(BlockArgumentError::DuplicateProperty {
                    cursor: self.reader.cursor(),
                    block: self.id.clone(),
                    property: key,
                });
            }
            self.reader.skip_whitespace();
            if !self.reader.can_read() || self.reader.peek() != '=' {
                return Err(BlockArgumentError::ExpectedValue {
                    cursor: self.reader.cursor(),
                    block: self.id.clone(),
                    property: key,
                });
            }
            self.reader.skip();
            self.reader.skip_whitespace();
            let value_start = self.reader.cursor();
            let value = self.reader.read_string();
            if !allowed.contains(&value) {
                self.reader.set_cursor(value_start);
                return Err(BlockArgumentError::InvalidValue {
                    cursor: self.reader.cursor(),
                    block: self.id.clone(),
                    property: key,
                    value,
                });
            }
            self.state
                .as_mut()
                .expect("state set")
                .properties
                .insert(key.clone(), value.clone());
            self.properties.insert(key, value);
            self.reader.skip_whitespace();
            if self.reader.can_read() {
                if self.reader.peek() == ',' {
                    self.reader.skip();
                } else if self.reader.peek() != ']' {
                    return Err(BlockArgumentError::ExpectedEndOfProperties {
                        cursor: self.reader.cursor(),
                    });
                }
            }
        }
        if self.reader.can_read() {
            self.reader.skip();
            Ok(())
        } else {
            Err(BlockArgumentError::ExpectedEndOfProperties {
                cursor: self.reader.cursor(),
            })
        }
    }

    fn read_vague_properties(&mut self) -> Result<(), BlockArgumentError> {
        self.reader.skip();
        self.reader.skip_whitespace();
        let mut value_start = None;
        while self.reader.can_read() && self.reader.peek() != ']' {
            self.reader.skip_whitespace();
            let key_start = self.reader.cursor();
            let key = self.reader.read_string();
            if self.vague_properties.contains_key(&key) {
                self.reader.set_cursor(key_start);
                return Err(BlockArgumentError::DuplicateProperty {
                    cursor: self.reader.cursor(),
                    block: self.id.clone(),
                    property: key,
                });
            }
            self.reader.skip_whitespace();
            if !self.reader.can_read() || self.reader.peek() != '=' {
                self.reader.set_cursor(key_start);
                return Err(BlockArgumentError::ExpectedValue {
                    cursor: self.reader.cursor(),
                    block: self.id.clone(),
                    property: key,
                });
            }
            self.reader.skip();
            self.reader.skip_whitespace();
            value_start = Some(self.reader.cursor());
            let value = self.reader.read_string();
            self.vague_properties.insert(key, value);
            self.reader.skip_whitespace();
            if self.reader.can_read() {
                value_start = None;
                if self.reader.peek() == ',' {
                    self.reader.skip();
                } else if self.reader.peek() != ']' {
                    return Err(BlockArgumentError::ExpectedEndOfProperties {
                        cursor: self.reader.cursor(),
                    });
                }
            }
        }
        if self.reader.can_read() {
            self.reader.skip();
            Ok(())
        } else {
            if let Some(cursor) = value_start {
                self.reader.set_cursor(cursor);
            }
            Err(BlockArgumentError::ExpectedEndOfProperties {
                cursor: self.reader.cursor(),
            })
        }
    }
}

fn fill_suggestions(
    registry: &BlockRegistryModel,
    remaining: &str,
    for_testing: bool,
    allow_nbt: bool,
) -> Vec<String> {
    let mut reader = StringReaderModel::new(remaining);
    let _ = BlockStateParserModel::new(registry, &mut reader, for_testing, allow_nbt).parse();
    let cursor = reader.cursor();
    let tail = &remaining[cursor..];
    if tail.is_empty() {
        if remaining.is_empty() {
            let mut values = registry.blocks.keys().cloned().collect::<Vec<_>>();
            if for_testing {
                values.extend(registry.tags.keys().map(|tag| format!("#{tag}")));
            }
            return values;
        }
        if let Ok(result) =
            parse_for_testing(registry, &mut StringReaderModel::new(remaining), true)
        {
            return match result {
                BlockPredicateResultModel::Block(input) => {
                    let Some(definition) = registry.block(&input.state.block_id) else {
                        return Vec::new();
                    };
                    let mut values = Vec::new();
                    if input.defined_properties.len() < definition.allowed_values.len() {
                        values.push("[".to_string());
                    }
                    if definition.has_block_entity {
                        values.push("{".to_string());
                    }
                    values
                }
                BlockPredicateResultModel::Tag { tag_id, .. } => {
                    tag_open_suggestions(registry, &tag_id)
                }
            };
        }
    }
    Vec::new()
}

fn tag_open_suggestions(registry: &BlockRegistryModel, tag_id: &str) -> Vec<String> {
    let Some(blocks) = registry.tag(tag_id) else {
        return Vec::new();
    };
    let mut has_properties = false;
    let mut has_entity = false;
    for block_id in blocks {
        if let Some(block) = registry.block(block_id) {
            has_properties |= !block.allowed_values.is_empty();
            has_entity |= block.has_block_entity;
        }
    }
    let mut values = Vec::new();
    if has_properties {
        values.push("[".to_string());
    }
    if has_entity {
        values.push("{".to_string());
    }
    values
}

fn read_nbt(
    reader: &mut StringReaderModel,
) -> Result<BTreeMap<String, String>, BlockArgumentError> {
    if !reader.can_read() || reader.peek() != '{' {
        return Err(BlockArgumentError::InvalidNbt {
            cursor: reader.cursor(),
        });
    }
    reader.skip();
    let mut result = BTreeMap::new();
    while reader.can_read() && reader.peek() != '}' {
        reader.skip_whitespace();
        let key_start = reader.cursor();
        let key = reader.read_string();
        if key.is_empty() {
            return Err(BlockArgumentError::InvalidNbt { cursor: key_start });
        }
        reader.skip_whitespace();
        if !reader.can_read() || reader.peek() != '=' {
            return Err(BlockArgumentError::InvalidNbt {
                cursor: reader.cursor(),
            });
        }
        reader.skip();
        reader.skip_whitespace();
        let value_start = reader.cursor();
        let value = reader.read_string();
        if value.is_empty() {
            return Err(BlockArgumentError::InvalidNbt {
                cursor: value_start,
            });
        }
        result.insert(key, value);
        reader.skip_whitespace();
        if reader.can_read() && reader.peek() == ',' {
            reader.skip();
        }
    }
    if reader.can_read() && reader.peek() == '}' {
        reader.skip();
        Ok(result)
    } else {
        Err(BlockArgumentError::InvalidNbt {
            cursor: reader.cursor(),
        })
    }
}

fn serialize(state: &BlockStateModel) -> String {
    let mut result = state.block_id.clone();
    if !state.properties.is_empty() {
        result.push('[');
        for (index, (key, value)) in state.properties.iter().enumerate() {
            if index > 0 {
                result.push(',');
            }
            result.push_str(key);
            result.push('=');
            result.push_str(value);
        }
        result.push(']');
    }
    result
}

fn normalize_id(value: &str) -> String {
    if value.contains(':') {
        value.to_string()
    } else {
        format!("minecraft:{value}")
    }
}

fn is_identifier_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.' | '/' | ':')
}

fn is_string_char(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-' | '.')
}

fn compare_nbt(expected: &BTreeMap<String, String>, actual: &BTreeMap<String, String>) -> bool {
    expected
        .iter()
        .all(|(key, value)| actual.get(key) == Some(value))
}

fn block_update_from_neighbor_shapes(state: &BlockStateModel) -> BlockStateModel {
    if state.block_id == "minecraft:fragile_air" {
        BlockStateModel::new("minecraft:air")
    } else {
        state.clone()
    }
}

#[cfg(test)]
#[path = "command_block_arguments_tests.rs"]
mod tests;
