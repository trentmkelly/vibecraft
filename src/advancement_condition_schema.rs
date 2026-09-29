//! Schema-driven codec for advancement criterion conditions.
//!
//! Java decodes `Criterion.conditions` through the trigger's instance codec, which is a tree
//! of `RecordCodecBuilder` records (`EntityPredicate`, `ItemPredicate`, `LocationPredicate`,
//! loot conditions, ...). VibeCraft's runtime criterion models are not JSON aware, so this
//! module mirrors those Java codecs as a declarative schema and walks a JSON document
//! against it. Decoding is strict, like DFU: an unknown field, an unknown registry entry, an
//! unknown loot-condition type or a value of the wrong JSON type is an error. The walk also
//! *re-encodes* the value the way the Java codec would (defaults omitted, single-element
//! holder sets collapsed, exact bounds collapsed to a number, `EntityPredicate` wrapped in a
//! context-aware list), so `decode(x) == x` for data-generator output is checkable.
//!
//! Deferrals (documented, not silently skipped):
//! * the *value* payload of `DataComponentExactPredicate` entries and of `DataComponentPatch`
//!   entries is only checked for a known component type key,
//! * `NbtPredicate` SNBT strings and text-component contents are checked structurally only,
//! * loot-condition types outside the set advancements use are rejected as unsupported.
//!
//! Java: `net.minecraft.advancements.criterion.*` and
//! `net.minecraft.world.level.storage.loot.predicates.*`.


use serde_json::{Map, Number, Value};

use crate::registry::Identifier;
use crate::registry_pipeline::builtin::BuiltinRegistries;

/// Registries a condition field can reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reg {
    Item,
    Block,
    EntityType,
    Fluid,
    MobEffect,
    Potion,
    StatType,
    DataComponentType,
    DataComponentPredicateType,
    EntitySubPredicateType,
    LootConditionType,
    /// `BuiltInRegistries.VILLAGER_TYPE` (the `villager/variant` component predicate).
    VillagerType,
    /// Data-driven registries (checked through [`RegistryLookup`] only).
    Biome,
    Structure,
    Enchantment,
    JukeboxSong,
    DamageType,
}

impl Reg {
    /// The `BuiltInRegistries` key for static registries, `None` for data-driven ones.
    fn static_key(self) -> Option<&'static str> {
        Some(match self {
            Reg::Item => "minecraft:item",
            Reg::Block => "minecraft:block",
            Reg::EntityType => "minecraft:entity_type",
            Reg::Fluid => "minecraft:fluid",
            Reg::MobEffect => "minecraft:mob_effect",
            Reg::Potion => "minecraft:potion",
            Reg::StatType => "minecraft:stat_type",
            Reg::VillagerType => "minecraft:villager_type",
            Reg::DataComponentType => "minecraft:data_component_type",
            Reg::DataComponentPredicateType => "minecraft:data_component_predicate_type",
            Reg::EntitySubPredicateType => "minecraft:entity_sub_predicate_type",
            Reg::LootConditionType => "minecraft:loot_condition_type",
            Reg::Biome
            | Reg::Structure
            | Reg::Enchantment
            | Reg::JukeboxSong
            | Reg::DamageType => return None,
        })
    }
}

/// Resolves registry entries and tags referenced by conditions.
pub(crate) trait RegistryLookup {
    /// Whether `registry` contains `id`.
    fn contains(&self, registry: Reg, id: &Identifier) -> bool;
    /// Whether the tag `#id` exists for `registry`.
    fn contains_tag(&self, registry: Reg, id: &Identifier) -> bool;
}

/// Lookup backed by the static `BuiltInRegistries` report only. Data-driven registries and
/// tags are accepted syntactically (they depend on the loaded data pack).
pub(crate) struct BuiltinLookup;

impl RegistryLookup for BuiltinLookup {
    fn contains(&self, registry: Reg, id: &Identifier) -> bool {
        let Some(key) = registry.static_key() else {
            return true;
        };
        let (Ok(registries), Ok(key)) = (BuiltinRegistries::vanilla(), Identifier::parse(key))
        else {
            return false;
        };
        registries.contains_element(&key, id)
    }

    fn contains_tag(&self, _registry: Reg, _id: &Identifier) -> bool {
        true
    }
}

/// Which map keys a `Node::Map` accepts.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Key {
    Reg(Reg),
    Id,
    Str,
}

/// Value emitted when a field is absent; equal values are omitted when encoding.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Def {
    /// No default (optional without value, or required).
    None,
    /// `MinMaxBounds.ANY`, an empty map or an empty record.
    EmptyObject,
    EmptyList,
    Bool(bool),
    Int(i64),
    Str(&'static str),
}

/// One record field.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Field {
    name: &'static str,
    node: Node,
    required: bool,
    default: Def,
}

const fn req(name: &'static str, node: Node) -> Field {
    Field { name, node, required: true, default: Def::None }
}

const fn opt(name: &'static str, node: Node) -> Field {
    Field { name, node, required: false, default: Def::None }
}

const fn dflt(name: &'static str, node: Node, default: Def) -> Field {
    Field { name, node, required: false, default }
}

/// Named record shapes; the field tables live in [`fields_of`].
#[derive(Debug, Clone, Copy)]
pub(crate) enum Rec {
    Entity,
    Distance,
    Movement,
    Location,
    Position,
    Light,
    Block,
    Fluid,
    Item,
    Flags,
    Equipment,
    Damage,
    DamageSource,
    TagPredicate,
    EffectInstance,
    Enchantment,
    ComponentDamage,
    JukeboxPlayable,
    Food,
    Input,
    InventorySlots,
    Slime,
    Raider,
    Sheep,
    FishingHook,
    Lightning,
    Player,
    /// `AdvancementRewards.CODEC`.
    Rewards,
    /// `DisplayInfo.CODEC`.
    Display,
}

