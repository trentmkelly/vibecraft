#![allow(dead_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::item_catalog::item_static_name;
use crate::item_stack::ItemStack;
use crate::player_inventory::{ItemCost, MerchantOffer};
use crate::resources::{DataResourceIndex, DataResourceKind};
use crate::villager_system::{VillagerLevel, VillagerProfession};

const VANILLA_DATA_ROOT_ENV: &str = "VIBECRAFT_VANILLA_DATA_ROOT";

pub type WanderingTraderOfferGroups = (Vec<MerchantOffer>, Vec<MerchantOffer>, Vec<MerchantOffer>);

#[derive(Debug, Clone, PartialEq)]
pub struct VillagerTradeResource {
    pub wants: TradeCostResource,
    pub additional_wants: Option<TradeCostResource>,
    pub gives: ItemStackTemplateResource,
    pub max_uses: NumberProviderResource,
    pub reputation_discount: NumberProviderResource,
    pub xp: NumberProviderResource,
    pub merchant_predicate: Option<serde_json::Value>,
    pub given_item_modifiers: Vec<serde_json::Value>,
    pub double_trade_price_enchantments: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeSetResource {
    pub trades: HolderSetResource,
    pub amount: NumberProviderResource,
    pub allow_duplicates: bool,
    pub random_sequence: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TradeCostResource {
    pub item_id: String,
    pub count: NumberProviderResource,
    pub components: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStackTemplateResource {
    pub item_id: String,
    pub count: i32,
    pub components: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumberProviderResource {
    Constant(f64),
    Provider(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSetResource {
    Tag(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VillagerTradeTagResource {
    pub values: Vec<HolderSetResource>,
}

pub fn profession_offers_from_default_data(
    profession: VillagerProfession,
    level: VillagerLevel,
) -> Result<Vec<MerchantOffer>, String> {
    let index = load_villager_trade_data_root(configured_vanilla_data_root()?)?;
    profession_offers_from_resources(&index, profession, level)
}

pub fn configured_vanilla_data_root() -> Result<PathBuf, String> {
    std::env::var_os(VANILLA_DATA_ROOT_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| {
            format!(
                "{VANILLA_DATA_ROOT_ENV} is not set; vanilla villager trade JSON is not bundled"
            )
        })
}

pub fn profession_offers_from_resources(
    resources: &DataResourceIndex,
    profession: VillagerProfession,
    level: VillagerLevel,
) -> Result<Vec<MerchantOffer>, String> {
    let Some(trade_set_id) = profession_trade_set_id(profession, level) else {
        return Ok(Vec::new());
    };
    offers_from_trade_set(resources, &trade_set_id)
}

pub fn wandering_trader_offers_from_resources(
    resources: &DataResourceIndex,
) -> Result<WanderingTraderOfferGroups, String> {
    Ok((
        offers_from_trade_set(resources, "wandering_trader/buying")?,
        offers_from_trade_set(resources, "wandering_trader/uncommon")?,
        offers_from_trade_set(resources, "wandering_trader/common")?,
    ))
}

pub fn offers_from_trade_set(
    resources: &DataResourceIndex,
    trade_set_id: &str,
) -> Result<Vec<MerchantOffer>, String> {
    let raw = resources
        .get("minecraft", DataResourceKind::TradeSet, trade_set_id)
        .ok_or_else(|| format!("missing trade set minecraft:{trade_set_id}"))?;
    let trade_set = parse_trade_set_resource(raw)?;
    let mut trade_ids = resolve_trade_holder_set(resources, &trade_set.trades)?;
    if !trade_set.allow_duplicates {
        let mut seen = BTreeSet::new();
        trade_ids.retain(|id| seen.insert(id.clone()));
    }
    let amount = number_provider_floor_i32(&trade_set.amount)?.max(0) as usize;
    // Java samples with the trade_set random sequence. VibeCraft does not yet
    // route RandomSequences into trade selection, so keep vanilla tag order and
    // apply the amount limit deterministically until that subsystem is wired.
    trade_ids.truncate(amount);

    trade_ids
        .iter()
        .map(|trade_id| {
            let raw = resources
                .get("minecraft", DataResourceKind::VillagerTrade, trade_id)
                .ok_or_else(|| format!("missing villager trade minecraft:{trade_id}"))?;
            parse_villager_trade_resource(raw)?.try_into_merchant_offer()
        })
        .collect()
}

pub fn load_villager_trade_data_root(
    data_minecraft_root: impl AsRef<Path>,
) -> Result<DataResourceIndex, String> {
    let root = data_minecraft_root.as_ref();
    let mut resources = Vec::new();
    for component in ["trade_set", "villager_trade", "tags/villager_trade"] {
        collect_json_resources(root, component, &mut resources)?;
    }
    DataResourceIndex::from_resources(
        resources
            .iter()
            .map(|(path, contents)| (path.as_str(), contents.as_str())),
    )
}

fn collect_json_resources(
    data_root: &Path,
    component: &str,
    resources: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let root = data_root.join(component);
    if !root.exists() {
        return Err(format!("missing vanilla data directory {}", root.display()));
    }
    let mut paths = Vec::new();
    collect_json_paths(&root, &mut paths)?;
    paths.sort();
    for path in paths {
        let relative = path.strip_prefix(data_root).map_err(|err| {
            format!(
                "failed to relativize {} against {}: {err}",
                path.display(),
                data_root.display()
            )
        })?;
        let resource_path = format!(
            "data/minecraft/{}",
            relative.to_string_lossy().replace('\\', "/")
        );
        let contents = std::fs::read_to_string(&path)
            .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
        resources.push((resource_path, contents));
    }
    Ok(())
}

fn collect_json_paths(root: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(root)
        .map_err(|err| format!("failed to read {}: {err}", root.display()))?
    {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        if path.is_dir() {
            collect_json_paths(&path, paths)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
            paths.push(path);
        }
    }
    Ok(())
}

fn resolve_trade_holder_set(
    resources: &DataResourceIndex,
    holder_set: &HolderSetResource,
) -> Result<Vec<String>, String> {
    let mut seen_tags = BTreeSet::new();
    resolve_trade_holder_set_inner(resources, holder_set, &mut seen_tags)
}

fn resolve_trade_holder_set_inner(
    resources: &DataResourceIndex,
    holder_set: &HolderSetResource,
    seen_tags: &mut BTreeSet<String>,
) -> Result<Vec<String>, String> {
    match holder_set {
        HolderSetResource::List(entries) => Ok(entries
            .iter()
            .map(|entry| resource_path_without_namespace(entry))
            .collect()),
        HolderSetResource::Tag(tag) => {
            let tag_path = format!("villager_trade/{}", resource_path_without_namespace(tag));
            if !seen_tags.insert(tag_path.clone()) {
                return Err(format!("recursive villager trade tag minecraft:{tag_path}"));
            }
            let raw = resources
                .get("minecraft", DataResourceKind::Tags, &tag_path)
                .ok_or_else(|| format!("missing villager trade tag minecraft:{tag_path}"))?;
            let tag = parse_villager_trade_tag_resource(raw)?;
            let mut resolved = Vec::new();
            for value in &tag.values {
                resolved.extend(resolve_trade_holder_set_inner(resources, value, seen_tags)?);
            }
            seen_tags.remove(&tag_path);
            Ok(resolved)
        }
    }
}

impl VillagerTradeResource {
    pub fn try_into_merchant_offer(&self) -> Result<MerchantOffer, String> {
        let cost_a = ItemCost::new(
            static_item_name(&self.wants.item_id),
            number_provider_floor_i32(&self.wants.count)?.max(1),
        );
        let cost_b = match &self.additional_wants {
            Some(cost) => Some(ItemCost::new(
                static_item_name(&cost.item_id),
                number_provider_floor_i32(&cost.count)?.max(1),
            )),
            None => None,
        };
        let mut offer = MerchantOffer::new(
            cost_a,
            cost_b,
            ItemStack::new(
                static_item_name(&self.gives.item_id),
                self.gives.count.max(1),
            ),
            number_provider_floor_i32(&self.max_uses)?.max(1),
            number_provider_floor_i32(&self.xp)?.max(0),
            number_provider_f32(&self.reputation_discount)?,
        );
        offer.ignore_discount = self.double_trade_price_enchantments.is_some();
        Ok(offer)
    }
}

fn profession_trade_set_id(profession: VillagerProfession, level: VillagerLevel) -> Option<String> {
    let profession = match profession {
        VillagerProfession::None | VillagerProfession::Nitwit => return None,
        VillagerProfession::Armorer => "armorer",
        VillagerProfession::Butcher => "butcher",
        VillagerProfession::Cartographer => "cartographer",
        VillagerProfession::Cleric => "cleric",
        VillagerProfession::Farmer => "farmer",
        VillagerProfession::Fisherman => "fisherman",
        VillagerProfession::Fletcher => "fletcher",
        VillagerProfession::Leatherworker => "leatherworker",
        VillagerProfession::Librarian => "librarian",
        VillagerProfession::Mason => "mason",
        VillagerProfession::Shepherd => "shepherd",
        VillagerProfession::Toolsmith => "toolsmith",
        VillagerProfession::Weaponsmith => "weaponsmith",
    };
    Some(format!("{profession}/level_{}", level as i32))
}

pub fn parse_villager_trade_resource(raw: &str) -> Result<VillagerTradeResource, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid villager trade JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "villager trade must be a JSON object".to_string())?;

    Ok(VillagerTradeResource {
        wants: parse_trade_cost(required_value(object, "wants")?)?,
        additional_wants: object
            .get("additional_wants")
            .map(parse_trade_cost)
            .transpose()?,
        gives: parse_item_stack_template(required_value(object, "gives")?)?,
        max_uses: object
            .get("max_uses")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(4.0)),
        reputation_discount: object
            .get("reputation_discount")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(0.0)),
        xp: object
            .get("xp")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(1.0)),
        merchant_predicate: object.get("merchant_predicate").cloned(),
        given_item_modifiers: object
            .get("given_item_modifiers")
            .map(parse_value_list)
            .transpose()?
            .unwrap_or_default(),
        double_trade_price_enchantments: object
            .get("double_trade_price_enchantments")
            .map(parse_holder_set_id)
            .transpose()?,
    })
}

pub fn parse_trade_set_resource(raw: &str) -> Result<TradeSetResource, String> {
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid trade set JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "trade set must be a JSON object".to_string())?;

    Ok(TradeSetResource {
        trades: parse_holder_set(required_value(object, "trades")?)?,
        amount: parse_number_provider(required_value(object, "amount")?)?,
        allow_duplicates: object
            .get("allow_duplicates")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        random_sequence: object
            .get("random_sequence")
            .map(json_string_value)
            .transpose()?,
    })
}

pub fn parse_villager_trade_tag_resource(raw: &str) -> Result<VillagerTradeTagResource, String> {
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|err| format!("invalid villager trade tag JSON: {err}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "villager trade tag must be a JSON object".to_string())?;
    let values = required_value(object, "values")?
        .as_array()
        .ok_or_else(|| "villager trade tag values must be a list".to_string())?;
    let mut parsed = Vec::new();
    for value in values {
        let id = if let Some(id) = value.as_str() {
            id
        } else {
            let object = value
                .as_object()
                .ok_or_else(|| "villager trade tag value must be a string or object".to_string())?;
            object
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "villager trade tag object value must have id".to_string())?
        };
        parsed.push(if let Some(tag) = id.strip_prefix('#') {
            HolderSetResource::Tag(tag.to_string())
        } else {
            HolderSetResource::List(vec![id.to_string()])
        });
    }
    Ok(VillagerTradeTagResource { values: parsed })
}

pub fn load_villager_trade_resource(
    path: impl AsRef<Path>,
) -> Result<VillagerTradeResource, String> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
    parse_villager_trade_resource(&raw)
}

pub fn load_trade_set_resource(path: impl AsRef<Path>) -> Result<TradeSetResource, String> {
    let raw = std::fs::read_to_string(path.as_ref())
        .map_err(|err| format!("failed to read {}: {err}", path.as_ref().display()))?;
    parse_trade_set_resource(&raw)
}

fn parse_trade_cost(value: &serde_json::Value) -> Result<TradeCostResource, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "trade cost must be a JSON object".to_string())?;
    Ok(TradeCostResource {
        item_id: json_string(object, "id")?,
        count: object
            .get("count")
            .map(parse_number_provider)
            .transpose()?
            .unwrap_or(NumberProviderResource::Constant(1.0)),
        components: object
            .get("components")
            .map(json_object_value)
            .transpose()?,
    })
}

