//! Runtime function models, shared by JSON decoding and evaluation.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub enum LootFunction {
    SetCount(NumberProvider),
    LimitCount {
        min: i32,
        max: i32,
    },
    AddLootingBonus {
        per_level: NumberProvider,
        limit: Option<i32>,
    },
    ApplyFortuneBonus {
        per_level: NumberProvider,
        limit: Option<i32>,
    },
    /// `ApplyBonusCount`: `formula` over the tool's level of `enchantment`.
    ApplyBonus {
        enchantment: String,
        formula: LootBonusFormula,
    },
    /// `SetItemCountFunction` with `add: true`.
    AddCount(NumberProvider),
    /// `EnchantedCountIncreaseFunction`: grows the count by
    /// `round(level * count)` for the attacker's level of `enchantment`, capped at
    /// `limit` when it is positive.
    EnchantedCountIncrease {
        enchantment: String,
        count: NumberProvider,
        limit: i32,
    },
    SetItem(String),
    /// `SetEnchantmentsFunction`: level providers in declaration order.
    SetEnchantments {
        enchantments: Vec<(String, NumberProvider)>,
        add: bool,
    },
    /// `SetItemDamageFunction`.
    SetDamage {
        damage: NumberProvider,
        add: bool,
    },
    SetNbt(HashMap<String, String>),
    /// `EnchantWithLevelsFunction`; `options` absent means every enchantment.
    EnchantWithLevels {
        levels: NumberProvider,
        options: Option<HolderSet>,
        include_additional_cost_component: bool,
    },
    /// `EnchantRandomlyFunction`; `options` absent means every enchantment.
    EnchantRandomly {
        options: Option<HolderSet>,
        only_compatible: bool,
        include_additional_cost_component: bool,
    },
    /// `SmeltItemFunction`.
    SmeltItem {
        use_input_count: bool,
    },
    /// `CopyComponentsFunction`.
    CopyComponents {
        source: ComponentSource,
        include: Option<Vec<String>>,
        exclude: Option<Vec<String>>,
    },
    CopyName {
        source: String,
    },
    CopyNbt {
        provider: NbtProvider,
        target: String,
    },
    /// `SetContainerContents`: expand every entry without weighted pool selection.
    SetContainerContents {
        component: ContentsComponent,
        entries: Vec<LootEntry>,
    },
    /// `ModifyContainerContents`: edit nonempty slots and clamp each result's size.
    ModifyContainerContents {
        component: ContentsComponent,
        modifier: Box<LootFunction>,
    },
    SetContents(Vec<LootStack>),
    ModifyContents(Vec<LootFunction>),
    ExplorationMap {
        destination: String,
        decoration: String,
    },
    FillPlayerHead,
    /// `CopyBlockState`: `properties` are those of `block` that exist on it.
    CopyState {
        block: String,
        properties: Vec<String>,
    },
    SetAttributes(Vec<String>),
    SetBannerPatterns(Vec<String>),
    SetBookContents {
        title: String,
        author: String,
        pages: Vec<String>,
    },
    /// `SetComponentsFunction`: the `DataComponentPatch` edits.
    SetComponents(Vec<ComponentEdit>),
    /// `SetInstrumentFunction`.
    SetInstrument(HolderSet),
    SetLore(Vec<String>),
    /// Decoded `SetLoreFunction` with Java list-operation semantics.
    EditLore(LootListEdit<serde_json::Value>),
    /// Decoded `SetWrittenBookPagesFunction`, preserving the book metadata.
    EditWrittenBookPages(LootListEdit<serde_json::Value>),
    /// Decoded `SetWritableBookPagesFunction`, preserving filterable page values.
    EditWritableBookPages(LootListEdit<serde_json::Value>),
    /// Decoded `SetCustomModelDataFunction`, including dynamic number providers.
    EditCustomModelData(CustomModelDataEdit),
    /// `SetNameFunction`: `name` is the compact JSON of the text component.
    SetName {
        name: Option<String>,
        target: NameTarget,
    },
    SetPotion(String),
    SetRandomPotion(Vec<String>),
    /// `SetStewEffectFunction`.
    SetStewEffects(Vec<StewEffect>),
    SetRandomDyes(Vec<String>),
    SetWrittenBookPages(Vec<String>),
    SetWritableBookPages(Vec<String>),
    SetBookCover {
        title: String,
        author: String,
    },
    ToggleTooltips(Vec<String>),
    SetFireworkExplosions(Vec<String>),
    SetFireworks {
        flight_duration: i32,
        explosions: Vec<String>,
    },
    SetOminousBottleAmplifier(NumberProvider),
    SetCustomModelData(String),
    SetLootTable(String),
    Reference(String),
    Discard,
    ApplyExplosionDecay,
    Filtered {
        // Conditions can contain recursive entity predicates; keep conditional
        // functions from inflating every other function variant's allocation.
        condition: Box<LootCondition>,
        function: Box<LootFunction>,
    },
    Sequence(Vec<LootFunction>),
    /// A `LootItemFunctions` type (or parameterisation) the runtime cannot apply
    /// yet. The codec validated the document; the function leaves the stack
    /// untouched and marks the table as not fully modeled.
    /// TODO(loot-json-codec): model these functions.
    Unmodeled {
        kind: String,
        data: serde_json::Value,
    },
}
