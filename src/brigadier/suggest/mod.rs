//! Server-side command suggestions: Brigadier's
//! `CommandDispatcher.parse` + `getCompletionSuggestions`, as driven by
//! `ServerGamePacketListenerImpl.handleCustomCommandSuggestions`.
//!
//! All ranges are UTF-16 code-unit indices, like Java's `String`, so the values
//! written to `ClientboundCommandSuggestionsPacket` are directly comparable.
//!
//! Argument handling status (see [`argument`]): the Brigadier primitives are
//! ported exactly; Minecraft argument types are consumed with a token heuristic
//! and only `bool`, `gamemode` and `entity_anchor` list suggestions, since the
//! others need live game data (players, registries, scoreboards).

pub mod argument;

use std::collections::HashSet;

use super::{CommandGraph, NodeKind, ROOT};
use argument::StringReader;

/// Suggestions truncate to this many entries in
/// `handleCustomCommandSuggestions`.
pub const MAX_SUGGESTIONS: usize = 1000;

/// Brigadier `Suggestions`: a shared replacement range plus the sorted texts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Suggestions {
    pub start: usize,
    pub end: usize,
    pub texts: Vec<String>,
}

/// One `Suggestion` before merging: its own range and text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Suggestion {
    start: usize,
    end: usize,
    text: String,
}

/// `SuggestionsBuilder`.
pub(crate) struct SuggestionsBuilder<'a> {
    input: &'a [u16],
    start: usize,
    result: Vec<Suggestion>,
}

impl<'a> SuggestionsBuilder<'a> {
    fn new(input: &'a [u16], start: usize) -> Self {
        Self {
            input,
            start,
            result: Vec::new(),
        }
    }

    /// `getRemaining`.
    pub(crate) fn remaining(&self) -> String {
        String::from_utf16_lossy(&self.input[self.start..])
    }

    /// `getRemainingLowerCase`.
    pub(crate) fn remaining_lower_case(&self) -> String {
        self.remaining().to_lowercase()
    }

    /// `suggest(String)`: a suggestion equal to the remaining input is dropped.
    pub(crate) fn suggest(&mut self, text: &str) {
        if text == self.remaining() {
            return;
        }
        self.result.push(Suggestion {
            start: self.start,
            end: self.input.len(),
            text: text.to_owned(),
        });
    }
}

/// `CommandContextBuilder` reduced to what suggestion lookup reads.
#[derive(Clone)]
struct ParseContext {
    root: usize,
    /// `(node, range start, range end)` per `ParsedCommandNode`.
    nodes: Vec<(usize, usize, usize)>,
    start: usize,
    end: usize,
    child: Option<Box<ParseContext>>,
}

impl ParseContext {
    fn new(root: usize, start: usize) -> Self {
        Self {
            root,
            nodes: Vec::new(),
            start,
            end: start,
            child: None,
        }
    }

    /// `withNode`: records the node and widens the context range.
    fn with_node(&mut self, node: usize, start: usize, end: usize) {
        self.nodes.push((node, start, end));
        self.start = self.start.min(start);
        self.end = self.end.max(end);
    }

    /// `findSuggestionContext`: the parent node whose children can complete
    /// `cursor`, and where the completed token starts.
    fn find_suggestion_context(&self, cursor: usize) -> (usize, usize) {
        if self.end < cursor {
            if let Some(child) = &self.child {
                return child.find_suggestion_context(cursor);
            }
            return match self.nodes.last() {
                Some(&(node, _, end)) => (node, end + 1),
                None => (self.root, self.start),
            };
        }
        let mut prev = self.root;
        for &(node, start, end) in &self.nodes {
            if start <= cursor && cursor <= end {
                return (prev, start);
            }
            prev = node;
        }
        (prev, self.start)
    }
}

struct ParseResults {
    context: ParseContext,
    cursor: usize,
    has_exceptions: bool,
}