/// A schema node.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Node {
    Bool,
    Int,
    PositiveInt,
    Str,
    Id,
    Ints,
    Doubles,
    Reg(Reg),
    /// `RegistryCodecs.homogeneousList`: `"id"`, `"#tag"` or a list of ids.
    Set(Reg),
    Rec(Rec),
    /// `TagKey.codec`: a plain tag id (no `#`).
    Tag(Reg),
    List(&'static Node),
    Map(Key, &'static Node),
    /// `EntityPredicate.ADVANCEMENT_CODEC`: a loot-condition list or one entity predicate.
    Ctx,
    /// `ContextAwarePredicate.CODEC`: a loot-condition list.
    CtxList,
    LootCondition,
    Component,
    ItemTemplate,
    StateProperties,
    StateValue,
    DataComponentsExact,
    DataComponentPredicates,
    DataComponentPatch,
    TypeSpecific,
    Stats,
    GameModes,
    AdvancementProgress,
    EntityTarget,
    /// `AdvancementType.CODEC` (`frame`).
    Frame,
}

const ANY: Def = Def::EmptyObject;
const ITEM: Node = Node::Rec(Rec::Item);
const ENTITY: Node = Node::Rec(Rec::Entity);
const LOCATION: Node = Node::Rec(Rec::Location);
const DISTANCE: Node = Node::Rec(Rec::Distance);
const PLAYER: Field = opt("player", Node::Ctx);

/// Field table for a record shape (Java: the matching `CODEC` definition).
fn fields_of(rec: Rec) -> &'static [Field] {
    use Node::{Bool, Doubles, Ints};
    match rec {
        Rec::Entity => const { &[
            opt("type", Node::Set(Reg::EntityType)),
            opt("distance", Node::Rec(Rec::Distance)),
            opt("movement", Node::Rec(Rec::Movement)),
            opt("location", LOCATION),
            opt("stepping_on", LOCATION),
            opt("movement_affected_by", LOCATION),
            opt("effects", Node::Map(Key::Reg(Reg::MobEffect), &Node::Rec(Rec::EffectInstance))),
            opt("nbt", Node::Str),
            opt("flags", Node::Rec(Rec::Flags)),
            opt("equipment", Node::Rec(Rec::Equipment)),
            opt("type_specific", Node::TypeSpecific),
            opt("periodic_tick", Node::PositiveInt),
            opt("vehicle", ENTITY),
            opt("passenger", ENTITY),
            opt("targeted_entity", ENTITY),
            opt("team", Node::Str),
            opt("slots", Node::Map(Key::Str, &Node::Rec(Rec::Item))),
            dflt("components", Node::DataComponentsExact, ANY),
            dflt("predicates", Node::DataComponentPredicates, ANY),
        ] },
        Rec::Distance => const { &[
            dflt("x", Doubles, ANY),
            dflt("y", Doubles, ANY),
            dflt("z", Doubles, ANY),
            dflt("horizontal", Doubles, ANY),
            dflt("absolute", Doubles, ANY),
        ] },
        Rec::Movement => const { &[
            dflt("x", Doubles, ANY),
            dflt("y", Doubles, ANY),
            dflt("z", Doubles, ANY),
            dflt("speed", Doubles, ANY),
            dflt("horizontal_speed", Doubles, ANY),
            dflt("vertical_speed", Doubles, ANY),
            dflt("fall_distance", Doubles, ANY),
        ] },
        Rec::Location => const { &[
            opt("position", Node::Rec(Rec::Position)),
            opt("biomes", Node::Set(Reg::Biome)),
            opt("structures", Node::Set(Reg::Structure)),
            opt("dimension", Node::Id),
            opt("smokey", Bool),
            opt("light", Node::Rec(Rec::Light)),
            opt("block", Node::Rec(Rec::Block)),
            opt("fluid", Node::Rec(Rec::Fluid)),
            opt("can_see_sky", Bool),
        ] },
        Rec::Position => const { &[dflt("x", Doubles, ANY), dflt("y", Doubles, ANY), dflt("z", Doubles, ANY)] },
        Rec::Light => const { &[dflt("light", Ints, ANY)] },
        Rec::Block => const { &[
            opt("blocks", Node::Set(Reg::Block)),
            opt("state", Node::StateProperties),
            opt("nbt", Node::Str),
            dflt("components", Node::DataComponentsExact, ANY),
            dflt("predicates", Node::DataComponentPredicates, ANY),
        ] },
        Rec::Fluid => const { &[opt("fluids", Node::Set(Reg::Fluid)), opt("state", Node::StateProperties)] },
        Rec::Item => const { &[
            opt("items", Node::Set(Reg::Item)),
            dflt("count", Ints, ANY),
            dflt("components", Node::DataComponentsExact, ANY),
            dflt("predicates", Node::DataComponentPredicates, ANY),
        ] },
        _ => fields_of_second(rec),
    }
}

