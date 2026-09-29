//! `TypeRewriteRule.everywhere` over NBT: visiting every occurrence of a type.
//!
//! `DataFix.fixTypeEverywhere(type, ..)` wraps its function in
//! `TypeRewriteRule.everywhere(ifSame(type, ..), .., recurse = true)`, which
//! (top-down) rewrites the current node when its type equals `type` and then
//! recurses into the children of the result. [`everywhere`] does the same over a
//! tag, using the schema templates to know which child values are which type.

use std::collections::HashSet;

use crate::storage::nbt::Tag;

use super::decode::{decode, is_opaque, read_as_or_right};
use super::dynamic::{get, get_mut};
use super::references::TypeReference;
use super::schema::Schema;
use super::template::{ChoiceSet, Tmpl};

/// The type a fix rewrites (the `Type<?>` argument of `fixTypeEverywhere`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A whole reference type, `schema.getType(References.X)`.
    Type(TypeReference),
    /// One branch of a tagged choice, `schema.getChoiceType(References.X, "id")`.
    Choice(ChoiceSet, String),
    /// The whole tagged-choice type of a registry (the `TaggedChoiceType`
    /// returned by `Schema.findChoiceType(References.X)`): every node that is an
    /// entity / block entity, whatever its id.
    AnyChoice(ChoiceSet),
}

impl Target {
    /// `getChoiceType(References.ENTITY, id)`.
    pub fn entity(id: &str) -> Self {
        Target::Choice(ChoiceSet::Entities, id.to_string())
    }

    /// `getChoiceType(References.BLOCK_ENTITY, id)`.
    pub fn block_entity(id: &str) -> Self {
        Target::Choice(ChoiceSet::BlockEntities, id.to_string())
    }
}

/// Whether `target` occurs anywhere in the type `root` (the static condition
/// under which the Java rule "matches" and produces a non-nop rewrite).
pub fn reaches(schema: &Schema, root: TypeReference, target: &Target) -> bool {
    let mut visited = HashSet::new();
    reaches_ref(schema, root, target, &mut visited)
}

fn reaches_ref(
    schema: &Schema,
    reference: TypeReference,
    target: &Target,
    visited: &mut HashSet<TypeReference>,
) -> bool {
    if *target == Target::Type(reference) {
        return true;
    }
    if !visited.insert(reference) {
        return false;
    }
    match schema.type_of(reference) {
        Some(tmpl) => reaches_tmpl(schema, tmpl, target, visited),
        None => false,
    }
}

fn reaches_tmpl(
    schema: &Schema,
    tmpl: &Tmpl,
    target: &Target,
    visited: &mut HashSet<TypeReference>,
) -> bool {
    match tmpl {
        Tmpl::Remainder | Tmpl::Leaf(_) => false,
        Tmpl::Ref(reference) => reaches_ref(schema, reference, target, visited),
        Tmpl::Fields(fields) => fields
            .iter()
            .any(|field| reaches_tmpl(schema, &field.tmpl, target, visited)),
        Tmpl::CompoundList(key, value) => {
            reaches_tmpl(schema, key, target, visited)
                || reaches_tmpl(schema, value, target, visited)
        }
        Tmpl::List(inner) | Tmpl::Hook(inner, _, _) => reaches_tmpl(schema, inner, target, visited),
        Tmpl::And(parts) => parts
            .iter()
            .any(|part| reaches_tmpl(schema, part, target, visited)),
        Tmpl::Or(left, right) => {
            reaches_tmpl(schema, left, target, visited)
                || reaches_tmpl(schema, right, target, visited)
        }
        Tmpl::Choice { set, .. } => {
            match target {
                Target::Choice(target_set, name)
                    if target_set == set && schema.choice(*set, name).is_some() =>
                {
                    return true
                }
                Target::AnyChoice(target_set) if target_set == set => return true,
                _ => {}
            }
            schema
                .choices(*set)
                .into_iter()
                .any(|(_, branch)| reaches_tmpl(schema, branch, target, visited))
        }
    }
}

/// Applies `f` to every occurrence of `target` inside `tag`, which is a value of
/// type `root`. The children of a rewritten node are visited as well: the types
/// fixes rewrite (entities, item stacks, ...) are registered recursive, and DFU
/// rewrites the whole recursive family, so nested occurrences are always reached.
pub fn everywhere(
    schema: &Schema,
    root: TypeReference,
    target: &Target,
    tag: &mut Tag,
    f: &mut dyn FnMut(&mut Tag),
) {
    walk_tmpl(schema, &Tmpl::Ref(root), target, tag, f);
}

fn walk_tmpl(
    schema: &Schema,
    tmpl: &Tmpl,
    target: &Target,
    tag: &mut Tag,
    f: &mut dyn FnMut(&mut Tag),
) {
    match tmpl {
        Tmpl::Remainder | Tmpl::Leaf(_) => {}
        Tmpl::Ref(reference) => {
            if *target == Target::Type(reference) {
                f(tag);
            }
            if let Some(inner) = schema.type_of(reference) {
                walk_tmpl(schema, inner, target, tag, f);
            }
        }
        Tmpl::Fields(fields) => {
            for field in fields {
                if is_opaque(tag, field.name) {
                    continue;
                }
                if let Some(value) = get_mut(tag, field.name) {
                    walk_tmpl(schema, &field.tmpl, target, value, f);
                }
            }
        }
        Tmpl::List(inner) => {
            if let Tag::List(items) = tag {
                for item in items {
                    walk_tmpl(schema, inner, target, item, f);
                }
            }
        }
        Tmpl::CompoundList(key, inner) => {
            if let Tag::Compound(entries) = tag {
                for (name, value) in entries {
                    let mut key_tag = Tag::String(std::mem::take(name));
                    walk_tmpl(schema, key, target, &mut key_tag, f);
                    if let Tag::String(walked) = key_tag {
                        *name = walked;
                    }
                    walk_tmpl(schema, inner, target, value, f);
                }
            }
        }
        Tmpl::And(parts) => {
            for part in parts {
                walk_tmpl(schema, part, target, tag, f);
            }
        }
        Tmpl::Or(left, right) => {
            let branch = if read_as_or_right(tag) {
                right
            } else {
                let mut attempt = tag.clone();
                if decode(schema, left, &mut attempt).is_ok() {
                    left
                } else {
                    right
                }
            };
            walk_tmpl(schema, branch, target, tag, f);
        }
        Tmpl::Choice { key, set, .. } => {
            let Some(id) = get(tag, key).and_then(|id| match id {
                Tag::String(id) => Some(id.clone()),
                _ => None,
            }) else {
                return;
            };
            let Some(branch) = schema.choice(*set, &id) else {
                return;
            };
            let hit = match target {
                Target::Choice(target_set, name) => target_set == set && *name == id,
                Target::AnyChoice(target_set) => target_set == set,
                Target::Type(_) => false,
            };
            if hit {
                f(tag);
            }
            walk_tmpl(schema, branch, target, tag, f);
        }
        Tmpl::Hook(inner, _, _) => walk_tmpl(schema, inner, target, tag, f),
    }
}
