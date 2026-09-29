//! Brigadier-compatible command graph.
//!
//! Mirrors the parts of `com.mojang.brigadier.tree.CommandNode` that vanilla
//! synchronises with clients (`ClientboundCommandsPacket`) and consults when
//! answering `ServerboundCommandSuggestionPacket`:
//!
//! * literal / argument / root nodes, children in registration order
//!   (`CommandNode.children` is a `LinkedHashMap`);
//! * `executable` (`node.getCommand() != null`) and `fork` flags;
//! * redirects;
//! * the permission requirement, reduced to the `PermissionLevel` a source
//!   needs (`Commands.LEVEL_*`); every requirement in the 26.1.2 command tree is
//!   a `PermissionCheck.Require(command_level)`;
//! * the argument parser id plus the properties `ArgumentTypeInfo` serialises;
//! * the custom `SuggestionProvider` identifier (`ArgumentCommandNode.getCustomSuggestions`).
//!
//! The vanilla tree is not hand-written: it is the exact dispatcher of the
//! 26.1.2 dedicated server, captured node-for-node by walking the real
//! `Commands` dispatcher (see `vanilla-data/reports/commands_graph_26_1_2.json`).
//! Unlike the `--reports` `commands.json` (a path-expanded tree that omits the
//! `run` redirect to the root, shared node identity and suggestion ids), the
//! graph keeps node identity plus the `ArgumentType`/`Command` identities that
//! `CommandNode.equals` compares, so packet node ids match vanilla byte for
//! byte (`ClientboundCommandsPacket` deduplicates equal nodes).
//!
//! Child order is registration order and, for registry driven nodes such as the
//! `gamerule` rules, depends on Java's registry iteration; it is therefore
//! not a stable specification, only what the captured dispatcher produced.
//!
//! * [`wire`] builds `ClientboundCommandsPacket` (Java `Commands.sendCommands`).
//! * [`suggest`] answers `CommandDispatcher.getCompletionSuggestions`.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

pub mod suggest;
pub mod sync;
pub mod wire;

#[cfg(test)]
mod tests;

/// Index of the root node in every [`CommandGraph`] (`RootCommandNode`).
pub const ROOT: usize = 0;

/// The vendored dispatcher graph of the official 26.1.2 dedicated server.
const VANILLA_GRAPH_JSON: &str =
    include_str!("../../vanilla-data/reports/commands_graph_26_1_2.json");

/// What kind of Brigadier node this is.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    /// `RootCommandNode`.
    Root,
    /// `LiteralCommandNode`.
    Literal { name: String },
    /// `ArgumentCommandNode`.
    Argument {
        name: String,
        /// Registry id of the `ArgumentTypeInfo` (`brigadier:integer`, `minecraft:entity`, ...).
        parser: String,
        /// `ArgumentTypeInfo.serializeToJson` properties (`Value::Null` when empty).
        properties: Value,
        /// `SuggestionProviders.getName` of the custom provider, when one is set.
        suggestions: Option<String>,
        /// Identity of the Brigadier `ArgumentType` under `equals`: two argument
        /// nodes may only be equal when their keys match. Types without an
        /// `equals` override never share a key, exactly like Java.
        type_key: u32,
    },
}

impl NodeKind {
    /// `CommandNode.getName`.
    pub fn name(&self) -> &str {
        match self {
            Self::Root => "",
            Self::Literal { name } | Self::Argument { name, .. } => name,
        }
    }

    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Literal { .. })
    }
}

/// One Brigadier command node.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandGraphNode {
    pub kind: NodeKind,
    /// `node.getCommand() != null`.
    pub executable: bool,
    /// Identity of the `Command` object. `CommandNode.equals` compares commands
    /// by `equals` (identity for lambdas), so nodes with different keys differ.
    pub command_key: Option<u32>,
    /// `node.isFork()`; not serialised by the packet but kept by `createBuilder`.
    pub fork: bool,
    /// Lowest `PermissionLevel` (0 = all .. 4 = owners) whose source passes `node.getRequirement()`.
    pub permission_level: u8,
    /// `node.getRedirect()`.
    pub redirect: Option<usize>,
    /// `node.getChildren()` in Brigadier order.
    pub children: Vec<usize>,
}

/// A command dispatcher tree with stable node identity.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandGraph {
    nodes: Vec<CommandGraphNode>,
}

impl CommandGraph {
    /// A graph containing only the root node.
    pub fn new() -> Self {
        Self {
            nodes: vec![CommandGraphNode {
                kind: NodeKind::Root,
                executable: false,
                command_key: None,
                fork: false,
                permission_level: 0,
                redirect: None,
                children: Vec::new(),
            }],
        }
    }

