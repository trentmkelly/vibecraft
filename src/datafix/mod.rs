//! DataFixerUpper port: upgrades old saved data to the current `DataVersion`.
//!
//! Java's `net.minecraft.util.datafix` package runs a chain of `DataFix`es over
//! typed values described by per-version `Schema`s. This module mirrors that
//! architecture over raw NBT [`Tag`](crate::storage::nbt::Tag) trees:
//!
//! * [`references`]: `References` (the `TypeReference` handles),
//! * [`template`]: the `DSL` type templates used by schemas,
//! * [`schema`] / [`schemas`]: `Schema` and the per-version `V*` schema classes,
//! * [`decode`]: the read step of `DataFixerUpper.update` (validation, leaf
//!   normalisation, pre-read hooks, lenient optional fields, remainder copies),
//! * [`typed`]: the typed view of decoded values used by fixes (`Typed` optics),
//! * [`walk`]: `TypeRewriteRule.everywhere`, visiting every occurrence of a type,
//! * [`fix`] / [`fixes`]: `DataFix` and the `fixes.*` classes,
//! * [`encode`]: the write step (post-write hooks, unencodable values),
//! * [`fixer`] / [`chain`]: the builder, `update`, and `DataFixers.addFixers`,
//! * [`json`] / [`packed_bit_storage`] / [`uuid`] / [`legacy_component_data_fix_utils`]:
//!   the small Java utilities the fixes depend on.
//!
//! See [`template`] for the modeling choice (typed values are replaced by
//! schema-guided traversal of NBT). The Java behaviour that the model preserves
//! includes: a value that cannot be read with the starting schema's type (for
//! example an entity with an unknown id) is returned unchanged, an optional field
//! that cannot be read is left raw and untouched by fixes, fields dropped by a
//! type change reappear from the raw remainder, and an update that matches no fix
//! is a no-op that does not even normalise the value. Every fix is verified
//! against the real Minecraft 26.1.2 fixer through recorded oracle fixtures (see
//! `tests`).
//!
//! The chain is ported from the oldest version upwards; [`chain::COMPLETE_THROUGH`]
//! is the highest data version whose fixes are all present. Older data can only
//! be upgraded to the current version once the chain is complete, so the loader
//! glue ([`crate::storage::datafix_upgrade`]) keeps refusing it until then.

// The port is a library of Java-parity building blocks: helpers for fixes that
// have not been ported yet are intentionally available before they are used.
#![allow(dead_code)]

pub mod chain;
pub mod decode;
pub mod dynamic;
pub mod encode;
pub mod fix;
pub mod fixer;
pub mod fixes;
pub mod int_open_hash_set;
pub mod json;
pub mod json_ops;
pub mod legacy_component_data_fix_utils;
pub mod packed_bit_storage;
pub mod references;
pub mod schema;
pub mod schemas;
pub mod template;
pub mod typed;
pub mod uuid;
pub mod walk;

#[cfg(test)]
mod tests;
