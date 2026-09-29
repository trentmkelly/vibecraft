//! Port of `net.minecraft.util.datafix.schemas.V1451_6`: statistics and scoreboard
//! objectives with structured criteria.

use crate::datafix::dynamic::{compound, get, get_str, remove, set};
use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
use crate::registry::Identifier;
use crate::storage::nbt::Tag;

/// `V1451_6.SPECIAL_OBJECTIVE_MARKER`.
pub const SPECIAL_OBJECTIVE_MARKER: &str = "_special";

/// Name of the schema-local criterion choice map (the `criterionTypes` variable).
pub const CRITERION_TYPES: &str = "criterionTypes";

/// `V1451_6.UNPACK_OBJECTIVE_ID` (pre-read hook): `CriteriaName` is exploded into a
/// `CriteriaType` of `{type, id}`.
pub fn unpack_objective_id(input: &mut Tag) {
    let Some(name) = get_str(input, "CriteriaName").map(str::to_string) else {
        return;
    };
    let (stat_type, stat_id) = match name.find(':') {
        None => (SPECIAL_OBJECTIVE_MARKER.to_string(), name),
        Some(colon_pos) => {
            let stat_type = Identifier::by_separator(&name[..colon_pos], '.');
            let stat_id = Identifier::by_separator(&name[colon_pos + 1..], '.');
            match (stat_type, stat_id) {
                (Ok(stat_type), Ok(stat_id)) => (stat_type.to_string(), stat_id.to_string()),
                _ => (SPECIAL_OBJECTIVE_MARKER.to_string(), name),
            }
        }
    };
    set(
        input,
        "CriteriaType",
        compound(vec![
            ("type", Tag::String(stat_type)),
            ("id", Tag::String(stat_id)),
        ]),
    );
}

/// `V1451_6.REPACK_OBJECTIVE_ID` (post-write hook): `CriteriaType` is packed back
/// into `CriteriaName`.
pub fn repack_objective_id(input: &mut Tag) {
    let Some(criteria_type) = get(input, "CriteriaType") else {
        return;
    };
    let (Some(stat_type), Some(stat_id)) =
        (get_str(criteria_type, "type"), get_str(criteria_type, "id"))
    else {
        return;
    };
    let repacked = if stat_type == SPECIAL_OBJECTIVE_MARKER {
        stat_id.to_string()
    } else {
        format!(
            "{}:{}",
            pack_namespaced_with_dot(stat_type),
            pack_namespaced_with_dot(stat_id)
        )
    };
    set(input, "CriteriaName", Tag::String(repacked));
    remove(input, "CriteriaType");
}

/// `V1451_6.packNamespacedWithDot`.
pub fn pack_namespaced_with_dot(location: &str) -> String {
    match Identifier::try_parse(location) {
        Some(parsed) => format!("{}.{}", parsed.namespace(), parsed.path()),
        None => location.to_string(),
    }
}

/// `V1451_6.createCriterionTypes`: registers the criterion type choices.
pub fn create_criterion_types(schema: &mut Schema) {
    let item = || dsl::optional_fields(vec![("id", dsl::reference(r::ITEM_NAME))]);
    let block = || dsl::optional_fields(vec![("id", dsl::reference(r::BLOCK_NAME))]);
    let entity = || dsl::optional_fields(vec![("id", dsl::reference(r::ENTITY_NAME))]);
    schema.register_named_choice(CRITERION_TYPES, "minecraft:mined", block());
    for name in [
        "minecraft:crafted",
        "minecraft:used",
        "minecraft:broken",
        "minecraft:picked_up",
        "minecraft:dropped",
    ] {
        schema.register_named_choice(CRITERION_TYPES, name, item());
    }
    schema.register_named_choice(CRITERION_TYPES, "minecraft:killed", entity());
    schema.register_named_choice(CRITERION_TYPES, "minecraft:killed_by", entity());
    schema.register_named_choice(
        CRITERION_TYPES,
        "minecraft:custom",
        dsl::optional_fields(vec![("id", dsl::namespaced_string())]),
    );
    schema.register_named_choice(
        CRITERION_TYPES,
        SPECIAL_OBJECTIVE_MARKER,
        dsl::optional_fields(vec![("id", dsl::string())]),
    );
}

/// `V1451_6.registerTypes`.
fn register_types(schema: &mut Schema) {
    let item_stats = || dsl::compound_list_keyed(dsl::reference(r::ITEM_NAME), dsl::int());
    schema.register_type(
        r::STATS,
        dsl::optional_fields(vec![(
            "stats",
            dsl::optional_fields(vec![
                (
                    "minecraft:mined",
                    dsl::compound_list_keyed(dsl::reference(r::BLOCK_NAME), dsl::int()),
                ),
                ("minecraft:crafted", item_stats()),
                ("minecraft:used", item_stats()),
                ("minecraft:broken", item_stats()),
                ("minecraft:picked_up", item_stats()),
                ("minecraft:dropped", item_stats()),
                (
                    "minecraft:killed",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::int()),
                ),
                (
                    "minecraft:killed_by",
                    dsl::compound_list_keyed(dsl::reference(r::ENTITY_NAME), dsl::int()),
                ),
                (
                    "minecraft:custom",
                    dsl::compound_list_keyed(dsl::namespaced_string(), dsl::int()),
                ),
            ]),
        )]),
    );
    create_criterion_types(schema);
    schema.register_type(
        r::OBJECTIVE,
        dsl::hook_full(
            dsl::optional_fields(vec![
                ("CriteriaType", dsl::named_choice("type", CRITERION_TYPES)),
                ("DisplayName", dsl::reference(r::TEXT_COMPONENT)),
            ]),
            unpack_objective_id,
            repack_objective_id,
        ),
    );
}

/// Applies `V1451_6` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_types(schema);
}