/// Second slice of [`fields_of`], split to keep each table function short.
fn fields_of_second(rec: Rec) -> &'static [Field] {
    use Node::{Bool, Doubles, Ints};
    match rec {
        Rec::Flags => const { &[
            opt("is_on_ground", Bool),
            opt("is_on_fire", Bool),
            opt("is_sneaking", Bool),
            opt("is_sprinting", Bool),
            opt("is_swimming", Bool),
            opt("is_flying", Bool),
            opt("is_baby", Bool),
            opt("is_in_water", Bool),
            opt("is_fall_flying", Bool),
        ] },
        Rec::Equipment => const { &[
            opt("head", ITEM),
            opt("chest", ITEM),
            opt("legs", ITEM),
            opt("feet", ITEM),
            opt("body", ITEM),
            opt("mainhand", ITEM),
            opt("offhand", ITEM),
        ] },
        Rec::Damage => const { &[
            dflt("dealt", Doubles, ANY),
            dflt("taken", Doubles, ANY),
            opt("source_entity", ENTITY),
            opt("blocked", Bool),
            opt("type", Node::Rec(Rec::DamageSource)),
        ] },
        Rec::DamageSource => const { &[
            dflt("tags", Node::List(&Node::Rec(Rec::TagPredicate)), Def::EmptyList),
            opt("direct_entity", ENTITY),
            opt("source_entity", ENTITY),
            opt("is_direct", Bool),
        ] },
        Rec::TagPredicate => const { &[req("id", Node::Tag(Reg::DamageType)), req("expected", Bool)] },
        Rec::EffectInstance => const { &[
            dflt("amplifier", Ints, ANY),
            dflt("duration", Ints, ANY),
            opt("ambient", Bool),
            opt("visible", Bool),
        ] },
        Rec::Enchantment => const { &[opt("enchantments", Node::Set(Reg::Enchantment)), dflt("levels", Ints, ANY)] },
        Rec::ComponentDamage => const { &[dflt("durability", Ints, ANY), dflt("damage", Ints, ANY)] },
        Rec::JukeboxPlayable => const { &[opt("song", Node::Set(Reg::JukeboxSong))] },
        Rec::Food => const { &[dflt("level", Ints, ANY), dflt("saturation", Doubles, ANY)] },
        _ => fields_of_third(rec),
    }
}

/// Third slice of [`fields_of`].
fn fields_of_third(rec: Rec) -> &'static [Field] {
    use Node::{Bool, Ints};
    match rec {
        Rec::Input => const { &[
            opt("forward", Bool),
            opt("backward", Bool),
            opt("left", Bool),
            opt("right", Bool),
            opt("jump", Bool),
            opt("sneak", Bool),
            opt("sprint", Bool),
        ] },
        Rec::InventorySlots => const { &[
            dflt("occupied", Ints, ANY),
            dflt("full", Ints, ANY),
            dflt("empty", Ints, ANY),
        ] },
        Rec::Slime => const { &[dflt("size", Ints, ANY)] },
        Rec::Raider => const { &[dflt("has_raid", Bool, Def::Bool(false)), dflt("is_captain", Bool, Def::Bool(false))] },
        Rec::Sheep => const { &[opt("sheared", Bool)] },
        Rec::FishingHook => const { &[opt("in_open_water", Bool)] },
        Rec::Lightning => const { &[dflt("blocks_set_on_fire", Ints, ANY), opt("entity_struck", ENTITY)] },
        Rec::Rewards => const { &[
            dflt("experience", Node::Int, Def::Int(0)),
            dflt("loot", Node::List(&Node::Id), Def::EmptyList),
            dflt("recipes", Node::List(&Node::Id), Def::EmptyList),
            opt("function", Node::Id),
        ] },
        Rec::Display => const { &[
            req("icon", Node::ItemTemplate),
            req("title", Node::Component),
            req("description", Node::Component),
            opt("background", Node::Id),
            dflt("frame", Node::Frame, Def::Str("task")),
            dflt("show_toast", Bool, Def::Bool(true)),
            dflt("announce_to_chat", Bool, Def::Bool(true)),
            dflt("hidden", Bool, Def::Bool(false)),
        ] },
        Rec::Player => const { &[
            dflt("level", Ints, ANY),
            dflt("food", Node::Rec(Rec::Food), ANY),
            dflt("gamemode", Node::GameModes, Def::EmptyList),
            dflt("stats", Node::Stats, Def::EmptyList),
            dflt("recipes", Node::Map(Key::Id, &Bool), ANY),
            dflt("advancements", Node::Map(Key::Id, &Node::AdvancementProgress), ANY),
            opt("looking_at", ENTITY),
            opt("input", Node::Rec(Rec::Input)),
        ] },
        // Handled by the earlier slices.
        _ => &[],
    }
}

