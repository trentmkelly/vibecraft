//! Type templates: the Rust model of DataFixerUpper's `TypeTemplate` DSL.
//!
//! # Modeling choice
//!
//! DFU builds a heavy algebraic type system (products, sums, optics, recursive
//! points, typed rewrite rules) and runs fixes against typed values. VibeCraft
//! keeps the same *shape* information but applies it to raw NBT [`Tag`] trees:
//!
//! * a [`Tmpl`] is the schema-side description of where nested, separately
//!   fixable types (item stacks, entities, block entities, ...) live inside a
//!   value. It mirrors the `DSL.*` calls in the Java `Schema` classes one to one
//!   (`optionalFields`, `list`, `compoundList`, `and`, `or`, `taggedChoice`,
//!   `hook`, `remainder`, `constType`, and `References.X.in(schema)`).
//! * [`decode`](super::decode) reproduces what DFU's *read* step does to the
//!   data (validation, integer/namespaced-string normalisation, pre-read
//!   hooks, failure when a tagged choice has an unknown key).
//! * [`walk`](super::walk) reproduces `TypeRewriteRule.everywhere`: it visits every
//!   occurrence of a target type and hands the NBT to the fix.
//!
//! Fixes therefore stay close to their Java bodies (which are almost always
//! `Dynamic` manipulations) while the schema classes are ported faithfully,
//! because *which* values get visited is decided entirely by the templates.

use super::references::TypeReference;

/// Which tagged-choice registry a [`Tmpl::Choice`] resolves against. In Java the
/// lazy `taggedChoiceLazy("id", string, entityTypes)` captures the schema's
/// `entityTypes` / `blockEntityTypes` maps; here the schema resolves them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChoiceSet {
    /// The schema's entity id -> template map.
    Entities,
    /// The schema's block entity id -> template map.
    BlockEntities,
    /// A schema-local choice map registered under a name (for example the
    /// scoreboard criterion types of `V1451_6`).
    Named(&'static str),
}

/// Primitive `DSL.constType(...)` leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leaf {
    /// `DSL.intType()`: any numeric tag, decoded to an int tag (`Number.intValue()`).
    Int,
    /// `DSL.string()`: string tags only.
    Str,
    /// `NamespacedSchema.namespacedString()`: string tags, normalised by
    /// `ensureNamespaced` when read.
    NsStr,
}

/// A `Hook.HookFunction`: rewrites the raw value before its element template
/// decodes it (pre-read) or after it has been encoded (post-write).
pub type HookFn = fn(&mut crate::storage::nbt::Tag);

/// One named field of an `optionalFields` / `fields` template.
#[derive(Debug, Clone)]
pub struct Field {
    /// NBT key.
    pub name: &'static str,
    /// Template of the value.
    pub tmpl: Tmpl,
    /// `true` for `DSL.optional(DSL.field(..))`, `false` for a required `DSL.field(..)`.
    pub optional: bool,
}

/// A type template (Java `TypeTemplate`).
#[derive(Debug, Clone)]
pub enum Tmpl {
    /// `DSL.remainder()`: opaque data; accepts anything.
    Remainder,
    /// `DSL.constType(..)`.
    Leaf(Leaf),
    /// `References.X.in(schema)`: resolved against the schema being used.
    Ref(TypeReference),
    /// `DSL.fields` / `DSL.optionalFields`: named fields plus an implicit
    /// trailing remainder (unlisted keys are preserved).
    Fields(Vec<Field>),
    /// `DSL.list(t)`.
    List(Box<Tmpl>),
    /// `DSL.compoundList(key, value)`: a compound whose keys are `key` typed
    /// strings and whose values are `value`.
    CompoundList(Box<Tmpl>, Box<Tmpl>),
    /// `DSL.and(a, b, ..)`: several templates over the same compound.
    And(Vec<Tmpl>),
    /// `DSL.or(a, b)`: an `Either`; the left side wins when it decodes.
    Or(Box<Tmpl>, Box<Tmpl>),
    /// `DSL.taggedChoiceLazy(key, keyType, map)`.
    Choice {
        /// Discriminator field (always `"id"` in vanilla schemas).
        key: &'static str,
        /// Whether the key type is `namespacedString()` rather than `string()`.
        namespaced_key: bool,
        /// Which registry provides the per-id templates.
        set: ChoiceSet,
    },
    /// `DSL.hook(t, preRead, postWrite)`; `None` is `HookFunction.IDENTITY`.
    Hook(Box<Tmpl>, HookFn, Option<HookFn>),
}

