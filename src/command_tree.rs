#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandNodeKind {
    Root,
    Literal(&'static str),
    Argument {
        name: &'static str,
        parser: ArgumentParser,
        signed: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgumentParser {
    Word,
    GreedyString,
    Integer,
    EntitySelector,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandNode {
    pub id: usize,
    pub kind: CommandNodeKind,
    pub requirement_level: u8,
    pub executable: bool,
    pub redirect: Option<usize>,
    pub fork: bool,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandTree {
    pub nodes: Vec<CommandNode>,
    pub root: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseResult {
    pub path: Vec<usize>,
    pub arguments: Vec<ParsedArgument>,
    pub executable: bool,
    pub redirected_to: Option<usize>,
    pub fork: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedArgument {
    pub name: &'static str,
    pub value: String,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    UnknownLiteral(String),
    PermissionDenied,
    InvalidArgument(&'static str),
    Incomplete,
}

impl CommandTree {
    pub fn new() -> Self {
        Self {
            nodes: vec![CommandNode {
                id: 0,
                kind: CommandNodeKind::Root,
                requirement_level: 0,
                executable: false,
                redirect: None,
                fork: false,
                children: Vec::new(),
            }],
            root: 0,
        }
    }

    pub fn add_child(
        &mut self,
        parent: usize,
        kind: CommandNodeKind,
        requirement_level: u8,
        executable: bool,
    ) -> usize {
        let id = self.nodes.len();
        self.nodes.push(CommandNode {
            id,
            kind,
            requirement_level,
            executable,
            redirect: None,
            fork: false,
            children: Vec::new(),
        });
        self.nodes[parent].children.push(id);
        id
    }

    pub fn set_redirect(&mut self, node: usize, target: usize, fork: bool) {
        self.nodes[node].redirect = Some(target);
        self.nodes[node].fork = fork;
    }

    pub fn parse(&self, input: &str, permission_level: u8) -> Result<ParseResult, ParseError> {
        let mut parts = input.split_whitespace().peekable();
        let first = parts.next().ok_or(ParseError::Empty)?;
        let mut current = self.root;
        let mut path = vec![current];
        let mut arguments = Vec::new();
        let mut token = first;

        loop {
            let child = self.nodes[current]
                .children
                .iter()
                .copied()
                .find(|child| self.node_accepts(*child, token));
            let child = match child {
                Some(child) => child,
                None => {
                    return if current == self.root {
                        Err(ParseError::UnknownLiteral(token.to_string()))
                    } else {
                        Err(ParseError::InvalidArgument(
                            self.argument_name_for_children(current)
                                .unwrap_or("argument"),
                        ))
                    };
                }
            };
            let node = &self.nodes[child];
            if permission_level < node.requirement_level {
                return Err(ParseError::PermissionDenied);
            }
            if let CommandNodeKind::Argument {
                name,
                parser,
                signed,
            } = node.kind
            {
                let value = if parser == ArgumentParser::GreedyString {
                    std::iter::once(token)
                        .chain(parts.by_ref())
                        .collect::<Vec<_>>()
                        .join(" ")
                } else {
                    token.to_string()
                };
                if !argument_matches(parser, &value) {
                    return Err(ParseError::InvalidArgument(name));
                }
                arguments.push(ParsedArgument {
                    name,
                    value,
                    signed,
                });
                current = child;
                path.push(current);
                break_if_greedy(parser);
                if parser == ArgumentParser::GreedyString {
                    break;
                }
            } else {
                current = child;
                path.push(current);
            }

            match parts.next() {
                Some(next) => token = next,
                None => break,
            }
        }

        let node = &self.nodes[current];
        Ok(ParseResult {
            path,
            arguments,
            executable: node.executable,
            redirected_to: node.redirect,
            fork: node.fork,
        })
        .and_then(|result| {
            if result.executable || result.redirected_to.is_some() {
                Ok(result)
            } else {
                Err(ParseError::Incomplete)
            }
        })
    }

    pub fn suggestions(&self, input: &str, permission_level: u8) -> Vec<String> {
        let trailing_space = input.ends_with(' ');
        let mut tokens: Vec<&str> = input.split_whitespace().collect();
        let prefix = if trailing_space {
            ""
        } else {
            tokens.pop().unwrap_or("")
        };
        let mut current = self.root;
        for token in tokens {
            let Some(child) = self.nodes[current]
                .children
                .iter()
                .copied()
                .find(|child| self.node_accepts(*child, token))
            else {
                return Vec::new();
            };
            if permission_level < self.nodes[child].requirement_level {
                return Vec::new();
            }
            current = child;
        }
        self.nodes[current]
            .children
            .iter()
            .filter(|child| permission_level >= self.nodes[**child].requirement_level)
            .filter_map(|child| match self.nodes[*child].kind {
                CommandNodeKind::Literal(name) if name.starts_with(prefix) => {
                    Some(name.to_string())
                }
                CommandNodeKind::Argument { name, .. } if prefix.is_empty() => {
                    Some(format!("<{name}>"))
                }
                _ => None,
            })
            .collect()
    }

    pub fn signed_arguments(&self, parse: &ParseResult) -> Vec<ParsedArgument> {
        parse
            .arguments
            .iter()
            .filter(|argument| argument.signed)
            .cloned()
            .collect()
    }

    fn node_accepts(&self, node: usize, token: &str) -> bool {
        match self.nodes[node].kind {
            CommandNodeKind::Root => false,
            CommandNodeKind::Literal(name) => name == token,
            CommandNodeKind::Argument { parser, .. } => argument_matches(parser, token),
        }
    }

    fn argument_name_for_children(&self, node: usize) -> Option<&'static str> {
        self.nodes[node].children.iter().find_map(|child| {
            if let CommandNodeKind::Argument { name, .. } = self.nodes[*child].kind {
                Some(name)
            } else {
                None
            }
        })
    }
}

pub fn vanilla_like_tree() -> CommandTree {
    let mut tree = CommandTree::new();
    let say = tree.add_child(tree.root, CommandNodeKind::Literal("say"), 2, false);
    tree.add_child(
        say,
        CommandNodeKind::Argument {
            name: "message",
            parser: ArgumentParser::GreedyString,
            signed: true,
        },
        2,
        true,
    );
    let execute = tree.add_child(tree.root, CommandNodeKind::Literal("execute"), 2, false);
    let as_node = tree.add_child(execute, CommandNodeKind::Literal("as"), 2, false);
    let targets = tree.add_child(
        as_node,
        CommandNodeKind::Argument {
            name: "targets",
            parser: ArgumentParser::EntitySelector,
            signed: false,
        },
        2,
        false,
    );
    let run = tree.add_child(targets, CommandNodeKind::Literal("run"), 2, false);
    tree.set_redirect(run, tree.root, true);
    let gamemode = tree.add_child(tree.root, CommandNodeKind::Literal("gamemode"), 2, false);
    tree.add_child(
        gamemode,
        CommandNodeKind::Argument {
            name: "mode",
            parser: ArgumentParser::Word,
            signed: false,
        },
        2,
        true,
    );
    let setidletimeout = tree.add_child(
        tree.root,
        CommandNodeKind::Literal("setidletimeout"),
        3,
        false,
    );
    tree.add_child(
        setidletimeout,
        CommandNodeKind::Argument {
            name: "minutes",
            parser: ArgumentParser::Integer,
            signed: false,
        },
        3,
        true,
    );
    tree
}

fn argument_matches(parser: ArgumentParser, value: &str) -> bool {
    match parser {
        ArgumentParser::Word => !value.is_empty() && !value.chars().any(char::is_whitespace),
        ArgumentParser::GreedyString => !value.is_empty(),
        ArgumentParser::Integer => value.parse::<i32>().is_ok(),
        ArgumentParser::EntitySelector => {
            value.starts_with('@') || value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
    }
}

fn break_if_greedy(_parser: ArgumentParser) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_tree_parses_literals_arguments_and_signed_metadata() {
        let tree = vanilla_like_tree();
        let parse = tree.parse("say hello signed world", 2).unwrap();
        assert!(parse.executable);
        assert_eq!(
            parse.arguments,
            vec![ParsedArgument {
                name: "message",
                value: "hello signed world".to_string(),
                signed: true,
            }]
        );
        assert_eq!(tree.signed_arguments(&parse), parse.arguments);
    }

    #[test]
    fn suggestions_filter_by_prefix_and_permission_level() {
        let tree = vanilla_like_tree();
        assert_eq!(tree.suggestions("ga", 2), vec!["gamemode".to_string()]);
        assert_eq!(tree.suggestions("gamemode ", 2), vec!["<mode>".to_string()]);
        assert!(!tree
            .suggestions("", 2)
            .contains(&"setidletimeout".to_string()));
        assert!(tree
            .suggestions("set", 3)
            .contains(&"setidletimeout".to_string()));
        assert!(tree.suggestions("set", 2).is_empty());
    }

    #[test]
    fn redirects_and_forks_represent_execute_run_semantics() {
        let tree = vanilla_like_tree();
        let parse = tree.parse("execute as @a run", 2).unwrap();
        assert_eq!(parse.redirected_to, Some(tree.root));
        assert!(parse.fork);
        assert!(!parse.executable);
    }

    #[test]
    fn parse_errors_distinguish_unknown_permission_incomplete_and_bad_argument() {
        let tree = vanilla_like_tree();
        assert_eq!(
            tree.parse("unknown", 4),
            Err(ParseError::UnknownLiteral("unknown".to_string()))
        );
        assert_eq!(
            tree.parse("setidletimeout 10", 2),
            Err(ParseError::PermissionDenied)
        );
        assert_eq!(tree.parse("gamemode", 2), Err(ParseError::Incomplete));
        assert_eq!(
            tree.parse("setidletimeout nope", 3),
            Err(ParseError::InvalidArgument("minutes"))
        );
    }
}