fn parse_item_stack_template(
    value: &serde_json::Value,
) -> Result<ItemStackTemplateResource, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "item stack template must be a JSON object".to_string())?;
    Ok(ItemStackTemplateResource {
        item_id: json_string(object, "id")?,
        count: object
            .get("count")
            .map(json_i32_value)
            .transpose()?
            .unwrap_or(1),
        components: object
            .get("components")
            .map(json_object_value)
            .transpose()?,
    })
}

fn parse_number_provider(value: &serde_json::Value) -> Result<NumberProviderResource, String> {
    if let Some(number) = value.as_f64() {
        return Ok(NumberProviderResource::Constant(number));
    }
    if value.as_object().is_some() {
        return Ok(NumberProviderResource::Provider(value.clone()));
    }
    Err("number provider must be a number or object".to_string())
}

fn parse_holder_set(value: &serde_json::Value) -> Result<HolderSetResource, String> {
    if let Some(id) = value.as_str() {
        return if let Some(tag) = id.strip_prefix('#') {
            Ok(HolderSetResource::Tag(tag.to_string()))
        } else {
            Ok(HolderSetResource::List(vec![id.to_string()]))
        };
    }
    let entries = value
        .as_array()
        .ok_or_else(|| "holder set must be a tag string, id string, or list".to_string())?;
    entries
        .iter()
        .map(json_string_value)
        .collect::<Result<Vec<_>, _>>()
        .map(HolderSetResource::List)
}