struct Parser<'a> {
    graph: &'a CommandGraph,
    permission_level: u8,
    input: &'a [u16],
}

impl Parser<'_> {
    /// `CommandDispatcher.parseNodes`.
    fn parse_nodes(
        &self,
        node: usize,
        cursor: usize,
        context_so_far: &ParseContext,
    ) -> ParseResults {
        let mut potentials: Vec<ParseResults> = Vec::new();
        let mut any_error = false;
        for child in self.relevant_nodes(node, cursor) {
            let child_node = self.graph.node(child);
            if child_node.permission_level > self.permission_level {
                continue; // `child.canUse(source)`
            }
            let mut context = context_so_far.clone();
            let mut reader = StringReader::new(self.input, cursor);
            if !self.parse_node(child, &mut reader, &mut context)
                || (reader.can_read(1) && reader.peek() != Some(b' ' as u16))
            {
                any_error = true;
                continue;
            }
            let redirect = child_node.redirect;
            if reader.can_read(if redirect.is_none() { 2 } else { 1 }) {
                reader.skip();
                if let Some(target) = redirect {
                    let child_context = ParseContext::new(target, reader.cursor());
                    let parse = self.parse_nodes(target, reader.cursor(), &child_context);
                    context.child = Some(Box::new(parse.context));
                    return ParseResults {
                        context,
                        cursor: parse.cursor,
                        has_exceptions: parse.has_exceptions,
                    };
                }
                potentials.push(self.parse_nodes(child, reader.cursor(), &context));
            } else {
                potentials.push(ParseResults {
                    context,
                    cursor: reader.cursor(),
                    has_exceptions: false,
                });
            }
        }
        if potentials.is_empty() {
            return ParseResults {
                context: context_so_far.clone(),
                cursor,
                has_exceptions: any_error,
            };
        }
        let total = self.input.len();
        // Stable sort with Brigadier's comparator: fully consumed input first,
        // then results without exceptions.
        potentials.sort_by_key(|p| (p.cursor < total, p.has_exceptions));
        potentials.swap_remove(0)
    }

    /// `CommandNode.getRelevantNodes`.
    fn relevant_nodes(&self, node: usize, cursor: usize) -> Vec<usize> {
        let children = &self.graph.node(node).children;
        let literal_of = |id: &usize| match &self.graph.node(*id).kind {
            NodeKind::Literal { name } => Some(name.as_str()),
            _ => None,
        };
        if children.iter().any(|c| literal_of(c).is_some()) {
            let end = self.input[cursor..]
                .iter()
                .position(|u| *u == b' ' as u16)
                .map_or(self.input.len(), |i| cursor + i);
            let text = String::from_utf16_lossy(&self.input[cursor..end]);
            if let Some(hit) = children
                .iter()
                .find(|c| literal_of(c) == Some(text.as_str()))
            {
                return vec![*hit];
            }
        }
        children
            .iter()
            .copied()
            .filter(|c| !self.graph.node(*c).kind.is_literal())
            .collect()
    }

    /// `CommandNode.parse`; returns `false` on a `CommandSyntaxException`.
    fn parse_node(&self, id: usize, reader: &mut StringReader, context: &mut ParseContext) -> bool {
        let start = reader.cursor();
        match &self.graph.node(id).kind {
            NodeKind::Root => true,
            NodeKind::Literal { name } => {
                if reader.read_literal(name) {
                    context.with_node(id, start, reader.cursor());
                    true
                } else {
                    false
                }
            }
            NodeKind::Argument {
                parser, properties, ..
            } => {
                if argument::parse(parser, properties, reader) {
                    context.with_node(id, start, reader.cursor());
                    true
                } else {
                    reader.set_cursor(start);
                    false
                }
            }
        }
    }
}