/// Instance field table for a trigger, keyed by the Java implementation class name
/// (see `advancement_trigger_registry`). `None` means the class has no schema here.
fn trigger_fields(implementation: &str) -> Option<&'static [Field]> {
    use Node::{Ints};
    Some(match implementation {
        "ImpossibleTrigger" => const { &[] },
        "PlayerTrigger" | "StartRidingTrigger" => const { &[PLAYER] },
        "AnyBlockInteractionTrigger" | "DefaultBlockInteractionTrigger" | "ItemUsedOnLocationTrigger" => const {
            &[PLAYER, opt("location", Node::CtxList)]
        }
        "BeeNestDestroyedTrigger" => const { &[
            PLAYER,
            opt("block", Node::Reg(Reg::Block)),
            opt("item", ITEM),
            dflt("num_bees_inside", Ints, ANY),
        ] },
        "BredAnimalsTrigger" => const { &[
            PLAYER,
            opt("parent", Node::Ctx),
            opt("partner", Node::Ctx),
            opt("child", Node::Ctx),
        ] },
        "BrewedPotionTrigger" => const { &[PLAYER, opt("potion", Node::Reg(Reg::Potion))] },
        "ChangeDimensionTrigger" => const { &[PLAYER, opt("from", Node::Id), opt("to", Node::Id)] },
        "ChanneledLightningTrigger" => const {
            &[PLAYER, dflt("victims", Node::List(&Node::Ctx), Def::EmptyList)]
        }
        "ConstructBeaconTrigger" => const { &[PLAYER, dflt("level", Ints, ANY)] },
        "ConsumeItemTrigger" | "FilledBucketTrigger" | "ShotCrossbowTrigger" | "UsedTotemTrigger"
        | "UsingItemTrigger" => const { &[PLAYER, opt("item", ITEM)] },
        "CuredZombieVillagerTrigger" => const {
            &[PLAYER, opt("zombie", Node::Ctx), opt("villager", Node::Ctx)]
        }
        "DistanceTrigger" => const { &[PLAYER, opt("start_position", LOCATION), opt("distance", DISTANCE)] },
        "EffectsChangedTrigger" => const { &[
            PLAYER,
            opt("effects", Node::Map(Key::Reg(Reg::MobEffect), &Node::Rec(Rec::EffectInstance))),
            opt("source", Node::Ctx),
        ] },
        "EnchantedItemTrigger" => const { &[PLAYER, opt("item", ITEM), dflt("levels", Ints, ANY)] },
        "EnterBlockTrigger" | "SlideDownBlockTrigger" => const {
            &[PLAYER, opt("block", Node::Reg(Reg::Block)), opt("state", Node::StateProperties)]
        }
        _ => return trigger_fields_second(implementation),
    })
}

/// Second slice of [`trigger_fields`].
fn trigger_fields_second(implementation: &str) -> Option<&'static [Field]> {
    use Node::{Doubles, Ints};
    Some(match implementation {
        "EntityHurtPlayerTrigger" => const { &[PLAYER, opt("damage", Node::Rec(Rec::Damage))] },
        "FallAfterExplosionTrigger" => const { &[
            PLAYER,
            opt("start_position", LOCATION),
            opt("distance", DISTANCE),
            opt("cause", Node::Ctx),
        ] },
        "FishingRodHookedTrigger" => const { &[
            PLAYER,
            opt("rod", ITEM),
            opt("entity", Node::Ctx),
            opt("item", ITEM),
        ] },
        "InventoryChangeTrigger" => const { &[
            PLAYER,
            dflt("slots", Node::Rec(Rec::InventorySlots), ANY),
            dflt("items", Node::List(&Node::Rec(Rec::Item)), Def::EmptyList),
        ] },
        "ItemDurabilityTrigger" => const { &[
            PLAYER,
            opt("item", ITEM),
            dflt("durability", Ints, ANY),
            dflt("delta", Ints, ANY),
        ] },
        "KilledByArrowTrigger" => const { &[
            PLAYER,
            dflt("victims", Node::List(&Node::Ctx), Def::EmptyList),
            dflt("unique_entity_types", Ints, ANY),
            opt("fired_from_weapon", ITEM),
        ] },
        "KilledTrigger" => const { &[
            PLAYER,
            opt("entity", Node::Ctx),
            opt("killing_blow", Node::Rec(Rec::DamageSource)),
        ] },
        "LevitationTrigger" => const { &[PLAYER, opt("distance", DISTANCE), dflt("duration", Ints, ANY)] },
        "LightningStrikeTrigger" => const {
            &[PLAYER, opt("lightning", Node::Ctx), opt("bystander", Node::Ctx)]
        }
        "LootTableTrigger" => const { &[PLAYER, req("loot_table", Node::Id)] },
        "PickedUpItemTrigger" | "PlayerInteractTrigger" => const {
            &[PLAYER, opt("item", ITEM), opt("entity", Node::Ctx)]
        }
        "PlayerHurtEntityTrigger" => const { &[
            PLAYER,
            opt("damage", Node::Rec(Rec::Damage)),
            opt("entity", Node::Ctx),
        ] },
        "RecipeCraftedTrigger" => const { &[
            PLAYER,
            req("recipe_id", Node::Id),
            dflt("ingredients", Node::List(&Node::Rec(Rec::Item)), Def::EmptyList),
        ] },
        "RecipeUnlockedTrigger" => const { &[PLAYER, req("recipe", Node::Id)] },
        "SpearMobsTrigger" => const { &[PLAYER, opt("count", Node::PositiveInt)] },
        "SummonedEntityTrigger" | "TameAnimalTrigger" => const { &[PLAYER, opt("entity", Node::Ctx)] },
        "TargetBlockTrigger" => const { &[
            PLAYER,
            dflt("signal_strength", Ints, ANY),
            opt("projectile", Node::Ctx),
        ] },
        "TradeTrigger" => const { &[PLAYER, opt("villager", Node::Ctx), opt("item", ITEM)] },
        "UsedEnderEyeTrigger" => const { &[PLAYER, dflt("distance", Doubles, ANY)] },
        _ => return None,
    })
}

/// Entity sub-predicate (`type_specific`) field tables, keyed by registry name.
fn sub_predicate_fields(kind: &str) -> Option<&'static [Field]> {
    Some(fields_of(match kind {
        "minecraft:lightning" => Rec::Lightning,
        "minecraft:fishing_hook" => Rec::FishingHook,
        "minecraft:player" => Rec::Player,
        "minecraft:slime" => Rec::Slime,
        "minecraft:raider" => Rec::Raider,
        "minecraft:sheep" => Rec::Sheep,
        _ => return None,
    }))
}

