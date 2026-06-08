use crate::command_shared_suggestion_provider::SuggestionsBuilderModel;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const ITEM_ARGUMENTS_PACKAGE_NULL_MARKED: bool = true;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ItemRegistryModel {
    items: BTreeMap<String, ItemDefinitionModel>,
    tags: BTreeMap<String, BTreeSet<String>>,
    components: BTreeMap<String, ComponentDefinitionModel>,
    predicates: BTreeSet<String>,
}

impl ItemRegistryModel {
    pub fn with_item(mut self, id: &str, max_stack_size: u32) -> Self {
        self.items.insert(
            normalize_id(id),
            ItemDefinitionModel {
                id: normalize_id(id),
                max_stack_size,
            },
        );
        self
    }

    pub fn with_tag(mut self, id: &str, items: &[&str]) -> Self {
        self.tags.insert(
            normalize_id(id),
            items.iter().map(|item| normalize_id(item)).collect(),
        );
        self
    }

    pub fn with_component(mut self, id: &str) -> Self {
        self.components.insert(
            normalize_id(id),
            ComponentDefinitionModel {
                id: normalize_id(id),
                transient: false,
                has_value_codec: true,
            },
        );
        self
    }

    pub fn with_transient_component(mut self, id: &str) -> Self {
        self.components.insert(
            normalize_id(id),
            ComponentDefinitionModel {
                id: normalize_id(id),
                transient: true,
                has_value_codec: true,
            },
        );
        self
    }

    pub fn with_marker_component(mut self, id: &str) -> Self {
        self.components.insert(
            normalize_id(id),
            ComponentDefinitionModel {
                id: normalize_id(id),
                transient: false,
                has_value_codec: false,
            },
        );
        self
    }

    pub fn with_predicate(mut self, id: &str) -> Self {
        self.predicates.insert(normalize_id(id));
        self
    }

    fn item(&self, id: &str) -> Option<&ItemDefinitionModel> {
        self.items.get(&normalize_id(id))
    }

    fn tag(&self, id: &str) -> Option<&BTreeSet<String>> {
        self.tags.get(&normalize_id(id))
    }

    fn component_for_item_parser(&self, id: &str) -> Option<&ComponentDefinitionModel> {
        self.components
            .get(&normalize_id(id))
            .filter(|component| !component.transient)
    }