/// Brigadier's `getCompletionSuggestions(parse)` for `command` (a leading `/`
/// is skipped first, as `handleCustomCommandSuggestions` does) as seen by a
/// source with `permission_level`.
pub fn command_suggestions(
    graph: &CommandGraph,
    permission_level: u8,
    command: &str,
) -> Suggestions {
    let input: Vec<u16> = command.encode_utf16().collect();
    let first = usize::from(input.first() == Some(&(b'/' as u16)));
    let parser = Parser {
        graph,
        permission_level,
        input: &input,
    };
    let root_context = ParseContext::new(ROOT, first);
    let parse = parser.parse_nodes(ROOT, first, &root_context);

    let cursor = input.len();
    let (parent, start_pos) = parse.context.find_suggestion_context(cursor);
    let start = start_pos.min(cursor);
    let mut all: Vec<Suggestion> = Vec::new();
    for &child in &graph.node(parent).children {
        let mut builder = SuggestionsBuilder::new(&input, start);
        list_node_suggestions(graph, child, &parse.context, &mut builder);
        all.extend(builder.result);
    }
    let mut suggestions = create_suggestions(&input, all);
    suggestions.texts.truncate(MAX_SUGGESTIONS);
    suggestions
}

/// `CommandNode.listSuggestions` for the three node kinds.
fn list_node_suggestions(
    graph: &CommandGraph,
    id: usize,
    _context: &ParseContext,
    builder: &mut SuggestionsBuilder<'_>,
) {
    match &graph.node(id).kind {
        NodeKind::Root => {}
        NodeKind::Literal { name } => {
            if name
                .to_lowercase()
                .starts_with(&builder.remaining_lower_case())
            {
                builder.suggest(name);
            }
        }
        NodeKind::Argument {
            parser,
            properties,
            suggestions,
            ..
        } => match suggestions.as_deref() {
            // `ask_server` resolves to `CommandSourceStack.customSuggestion`
            // (`Suggestions.empty()`); the other providers need server data.
            // TODO(command-suggestion-providers): available_sounds,
            // summonable_entities and the per-command lambdas.
            Some(_) => {}
            None => argument::list_suggestions(parser, properties, builder),
        },
    }
}

/// `Suggestions.create`: merges ranges, expands each suggestion to the merged
/// range, removes duplicates and sorts with `compareToIgnoreCase`.
fn create_suggestions(input: &[u16], suggestions: Vec<Suggestion>) -> Suggestions {
    if suggestions.is_empty() {
        return Suggestions::default();
    }
    let start = suggestions.iter().map(|s| s.start).min().unwrap_or(0);
    let end = suggestions.iter().map(|s| s.end).max().unwrap_or(0);
    let mut seen = HashSet::new();
    let mut expanded: Vec<String> = Vec::new();
    for suggestion in suggestions {
        // `Suggestion.expand`
        let mut text = String::new();
        if start < suggestion.start {
            text.push_str(&String::from_utf16_lossy(&input[start..suggestion.start]));
        }
        text.push_str(&suggestion.text);
        if end > suggestion.end {
            text.push_str(&String::from_utf16_lossy(&input[suggestion.end..end]));
        }
        if seen.insert(text.clone()) {
            expanded.push(text);
        }
    }
    expanded.sort_by(|a, b| compare_to_ignore_case(a, b));
    Suggestions {
        start,
        end,
        texts: expanded,
    }
}

/// `String.compareToIgnoreCase`.
fn compare_to_ignore_case(a: &str, b: &str) -> std::cmp::Ordering {
    fn fold(unit: u16) -> u16 {
        let single = |mut it: std::vec::IntoIter<char>| match (it.next(), it.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        };
        let Some(c) = char::from_u32(u32::from(unit)) else {
            return unit;
        };
        let upper = single(c.to_uppercase().collect::<Vec<_>>().into_iter()).unwrap_or(c);
        let lower = single(upper.to_lowercase().collect::<Vec<_>>().into_iter()).unwrap_or(upper);
        u16::try_from(u32::from(lower)).unwrap_or(unit)
    }
    a.encode_utf16().map(fold).cmp(b.encode_utf16().map(fold))
}
