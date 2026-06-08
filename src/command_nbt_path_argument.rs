use std::collections::HashMap;

use crate::storage::nbt::Tag;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NbtPathArgumentModel;

impl NbtPathArgumentModel {
    pub fn nbt_path() -> Self {
        Self
    }

    pub fn parse(&self, reader: &mut StringReaderModel) -> Result<NbtPathModel, NbtPathError> {
        parse_path(reader)
    }

    pub fn examples(&self) -> [&'static str; 6] {
        ["foo", "foo.bar", "foo[0]", "[0]", "[]", "{foo=bar}"]
    }
}

pub fn get_path(context: &CommandContextModel, name: &str) -> Option<NbtPathModel> {
    context.paths.get(name).cloned()
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommandContextModel {
    paths: HashMap<String, NbtPathModel>,
}

impl CommandContextModel {
    pub fn with_path(mut self, name: impl Into<String>, path: NbtPathModel) -> Self {
        self.paths.insert(name.into(), path);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NbtPathModel {
    original: String,
    nodes: Vec<PathNodeModel>,
    node_end_positions: Vec<usize>,
}

impl NbtPathModel {
    pub fn of(input: &str) -> Result<Self, NbtPathError> {
        NbtPathArgumentModel::nbt_path().parse(&mut StringReaderModel::new(input))
    }

    pub fn as_string(&self) -> &str {
        &self.original
    }

    pub fn nodes(&self) -> &[PathNodeModel] {
        &self.nodes
    }

    pub fn get(&self, tag: &Tag) -> Result<Vec<Tag>, NbtPathError> {
        let mut result = vec![tag.clone()];
        for (index, node) in self.nodes.iter().enumerate() {
            result = result
                .iter()
                .flat_map(|tag| node.get(tag))
                .collect::<Vec<_>>();
            if result.is_empty() {
                return Err(self.not_found(index));
            }
        }
        Ok(result)
    }

    pub fn count_matching(&self, tag: &Tag) -> usize {
        let mut result = vec![tag.clone()];
        for node in &self.nodes {
            result = result
                .iter()
                .flat_map(|tag| node.get(tag))
                .collect::<Vec<_>>();
            if result.is_empty() {
                return 0;
            }
        }
        result.len()
    }

    pub fn get_or_create(
        &self,
        tag: &mut Tag,
        new_tag_value: impl Fn() -> Tag + Copy,
    ) -> Result<Vec<Tag>, NbtPathError> {
        let result = get_or_create_at(tag, &self.nodes, new_tag_value);
        if result.is_empty() && self.nodes.len() > 1 {
            Err(self.not_found(self.nodes.len() - 2))
        } else {
            Ok(result)
        }
    }

    pub fn set(&self, tag: &mut Tag, to_add: Tag) -> Result<usize, NbtPathError> {
        if is_too_deep(&to_add, self.estimate_path_depth()) {
            return Err(NbtPathError::DataTooDeep);
        }
        Ok(set_at(tag, &self.nodes, &to_add))
    }

    pub fn insert(
        &self,
        index: isize,
        target: &mut Tag,
        to_insert: &[Tag],
    ) -> Result<usize, NbtPathError> {
        for tag in to_insert {
            if is_too_deep(tag, self.estimate_path_depth()) {
                return Err(NbtPathError::DataTooDeep);
            }
        }
        let targets = self.get_or_create(target, || Tag::List(Vec::new()))?;
        if targets.iter().any(|tag| !matches!(tag, Tag::List(_))) {
            return Err(NbtPathError::ExpectedList);
        }
        insert_at(target, &self.nodes, index, to_insert)
    }

    pub fn remove(&self, tag: &mut Tag) -> usize {
        remove_at(tag, &self.nodes)
    }

    pub fn is_too_deep(tag: &Tag, depth: usize) -> bool {
        is_too_deep(tag, depth)
    }

    fn estimate_path_depth(&self) -> usize {
        self.nodes.len()
    }

    fn not_found(&self, node_index: usize) -> NbtPathError {
        let end = self.node_end_positions[node_index];
        NbtPathError::NothingFound(self.original[..end].to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathNodeModel {
    CompoundChild(String),
    IndexedElement(isize),
    AllElements,
    MatchElement(Tag),
    MatchObject { name: String, pattern: Tag },
    MatchRootObject(Tag),
}

impl PathNodeModel {
    fn get(&self, parent: &Tag) -> Vec<Tag> {
        match self {
            Self::CompoundChild(name) => compound_get(parent, name).cloned().into_iter().collect(),
            Self::IndexedElement(index) => list_index(parent, *index)
                .and_then(|actual| list_get(parent, actual))
                .cloned()
                .into_iter()
                .collect(),
            Self::AllElements => match parent {
                Tag::List(values) => values.clone(),
                _ => Vec::new(),
            },
            Self::MatchElement(pattern) => match parent {
                Tag::List(values) => values
                    .iter()
                    .filter(|tag| compare_nbt(pattern, tag))
                    .cloned()
                    .collect(),
                _ => Vec::new(),
            },
            Self::MatchObject { name, pattern } => compound_get(parent, name)
                .filter(|tag| compare_nbt(pattern, tag))
                .cloned()
                .into_iter()
                .collect(),
            Self::MatchRootObject(pattern) => {
                if matches!(parent, Tag::Compound(_)) && compare_nbt(pattern, parent) {
                    vec![parent.clone()]
                } else {
                    Vec::new()
                }
            }
        }
    }

    fn get_or_create(&self, parent: &mut Tag, child: impl Fn() -> Tag + Copy) -> Vec<Tag> {
        match self {
            Self::CompoundChild(name) => match parent {
                Tag::Compound(fields) => {
                    if !fields.iter().any(|(key, _)| key == name) {
                        fields.push((name.clone(), child()));
                    }
                    compound_get(parent, name).cloned().into_iter().collect()
                }
                _ => Vec::new(),
            },
            Self::IndexedElement(_) => self.get(parent),
            Self::AllElements => match parent {
                Tag::List(values) => {
                    if values.is_empty() {
                        values.push(child());
                    }
                    values.clone()
                }
                _ => Vec::new(),
            },
            Self::MatchElement(pattern) => match parent {
                Tag::List(values) => {
                    let matches = values
                        .iter()
                        .filter(|tag| compare_nbt(pattern, tag))
                        .cloned()
                        .collect::<Vec<_>>();
                    if matches.is_empty() {
                        values.push(pattern.clone());
                        vec![pattern.clone()]
                    } else {
                        matches
                    }
                }
                _ => Vec::new(),
            },
            Self::MatchObject { name, pattern } => match parent {
                Tag::Compound(fields) => {
                    if let Some(value) = fields
                        .iter()
                        .find(|(key, _)| key == name)
                        .map(|(_, value)| value)
                    {
                        if compare_nbt(pattern, value) {
                            vec![value.clone()]
                        } else {
                            Vec::new()
                        }
                    } else {
                        fields.push((name.clone(), pattern.clone()));
                        vec![pattern.clone()]
                    }
                }
                _ => Vec::new(),
            },
            Self::MatchRootObject(_) => self.get(parent),
        }
    }

    fn preferred_parent_tag(&self) -> Tag {
        match self {
            Self::CompoundChild(_) | Self::MatchObject { .. } | Self::MatchRootObject(_) => {
                Tag::Compound(Vec::new())
            }
            Self::IndexedElement(_) | Self::AllElements | Self::MatchElement(_) => {
                Tag::List(Vec::new())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NbtPathError {
    InvalidNode,
    NothingFound(String),
    DataTooDeep,
    ExpectedList,
    InvalidIndex(isize),
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

    fn can_read(&self) -> bool {
        self.cursor < self.input.len()
    }

    fn peek(&self) -> char {
        self.input.as_bytes()[self.cursor] as char
    }

    fn skip(&mut self) {
        self.cursor += 1;
    }

    fn expect(&mut self, expected: char) -> Result<(), NbtPathError> {
        if self.can_read() && self.peek() == expected {
            self.skip();
            Ok(())
        } else {
            Err(NbtPathError::InvalidNode)
        }
    }

    fn read_string(&mut self) -> Result<String, NbtPathError> {
        let quote = self.peek();
        if !matches!(quote, '"' | '\'') {
            return Err(NbtPathError::InvalidNode);
        }
        self.skip();
        let start = self.cursor;
        while self.can_read() && self.peek() != quote {
            self.skip();
        }
        if !self.can_read() {
            return Err(NbtPathError::InvalidNode);
        }
        let value = self.input[start..self.cursor].to_string();
        self.skip();
        Ok(value)
    }
}

fn parse_path(reader: &mut StringReaderModel) -> Result<NbtPathModel, NbtPathError> {
    let mut nodes = Vec::new();
    let mut positions = Vec::new();
    let start = reader.cursor();
    let mut first_node = true;

    while reader.can_read() && reader.peek() != ' ' {
        nodes.push(parse_node(reader, first_node)?);
        positions.push(reader.cursor() - start);
        first_node = false;
        if reader.can_read() {
            let next = reader.peek();
            if next != ' ' && next != '[' && next != '{' {
                reader.expect('.')?;
            }
        }
    }

    Ok(NbtPathModel {
        original: reader.input[start..reader.cursor()].to_string(),
        nodes,
        node_end_positions: positions,
    })
}

fn parse_node(
    reader: &mut StringReaderModel,
    first_node: bool,
) -> Result<PathNodeModel, NbtPathError> {
    match reader.peek() {
        '"' | '\'' => {
            let name = reader.read_string()?;
            read_object_node(reader, name)
        }
        '[' => {
            reader.skip();
            match reader.peek() {
                '{' => {
                    let pattern = parse_compound_pattern(reader)?;
                    reader.expect(']')?;
                    Ok(PathNodeModel::MatchElement(pattern))
                }
                ']' => {
                    reader.skip();
                    Ok(PathNodeModel::AllElements)
                }
                _ => {
                    let index = read_int(reader)?;
                    reader.expect(']')?;
                    Ok(PathNodeModel::IndexedElement(index))
                }
            }
        }
        '{' => {
            if !first_node {
                return Err(NbtPathError::InvalidNode);
            }
            Ok(PathNodeModel::MatchRootObject(parse_compound_pattern(
                reader,
            )?))
        }
        _ => {
            let name = read_unquoted_name(reader)?;
            read_object_node(reader, name)
        }
    }
}

fn read_object_node(
    reader: &mut StringReaderModel,
    name: String,
) -> Result<PathNodeModel, NbtPathError> {
    if name.is_empty() {
        Err(NbtPathError::InvalidNode)
    } else if reader.can_read() && reader.peek() == '{' {
        Ok(PathNodeModel::MatchObject {
            name,
            pattern: parse_compound_pattern(reader)?,
        })
    } else {
        Ok(PathNodeModel::CompoundChild(name))
    }
}

fn read_unquoted_name(reader: &mut StringReaderModel) -> Result<String, NbtPathError> {
    let start = reader.cursor();
    while reader.can_read() && is_allowed_in_unquoted_name(reader.peek()) {
        reader.skip();
    }
    if reader.cursor() == start {
        Err(NbtPathError::InvalidNode)
    } else {
        Ok(reader.input[start..reader.cursor()].to_string())
    }
}

fn is_allowed_in_unquoted_name(ch: char) -> bool {
    !matches!(ch, ' ' | '"' | '\'' | '[' | ']' | '.' | '{' | '}')
}

fn read_int(reader: &mut StringReaderModel) -> Result<isize, NbtPathError> {
    let start = reader.cursor();
    if reader.can_read() && reader.peek() == '-' {
        reader.skip();
    }
    while reader.can_read() && reader.peek().is_ascii_digit() {
        reader.skip();
    }
    reader.input[start..reader.cursor()]
        .parse()
        .map_err(|_| NbtPathError::InvalidNode)
}

fn parse_compound_pattern(reader: &mut StringReaderModel) -> Result<Tag, NbtPathError> {
    reader.expect('{')?;
    let mut fields = Vec::new();
    loop {
        if !reader.can_read() {
            return Err(NbtPathError::InvalidNode);
        }
        if reader.peek() == '}' {
            reader.skip();
            return Ok(Tag::Compound(fields));
        }
        let key = if matches!(reader.peek(), '"' | '\'') {
            reader.read_string()?
        } else {
            read_pattern_token(reader)?
        };
        if !reader.can_read() || !matches!(reader.peek(), ':' | '=') {
            return Err(NbtPathError::InvalidNode);
        }
        reader.skip();
        let value = parse_pattern_value(reader)?;
        fields.push((key, value));
        if reader.can_read() && reader.peek() == ',' {
            reader.skip();
        }
    }
}

fn parse_pattern_value(reader: &mut StringReaderModel) -> Result<Tag, NbtPathError> {
    match reader.peek() {
        '{' => parse_compound_pattern(reader),
        '"' | '\'' => reader.read_string().map(Tag::String),
        _ => {
            let token = read_pattern_token(reader)?;
            if let Ok(value) = token.parse::<i32>() {
                Ok(Tag::Int(value))
            } else {
                Ok(Tag::String(token))
            }
        }
    }
}

fn read_pattern_token(reader: &mut StringReaderModel) -> Result<String, NbtPathError> {
    let start = reader.cursor();
    while reader.can_read() && !matches!(reader.peek(), ':' | '=' | ',' | '}' | ']') {
        reader.skip();
    }
    if reader.cursor() == start {
        Err(NbtPathError::InvalidNode)
    } else {
        Ok(reader.input[start..reader.cursor()].to_string())
    }
}

fn compound_get<'a>(tag: &'a Tag, name: &str) -> Option<&'a Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value),
        _ => None,
    }
}

fn compound_get_mut<'a>(tag: &'a mut Tag, name: &str) -> Option<&'a mut Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter_mut()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value),
        _ => None,
    }
}

fn compound_put(tag: &mut Tag, name: String, value: Tag) -> Option<Tag> {
    match tag {
        Tag::Compound(fields) => {
            if let Some((_, previous)) = fields.iter_mut().find(|(field, _)| field == &name) {
                Some(std::mem::replace(previous, value))
            } else {
                fields.push((name, value));
                None
            }
        }
        _ => None,
    }
}

fn compound_remove(tag: &mut Tag, name: &str) -> Option<Tag> {
    match tag {
        Tag::Compound(fields) => fields
            .iter()
            .position(|(field, _)| field == name)
            .map(|index| fields.remove(index).1),
        _ => None,
    }
}

fn list_get(tag: &Tag, index: usize) -> Option<&Tag> {
    match tag {
        Tag::List(values) => values.get(index),
        _ => None,
    }
}

fn list_index(tag: &Tag, index: isize) -> Option<usize> {
    match tag {
        Tag::List(values) => {
            let size = values.len() as isize;
            let actual = if index < 0 { size + index } else { index };
            (0..size).contains(&actual).then_some(actual as usize)
        }
        _ => None,
    }
}

fn compare_nbt(pattern: &Tag, tag: &Tag) -> bool {
    match (pattern, tag) {
        (Tag::Compound(pattern), Tag::Compound(fields)) => pattern.iter().all(|(key, value)| {
            fields
                .iter()
                .find(|(field, _)| field == key)
                .is_some_and(|(_, tag_value)| compare_nbt(value, tag_value))
        }),
        _ => pattern == tag,
    }
}

fn get_or_create_at(
    tag: &mut Tag,
    nodes: &[PathNodeModel],
    new_tag: impl Fn() -> Tag + Copy,
) -> Vec<Tag> {
    let Some((node, rest)) = nodes.split_first() else {
        return vec![tag.clone()];
    };
    if rest.is_empty() {
        return node.get_or_create(tag, new_tag);
    }
    match node {
        PathNodeModel::CompoundChild(name) => {
            if compound_get(tag, name).is_none() {
                compound_put(tag, name.clone(), rest[0].preferred_parent_tag());
            }
            compound_get_mut(tag, name)
                .map(|child| get_or_create_at(child, rest, new_tag))
                .unwrap_or_default()
        }
        PathNodeModel::IndexedElement(index) => match tag {
            Tag::List(values) => {
                let size = values.len() as isize;
                let actual = if *index < 0 { size + index } else { *index };
                if (0..size).contains(&actual) {
                    get_or_create_at(&mut values[actual as usize], rest, new_tag)
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        },
        PathNodeModel::AllElements => match tag {
            Tag::List(values) => {
                if values.is_empty() {
                    values.push(rest[0].preferred_parent_tag());
                }
                values
                    .iter_mut()
                    .flat_map(|tag| get_or_create_at(tag, rest, new_tag))
                    .collect()
            }
            _ => Vec::new(),
        },
        PathNodeModel::MatchElement(pattern) => match tag {
            Tag::List(values) => {
                if !values.iter().any(|tag| compare_nbt(pattern, tag)) {
                    values.push(pattern.clone());
                }
                values
                    .iter_mut()
                    .filter(|tag| compare_nbt(pattern, tag))
                    .flat_map(|tag| get_or_create_at(tag, rest, new_tag))
                    .collect()
            }
            _ => Vec::new(),
        },
        PathNodeModel::MatchObject { name, pattern } => match tag {
            Tag::Compound(fields) => {
                if !fields.iter().any(|(key, _)| key == name) {
                    fields.push((name.clone(), pattern.clone()));
                }
                fields
                    .iter_mut()
                    .find(|(key, value)| key == name && compare_nbt(pattern, value))
                    .map(|(_, value)| get_or_create_at(value, rest, new_tag))
                    .unwrap_or_default()
            }
            _ => Vec::new(),
        },
        PathNodeModel::MatchRootObject(pattern) => {
            if compare_nbt(pattern, tag) {
                get_or_create_at(tag, rest, new_tag)
            } else {
                Vec::new()
            }
        }
    }
}

fn set_at(tag: &mut Tag, nodes: &[PathNodeModel], value: &Tag) -> usize {
    let Some((node, rest)) = nodes.split_first() else {
        return 0;
    };
    if rest.is_empty() {
        return set_last(tag, node, value);
    }
    match node {
        PathNodeModel::CompoundChild(name) => {
            if compound_get(tag, name).is_none() {
                compound_put(tag, name.clone(), rest[0].preferred_parent_tag());
            }
            compound_get_mut(tag, name)
                .map(|child| set_at(child, rest, value))
                .unwrap_or(0)
        }
        PathNodeModel::IndexedElement(index) => match tag {
            Tag::List(values) => {
                let size = values.len() as isize;
                let actual = if *index < 0 { size + index } else { *index };
                if (0..size).contains(&actual) {
                    set_at(&mut values[actual as usize], rest, value)
                } else {
                    0
                }
            }
            _ => 0,
        },
        PathNodeModel::AllElements => match tag {
            Tag::List(values) => values.iter_mut().map(|tag| set_at(tag, rest, value)).sum(),
            _ => 0,
        },
        PathNodeModel::MatchElement(pattern) => match tag {
            Tag::List(values) => values
                .iter_mut()
                .filter(|tag| compare_nbt(pattern, tag))
                .map(|tag| set_at(tag, rest, value))
                .sum(),
            _ => 0,
        },
        PathNodeModel::MatchObject { name, pattern } => compound_get_mut(tag, name)
            .filter(|child| compare_nbt(pattern, child))
            .map(|child| set_at(child, rest, value))
            .unwrap_or(0),
        PathNodeModel::MatchRootObject(pattern) => {
            if compare_nbt(pattern, tag) {
                set_at(tag, rest, value)
            } else {
                0
            }
        }
    }
}

fn set_last(parent: &mut Tag, node: &PathNodeModel, value: &Tag) -> usize {
    match node {
        PathNodeModel::CompoundChild(name) => {
            let previous = compound_put(parent, name.clone(), value.clone());
            usize::from(previous.as_ref() != Some(value))
        }
        PathNodeModel::IndexedElement(index) => match parent {
            Tag::List(values) => {
                let size = values.len() as isize;
                let actual = if *index < 0 { size + index } else { *index };
                if (0..size).contains(&actual) && values[actual as usize] != *value {
                    values[actual as usize] = value.clone();
                    1
                } else {
                    0
                }
            }
            _ => 0,
        },
        PathNodeModel::AllElements => match parent {
            Tag::List(values) => {
                if values.is_empty() {
                    values.push(value.clone());
                    1
                } else {
                    let changed = values.iter().filter(|tag| *tag != value).count();
                    if changed > 0 {
                        values.fill(value.clone());
                    }
                    changed
                }
            }
            _ => 0,
        },
        PathNodeModel::MatchElement(pattern) => match parent {
            Tag::List(values) => {
                if values.is_empty() {
                    values.push(value.clone());
                    1
                } else {
                    let mut changed = 0;
                    for tag in values {
                        if compare_nbt(pattern, tag) && *tag != *value {
                            *tag = value.clone();
                            changed += 1;
                        }
                    }
                    changed
                }
            }
            _ => 0,
        },
        PathNodeModel::MatchObject { name, pattern } => compound_get_mut(parent, name)
            .filter(|tag| compare_nbt(pattern, tag) && **tag != *value)
            .map(|tag| {
                *tag = value.clone();
                1
            })
            .unwrap_or(0),
        PathNodeModel::MatchRootObject(_) => 0,
    }
}

fn insert_at(
    tag: &mut Tag,
    nodes: &[PathNodeModel],
    index: isize,
    values: &[Tag],
) -> Result<usize, NbtPathError> {
    let Some((node, rest)) = nodes.split_first() else {
        return insert_into_list(tag, index, values);
    };
    if rest.is_empty() {
        for target in node.get(tag) {
            if !matches!(target, Tag::List(_)) {
                return Err(NbtPathError::ExpectedList);
            }
        }
        return insert_target(tag, node, index, values);
    }
    match node {
        PathNodeModel::CompoundChild(name) => compound_get_mut(tag, name)
            .map(|child| insert_at(child, rest, index, values))
            .unwrap_or(Ok(0)),
        _ => Ok(0),
    }
}

fn insert_target(
    tag: &mut Tag,
    node: &PathNodeModel,
    index: isize,
    values: &[Tag],
) -> Result<usize, NbtPathError> {
    match node {
        PathNodeModel::CompoundChild(name) => compound_get_mut(tag, name)
            .map(|target| insert_into_list(target, index, values))
            .unwrap_or(Ok(0)),
        _ => Ok(0),
    }
}

fn insert_into_list(tag: &mut Tag, index: isize, values: &[Tag]) -> Result<usize, NbtPathError> {
    let Tag::List(target) = tag else {
        return Err(NbtPathError::ExpectedList);
    };
    let mut actual = if index < 0 {
        target.len() as isize + index + 1
    } else {
        index
    };
    if actual < 0 || actual > target.len() as isize {
        return Err(NbtPathError::InvalidIndex(actual));
    }
    for value in values {
        target.insert(actual as usize, value.clone());
        actual += 1;
    }
    Ok(usize::from(!values.is_empty()))
}

fn remove_at(tag: &mut Tag, nodes: &[PathNodeModel]) -> usize {
    let Some((node, rest)) = nodes.split_first() else {
        return 0;
    };
    if rest.is_empty() {
        return remove_last(tag, node);
    }
    match node {
        PathNodeModel::CompoundChild(name) => compound_get_mut(tag, name)
            .map(|child| remove_at(child, rest))
            .unwrap_or(0),
        _ => 0,
    }
}

fn remove_last(parent: &mut Tag, node: &PathNodeModel) -> usize {
    match node {
        PathNodeModel::CompoundChild(name) => usize::from(compound_remove(parent, name).is_some()),
        PathNodeModel::IndexedElement(index) => match parent {
            Tag::List(values) => {
                let size = values.len() as isize;
                let actual = if *index < 0 { size + index } else { *index };
                if (0..size).contains(&actual) {
                    values.remove(actual as usize);
                    1
                } else {
                    0
                }
            }
            _ => 0,
        },
        PathNodeModel::AllElements => match parent {
            Tag::List(values) => {
                let size = values.len();
                values.clear();
                size
            }
            _ => 0,
        },
        PathNodeModel::MatchElement(pattern) => match parent {
            Tag::List(values) => {
                let old = values.len();
                values.retain(|tag| !compare_nbt(pattern, tag));
                old - values.len()
            }
            _ => 0,
        },
        PathNodeModel::MatchObject { name, pattern } => {
            let should_remove =
                compound_get(parent, name).is_some_and(|tag| compare_nbt(pattern, tag));
            if should_remove && compound_remove(parent, name).is_some() {
                1
            } else {
                0
            }
        }
        PathNodeModel::MatchRootObject(_) => 0,
    }
}

fn is_too_deep(tag: &Tag, depth: usize) -> bool {
    if depth >= 512 {
        return true;
    }
    match tag {
        Tag::Compound(fields) => fields.iter().any(|(_, tag)| is_too_deep(tag, depth + 1)),
        Tag::List(values) => values.iter().any(|tag| is_too_deep(tag, depth + 1)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(fields: Vec<(&str, Tag)>) -> Tag {
        Tag::Compound(
            fields
                .into_iter()
                .map(|(name, value)| (name.to_string(), value))
                .collect(),
        )
    }

    #[test]
    fn java_factory_examples_context_getter_and_codec_like_of_match_source() {
        let argument = NbtPathArgumentModel::nbt_path();
        assert_eq!(
            argument.examples(),
            ["foo", "foo.bar", "foo[0]", "[0]", "[]", "{foo=bar}"]
        );
        let path = NbtPathModel::of("foo.bar").unwrap();
        assert_eq!(path.as_string(), "foo.bar");
        let context = CommandContextModel::default().with_path("path", path.clone());
        assert_eq!(get_path(&context, "path"), Some(path));
    }

    #[test]
    fn java_parse_covers_node_kinds_and_original_positions() {
        let path = NbtPathModel::of("{foo=bar}.items[{id=stone}][0][]").unwrap();
        assert!(matches!(path.nodes()[0], PathNodeModel::MatchRootObject(_)));
        assert!(matches!(path.nodes()[1], PathNodeModel::CompoundChild(_)));
        assert!(matches!(path.nodes()[2], PathNodeModel::MatchElement(_)));
        assert!(matches!(path.nodes()[3], PathNodeModel::IndexedElement(0)));
        assert!(matches!(path.nodes()[4], PathNodeModel::AllElements));
        assert_eq!(
            NbtPathModel::of("foo.{bar=1}"),
            Err(NbtPathError::InvalidNode)
        );
    }

    #[test]
    fn java_get_and_count_matching_walk_paths_and_report_prefix_on_miss() {
        let tag = compound(vec![(
            "foo",
            compound(vec![(
                "list",
                Tag::List(vec![compound(vec![(
                    "id",
                    Tag::String("stone".to_string()),
                )])]),
            )]),
        )]);
        let path = NbtPathModel::of("foo.list[{id=stone}].id").unwrap();
        assert_eq!(path.count_matching(&tag), 1);
        assert_eq!(path.get(&tag), Ok(vec![Tag::String("stone".to_string())]));

        let missing = NbtPathModel::of("foo.missing.name").unwrap();
        assert_eq!(
            missing.get(&tag),
            Err(NbtPathError::NothingFound("foo.missing".to_string()))
        );
    }

    #[test]
    fn java_set_creates_parent_compounds_and_reports_changed_count() {
        let mut tag = Tag::Compound(Vec::new());
        let path = NbtPathModel::of("foo.bar").unwrap();
        assert_eq!(path.set(&mut tag, Tag::Int(3)), Ok(1));
        assert_eq!(
            tag,
            compound(vec![("foo", compound(vec![("bar", Tag::Int(3))]))])
        );
        assert_eq!(path.set(&mut tag, Tag::Int(3)), Ok(0));
    }

    #[test]
    fn java_index_all_and_remove_mutations_match_list_behavior() {
        let mut tag = compound(vec![(
            "items",
            Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)]),
        )]);
        assert_eq!(
            NbtPathModel::of("items[-1]")
                .unwrap()
                .set(&mut tag, Tag::Int(9)),
            Ok(1)
        );
        assert_eq!(NbtPathModel::of("items[]").unwrap().remove(&mut tag), 3);
        assert_eq!(tag, compound(vec![("items", Tag::List(Vec::new()))]));
    }

    #[test]
    fn java_insert_requires_list_checks_index_and_counts_modified_targets() {
        let mut tag = compound(vec![("items", Tag::List(vec![Tag::Int(1), Tag::Int(3)]))]);
        let path = NbtPathModel::of("items").unwrap();
        assert_eq!(path.insert(1, &mut tag, &[Tag::Int(2)]), Ok(1));
        assert_eq!(
            tag,
            compound(vec![(
                "items",
                Tag::List(vec![Tag::Int(1), Tag::Int(2), Tag::Int(3)])
            )])
        );
        assert_eq!(
            path.insert(9, &mut tag, &[Tag::Int(4)]),
            Err(NbtPathError::InvalidIndex(9))
        );
        let mut not_list = compound(vec![("items", Tag::Int(1))]);
        assert_eq!(
            path.insert(0, &mut not_list, &[Tag::Int(4)]),
            Err(NbtPathError::ExpectedList)
        );
    }

    #[test]
    fn java_match_object_and_match_element_create_or_filter_by_partial_compound() {
        let mut tag = compound(vec![("items", Tag::List(Vec::new()))]);
        let path = NbtPathModel::of("items[{id=stone}]").unwrap();
        assert_eq!(
            path.get_or_create(&mut tag, || compound(vec![("count", Tag::Int(1))])),
            Ok(vec![compound(vec![(
                "id",
                Tag::String("stone".to_string())
            )])])
        );
        assert_eq!(path.count_matching(&tag), 1);
    }

    #[test]
    fn java_depth_guard_trips_at_512_plus_estimated_path_depth() {
        let mut deep = Tag::Int(1);
        for _ in 0..512 {
            deep = Tag::List(vec![deep]);
        }
        assert!(NbtPathModel::is_too_deep(&deep, 0));
        let mut tag = Tag::Compound(Vec::new());
        assert_eq!(
            NbtPathModel::of("foo").unwrap().set(&mut tag, deep),
            Err(NbtPathError::DataTooDeep)
        );
    }
}