/// Entity targets accepted by `entity_properties` (`LootContext.EntityTarget`).
const ENTITY_TARGETS: [&str; 6] = [
    "this",
    "attacker",
    "direct_attacker",
    "attacking_player",
    "target_entity",
    "interacting_entity",
];

/// Vanilla `GameType` serialized names.
const GAME_TYPES: [&str; 4] = ["survival", "creative", "adventure", "spectator"];

/// Text-component keys (content and style) accepted on an object component.
const COMPONENT_KEYS: [&str; 36] = [
    "text", "translate", "fallback", "with", "keybind", "score", "selector", "nbt", "block",
    "entity", "storage", "interpret", "separator", "source", "extra", "color", "shadow_color",
    "bold", "italic", "underlined", "strikethrough", "obfuscated", "insertion", "click_event",
    "hover_event", "font", "type", "atlas", "sprite", "player", "hat", "object", "fallback_text",
    "plain", "sprite_id", "id",
];

/// Decoder/re-encoder over the schema.
pub(crate) struct ConditionCodec<'a> {
    lookup: &'a dyn RegistryLookup,
}

type Decoded = Result<Value, String>;

impl<'a> ConditionCodec<'a> {
    /// Creates a codec resolving registry references through `lookup`.
    pub(crate) fn new(lookup: &'a dyn RegistryLookup) -> Self {
        Self { lookup }
    }

    /// Decodes and re-encodes a criterion's `conditions` for `trigger` (a full id such as
    /// `minecraft:tick`). Returns `None` when Java would omit the field (an empty instance).
    pub(crate) fn conditions(
        &self,
        trigger: &Identifier,
        conditions: Option<&Value>,
    ) -> Result<Option<Value>, String> {
        let name = trigger
            .to_string()
            .strip_prefix("minecraft:")
            .map(ToString::to_string)
            .ok_or_else(|| format!("unknown criterion trigger {trigger}"))?;
        let registry = crate::advancement_trigger_registry::CriteriaTriggersModel::java_default_registry();
        let entry = registry
            .by_name(&name)
            .ok_or_else(|| format!("unknown criterion trigger {trigger}"))?;
        let fields = trigger_fields(entry.implementation).ok_or_else(|| {
            format!("trigger {trigger} ({}) has no condition schema", entry.implementation)
        })?;
        let Some(conditions) = conditions else {
            self.record(fields, &Value::Object(Map::new()))?;
            return Ok(None);
        };
        if entry.implementation == "ImpossibleTrigger" {
            // `Codec.unit`: whatever is present decodes to the singleton instance.
            return Ok(None);
        }
        let encoded = self.record(fields, conditions)?;
        let is_empty = encoded.as_object().is_none_or(Map::is_empty);
        Ok((!is_empty).then_some(encoded))
    }

    /// Decodes one text component (structural check; returned unchanged).
    pub(crate) fn component(&self, value: &Value) -> Decoded {
        match value {
            Value::String(_) => Ok(value.clone()),
            Value::Array(items) => {
                items.iter().try_for_each(|item| self.component(item).map(drop))?;
                Ok(value.clone())
            }
            Value::Object(map) => {
                if let Some(key) = map.keys().find(|key| !COMPONENT_KEYS.contains(&key.as_str())) {
                    return Err(format!("unknown text component field `{key}`"));
                }
                let has_content = ["text", "translate", "keybind", "score", "selector", "nbt", "object"]
                    .iter()
                    .any(|key| map.contains_key(*key));
                if !has_content {
                    return Err("text component object has no content field".to_string());
                }
                Ok(value.clone())
            }
            other => Err(format!("text component must be a string, list or object, got {other}")),
        }
    }

    /// Decodes an `ItemStackTemplate` (`{id, count?, components?}` or a bare item id).
    pub(crate) fn item_template(&self, value: &Value) -> Decoded {
        if value.is_string() {
            let id = self.registry_id(Reg::Item, value)?;
            let mut out = Map::new();
            out.insert("id".to_string(), id);
            return Ok(Value::Object(out));
        }
        let object = value.as_object().ok_or("item stack template must be an object or id")?;
        Self::reject_unknown(object, &["id", "count", "components"])?;
        let mut out = Map::new();
        let id = object.get("id").ok_or("missing required field `id`")?;
        out.insert("id".to_string(), self.registry_id(Reg::Item, id)?);
        if let Some(count) = object.get("count") {
            let count = Self::int(count)?;
            if !(1..=99).contains(&count) {
                return Err(format!("Value must be within range [1;99]: {count}"));
            }
            if count != 1 {
                out.insert("count".to_string(), Value::from(count));
            }
        }
        if let Some(components) = object.get("components") {
            let patch = self.node(Node::DataComponentPatch, components)?;
            if patch.as_object().is_some_and(|map| !map.is_empty()) {
                out.insert("components".to_string(), patch);
            }
        }
        Ok(Value::Object(out))
    }

    /// Decodes and re-encodes one schema node (for codecs of other data types that embed
    /// shared Java codecs such as `DataComponentExactPredicate.CODEC`).
    pub(crate) fn decode_node(&self, node: Node, value: &Value) -> Decoded {
        self.node(node, value)
    }

    fn reject_unknown(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), String> {
        match object.keys().find(|key| !allowed.contains(&key.as_str())) {
            Some(key) => Err(format!("unknown field `{key}`")),
            None => Ok(()),
        }
    }