    fn component_with_value_codec(&self, id: &str) -> Option<&ComponentDefinitionModel> {
        self.components
            .get(&normalize_id(id))
            .filter(|component| !component.transient && component.has_value_codec)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemDefinitionModel {
    id: String,
    max_stack_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComponentDefinitionModel {
    id: String,
    transient: bool,
    has_value_codec: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataComponentPatchModel {
    set: BTreeMap<String, String>,
    removed: BTreeSet<String>,
}

impl DataComponentPatchModel {
    pub fn set_components(&self) -> &BTreeMap<String, String> {
        &self.set
    }

    pub fn removed_components(&self) -> &BTreeSet<String> {
        &self.removed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemInputModel {
    item_id: String,
    components: DataComponentPatchModel,
}

impl ItemInputModel {
    fn new(item_id: String, components: DataComponentPatchModel) -> Self {
        Self {
            item_id,
            components,
        }
    }

    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub fn components(&self) -> &DataComponentPatchModel {
        &self.components
    }

    pub fn create_item_stack(
        &self,
        registry: &ItemRegistryModel,
        count: u32,
    ) -> Result<ItemStackModel, ItemArgumentError> {
        let item = registry
            .item(&self.item_id)
            .expect("parsed item still exists");
        if count > item.max_stack_size {
            return Err(ItemArgumentError::StackTooBig {
                item: self.item_id.clone(),
                max: item.max_stack_size,
            });
        }
        if self
            .components
            .set
            .values()
            .any(|value| value == "malformed")
        {
            return Err(ItemArgumentError::MalformedItem {
                message: "strict item validation failed".to_string(),
            });
        }
        Ok(ItemStackModel {
            item_id: self.item_id.clone(),
            count,
            components: self.components.set.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackModel {
    item_id: String,
    count: u32,
    components: BTreeMap<String, String>,
}

impl ItemStackModel {
    pub fn new(item_id: &str, count: u32) -> Self {
        Self {
            item_id: normalize_id(item_id),
            count,
            components: BTreeMap::new(),
        }
    }

    pub fn with_component(mut self, id: &str, value: &str) -> Self {
        self.components.insert(normalize_id(id), value.to_string());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemArgumentError {
    UnknownItem {
        cursor: usize,
        id: String,
    },
    UnknownTag {
        cursor: usize,
        id: String,
    },
    UnknownComponent {
        cursor: usize,
        id: String,
    },
    UnknownPredicate {
        cursor: usize,
        id: String,
    },
    RepeatedComponent {
        id: String,
    },
    ExpectedComponent {
        cursor: usize,
    },
    ExpectedChar {
        cursor: usize,
        value: char,
    },
    MalformedComponent {
        cursor: usize,
        id: String,
        message: String,
    },
    MalformedPredicate {
        cursor: usize,
        id: String,
        message: String,
    },
    StackTooBig {
        item: String,
        max: u32,
    },
    MalformedItem {
        message: String,
    },
    UnknownFunction {
        id: String,
    },
    UnknownFunctionTag {
        id: String,
    },
    InvalidIdentifier {
        cursor: usize,
    },
    InvalidValue {
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

    fn expect(&mut self, value: char) -> Result<(), ItemArgumentError> {
        if self.can_read() && self.peek() == value {
            self.skip();
            Ok(())
        } else {
            Err(ItemArgumentError::ExpectedChar {
                cursor: self.cursor,
                value,
            })
        }
    }

    fn read_identifier(&mut self) -> Result<String, ItemArgumentError> {
        let start = self.cursor;
        while self.can_read() && is_identifier_char(self.peek()) {
            self.skip();
        }
        if start == self.cursor {
            Err(ItemArgumentError::InvalidIdentifier { cursor: start })
        } else {
            Ok(normalize_id(&self.input[start..self.cursor]))
        }
    }

    fn read_value(&mut self) -> Result<String, ItemArgumentError> {
        let start = self.cursor;
        if self.can_read() && self.peek() == '{' {
            let mut depth = 0usize;
            while self.can_read() {
                match self.peek() {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        self.skip();
                        if depth == 0 {
                            return Ok(self.input[start..self.cursor].to_string());
                        }
                        continue;
                    }
                    _ => {}
                }
                self.skip();
            }
            return Err(ItemArgumentError::InvalidValue { cursor: start });
        }
        while self.can_read() && !matches!(self.peek(), ',' | ']' | '|') {
            self.skip();
        }
        let value = self.input[start..self.cursor].trim().to_string();
        if value.is_empty() {
            Err(ItemArgumentError::InvalidValue { cursor: start })
        } else {
            Ok(value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemArgumentModel;

impl ItemArgumentModel {
    pub fn item(_context: &CommandBuildContextModel) -> Self {
        Self
    }

    pub fn parse(
        &self,
        registry: &ItemRegistryModel,
        reader: &mut StringReaderModel,
    ) -> Result<ItemInputModel, ItemArgumentError> {
        ItemParserModel::new(registry).parse(reader)
    }

    pub fn get_item(context: &CommandContextModel, name: &str) -> Option<ItemInputModel> {
        context.items.get(name).cloned()
    }

    pub fn list_suggestions(
        &self,
        registry: &ItemRegistryModel,
        builder: &mut SuggestionsBuilderModel,
    ) -> Vec<String> {
        ItemParserModel::new(registry).fill_suggestions(builder.remaining())
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["stick", "minecraft:stick", "stick{foo=bar}"]
    }
}

struct ItemParserModel<'a> {
    registry: &'a ItemRegistryModel,
}

impl<'a> ItemParserModel<'a> {
    fn new(registry: &'a ItemRegistryModel) -> Self {
        Self { registry }
    }

    fn parse(&self, reader: &mut StringReaderModel) -> Result<ItemInputModel, ItemArgumentError> {
        let start = reader.cursor();
        match self.parse_inner(reader) {
            Ok(input) => Ok(input),
            Err(error) => {
                reader.set_cursor(start);
                Err(error)
            }
        }
    }

    fn parse_inner(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<ItemInputModel, ItemArgumentError> {
        let item_start = reader.cursor();
        let item_id = reader.read_identifier()?;
        if self.registry.item(&item_id).is_none() {
            reader.set_cursor(item_start);
            return Err(ItemArgumentError::UnknownItem {
                cursor: item_start,
                id: item_id,
            });
        }
        let components = if reader.can_read() && reader.peek() == '[' {
            self.read_components(reader)?
        } else {
            DataComponentPatchModel::default()
        };
        Ok(ItemInputModel::new(item_id, components))
    }

    fn read_components(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<DataComponentPatchModel, ItemArgumentError> {
        reader.expect('[')?;
        let mut patch = DataComponentPatchModel::default();
        let mut seen = BTreeSet::new();
        while reader.can_read() && reader.peek() != ']' {
            reader.skip_whitespace();
            if !reader.can_read() {
                break;
            }
            let removed = reader.peek() == '!';
            if removed {
                reader.skip();
            }
            let component = self.read_component_type(reader, true)?;
            if !seen.insert(component.clone()) {
                return Err(ItemArgumentError::RepeatedComponent { id: component });
            }
            if removed {
                patch.removed.insert(component);
            } else {
                reader.skip_whitespace();
                reader.expect('=')?;
                reader.skip_whitespace();
                let value_start = reader.cursor();
                let value = reader.read_value()?;
                if value == "bad" {
                    reader.set_cursor(value_start);
                    return Err(ItemArgumentError::MalformedComponent {
                        cursor: value_start,
                        id: component,
                        message: "component codec rejected value".to_string(),
                    });
                }
                patch.set.insert(component, value);
            }
            reader.skip_whitespace();
            if reader.can_read() && reader.peek() == ',' {
                reader.skip();
                reader.skip_whitespace();
                if !reader.can_read() {
                    return Err(ItemArgumentError::ExpectedComponent {
                        cursor: reader.cursor(),
                    });
                }
            } else {
                break;
            }
        }
        reader.expect(']')?;
        Ok(patch)
    }

    fn read_component_type(
        &self,
        reader: &mut StringReaderModel,
        allow_marker: bool,
    ) -> Result<String, ItemArgumentError> {
        if !reader.can_read() {
            return Err(ItemArgumentError::ExpectedComponent {
                cursor: reader.cursor(),
            });
        }
        let start = reader.cursor();
        let id = reader.read_identifier()?;
        let known = if allow_marker {
            self.registry.component_for_item_parser(&id)
        } else {
            self.registry.component_with_value_codec(&id)
        };
        if let Some(component) = known {
            Ok(component.id.clone())
        } else {
            reader.set_cursor(start);
            Err(ItemArgumentError::UnknownComponent { cursor: start, id })
        }
    }

    fn fill_suggestions(&self, remaining: &str) -> Vec<String> {
        if remaining.is_empty() {
            return self.registry.items.keys().cloned().collect();
        }
        let mut reader = StringReaderModel::new(remaining);
        let _ = self.parse_inner(&mut reader);
        if reader.cursor() == remaining.len() && self.registry.item(remaining).is_some() {
            return vec!["[".to_string()];
        }
        if remaining.ends_with('[') || remaining.ends_with(',') {
            let mut values = vec!["!".to_string()];
            values.extend(
                self.registry
                    .components
                    .values()
                    .filter(|component| !component.transient && component.has_value_codec)
                    .map(|component| format!("{}=", component.id)),
            );
            return values;
        }
        if remaining.ends_with('=') {
            return Vec::new();
        }
        if self.parse(&mut StringReaderModel::new(remaining)).is_ok() && remaining.contains('[') {
            return vec![",".to_string(), "]".to_string()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemPredicateArgumentModel;

impl ItemPredicateArgumentModel {
    pub fn item_predicate(_context: &CommandBuildContextModel) -> Self {
        Self
    }

    pub fn parse(
        &self,
        registry: &ItemRegistryModel,
        reader: &mut StringReaderModel,
    ) -> Result<ItemPredicateModel, ItemArgumentError> {
        let start = reader.cursor();
        match PredicateParserModel::new(registry, reader).parse_top() {
            Ok(predicate) => Ok(predicate),
            Err(error) => {
                reader.set_cursor(start);
                Err(error)
            }
        }
    }

    pub fn get_item_predicate(
        context: &CommandContextModel,
        name: &str,
    ) -> Option<ItemPredicateModel> {
        context.predicates.get(name).cloned()
    }

    pub fn examples(&self) -> [&'static str; 4] {
        ["stick", "minecraft:stick", "#stick", "#stick{foo:'bar'}"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemPredicateModel {
    All(Vec<ItemPredicateModel>),
    Any(Vec<ItemPredicateModel>),
    Not(Box<ItemPredicateModel>),
    Item(String),
    Tag(String),
    AnyItem,
    ComponentPresent(String),
    ComponentEquals(String, String),
    PredicateValue(String, String),
    CountRange(CountRangeModel),
}

impl ItemPredicateModel {
    pub fn test(&self, registry: &ItemRegistryModel, stack: &ItemStackModel) -> bool {
        match self {
            Self::All(values) => values.iter().all(|value| value.test(registry, stack)),
            Self::Any(values) => values.iter().any(|value| value.test(registry, stack)),
            Self::Not(value) => !value.test(registry, stack),
            Self::Item(id) => stack.item_id == *id,
            Self::Tag(id) => registry
                .tag(id)
                .is_some_and(|items| items.contains(&stack.item_id)),
            Self::AnyItem => true,
            Self::ComponentPresent(id) => stack.components.contains_key(id),
            Self::ComponentEquals(id, value) => stack.components.get(id) == Some(value),
            Self::PredicateValue(id, value) => {
                id == "minecraft:custom_data" && stack.components.get(id) == Some(value)
            }
            Self::CountRange(range) => range.matches(stack.count),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CountRangeModel {
    min: Option<u32>,
    max: Option<u32>,
}

impl CountRangeModel {
    fn exact(value: u32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    fn matches(&self, count: u32) -> bool {
        self.min.is_none_or(|min| count >= min) && self.max.is_none_or(|max| count <= max)
    }
}

struct PredicateParserModel<'a, 'b> {
    registry: &'a ItemRegistryModel,
    reader: &'b mut StringReaderModel,
}

impl<'a, 'b> PredicateParserModel<'a, 'b> {
    fn new(registry: &'a ItemRegistryModel, reader: &'b mut StringReaderModel) -> Self {
        Self { registry, reader }
    }

    fn parse_top(&mut self) -> Result<ItemPredicateModel, ItemArgumentError> {
        let mut tests = Vec::new();
        if self.reader.can_read() && self.reader.peek() == '*' {
            self.reader.skip();
            tests.push(ItemPredicateModel::AnyItem);
        } else if self.reader.can_read() && self.reader.peek() == '#' {
            self.reader.skip();
            let start = self.reader.cursor();
            let id = self.reader.read_identifier()?;
            if self.registry.tag(&id).is_none() {
                self.reader.set_cursor(start);
                return Err(ItemArgumentError::UnknownTag { cursor: start, id });
            }
            tests.push(ItemPredicateModel::Tag(id));
        } else if self.reader.can_read() && self.reader.peek() != '[' {
            let start = self.reader.cursor();
            let id = self.reader.read_identifier()?;
            if self.registry.item(&id).is_none() {
                self.reader.set_cursor(start);
                return Err(ItemArgumentError::UnknownItem { cursor: start, id });
            }
            tests.push(ItemPredicateModel::Item(id));
        }
        if self.reader.can_read() && self.reader.peek() == '[' {
            self.reader.skip();
            if self.reader.can_read() && self.reader.peek() != ']' {
                tests.extend(self.parse_conditions()?);
            }
            self.reader.expect(']')?;
        }
        Ok(if tests.len() == 1 {
            tests.remove(0)
        } else {
            ItemPredicateModel::All(tests)
        })
    }

    fn parse_conditions(&mut self) -> Result<Vec<ItemPredicateModel>, ItemArgumentError> {
        let mut values = Vec::new();
        loop {
            values.push(self.parse_alternatives()?);
            if self.reader.can_read() && self.reader.peek() == ',' {
                self.reader.skip();
            } else {
                break;
            }
        }
        Ok(values)
    }

    fn parse_alternatives(&mut self) -> Result<ItemPredicateModel, ItemArgumentError> {
        let mut values = vec![self.parse_term()?];
        while self.reader.can_read() && self.reader.peek() == '|' {
            self.reader.skip();
            values.push(self.parse_term()?);
        }
        Ok(if values.len() == 1 {
            values.remove(0)
        } else {
            ItemPredicateModel::Any(values)
        })
    }

    fn parse_term(&mut self) -> Result<ItemPredicateModel, ItemArgumentError> {
        if self.reader.can_read() && self.reader.peek() == '!' {
            self.reader.skip();
            return Ok(ItemPredicateModel::Not(Box::new(self.parse_test()?)));
        }
        self.parse_test()
    }

    fn parse_test(&mut self) -> Result<ItemPredicateModel, ItemArgumentError> {
        let start = self.reader.cursor();
        let id = self.reader.read_identifier()?;
        if self.reader.can_read() && self.reader.peek() == '~' {
            self.reader.skip();
            let value_start = self.reader.cursor();
            let value = self.reader.read_value()?;
            if id == "minecraft:count" {
                return Ok(ItemPredicateModel::CountRange(parse_count_range(
                    value_start,
                    &value,
                )?));
            }
            if !self.registry.predicates.contains(&id) {
                self.reader.set_cursor(start);
                return Err(ItemArgumentError::UnknownPredicate { cursor: start, id });
            }
            if value == "bad" {
                self.reader.set_cursor(value_start);
                return Err(ItemArgumentError::MalformedPredicate {
                    cursor: value_start,
                    id,
                    message: "predicate codec rejected value".to_string(),
                });
            }
            return Ok(ItemPredicateModel::PredicateValue(id, value));
        }
        if self.reader.can_read() && self.reader.peek() == '=' {
            self.reader.skip();
            let value_start = self.reader.cursor();
            let value = self.reader.read_value()?;
            if id == "minecraft:count" {
                return Ok(ItemPredicateModel::CountRange(parse_count_range(
                    value_start,
                    &value,
                )?));
            }
            let component = self.lookup_component_for_predicate(start, &id, true)?;
            if value == "bad" {
                self.reader.set_cursor(value_start);
                return Err(ItemArgumentError::MalformedComponent {
                    cursor: value_start,
                    id: component,
                    message: "component codec rejected value".to_string(),
                });
            }
            return Ok(ItemPredicateModel::ComponentEquals(component, value));
        }
        if id == "minecraft:count" {
            return Ok(ItemPredicateModel::CountRange(CountRangeModel {
                min: Some(1),
                max: None,
            }));
        }
        let component = self.lookup_component_for_predicate(start, &id, false)?;
        Ok(ItemPredicateModel::ComponentPresent(component))
    }

    fn lookup_component_for_predicate(
        &mut self,
        start: usize,
        id: &str,
        require_value_codec: bool,
    ) -> Result<String, ItemArgumentError> {
        let component = if require_value_codec {
            self.registry.component_with_value_codec(id)
        } else {
            self.registry.component_for_item_parser(id)
        };
        if let Some(component) = component {
            Ok(component.id.clone())
        } else {
            self.reader.set_cursor(start);
            Err(ItemArgumentError::UnknownComponent {
                cursor: start,
                id: id.to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionArgumentModel;

impl FunctionArgumentModel {
    pub fn functions() -> Self {
        Self
    }

    pub fn parse(
        &self,
        reader: &mut StringReaderModel,
    ) -> Result<FunctionResultModel, ItemArgumentError> {
        if reader.can_read() && reader.peek() == '#' {
            reader.skip();
            Ok(FunctionResultModel::Tag(reader.read_identifier()?))
        } else {
            Ok(FunctionResultModel::Function(reader.read_identifier()?))
        }
    }

    pub fn get_functions(
        context: &CommandFunctionContextModel,
        name: &str,
    ) -> Result<Vec<CommandFunctionModel>, ItemArgumentError> {
        context
            .arguments
            .get(name)
            .expect("typed argument exists")
            .create(context)
    }

    pub fn get_function_or_tag(
        context: &CommandFunctionContextModel,
        name: &str,
    ) -> Result<FunctionUnwrapModel, ItemArgumentError> {
        context
            .arguments
            .get(name)
            .expect("typed argument exists")
            .unwrap(context)
    }

    pub fn get_function_collection(
        context: &CommandFunctionContextModel,
        name: &str,
    ) -> Result<(String, Vec<CommandFunctionModel>), ItemArgumentError> {
        context
            .arguments
            .get(name)
            .expect("typed argument exists")
            .unwrap_to_collection(context)
    }

    pub fn examples(&self) -> [&'static str; 3] {
        ["foo", "foo:bar", "#foo"]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionResultModel {
    Function(String),
    Tag(String),
}

impl FunctionResultModel {
    fn create(
        &self,
        context: &CommandFunctionContextModel,
    ) -> Result<Vec<CommandFunctionModel>, ItemArgumentError> {
        match self {
            Self::Function(id) => context
                .manager
                .functions
                .get(id)
                .cloned()
                .map(|function| vec![function])
                .ok_or_else(|| ItemArgumentError::UnknownFunction { id: id.clone() }),
            Self::Tag(id) => context
                .manager
                .tags
                .get(id)
                .cloned()
                .ok_or_else(|| ItemArgumentError::UnknownFunctionTag { id: id.clone() }),
        }
    }

    fn unwrap(
        &self,
        context: &CommandFunctionContextModel,
    ) -> Result<FunctionUnwrapModel, ItemArgumentError> {
        match self {
            Self::Function(id) => Ok(FunctionUnwrapModel::Function(
                id.clone(),
                context
                    .manager
                    .functions
                    .get(id)
                    .cloned()
                    .ok_or_else(|| ItemArgumentError::UnknownFunction { id: id.clone() })?,
            )),
            Self::Tag(id) => Ok(FunctionUnwrapModel::Tag(
                id.clone(),
                context
                    .manager
                    .tags
                    .get(id)
                    .cloned()
                    .ok_or_else(|| ItemArgumentError::UnknownFunctionTag { id: id.clone() })?,
            )),
        }
    }

    fn unwrap_to_collection(
        &self,
        context: &CommandFunctionContextModel,
    ) -> Result<(String, Vec<CommandFunctionModel>), ItemArgumentError> {
        match self.unwrap(context)? {
            FunctionUnwrapModel::Function(id, function) => Ok((id, vec![function])),
            FunctionUnwrapModel::Tag(id, functions) => Ok((id, functions)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionUnwrapModel {
    Function(String, CommandFunctionModel),
    Tag(String, Vec<CommandFunctionModel>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFunctionModel {
    id: String,
    commands: Vec<String>,
}

impl CommandFunctionModel {
    pub fn new(id: &str, commands: &[&str]) -> Self {
        Self {
            id: normalize_id(id),
            commands: commands
                .iter()
                .map(|command| (*command).to_string())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandFunctionManagerModel {
    functions: BTreeMap<String, CommandFunctionModel>,
    tags: BTreeMap<String, Vec<CommandFunctionModel>>,
}

impl CommandFunctionManagerModel {
    pub fn with_function(mut self, function: CommandFunctionModel) -> Self {
        self.functions.insert(function.id.clone(), function);
        self
    }

    pub fn with_tag(mut self, id: &str, functions: Vec<CommandFunctionModel>) -> Self {
        self.tags.insert(normalize_id(id), functions);
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandBuildContextModel;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandContextModel {
    items: HashMap<String, ItemInputModel>,
    predicates: HashMap<String, ItemPredicateModel>,
}

impl CommandContextModel {
    pub fn with_item(mut self, name: &str, value: ItemInputModel) -> Self {
        self.items.insert(name.to_string(), value);
        self
    }

    pub fn with_predicate(mut self, name: &str, value: ItemPredicateModel) -> Self {
        self.predicates.insert(name.to_string(), value);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandFunctionContextModel {
    manager: CommandFunctionManagerModel,
    arguments: HashMap<String, FunctionResultModel>,
}

impl CommandFunctionContextModel {
    pub fn new(manager: CommandFunctionManagerModel) -> Self {
        Self {
            manager,
            arguments: HashMap::new(),
        }
    }

    pub fn with_argument(mut self, name: &str, value: FunctionResultModel) -> Self {
        self.arguments.insert(name.to_string(), value);
        self
    }
}

fn parse_count_range(cursor: usize, value: &str) -> Result<CountRangeModel, ItemArgumentError> {
    let parse_bound = |raw: &str| {
        raw.parse::<u32>()
            .map_err(|_| ItemArgumentError::InvalidValue { cursor })
    };
    if let Some((min, max)) = value.split_once("..") {
        Ok(CountRangeModel {
            min: (!min.is_empty()).then(|| parse_bound(min)).transpose()?,
            max: (!max.is_empty()).then(|| parse_bound(max)).transpose()?,
        })
    } else {
        Ok(CountRangeModel::exact(parse_bound(value)?))
    }
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

#[cfg(test)]
#[path = "command_item_arguments_tests.rs"]
mod tests;
