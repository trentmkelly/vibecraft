use super::*;

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