    /// Decodes and re-encodes a record shape (`rewards`, `display`, predicates).
    pub(crate) fn decode_rec(&self, rec: Rec, value: &Value) -> Decoded {
        self.record(fields_of(rec), value)
    }

    fn record(&self, fields: &[Field], value: &Value) -> Decoded {
        let object = value.as_object().ok_or("expected a JSON object")?;
        let names = fields.iter().map(|field| field.name).collect::<Vec<_>>();
        Self::reject_unknown(object, &names)?;
        let mut out = Map::new();
        for field in fields {
            match object.get(field.name) {
                Some(raw) => {
                    let encoded = self
                        .node(field.node, raw)
                        .map_err(|err| format!("{}: {err}", field.name))?;
                    if !Self::is_default(field.default, &encoded) {
                        out.insert(field.name.to_string(), encoded);
                    }
                }
                None if field.required => {
                    return Err(format!("missing required field `{}`", field.name));
                }
                None => {}
            }
        }
        Ok(Value::Object(out))
    }

    fn is_default(default: Def, encoded: &Value) -> bool {
        match default {
            Def::None => false,
            Def::EmptyObject => encoded.as_object().is_some_and(Map::is_empty),
            Def::EmptyList => encoded.as_array().is_some_and(Vec::is_empty),
            Def::Bool(expected) => encoded.as_bool() == Some(expected),
            Def::Int(expected) => encoded.as_i64() == Some(expected),
            Def::Str(expected) => encoded.as_str() == Some(expected),
        }
    }

    /// `Codec.INT`: any JSON number, narrowed with `Number.intValue()` semantics.
    fn int(value: &Value) -> Result<i64, String> {
        let number = value.as_number().ok_or_else(|| format!("expected a number, got {value}"))?;
        let wide = number
            .as_i64()
            .or_else(|| number.as_f64().map(|float| float as i64))
            .ok_or_else(|| format!("number {number} out of range"))?;
        Ok(i64::from(wide as i32))
    }

    fn double(value: &Value) -> Result<Number, String> {
        let number = value.as_number().ok_or_else(|| format!("expected a number, got {value}"))?;
        let float = number.as_f64().ok_or_else(|| format!("number {number} out of range"))?;
        Number::from_f64(float).ok_or_else(|| format!("number {number} is not finite"))
    }

    fn registry_id(&self, registry: Reg, value: &Value) -> Decoded {
        let raw = value.as_str().ok_or_else(|| format!("expected an id string, got {value}"))?;
        let id = Identifier::parse(raw)?;
        if !self.lookup.contains(registry, &id) {
            return Err(format!("unknown {registry:?} entry {id}"));
        }
        Ok(Value::String(id.to_string()))
    }

    fn holder_set(&self, registry: Reg, value: &Value) -> Decoded {
        match value {
            Value::String(raw) => {
                if let Some(tag) = raw.strip_prefix('#') {
                    let id = Identifier::parse(tag)?;
                    if !self.lookup.contains_tag(registry, &id) {
                        return Err(format!("unknown {registry:?} tag #{id}"));
                    }
                    return Ok(Value::String(format!("#{id}")));
                }
                self.registry_id(registry, value)
            }
            Value::Array(items) => {
                let ids = items
                    .iter()
                    .map(|item| self.registry_id(registry, item))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(if let [only] = ids.as_slice() { only.clone() } else { Value::Array(ids) })
            }
            other => Err(format!("expected an id, tag or list of ids, got {other}")),
        }
    }

    fn bounds(value: &Value, integer: bool) -> Decoded {
        let number = |raw: &Value| -> Result<Value, String> {
            if integer {
                Self::int(raw).map(Value::from)
            } else {
                Self::double(raw).map(Value::Number)
            }
        };
        if value.is_number() {
            return number(value);
        }
        let object = value.as_object().ok_or_else(|| format!("expected bounds, got {value}"))?;
        Self::reject_unknown(object, &["min", "max"])?;
        let min = object.get("min").map(number).transpose()?;
        let max = object.get("max").map(number).transpose()?;
        if let (Some(low), Some(high)) = (&min, &max) {
            if low == high {
                return Ok(low.clone());
            }
        }
        let mut out = Map::new();
        if let Some(low) = min {
            out.insert("min".to_string(), low);
        }
        if let Some(high) = max {
            out.insert("max".to_string(), high);
        }
        Ok(Value::Object(out))
    }

    fn map(&self, key: Key, node: Node, value: &Value) -> Decoded {
        let object = value.as_object().ok_or("expected a JSON object")?;
        let mut out = Map::new();
        for (name, raw) in object {
            let normalized = match key {
                Key::Reg(registry) => self
                    .registry_id(registry, &Value::String(name.clone()))?
                    .as_str()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                Key::Id => Identifier::parse(name)?.to_string(),
                Key::Str => name.clone(),
            };
            let encoded = self.node(node, raw).map_err(|err| format!("{name}: {err}"))?;
            out.insert(normalized, encoded);
        }
        Ok(Value::Object(out))
    }