fn parse_holder_set_id(value: &serde_json::Value) -> Result<String, String> {
    match parse_holder_set(value)? {
        HolderSetResource::Tag(tag) => Ok(format!("#{tag}")),
        HolderSetResource::List(entries) => Ok(entries.join(",")),
    }
}

fn number_provider_floor_i32(provider: &NumberProviderResource) -> Result<i32, String> {
    let value = number_provider_f64(provider)?;
    if !value.is_finite() {
        return Err("number provider produced a non-finite value".to_string());
    }
    Ok(value.floor() as i32)
}

fn number_provider_f32(provider: &NumberProviderResource) -> Result<f32, String> {
    Ok(number_provider_f64(provider)? as f32)
}

fn number_provider_f64(provider: &NumberProviderResource) -> Result<f64, String> {
    match provider {
        NumberProviderResource::Constant(value) => Ok(*value),
        NumberProviderResource::Provider(value) => {
            let object = value
                .as_object()
                .ok_or_else(|| "number provider must be an object".to_string())?;
            match object.get("type").and_then(serde_json::Value::as_str) {
                Some("minecraft:uniform") => number_provider_bound(object, "min"),
                Some("minecraft:binomial") => {
                    Ok(number_provider_bound(object, "n")? * number_provider_bound(object, "p")?)
                }
                Some("minecraft:sum") => {
                    let values = object
                        .get("values")
                        .and_then(serde_json::Value::as_array)
                        .ok_or_else(|| "sum number provider requires values".to_string())?;
                    values
                        .iter()
                        .map(parse_number_provider)
                        .map(|provider| {
                            provider.and_then(|provider| number_provider_f64(&provider))
                        })
                        .sum()
                }
                Some(kind) => Err(format!("unsupported villager trade number provider {kind}")),
                None => Err("number provider object requires type".to_string()),
            }
        }
    }
}

