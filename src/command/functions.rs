use super::*;
use std::collections::VecDeque;

pub const COMMAND_FUNCTIONS_PACKAGE_NULL_MARKED: bool = true;

pub fn load_command_functions_from_resources<'a>(
    resources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<CommandFunctionDefinition>, String> {
    let mut functions = Vec::new();
    for (path, contents) in resources {
        let Some(id) = function_id_from_path(path, ".mcfunction", "function")? else {
            continue;
        };
        functions.push(parse_command_function(&id, contents)?);
    }
    Ok(functions)
}

pub fn load_command_function_tags_from_resources<'a>(
    resources: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<Vec<CommandFunctionTag>, String> {
    let mut tags = Vec::new();
    for (path, contents) in resources {
        let Some(id) = function_id_from_path(path, ".json", "tags/function")? else {
            continue;
        };
        let value: serde_json::Value = serde_json::from_str(contents)
            .map_err(|err| format!("Failed to parse function tag {id}: {err}"))?;
        let values = value
            .get("values")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| format!("Function tag {id} is missing values array"))?;
        let mut functions = Vec::new();
        for value in values {
            match value {
                serde_json::Value::String(function) => {
                    let (parsed, tag) = parse_schedule_function(function)
                        .map_err(|_| format!("Invalid function tag value {function:?} in {id}"))?;
                    functions.push(if tag { format!("#{parsed}") } else { parsed });
                }
                serde_json::Value::Object(object) => {
                    let function = object
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| format!("Function tag {id} has object value without id"))?;
                    let (parsed, tag) = parse_schedule_function(function)
                        .map_err(|_| format!("Invalid function tag value {function:?} in {id}"))?;
                    functions.push(if tag { format!("#{parsed}") } else { parsed });
                }
                _ => return Err(format!("Function tag {id} contains a non-string value")),
            }
        }
        tags.push(CommandFunctionTag { id, functions });
    }
    Ok(tags)
}

pub(super) fn parse_command_function(
    id: &str,
    contents: &str,
) -> Result<CommandFunctionDefinition, String> {
    parse_resource_identifier(id).map_err(|_| format!("Invalid function id {id}"))?;
    let mut commands = Vec::new();
    let mut macro_parameters = Vec::new();
    let mut lines = contents.lines().enumerate();
    while let Some((line_index, raw_line)) = lines.next() {
        let line_number = line_index + 1;
        let mut line = raw_line.trim().to_string();
        if should_concatenate_next_function_line(&line) {
            loop {
                if line.pop() != Some('\\') {
                    return Err(format!("Invalid function line continuation in {id}"));
                }
                let Some((_, next_line)) = lines.next() else {
                    return Err(format!("Line continuation at end of function {id}"));
                };
                line.push_str(next_line.trim());
                check_function_line_length(id, &line)?;
                if !should_concatenate_next_function_line(&line) {
                    break;
                }
            }
        }
        check_function_line_length(id, &line)?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(command) = line.strip_prefix('/') {
            let hint = command.split_whitespace().next().unwrap_or_default();
            return Err(format!(
                "Invalid leading slash in function {id} line {line_number}; use '{hint}' without '/'"
            ));
        }
        if let Some(macro_line) = line.strip_prefix('$') {
            let parameters = parse_function_macro_variables(id, line_number, macro_line)?;
            for parameter in parameters {
                if !macro_parameters.contains(&parameter) {
                    macro_parameters.push(parameter);
                }
            }
            commands.push(macro_line.to_string());
        } else {
            commands.push(line);
        }
    }
    Ok(CommandFunctionDefinition {
        id: id.to_string(),
        commands,
        macro_parameters,
    })
}

