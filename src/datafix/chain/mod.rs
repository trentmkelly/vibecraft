//! The fixer chain, ported from `net.minecraft.util.datafix.DataFixers.addFixers`.
//!
//! Schemas and fixes are registered in exactly the order Java registers them
//! (the order defines both the schema parent chain and the fix execution order).
//! The Java method is split by era into submodules; only a prefix of the chain
//! has been ported so far and [`COMPLETE_THROUGH`] records how far.

use std::sync::OnceLock;

use super::fixer::{DataFixer, DataFixerBuilder};

mod flattening;
mod late_flattening;
mod legacy;

/// The highest data version for which every Java fix is registered here.
pub const COMPLETE_THROUGH: i32 = 1624;

/// `DataFixers.createRenamer(from, to)`: replaces exactly `from` with `to`.
pub fn create_renamer(from: &'static str, to: &'static str) -> impl Fn(&str) -> String {
    move |name| {
        if name == from {
            to.to_string()
        } else {
            name.to_string()
        }
    }
}

/// `DataFixers.createRenamer(Map)`: renames the names present in `map`.
pub fn create_renamer_map(map: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> String {
    move |name| {
        map.iter()
            .find(|(from, _)| *from == name)
            .map_or(name, |(_, to)| *to)
            .to_string()
    }
}

/// `DataFixers.addFixers`.
pub fn build() -> DataFixer {
    let mut builder = DataFixerBuilder::new();
    legacy::add_fixers(&mut builder);
    flattening::add_fixers(&mut builder);
    late_flattening::add_fixers(&mut builder);
    builder.build(COMPLETE_THROUGH)
}

/// `DataFixers.getDataFixer()`.
pub fn data_fixer() -> &'static DataFixer {
    static FIXER: OnceLock<DataFixer> = OnceLock::new();
    FIXER.get_or_init(build)
}