/// Builders mirroring the `DSL.*` static methods used by the Java schemas.
pub mod dsl {
    use super::{ChoiceSet, Field, HookFn, Leaf, Tmpl};
    use crate::datafix::references::TypeReference;

    /// `DSL.remainder()`.
    pub fn remainder() -> Tmpl {
        Tmpl::Remainder
    }

    /// `DSL.constType(DSL.intType())`.
    pub fn int() -> Tmpl {
        Tmpl::Leaf(Leaf::Int)
    }

    /// `DSL.constType(DSL.string())`.
    pub fn string() -> Tmpl {
        Tmpl::Leaf(Leaf::Str)
    }

    /// `DSL.constType(NamespacedSchema.namespacedString())`.
    pub fn namespaced_string() -> Tmpl {
        Tmpl::Leaf(Leaf::NsStr)
    }

    /// `References.X.in(schema)`.
    pub fn reference(reference: TypeReference) -> Tmpl {
        Tmpl::Ref(reference)
    }

    /// `DSL.optionalFields(name, tmpl, name, tmpl, ..)`.
    pub fn optional_fields(fields: Vec<(&'static str, Tmpl)>) -> Tmpl {
        Tmpl::Fields(
            fields
                .into_iter()
                .map(|(name, tmpl)| Field {
                    name,
                    tmpl,
                    optional: true,
                })
                .collect(),
        )
    }

    /// `DSL.optionalFields(name, tmpl, .., rest)`: fields plus a template over the
    /// same compound.
    pub fn optional_fields_with_rest(fields: Vec<(&'static str, Tmpl)>, rest: Tmpl) -> Tmpl {
        Tmpl::And(vec![optional_fields(fields), rest])
    }

    /// `DSL.fields(name, tmpl, ..)`: required fields.
    pub fn fields(fields: Vec<(&'static str, Tmpl)>) -> Tmpl {
        Tmpl::Fields(
            fields
                .into_iter()
                .map(|(name, tmpl)| Field {
                    name,
                    tmpl,
                    optional: false,
                })
                .collect(),
        )
    }

    /// `DSL.list(tmpl)`.
    pub fn list(tmpl: Tmpl) -> Tmpl {
        Tmpl::List(Box::new(tmpl))
    }

    /// `DSL.compoundList(tmpl)`: plain string keys.
    pub fn compound_list(tmpl: Tmpl) -> Tmpl {
        Tmpl::CompoundList(Box::new(string()), Box::new(tmpl))
    }

    /// `DSL.compoundList(keyTmpl, valueTmpl)`.
    pub fn compound_list_keyed(key: Tmpl, value: Tmpl) -> Tmpl {
        Tmpl::CompoundList(Box::new(key), Box::new(value))
    }

    /// `DSL.taggedChoiceLazy(key, DSL.string(), map)` over a schema-local map.
    pub fn named_choice(key: &'static str, name: &'static str) -> Tmpl {
        Tmpl::Choice {
            key,
            namespaced_key: false,
            set: ChoiceSet::Named(name),
        }
    }

    /// `DSL.and(..)`.
    pub fn and(parts: Vec<Tmpl>) -> Tmpl {
        Tmpl::And(parts)
    }

    /// `DSL.or(a, b)`.
    pub fn or(left: Tmpl, right: Tmpl) -> Tmpl {
        Tmpl::Or(Box::new(left), Box::new(right))
    }

    /// `DSL.taggedChoiceLazy("id", DSL.string(), map)`.
    pub fn choice(set: ChoiceSet) -> Tmpl {
        Tmpl::Choice {
            key: "id",
            namespaced_key: false,
            set,
        }
    }

    /// `DSL.taggedChoiceLazy("id", NamespacedSchema.namespacedString(), map)`.
    pub fn namespaced_choice(set: ChoiceSet) -> Tmpl {
        Tmpl::Choice {
            key: "id",
            namespaced_key: true,
            set,
        }
    }

    /// `DSL.hook(tmpl, preRead, HookFunction.IDENTITY)`.
    pub fn hook(tmpl: Tmpl, pre_read: HookFn) -> Tmpl {
        Tmpl::Hook(Box::new(tmpl), pre_read, None)
    }

    /// `DSL.hook(tmpl, preRead, postWrite)`.
    pub fn hook_full(tmpl: Tmpl, pre_read: HookFn, post_write: HookFn) -> Tmpl {
        Tmpl::Hook(Box::new(tmpl), pre_read, Some(post_write))
    }
}