pub fn instantiate_command_function(
    function: &CommandFunctionDefinition,
    arguments: Option<&str>,
) -> Result<InstantiatedCommandFunction, String> {
    if function.macro_parameters.is_empty() {
        return Ok(InstantiatedCommandFunction {
            commands: function.commands.clone(),
        });
    }
    let arguments =
        arguments.ok_or_else(|| format!("Missing macro arguments for function {}", function.id))?;
    let Tag::Compound(values) = parse_snbt(arguments).map_err(|err| {
        format!(
            "Invalid macro arguments for function {}: {err}",
            function.id
        )
    })?
    else {
        return Err(format!(
            "Macro arguments for function {} are not a compound tag",
            function.id
        ));
    };
    let mut parameter_values = Vec::with_capacity(function.macro_parameters.len());
    for parameter in &function.macro_parameters {
        let Some((_, tag)) = values.iter().find(|(name, _)| name == parameter) else {
            return Err(format!(
                "Missing macro argument {parameter} for function {}",
                function.id
            ));
        };
        parameter_values.push(stringify_macro_argument(tag));
    }
    let mut commands = Vec::with_capacity(function.commands.len());
    for command in &function.commands {
        if command.contains("$(") {
            commands.push(substitute_function_macro_command(
                &function.id,
                command,
                &function.macro_parameters,
                &parameter_values,
            )?);
        } else {
            commands.push(command.clone());
        }
    }
    Ok(InstantiatedCommandFunction { commands })
}

pub(super) fn stringify_macro_argument(tag: &Tag) -> String {
    match tag {
        Tag::Byte(value) => value.to_string(),
        Tag::Short(value) => value.to_string(),
        Tag::Int(value) => value.to_string(),
        Tag::Long(value) => value.to_string(),
        Tag::Float(value) => format_macro_decimal(f64::from(*value)),
        Tag::Double(value) => format_macro_decimal(*value),
        Tag::String(value) => value.clone(),
        other => other.to_snbt(),
    }
}

pub(super) fn format_macro_decimal(value: f64) -> String {
    let mut formatted = format!("{value:.15}");
    while formatted.contains('.') && formatted.ends_with('0') {
        formatted.pop();
    }
    if formatted.ends_with('.') {
        formatted.pop();
    }
    if formatted == "-0" {
        "0".to_string()
    } else {
        formatted
    }
}

