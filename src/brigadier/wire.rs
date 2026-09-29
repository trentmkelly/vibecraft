//! `ClientboundCommandsPacket` construction (Java `Commands.sendCommands` and the
//! `ClientboundCommandsPacket(RootCommandNode, NodeInspector)` constructor).

use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};

use serde_json::Value;

use super::{CommandGraph, NodeKind, ROOT};
use crate::command_synchronization::{
    argument_type_bootstrap_order, ArgumentInfoKind, BrigadierStringType, NumericArgumentKind,
};
use crate::network::play::{ClientboundCommandsPacket, CommandNodeEntryData, CommandNodeStubData};
use crate::network::varint::write_var_i32;
use crate::registry::Identifier;

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

impl CommandGraph {
    /// Java `Commands.sendCommands`: the per-player packet is built from the tree
    /// filtered to what a source at `permission_level` may use.
    pub fn commands_packet_for_permission_level(
        &self,
        permission_level: u8,
    ) -> io::Result<ClientboundCommandsPacket> {
        self.filter_for_permission_level(permission_level)
            .to_packet()
    }

    /// Java `ClientboundCommandsPacket(root, inspector)`: nodes are numbered in
    /// breadth-first order (children, then the redirect target), the flags come
    /// from `Commands.COMMAND_NODE_INSPECTOR` (`restricted` = the requirement
    /// fails for a source without permissions).
    ///
    /// `enumerateNodes` keys an `Object2IntOpenHashMap` by `CommandNode`, and
    /// `CommandNode.equals` is structural (see [`Self::equality_classes`]), so
    /// equal nodes share one id and only the first one is serialised.
    pub fn to_packet(&self) -> io::Result<ClientboundCommandsPacket> {
        let classes = self.equality_classes();
        let mut class_ids: HashMap<u32, i32> = HashMap::new();
        let mut order = Vec::new();
        let mut queue = VecDeque::from([ROOT]);
        while let Some(node) = queue.pop_front() {
            if class_ids.contains_key(&classes[node]) {
                continue;
            }
            class_ids.insert(classes[node], order.len() as i32);
            order.push(node);
            queue.extend(self.node(node).children.iter().copied());
            queue.extend(self.node(node).redirect);
        }
        let id_of = |node: usize| class_ids[&classes[node]];

        let entries = order
            .iter()
            .map(|&index| {
                let node = self.node(index);
                Ok(CommandNodeEntryData {
                    stub: stub_for(&node.kind)?,
                    executable: node.executable,
                    restricted: node.permission_level > 0,
                    redirect: node.redirect.map(id_of),
                    children: node.children.iter().map(|c| id_of(*c)).collect(),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        Ok(ClientboundCommandsPacket {
            root_index: id_of(ROOT),
            entries,
        })
    }

    /// Assigns every node an equivalence class following Brigadier's
    /// `CommandNode.equals`/`hashCode`: same node kind and name, equal argument
    /// type, same command object and equal children (compared as a map by
    /// name). Redirects, requirements and forks do not take part.
    fn equality_classes(&self) -> Vec<u32> {
        type Signature = (u8, String, u32, Option<u32>, Vec<(String, u32)>);
        fn classify(
            graph: &CommandGraph,
            node: usize,
            done: &mut Vec<Option<u32>>,
            interned: &mut HashMap<Signature, u32>,
        ) -> u32 {
            if let Some(class) = done[node] {
                return class;
            }
            let n = graph.node(node);
            let mut children: Vec<(String, u32)> = n
                .children
                .iter()
                .map(|c| {
                    let class = classify(graph, *c, done, interned);
                    (graph.node(*c).kind.name().to_owned(), class)
                })
                .collect();
            children.sort();
            let (tag, name, type_key) = match &n.kind {
                NodeKind::Root => (0, String::new(), 0),
                NodeKind::Literal { name } => (1, name.clone(), 0),
                NodeKind::Argument { name, type_key, .. } => (2, name.clone(), *type_key),
            };
            let next = interned.len() as u32;
            let class = *interned
                .entry((tag, name, type_key, n.command_key, children))
                .or_insert(next);
            done[node] = Some(class);
            class
        }
        let mut done = vec![None; self.node_count()];
        let mut interned = HashMap::new();
        (0..self.node_count())
            .map(|node| classify(self, node, &mut done, &mut interned))
            .collect()
    }
}

fn stub_for(kind: &NodeKind) -> io::Result<CommandNodeStubData> {
    Ok(match kind {
        NodeKind::Root => CommandNodeStubData::Root,
        NodeKind::Literal { name } => CommandNodeStubData::Literal { name: name.clone() },
        NodeKind::Argument {
            name,
            parser,
            properties,
            suggestions,
            ..
        } => {
            let (parser_type_id, payload) = encode_argument_type(parser, properties)?;
            CommandNodeStubData::Argument {
                name: name.clone(),
                parser_type_id,
                parser_payload: payload,
                suggestion_id: suggestions
                    .as_deref()
                    .map(|id| Identifier::parse(id).map_err(invalid))
                    .transpose()?,
            }
        }
    })
}

/// Registry id (`BuiltInRegistries.COMMAND_ARGUMENT_TYPE.getId`) and the
/// `ArgumentTypeInfo.serializeToNetwork` payload for a parser and its JSON
/// properties (the inverse of `ArgumentTypeInfo.serializeToJson`).
pub fn encode_argument_type(parser: &str, properties: &Value) -> io::Result<(i32, Vec<u8>)> {
    let qualified = |id: &str| id.strip_prefix("minecraft:").unwrap_or(id).to_owned();
    let wanted = qualified(parser);
    let (id, registration) = argument_type_bootstrap_order()
        .iter()
        .enumerate()
        .find(|(_, r)| qualified(r.id) == wanted)
        .ok_or_else(|| invalid(format!("unknown argument type {parser}")))?;
    let mut out = Vec::new();
    match registration.info {
        ArgumentInfoKind::Singleton { .. } => {}
        ArgumentInfoKind::Numeric(kind) => encode_numeric(kind, properties, &mut out)?,
        ArgumentInfoKind::String => {
            let ty = match properties["type"].as_str() {
                Some("word") => BrigadierStringType::SingleWord,
                Some("phrase") => BrigadierStringType::QuotablePhrase,
                Some("greedy") => BrigadierStringType::GreedyPhrase,
                other => return Err(invalid(format!("bad string type {other:?}"))),
            };
            write_var_i32(&mut out, ty.network_ordinal())?;
        }
        // EntityArgument.Info: bit0 single, bit1 playersOnly.
        ArgumentInfoKind::Entity => {
            let single = properties["amount"].as_str() == Some("single");
            let players = properties["type"].as_str() == Some("players");
            out.push(u8::from(single) | (u8::from(players) << 1));
        }
        // ScoreHolderArgument.Info: bit0 multiple.
        ArgumentInfoKind::ScoreHolder => {
            out.push(u8::from(properties["amount"].as_str() == Some("multiple")));
        }
        // TimeArgument.Info: the minimum as a big-endian int.
        ArgumentInfoKind::Time => {
            let min = properties["min"]
                .as_i64()
                .ok_or_else(|| invalid("time min"))?;
            out.write_all(&(min as i32).to_be_bytes())?;
        }
        // Resource*Argument.Info: the registry key identifier.
        ArgumentInfoKind::RegistryBacked => {
            let registry = properties["registry"]
                .as_str()
                .ok_or_else(|| invalid("registry"))?;
            let key = Identifier::parse(registry).map_err(invalid)?;
            crate::network::codec::write_identifier(&mut out, &key)?;
        }
    }
    Ok((id as i32, out))
}

/// Numeric infos: a flags byte (`ArgumentUtils.createNumberFlags`) then the
/// present bounds.
fn encode_numeric(kind: NumericArgumentKind, props: &Value, out: &mut Vec<u8>) -> io::Result<()> {
    let (min, max) = (props.get("min"), props.get("max"));
    out.push(u8::from(min.is_some()) | (u8::from(max.is_some()) << 1));
    for bound in [min, max].into_iter().flatten() {
        match kind {
            NumericArgumentKind::Float => out.extend((num_f64(bound)? as f32).to_be_bytes()),
            NumericArgumentKind::Double => out.extend(num_f64(bound)?.to_be_bytes()),
            NumericArgumentKind::Integer => out.extend((num_i64(bound)? as i32).to_be_bytes()),
            NumericArgumentKind::Long => out.extend(num_i64(bound)?.to_be_bytes()),
        }
    }
    Ok(())
}

fn num_f64(v: &Value) -> io::Result<f64> {
    v.as_f64().ok_or_else(|| invalid("numeric bound"))
}

fn num_i64(v: &Value) -> io::Result<i64> {
    v.as_i64().ok_or_else(|| invalid("integral bound"))
}
