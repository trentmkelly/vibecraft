//! Codecs for `villager_trade` (`VillagerTrade.CODEC`) and `trade_set`
//! (`TradeSet.CODEC`) with `TradeCost`.
//!
//! Number providers, loot conditions and loot functions are decoded by the loot codec
//! ([`crate::loot_system`]) and item stacks / exact component predicates by the
//! advancement schema codec ([`crate::advancement_condition_schema`]); both validate
//! structurally but do not decode data component payloads yet (documented there), so
//! trade component payloads are only checked for known component type keys.
//!
//! TODO(registry-pipeline-trade-validation): `Validatable.validatorForContext(
//! LootContextParamSets.VILLAGER_TRADE)` (loot params required by the conditions and
//! functions of a trade) is not applied.

use serde_json::{json, Value};

use crate::advancement_condition_schema::{BuiltinLookup, ConditionCodec, Node};
use crate::loot_system::{
    validate_loot_condition, validate_loot_function, validate_number_provider,
};
use crate::registry_pipeline::codec::{
    bool_codec, holder_fixed, holder_set, identifier_codec, json_to_tag, lenient_opt_default, list,
    opt, opt_default, record, req, Codec,
};
use crate::storage::nbt::Tag;

const ITEM_REGISTRY: &str = "minecraft:item";
const ENCHANTMENT_REGISTRY: &str = "minecraft:enchantment";
const VILLAGER_TRADE_REGISTRY: &str = "minecraft:villager_trade";

/// A shared Java codec modelled by the advancement schema: decode, then keep the
/// re-encoded JSON as NBT.
fn schema_codec(node: Node) -> Codec {
    Codec::new(move |json, _| {
        let lookup = BuiltinLookup;
        ConditionCodec::new(&lookup)
            .decode_node(node, json)
            .map(|encoded| json_to_tag(&encoded))
    })
}

/// `NumberProviders.CODEC`: a bare number is a `ConstantValue` (a float); typed
/// providers are validated by the loot codec and kept as written.
fn number_provider() -> Codec {
    Codec::new(|json, _| match json {
        Value::Number(number) => Ok(Tag::Float(number.as_f64().unwrap_or(0.0) as f32)),
        other => {
            validate_number_provider(other)?;
            Ok(json_to_tag(other))
        }
    })
}

/// `ItemStackTemplate.CODEC`: `{id, count?, components?}` or a bare item id. The
/// constructor rejects air ("Item must be non-empty").
fn item_stack_template() -> Codec {
    let schema = schema_codec(Node::ItemTemplate);
    Codec::new(move |json, ctx| {
        let tag = schema.parse(json, ctx)?;
        let is_air = matches!(&tag, Tag::Compound(fields) if fields
            .iter()
            .any(|(key, value)| key == "id" && *value == Tag::String("minecraft:air".to_string())));
        if is_air {
            return Err("Item must be non-empty".to_string());
        }
        Ok(tag)
    })
}

/// `TradeCost.CODEC`.
fn trade_cost() -> Codec {
    record(vec![
        req("id", holder_fixed(ITEM_REGISTRY)),
        opt_default("count", number_provider(), json!(1.0)),
        opt_default(
            "components",
            schema_codec(Node::DataComponentsExact),
            json!({}),
        ),
    ])
}

/// `LootItemCondition.DIRECT_CODEC`, validated by the loot codec.
fn loot_condition() -> Codec {
    Codec::new(|json, _| {
        validate_loot_condition(json)?;
        Ok(json_to_tag(json))
    })
}

/// `LootItemFunctions.ROOT_CODEC`, validated by the loot codec.
fn loot_function() -> Codec {
    Codec::new(|json, _| {
        validate_loot_function(json)?;
        Ok(json_to_tag(json))
    })
}

/// `VillagerTrade.CODEC`.
pub fn villager_trade() -> Codec {
    record(vec![
        req("wants", trade_cost()),
        opt("additional_wants", trade_cost()),
        req("gives", item_stack_template()),
        lenient_opt_default("max_uses", number_provider(), json!(4.0)),
        lenient_opt_default("reputation_discount", number_provider(), json!(0.0)),
        lenient_opt_default("xp", number_provider(), json!(1.0)),
        opt("merchant_predicate", loot_condition()),
        opt_default("given_item_modifiers", list(loot_function()), json!([])),
        opt(
            "double_trade_price_enchantments",
            holder_set(ENCHANTMENT_REGISTRY, false),
        ),
    ])
}

/// `TradeSet.CODEC`.
pub fn trade_set() -> Codec {
    record(vec![
        req("trades", holder_set(VILLAGER_TRADE_REGISTRY, false)),
        req("amount", number_provider()),
        opt_default("allow_duplicates", bool_codec(), json!(false)),
        opt("random_sequence", identifier_codec()),
    ])
}