fn number_provider_bound(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<f64, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{field} is required"))
        .and_then(parse_number_provider)
        .and_then(|provider| number_provider_f64(&provider))
}

fn static_item_name(item_id: &str) -> &'static str {
    item_static_name(item_id).unwrap_or_else(|| Box::leak(item_id.to_string().into_boxed_str()))
}

fn resource_path_without_namespace(id: &str) -> String {
    id.strip_prefix("minecraft:").unwrap_or(id).to_string()
}

fn parse_value_list(value: &serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    value
        .as_array()
        .cloned()
        .ok_or_else(|| "value must be a list".to_string())
}

fn required_value<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<&'a serde_json::Value, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{field} is required"))
}

fn json_string(
    object: &serde_json::Map<String, serde_json::Value>,
    field: &str,
) -> Result<String, String> {
    object
        .get(field)
        .map(json_string_value)
        .transpose()?
        .ok_or_else(|| format!("{field} must be a string"))
}

fn json_string_value(value: &serde_json::Value) -> Result<String, String> {
    value
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| "value must be a string".to_string())
}

fn json_i32_value(value: &serde_json::Value) -> Result<i32, String> {
    if let Some(integer) = value.as_i64() {
        return i32::try_from(integer).map_err(|_| "value is out of i32 range".to_string());
    }
    let number = value
        .as_f64()
        .ok_or_else(|| "value must be a number".to_string())?;
    if number.fract() != 0.0 {
        return Err("value must be an integer".to_string());
    }
    i32::try_from(number as i64).map_err(|_| "value is out of i32 range".to_string())
}

fn json_object_value(
    value: &serde_json::Value,
) -> Result<serde_json::Map<String, serde_json::Value>, String> {
    value
        .as_object()
        .cloned()
        .ok_or_else(|| "value must be an object".to_string())
}