    /// The unmodified official 26.1.2 dedicated-server command graph.
    pub fn vanilla() -> &'static CommandGraph {
        static VANILLA: OnceLock<CommandGraph> = OnceLock::new();
        VANILLA.get_or_init(|| {
            // The document is embedded at compile time and validated by the
            // `vanilla_graph_*` tests; a parse failure can only mean a corrupted
            // build, in which case serving an empty tree beats aborting the server.
            Self::from_json(VANILLA_GRAPH_JSON).unwrap_or_else(|error| {
                eprintln!("embedded vanilla command graph is invalid: {error}");
                Self::new()
            })
        })
    }

    /// The graph VibeCraft actually serves: [`Self::vanilla`] plus VibeCraft's
    /// documented debug extension `/biome`.
    ///
    /// Intentional Java parity divergence: vanilla 26.1.2 has no root `/biome`
    /// command, but exposing it keeps the in-game biome debugging helper usable
    /// from the slash-command UI.
    pub fn served() -> &'static CommandGraph {
        static SERVED: OnceLock<CommandGraph> = OnceLock::new();
        SERVED.get_or_init(|| {
            let mut graph = Self::vanilla().clone();
            let biome = graph.push(CommandGraphNode {
                kind: NodeKind::Literal {
                    name: "biome".into(),
                },
                executable: true,
                command_key: Some(u32::MAX),
                fork: false,
                permission_level: 0,
                redirect: None,
                children: Vec::new(),
            });
            graph.add_child(ROOT, biome);
            graph
        })
    }

    /// Parses the graph document written by the vanilla dump tool.
    pub fn from_json(text: &str) -> Result<Self, String> {
        let document: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let raw = document["nodes"].as_array().ok_or("missing nodes array")?;
        let mut nodes = Vec::with_capacity(raw.len());
        for entry in raw {
            nodes.push(Self::node_from_json(entry)?);
        }
        let count = nodes.len();
        let dangling = |id: usize| id >= count;
        if nodes
            .iter()
            .any(|n| n.children.iter().any(|c| dangling(*c)) || n.redirect.is_some_and(dangling))
        {
            return Err("node reference out of range".into());
        }
        if !matches!(nodes.first().map(|n| &n.kind), Some(NodeKind::Root)) {
            return Err("node 0 must be the root".into());
        }
        Ok(Self { nodes })
    }

    fn node_from_json(entry: &Value) -> Result<CommandGraphNode, String> {
        let text = |key: &str| entry[key].as_str().map(str::to_owned);
        let kind = match entry["kind"].as_str() {
            Some("root") => NodeKind::Root,
            Some("literal") => NodeKind::Literal {
                name: text("name").ok_or("literal without name")?,
            },
            Some("argument") => NodeKind::Argument {
                name: text("name").ok_or("argument without name")?,
                parser: text("parser").ok_or("argument without parser")?,
                properties: entry.get("properties").cloned().unwrap_or(Value::Null),
                suggestions: text("suggestions"),
                type_key: entry["type_key"]
                    .as_u64()
                    .ok_or("argument without type_key")? as u32,
            },
            other => return Err(format!("unknown node kind {other:?}")),
        };
        let index = |v: &Value| v.as_u64().map(|i| i as usize).ok_or("bad node index");
        Ok(CommandGraphNode {
            kind,
            executable: entry["executable"].as_bool().unwrap_or(false),
            command_key: entry["command_key"].as_u64().map(|k| k as u32),
            fork: entry["fork"].as_bool().unwrap_or(false),
            permission_level: entry["permission_level"].as_u64().unwrap_or(0) as u8,
            redirect: match entry.get("redirect") {
                Some(v) => Some(index(v)?),
                None => None,
            },
            children: entry["children"]
                .as_array()
                .map(|a| a.iter().map(index).collect::<Result<Vec<_>, _>>())
                .transpose()?
                .unwrap_or_default(),
        })
    }

    pub fn node(&self, id: usize) -> &CommandGraphNode {
        &self.nodes[id]
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Appends a detached node and returns its id.
    pub fn push(&mut self, node: CommandGraphNode) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// `CommandNode.addChild`: attaches `child`; Brigadier 1.3 keeps children in
    /// insertion order (`LinkedHashMap`), so the new node goes last.
    pub fn add_child(&mut self, parent: usize, child: usize) {
        self.nodes[parent].children.push(child);
    }

    /// Java `Commands.fillUsableCommands` (called from `Commands.sendCommands`):
    /// copies every node whose requirement the source satisfies, depth first,
    /// re-pointing redirects at the copies made so far. A redirect whose target
    /// has not been copied yet (or is not usable) becomes `null`, exactly as
    /// `builder.redirect(converted.get(target))` does. Shared nodes reached
    /// through several parents are copied once per parent.
    pub fn filter_for_permission_level(&self, level: u8) -> CommandGraph {
        let mut out = CommandGraph::new();
        let mut converted = HashMap::from([(ROOT, ROOT)]);
        self.fill_usable_commands(ROOT, ROOT, level, &mut converted, &mut out);
        out
    }

    fn fill_usable_commands(
        &self,
        source: usize,
        target: usize,
        level: u8,
        converted: &mut HashMap<usize, usize>,
        out: &mut CommandGraph,
    ) {
        for &child in &self.nodes[source].children {
            let original = &self.nodes[child];
            if original.permission_level > level {
                continue; // `child.canUse(source)` failed
            }
            let mut copy = original.clone();
            copy.children.clear();
            copy.redirect = original.redirect.and_then(|r| converted.get(&r).copied());
            let copy_id = out.push(copy);
            converted.insert(child, copy_id);
            out.add_child(target, copy_id);
            if !original.children.is_empty() {
                self.fill_usable_commands(child, copy_id, level, converted, out);
            }
        }
    }
}

impl Default for CommandGraph {
    fn default() -> Self {
        Self::new()
    }
}