pub(super) fn substitute_function_macro_command(
    id: &str,
    command: &str,
    parameters: &[String],
    parameter_values: &[String],
) -> Result<String, String> {
    let mut substituted = String::new();
    let mut start = 0usize;
    while let Some(relative_index) = command[start..].find('$') {
        let index = start + relative_index;
        if command.as_bytes().get(index + 1) != Some(&b'(') {
            start = index + 1;
            continue;
        }
        substituted.push_str(&command[start..index]);
        let Some(relative_end) = command[index + 2..].find(')') else {
            return Err(format!("Unterminated macro variable in function {id}"));
        };
        let end = index + 2 + relative_end;
        let variable = &command[index + 2..end];
        let Some(parameter_index) = parameters
            .iter()
            .position(|parameter| parameter == variable)
        else {
            return Err(format!(
                "Unknown macro variable {variable} in function {id}"
            ));
        };
        substituted.push_str(&parameter_values[parameter_index]);
        check_function_line_length(id, &substituted)?;
        start = end + 1;
    }
    substituted.push_str(&command[start..]);
    check_function_line_length(id, &substituted)?;
    Ok(substituted)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringTemplateModel {
    pub segments: Vec<String>,
    pub variables: Vec<String>,
}

impl StringTemplateModel {
    pub fn from_string(input: &str) -> Result<Self, String> {
        let mut segments = Vec::new();
        let mut variables = Vec::new();
        let mut start = 0usize;
        let mut index = input.find('$');
        while let Some(position) = index {
            if input.as_bytes().get(position + 1) == Some(&b'(') {
                segments.push(input[start..position].to_string());
                let Some(relative_end) = input[position + 2..].find(')') else {
                    return Err("Unterminated macro variable".to_string());
                };
                let end = position + 2 + relative_end;
                let variable = &input[position + 2..end];
                if !Self::is_valid_variable_name(variable) {
                    return Err(format!("Invalid macro variable name '{variable}'"));
                }
                variables.push(variable.to_string());
                start = end + 1;
                index = input[start..].find('$').map(|offset| start + offset);
            } else {
                index = input[position + 1..]
                    .find('$')
                    .map(|offset| position + 1 + offset);
            }
        }
        if start == 0 {
            return Err("No variables in macro".to_string());
        }
        if start != input.len() {
            segments.push(input[start..].to_string());
        }
        Ok(Self {
            segments,
            variables,
        })
    }

    pub fn is_valid_variable_name(variable: &str) -> bool {
        variable
            .chars()
            .all(|character| character.is_alphanumeric() || character == '_')
    }

    pub fn substitute(&self, arguments: &[String]) -> Result<String, String> {
        if arguments.len() < self.variables.len() {
            return Err("Missing template argument".to_string());
        }
        let mut result = String::new();
        for (index, argument) in arguments.iter().enumerate().take(self.variables.len()) {
            result.push_str(&self.segments[index]);
            result.push_str(argument);
            check_function_line_length("template", &result)?;
        }
        if self.segments.len() > self.variables.len() {
            if let Some(segment) = self.segments.last() {
                result.push_str(segment);
            }
        }
        check_function_line_length("template", &result)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionEntryModel {
    Plain(String),
    Macro {
        template: StringTemplateModel,
        parameter_indices: Vec<usize>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FunctionBuilderModel {
    plain_entries: Option<Vec<String>>,
    macro_entries: Option<Vec<FunctionEntryModel>>,
    macro_arguments: Vec<String>,
}

impl FunctionBuilderModel {
    pub fn new() -> Self {
        Self {
            plain_entries: Some(Vec::new()),
            macro_entries: None,
            macro_arguments: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: impl Into<String>) {
        let command = command.into();
        if let Some(macro_entries) = &mut self.macro_entries {
            macro_entries.push(FunctionEntryModel::Plain(command));
        } else if let Some(plain_entries) = &mut self.plain_entries {
            plain_entries.push(command);
        }
    }

    pub fn add_macro(&mut self, command: &str, line: usize) -> Result<(), String> {
        let template = StringTemplateModel::from_string(command)
            .map_err(|err| format!("Can't parse function line {line}: '{command}': {err}"))?;
        if let Some(plain_entries) = self.plain_entries.take() {
            self.macro_entries = Some(
                plain_entries
                    .into_iter()
                    .map(FunctionEntryModel::Plain)
                    .collect(),
            );
        }
        let parameter_indices = self.convert_to_indices(&template.variables);
        if let Some(macro_entries) = &mut self.macro_entries {
            macro_entries.push(FunctionEntryModel::Macro {
                template,
                parameter_indices,
            });
        }
        Ok(())
    }

    pub fn build(self, id: impl Into<String>) -> CommandFunctionModel {
        let id = id.into();
        if let Some(entries) = self.macro_entries {
            CommandFunctionModel::Macro(MacroFunctionModel {
                id,
                entries,
                parameters: self.macro_arguments,
                cache: MacroFunctionCacheModel::default(),
            })
        } else {
            CommandFunctionModel::Plain(PlainTextFunctionModel {
                id,
                entries: self.plain_entries.unwrap_or_default(),
            })
        }
    }

    fn convert_to_indices(&mut self, ids: &[String]) -> Vec<usize> {
        ids.iter().map(|id| self.get_argument_index(id)).collect()
    }

    fn get_argument_index(&mut self, id: &str) -> usize {
        if let Some(index) = self
            .macro_arguments
            .iter()
            .position(|argument| argument == id)
        {
            index
        } else {
            let index = self.macro_arguments.len();
            self.macro_arguments.push(id.to_string());
            index
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandFunctionModel {
    Plain(PlainTextFunctionModel),
    Macro(MacroFunctionModel),
}

impl CommandFunctionModel {
    pub fn id(&self) -> &str {
        match self {
            Self::Plain(function) => &function.id,
            Self::Macro(function) => &function.id,
        }
    }

    pub fn instantiate(
        &mut self,
        arguments: Option<&str>,
    ) -> Result<InstantiatedFunctionModel, String> {
        match self {
            Self::Plain(function) => Ok(function.instantiate()),
            Self::Macro(function) => function.instantiate(arguments),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlainTextFunctionModel {
    pub id: String,
    pub entries: Vec<String>,
}

impl PlainTextFunctionModel {
    pub fn instantiate(&self) -> InstantiatedFunctionModel {
        InstantiatedFunctionModel {
            id: self.id.clone(),
            entries: self.entries.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroFunctionModel {
    pub id: String,
    pub entries: Vec<FunctionEntryModel>,
    pub parameters: Vec<String>,
    pub cache: MacroFunctionCacheModel,
}

impl MacroFunctionModel {
    pub fn instantiate(
        &mut self,
        arguments: Option<&str>,
    ) -> Result<InstantiatedFunctionModel, String> {
        let values = parse_macro_arguments(&self.id, &self.parameters, arguments)?;
        if let Some(cached) = self.cache.get_and_move_to_last(&values) {
            return Ok(cached);
        }
        if self.cache.len() >= MacroFunctionCacheModel::MAX_CACHE_ENTRIES {
            self.cache.remove_first();
        }
        let instantiated = self.substitute_and_parse(&values)?;
        self.cache.put(values, instantiated.clone());
        Ok(instantiated)
    }

    pub fn cached_keys(&self) -> Vec<Vec<String>> {
        self.cache.cached_keys()
    }

    fn substitute_and_parse(&self, values: &[String]) -> Result<InstantiatedFunctionModel, String> {
        let mut entries = Vec::with_capacity(self.entries.len());
        for entry in &self.entries {
            match entry {
                FunctionEntryModel::Plain(command) => entries.push(command.clone()),
                FunctionEntryModel::Macro {
                    template,
                    parameter_indices,
                } => {
                    let substitutions = parameter_indices
                        .iter()
                        .map(|index| values[*index].clone())
                        .collect::<Vec<_>>();
                    entries.push(template.substitute(&substitutions)?);
                }
            }
        }
        Ok(InstantiatedFunctionModel {
            id: instantiated_function_id(&self.id, &self.parameters),
            entries,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantiatedFunctionModel {
    pub id: String,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MacroFunctionCacheModel {
    entries: VecDeque<(Vec<String>, InstantiatedFunctionModel)>,
}

impl MacroFunctionCacheModel {
    const MAX_CACHE_ENTRIES: usize = 8;

    fn get_and_move_to_last(&mut self, values: &[String]) -> Option<InstantiatedFunctionModel> {
        let index = self.entries.iter().position(|(key, _)| key == values)?;
        let (key, value) = self.entries.remove(index)?;
        self.entries.push_back((key, value.clone()));
        Some(value)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn remove_first(&mut self) {
        self.entries.pop_front();
    }

    fn put(&mut self, values: Vec<String>, function: InstantiatedFunctionModel) {
        self.entries.push_back((values, function));
    }

    pub fn cached_keys(&self) -> Vec<Vec<String>> {
        self.entries.iter().map(|(key, _)| key.clone()).collect()
    }
}

pub fn instantiated_function_id(base_id: &str, parameter_names: &[String]) -> String {
    format!("{base_id}/{}", java_string_list_hash(parameter_names))
}

fn macro_parameter_values(
    function: &CommandFunctionDefinition,
    arguments: Option<&str>,
) -> Result<Vec<String>, String> {
    parse_macro_arguments(&function.id, &function.macro_parameters, arguments)
}

fn parse_macro_arguments(
    function_id: &str,
    parameters: &[String],
    arguments: Option<&str>,
) -> Result<Vec<String>, String> {
    if parameters.is_empty() {
        return Ok(Vec::new());
    }
    let arguments =
        arguments.ok_or_else(|| format!("Missing macro arguments for function {function_id}"))?;
    let Tag::Compound(values) = parse_snbt(arguments)
        .map_err(|err| format!("Invalid macro arguments for function {function_id}: {err}"))?
    else {
        return Err(format!(
            "Macro arguments for function {function_id} are not a compound tag"
        ));
    };
    parameters
        .iter()
        .map(|parameter| {
            let Some((_, tag)) = values.iter().find(|(name, _)| name == parameter) else {
                return Err(format!(
                    "Missing macro argument {parameter} for function {function_id}"
                ));
            };
            Ok(stringify_macro_argument(tag))
        })
        .collect()
}

fn java_string_list_hash(values: &[String]) -> i32 {
    values.iter().fold(1i32, |hash, value| {
        hash.wrapping_mul(31).wrapping_add(java_string_hash(value))
    })
}

fn java_string_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
    })
}

pub(super) fn should_concatenate_next_function_line(line: &str) -> bool {
    line.ends_with('\\')
}

pub(super) fn check_function_line_length(id: &str, line: &str) -> Result<(), String> {
    if line.len() > MAX_COMMAND_FUNCTION_LINE_LENGTH {
        Err(format!(
            "Command too long in function {id}: {} characters",
            line.len()
        ))
    } else {
        Ok(())
    }
}

pub(super) fn parse_function_macro_variables(
    id: &str,
    line_number: usize,
    line: &str,
) -> Result<Vec<String>, String> {
    let mut variables = Vec::new();
    let mut start = 0usize;
    while let Some(relative_index) = line[start..].find('$') {
        let index = start + relative_index;
        if line.as_bytes().get(index + 1) != Some(&b'(') {
            start = index + 1;
            continue;
        }
        let Some(relative_end) = line[index + 2..].find(')') else {
            return Err(format!(
                "Unterminated macro variable in function {id} line {line_number}"
            ));
        };
        let end = index + 2 + relative_end;
        let variable = &line[index + 2..end];
        if variable.is_empty()
            || variable
                .bytes()
                .any(|byte| !byte.is_ascii_alphanumeric() && byte != b'_')
        {
            return Err(format!(
                "Invalid macro variable {variable:?} in function {id} line {line_number}"
            ));
        }
        variables.push(variable.to_string());
        start = end + 1;
    }
    if variables.is_empty() {
        return Err(format!(
            "Macro function line in {id} line {line_number} has no variables"
        ));
    }
    Ok(variables)
}

pub(super) fn function_id_from_path(
    path: &str,
    suffix: &str,
    directory: &str,
) -> Result<Option<String>, String> {
    let Some(rest) = path.strip_prefix("data/") else {
        return Ok(None);
    };
    let Some((namespace, rest)) = rest.split_once('/') else {
        return Ok(None);
    };
    let Some(rest) = rest
        .strip_prefix(directory)
        .and_then(|rest| rest.strip_prefix('/'))
    else {
        return Ok(None);
    };
    let Some(path_id) = rest.strip_suffix(suffix) else {
        return Ok(None);
    };
    if path_id.is_empty() {
        return Err(format!("Empty function id in path {path}"));
    }
    let id = parse_resource_identifier(&format!("{namespace}:{path_id}"))
        .map_err(|_| format!("Invalid function resource path {path}"))?;
    Ok(Some(id))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedFunctionCall {
    pub id: String,
    pub commands: Vec<String>,
    pub arguments: Option<String>,
    pub source_dimension: String,
    pub suppressed_output: bool,
    pub permission_level: PermissionLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerFunctionTickState {
    pub post_reload: bool,
    pub tick_tag: String,
    pub load_tag: String,
}

impl Default for ServerFunctionTickState {
    fn default() -> Self {
        Self {
            post_reload: true,
            tick_tag: "minecraft:tick".to_string(),
            load_tag: "minecraft:load".to_string(),
        }
    }
}

pub fn queue_server_function_tick(
    command_state: &mut ServerCommandState,
    function_state: &mut ServerFunctionTickState,
    runs_normally: bool,
) -> Vec<String> {
    if !runs_normally {
        return Vec::new();
    }
    let mut queued = Vec::new();
    if function_state.post_reload {
        function_state.post_reload = false;
        queued.extend(queue_function_tag_for_game_loop(
            command_state,
            &function_state.load_tag,
        ));
    }
    queued.extend(queue_function_tag_for_game_loop(
        command_state,
        &function_state.tick_tag,
    ));
    queued
}

pub fn queue_function_tag_for_game_loop(
    state: &mut ServerCommandState,
    tag_id: &str,
) -> Vec<String> {
    let Some(tag) = state.function_tags.iter().find(|tag| tag.id == tag_id) else {
        return Vec::new();
    };
    let function_ids = tag.functions.clone();
    let mut queued = Vec::new();
    for id in function_ids {
        if id.starts_with('#') {
            continue;
        }
        if let Some(function) = state
            .available_functions
            .iter()
            .find(|function| function.id == id)
            .cloned()
        {
            state.queued_functions.push(QueuedFunctionCall {
                id: function.id.clone(),
                commands: function.commands,
                arguments: None,
                source_dimension: state.command_source_dimension.clone(),
                suppressed_output: true,
                permission_level: PermissionLevel::Gamemasters,
            });
            queued.push(function.id);
        }
    }
    queued
}