    fn node(&self, node: Node, value: &Value) -> Decoded {
        match node {
            Node::Bool => value
                .as_bool()
                .map(Value::Bool)
                .ok_or_else(|| format!("expected a boolean, got {value}")),
            Node::Int => Self::int(value).map(Value::from),
            Node::PositiveInt => {
                let int = Self::int(value)?;
                if int > 0 {
                    Ok(Value::from(int))
                } else {
                    Err(format!("Value must be positive: {int}"))
                }
            }
            Node::Str => value
                .as_str()
                .map(|text| Value::String(text.to_string()))
                .ok_or_else(|| format!("expected a string, got {value}")),
            Node::Id => {
                let raw = value.as_str().ok_or_else(|| format!("expected an id, got {value}"))?;
                Ok(Value::String(Identifier::parse(raw)?.to_string()))
            }
            Node::Ints => Self::bounds(value, true),
            Node::Doubles => Self::bounds(value, false),
            Node::Reg(registry) => self.registry_id(registry, value),
            Node::Set(registry) => self.holder_set(registry, value),
            Node::Tag(registry) => {
                let raw = value.as_str().ok_or_else(|| format!("expected a tag id, got {value}"))?;
                let id = Identifier::parse(raw)?;
                if !self.lookup.contains_tag(registry, &id) {
                    return Err(format!("unknown {registry:?} tag {id}"));
                }
                Ok(Value::String(id.to_string()))
            }
            Node::Rec(rec) => self.record(fields_of(rec), value),
            Node::List(element) => {
                let items = value.as_array().ok_or("expected a JSON list")?;
                items
                    .iter()
                    .enumerate()
                    .map(|(index, item)| {
                        self.node(*element, item).map_err(|err| format!("[{index}]: {err}"))
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::Array)
            }
            Node::Map(key, element) => self.map(key, *element, value),
            Node::Ctx => self.context_aware(value, true),
            Node::CtxList => self.context_aware(value, false),
            Node::LootCondition => self.loot_condition(value),
            Node::Component => self.component(value),
            Node::ItemTemplate => self.item_template(value),
            Node::StateProperties => self.map(Key::Str, Node::StateValue, value),
            Node::StateValue => Self::state_value(value),
            Node::DataComponentsExact => self.data_components(value, false),
            Node::DataComponentPatch => self.data_components(value, true),
            Node::DataComponentPredicates => self.component_predicates(value),
            Node::TypeSpecific => self.type_specific(value),
            Node::Stats => self.stats(value),
            Node::GameModes => Self::game_modes(value),
            Node::AdvancementProgress => self.advancement_progress(value),
            Node::EntityTarget => Self::entity_target(value),
            Node::Frame => Self::frame(value),
        }
    }

    /// `StatePropertiesPredicate.ValueMatcher`: an exact string or `{min?, max?}` strings.
    fn state_value(value: &Value) -> Decoded {
        match value {
            Value::String(_) => Ok(value.clone()),
            Value::Object(object) => {
                Self::reject_unknown(object, &["min", "max"])?;
                object
                    .values()
                    .all(Value::is_string)
                    .then(|| value.clone())
                    .ok_or_else(|| "state range bounds must be strings".to_string())
            }
            other => Err(format!("state matcher must be a string or range, got {other}")),
        }
    }

    fn game_modes(value: &Value) -> Decoded {
        let items = value.as_array().ok_or("expected a JSON list")?;
        for item in items {
            let name = item.as_str().ok_or("game type must be a string")?;
            if !GAME_TYPES.contains(&name) {
                return Err(format!("unknown game type {name}"));
            }
        }
        Ok(value.clone())
    }

    fn frame(value: &Value) -> Decoded {
        match value.as_str() {
            Some("task" | "challenge" | "goal") => Ok(value.clone()),
            _ => Err(format!("Unknown element name: {value}")),
        }
    }

    fn entity_target(value: &Value) -> Decoded {
        match value.as_str() {
            Some(name) if ENTITY_TARGETS.contains(&name) => Ok(value.clone()),
            _ => Err(format!("Invalid entity target {value}")),
        }
    }

    /// `PlayerPredicate.AdvancementPredicate`: `true`/`false` or a criterion-to-bool map.
    fn advancement_progress(&self, value: &Value) -> Decoded {
        if value.is_boolean() {
            return Ok(value.clone());
        }
        self.map(Key::Str, Node::Bool, value)
    }

    /// `PlayerPredicate.StatMatcher`: `{type, stat, value?}` dispatched on the stat type.
    fn stats(&self, value: &Value) -> Decoded {
        let items = value.as_array().ok_or("expected a JSON list")?;
        let mut out = Vec::new();
        for item in items {
            let object = item.as_object().ok_or("stat matcher must be an object")?;
            Self::reject_unknown(object, &["type", "stat", "value"])?;
            let mut encoded = Map::new();
            let stat_type = object.get("type").ok_or("missing required field `type`")?;
            encoded.insert("type".to_string(), self.registry_id(Reg::StatType, stat_type)?);
            let stat = object.get("stat").ok_or("missing required field `stat`")?;
            encoded.insert("stat".to_string(), self.node(Node::Id, stat)?);
            if let Some(range) = object.get("value") {
                let range = Self::bounds(range, true)?;
                if !Self::is_default(ANY, &range) {
                    encoded.insert("value".to_string(), range);
                }
            }
            out.push(Value::Object(encoded));
        }
        Ok(Value::Array(out))
    }

    /// `EntitySubPredicate.CODEC`: a `type`-dispatched record.
    fn type_specific(&self, value: &Value) -> Decoded {
        let object = value.as_object().ok_or("expected a JSON object")?;
        let kind = object.get("type").ok_or("missing required field `type`")?;
        let kind = self.registry_id(Reg::EntitySubPredicateType, kind)?;
        let name = kind.as_str().unwrap_or_default();
        let fields = sub_predicate_fields(name)
            .ok_or_else(|| format!("entity sub-predicate {name} has no schema"))?;
        let mut body = object.clone();
        body.remove("type");
        let mut encoded = self.record(fields, &Value::Object(body))?;
        if let Some(map) = encoded.as_object_mut() {
            map.insert("type".to_string(), kind);
        }
        Ok(encoded)
    }

    /// Exact data component matchers / patches: known component keys, payload deferred.
    fn data_components(&self, value: &Value, patch: bool) -> Decoded {
        let object = value.as_object().ok_or("expected a JSON object")?;
        let mut out = Map::new();
        for (key, payload) in object {
            let raw = if patch { key.strip_prefix('!').unwrap_or(key) } else { key };
            let component = Identifier::parse(raw)?;
            if !self.lookup.contains(Reg::DataComponentType, &component) {
                return Err(format!("unknown data component type {component}"));
            }
            let prefix = if patch && key.starts_with('!') { "!" } else { "" };
            out.insert(format!("{prefix}{component}"), payload.clone());
        }
        Ok(Value::Object(out))
    }

    /// `DataComponentPredicate.CODEC`: partial matchers keyed by predicate type.
    fn component_predicates(&self, value: &Value) -> Decoded {
        let object = value.as_object().ok_or("expected a JSON object")?;
        let mut out = Map::new();
        for (key, payload) in object {
            let kind = self.registry_id(Reg::DataComponentPredicateType, &Value::String(key.clone()))?;
            let name = kind.as_str().unwrap_or_default().to_string();
            let encoded = match name.as_str() {
                "minecraft:enchantments" | "minecraft:stored_enchantments" => {
                    self.node(Node::List(&Node::Rec(Rec::Enchantment)), payload)
                }
                "minecraft:damage" => self.node(Node::Rec(Rec::ComponentDamage), payload),
                "minecraft:jukebox_playable" => self.node(Node::Rec(Rec::JukeboxPlayable), payload),
                // `VillagerTypePredicate.CODEC = RegistryCodecs.homogeneousList(VILLAGER_TYPE)`.
                "minecraft:villager/variant" => self.node(Node::Set(Reg::VillagerType), payload),
                _ => Err(format!("component predicate {name} has no schema")),
            }
            .map_err(|err| format!("{name}: {err}"))?;
            out.insert(name, encoded);
        }
        Ok(Value::Object(out))
    }

    /// `EntityPredicate.ADVANCEMENT_CODEC` (`allow_entity`) or `ContextAwarePredicate.CODEC`.
    fn context_aware(&self, value: &Value, allow_entity: bool) -> Decoded {
        if let Value::Array(items) = value {
            return items
                .iter()
                .map(|item| self.loot_condition(item))
                .collect::<Result<Vec<_>, _>>()
                .map(Value::Array);
        }
        if !allow_entity {
            return Err("context-aware predicate must be a list of loot conditions".to_string());
        }
        // `EntityPredicate.wrap`: the alternative re-encodes as an `entity_properties` check.
        let predicate = self.record(fields_of(Rec::Entity), value)?;
        let mut wrapped = Map::new();
        wrapped.insert("condition".to_string(), Value::from("minecraft:entity_properties"));
        wrapped.insert("entity".to_string(), Value::from("this"));
        wrapped.insert("predicate".to_string(), predicate);
        Ok(Value::Array(vec![Value::Object(wrapped)]))
    }

    /// `LootItemCondition.DIRECT_CODEC`, restricted to the types advancements use.
    fn loot_condition(&self, value: &Value) -> Decoded {
        let object = value.as_object().ok_or("loot condition must be a JSON object")?;
        let kind = object.get("condition").ok_or("missing required field `condition`")?;
        let kind = self.registry_id(Reg::LootConditionType, kind)?;
        let name = kind.as_str().unwrap_or_default().to_string();
        let fields: &'static [Field] = match name.as_str() {
            "minecraft:all_of" | "minecraft:any_of" => const {
                &[req("terms", Node::List(&Node::LootCondition))]
            }
            "minecraft:inverted" => const { &[req("term", Node::LootCondition)] },
            "minecraft:block_state_property" => const {
                &[req("block", Node::Reg(Reg::Block)), opt("properties", Node::StateProperties)]
            }
            "minecraft:entity_properties" => const {
                &[opt("predicate", Node::Rec(Rec::Entity)), req("entity", Node::EntityTarget)]
            }
            "minecraft:location_check" => const { &[
                opt("predicate", Node::Rec(Rec::Location)),
                dflt("offsetX", Node::Int, Def::Int(0)),
                dflt("offsetY", Node::Int, Def::Int(0)),
                dflt("offsetZ", Node::Int, Def::Int(0)),
            ] },
            "minecraft:match_tool" => const { &[opt("predicate", Node::Rec(Rec::Item))] },
            "minecraft:damage_source_properties" => const { &[opt("predicate", Node::Rec(Rec::DamageSource))] },
            "minecraft:killed_by_player" | "minecraft:survives_explosion" => &[],
            _ => return Err(format!("loot condition {name} has no advancement schema")),
        };
        let mut body = object.clone();
        body.remove("condition");
        let mut encoded = self.record(fields, &Value::Object(body))?;
        if let Some(map) = encoded.as_object_mut() {
            map.insert("condition".to_string(), kind);
        }
        Ok(encoded)
    }
}

/// Trigger implementations that have a condition schema (used by coverage tests).
#[cfg(test)]
pub(crate) fn supported_trigger_implementations() -> std::collections::BTreeSet<&'static str> {
    crate::advancement_trigger_registry::CriteriaTriggersModel::JAVA_ENTRIES
        .iter()
        .map(|entry| entry.implementation)
        .filter(|implementation| trigger_fields(implementation).is_some())
        .collect()
}
