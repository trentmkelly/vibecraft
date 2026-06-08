use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComponentTypeDef {
    id: &'static str,
    persistent: bool,
    network: bool,
    cache_encoding: bool,
    ignore_swap_animation: bool,
}

impl ComponentTypeDef {
    const fn new(id: &'static str) -> Self {
        Self {
            id,
            persistent: false,
            network: false,
            cache_encoding: false,
            ignore_swap_animation: false,
        }
    }

    const fn persistent(mut self) -> Self {
        self.persistent = true;
        self
    }

    const fn network(mut self) -> Self {
        self.network = true;
        self
    }

    const fn cache_encoding(mut self) -> Self {
        self.cache_encoding = true;
        self
    }

    const fn ignore_swap_animation(mut self) -> Self {
        self.ignore_swap_animation = true;
        self
    }

    fn is_transient(self) -> bool {
        !self.persistent
    }

    fn codec_or_throw(self) -> Result<&'static str, String> {
        self.persistent
            .then_some("codec")
            .ok_or_else(|| format!("{} is not a persistent component", self.id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TypedComponent {
    ty: &'static str,
    value: &'static str,
}

impl TypedComponent {
    fn to_java_string(self) -> String {
        format!("{}=>{}", self.ty, self.value)
    }

    fn encode_value(self) -> Result<&'static str, String> {
        component_type(self.ty)
            .ok_or_else(|| format!("Component of type {} is not encodable", self.ty))
            .and_then(ComponentTypeDef::codec_or_throw)
            .map(|_| self.value)
            .map_err(|_| format!("Component of type {} is not encodable", self.ty))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ComponentMapModel {
    values: BTreeMap<&'static str, &'static str>,
}

type ComponentMapValidator = fn(&ComponentMapModel) -> Result<(), String>;

impl ComponentMapModel {
    fn builder() -> ComponentMapBuilder {
        ComponentMapBuilder::default()
    }

    fn get(&self, ty: &'static str) -> Option<&'static str> {
        self.values.get(ty).copied()
    }

    fn get_or_default(&self, ty: &'static str, default: &'static str) -> &'static str {
        self.get(ty).unwrap_or(default)
    }

    fn get_typed(&self, ty: &'static str) -> Option<TypedComponent> {
        self.get(ty).map(|value| TypedComponent { ty, value })
    }

    fn has(&self, ty: &'static str) -> bool {
        self.values.contains_key(ty)
    }

    fn key_set(&self) -> BTreeSet<&'static str> {
        self.values.keys().copied().collect()
    }

    fn iter(&self) -> Vec<TypedComponent> {
        self.values
            .iter()
            .map(|(ty, value)| TypedComponent { ty, value })
            .collect()
    }

    fn filter(&self, predicate: impl Fn(&str) -> bool) -> Self {
        Self {
            values: self
                .values
                .iter()
                .filter_map(|(ty, value)| predicate(ty).then_some((*ty, *value)))
                .collect(),
        }
    }

    fn composite(prototype: &Self, overrides: &Self) -> Self {
        let mut values = prototype.values.clone();
        values.extend(overrides.values.iter().map(|(ty, value)| (*ty, *value)));
        Self { values }
    }

    fn persistent_value_map(&self) -> BTreeMap<&'static str, &'static str> {
        self.values
            .iter()
            .filter_map(|(ty, value)| {
                component_type(ty)
                    .is_some_and(|component| !component.is_transient())
                    .then_some((*ty, *value))
            })
            .collect()
    }
}

#[derive(Default)]
struct ComponentMapBuilder {
    values: BTreeMap<&'static str, &'static str>,
    validators: Vec<ComponentMapValidator>,
}

impl ComponentMapBuilder {
    fn set(mut self, ty: &'static str, value: Option<&'static str>) -> Self {
        if let Some(value) = value {
            self.values.insert(ty, value);
        } else {
            self.values.remove(ty);
        }
        self
    }

    fn add_all(mut self, map: &ComponentMapModel) -> Self {
        self.values
            .extend(map.values.iter().map(|(ty, value)| (*ty, *value)));
        self
    }

    fn add_validator(mut self, validator: ComponentMapValidator) -> Self {
        self.validators.push(validator);
        self
    }

    fn build(self) -> Result<ComponentMapModel, String> {
        let result = ComponentMapModel {
            values: self.values,
        };
        for validator in self.validators {
            validator(&result)?;
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PatchValue {
    Set(&'static str),
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ComponentPatchModel {
    values: BTreeMap<&'static str, PatchValue>,
}

impl ComponentPatchModel {
    fn builder() -> ComponentPatchBuilder {
        ComponentPatchBuilder::default()
    }

    fn get(&self, prototype: &ComponentMapModel, ty: &'static str) -> Option<&'static str> {
        match self.values.get(ty) {
            Some(PatchValue::Set(value)) => Some(*value),
            Some(PatchValue::Remove) => None,
            None => prototype.get(ty),
        }
    }

    fn forget(&self, predicate: impl Fn(&str) -> bool) -> Self {
        Self {
            values: self
                .values
                .iter()
                .filter_map(|(ty, value)| (!predicate(ty)).then_some((*ty, value.clone())))
                .collect(),
        }
    }

    fn split(&self) -> (ComponentMapModel, BTreeSet<&'static str>) {
        let mut added = BTreeMap::new();
        let mut removed = BTreeSet::new();
        for (ty, value) in &self.values {
            match value {
                PatchValue::Set(value) => {
                    added.insert(*ty, *value);
                }
                PatchValue::Remove => {
                    removed.insert(*ty);
                }
            }
        }
        (ComponentMapModel { values: added }, removed)
    }

    fn encode_order(&self) -> (Vec<TypedComponent>, Vec<&'static str>) {
        let mut positive = Vec::new();
        let mut negative = Vec::new();
        for (ty, value) in &self.values {
            match value {
                PatchValue::Set(value) => positive.push(TypedComponent { ty, value }),
                PatchValue::Remove => negative.push(*ty),
            }
        }
        (positive, negative)
    }

    fn codec_key(ty: &'static str, removed: bool) -> Result<String, String> {
        let component =
            component_type(ty).ok_or_else(|| format!("No component with type: '{}'", ty))?;
        if component.is_transient() {
            return Err(format!("'{}' is not a persistent component", ty));
        }
        Ok(if removed {
            format!("!{}", ty)
        } else {
            ty.to_string()
        })
    }

    fn java_string(&self) -> String {
        let entries: Vec<_> = self
            .values
            .iter()
            .map(|(ty, value)| match value {
                PatchValue::Set(value) => format!("{}=>{}", ty, value),
                PatchValue::Remove => format!("!{}", ty),
            })
            .collect();
        format!("{{{}}}", entries.join(", "))
    }
}

#[derive(Default)]
struct ComponentPatchBuilder {
    values: BTreeMap<&'static str, PatchValue>,
}

impl ComponentPatchBuilder {
    fn set(mut self, ty: &'static str, value: &'static str) -> Self {
        self.values.insert(ty, PatchValue::Set(value));
        self
    }

    fn remove(mut self, ty: &'static str) -> Self {
        self.values.insert(ty, PatchValue::Remove);
        self
    }

    fn set_all(mut self, components: &[TypedComponent]) -> Self {
        for component in components {
            self.values
                .insert(component.ty, PatchValue::Set(component.value));
        }
        self
    }

    fn build(self) -> ComponentPatchModel {
        ComponentPatchModel {
            values: self.values,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PatchedComponentMapModel {
    prototype: ComponentMapModel,
    patch: ComponentPatchModel,
}

impl PatchedComponentMapModel {
    fn new(prototype: ComponentMapModel) -> Self {
        Self {
            prototype,
            patch: ComponentPatchModel::default(),
        }
    }

    fn from_patch(prototype: ComponentMapModel, patch: ComponentPatchModel) -> Self {
        let mut result = Self::new(prototype);
        result.apply_patch(&patch);
        result
    }

    fn get(&self, ty: &'static str) -> Option<&'static str> {
        self.patch.get(&self.prototype, ty)
    }

    fn set(&mut self, ty: &'static str, value: Option<&'static str>) -> Option<&'static str> {
        let default = self.prototype.get(ty);
        let previous = self.get(ty);
        if value == default {
            self.patch.values.remove(ty);
        } else if let Some(value) = value {
            self.patch.values.insert(ty, PatchValue::Set(value));
        } else {
            self.patch.values.insert(ty, PatchValue::Remove);
        }
        previous
    }

    fn remove(&mut self, ty: &'static str) -> Option<&'static str> {
        self.set(ty, None)
    }

    fn apply_patch(&mut self, patch: &ComponentPatchModel) {
        for (ty, value) in &patch.values {
            match value {
                PatchValue::Set(value) => {
                    self.set(ty, Some(value));
                }
                PatchValue::Remove => {
                    self.remove(ty);
                }
            }
        }
    }

    fn restore_patch(&mut self, patch: ComponentPatchModel) {
        self.patch = patch;
    }

    fn clear_patch(&mut self) {
        self.patch.values.clear();
    }

    fn has_non_default(&self, ty: &'static str) -> bool {
        self.patch.values.contains_key(ty)
    }

    fn key_set(&self) -> BTreeSet<&'static str> {
        let mut keys = self.prototype.key_set();
        for (ty, value) in &self.patch.values {
            match value {
                PatchValue::Set(_) => {
                    keys.insert(*ty);
                }
                PatchValue::Remove => {
                    keys.remove(ty);
                }
            }
        }
        keys
    }

    fn iter(&self) -> Vec<TypedComponent> {
        let mut components = Vec::new();
        for (ty, value) in &self.patch.values {
            if let PatchValue::Set(value) = value {
                components.push(TypedComponent { ty, value });
            }
        }
        for component in self.prototype.iter() {
            if !self.patch.values.contains_key(component.ty) {
                components.push(component);
            }
        }
        components
    }

    fn as_patch(&self) -> ComponentPatchModel {
        self.patch.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExactPredicateModel {
    expected: Vec<TypedComponent>,
}

impl ExactPredicateModel {
    fn empty() -> Self {
        Self {
            expected: Vec::new(),
        }
    }

    fn expect(ty: &'static str, value: &'static str) -> Self {
        Self {
            expected: vec![TypedComponent { ty, value }],
        }
    }

    fn all_of(components: &ComponentMapModel) -> Self {
        Self {
            expected: components.iter(),
        }
    }

    fn some_of(components: &ComponentMapModel, types: &[&'static str]) -> Self {
        Self {
            expected: types
                .iter()
                .filter_map(|ty| components.get_typed(ty))
                .collect(),
        }
    }

    fn test(&self, actual: &ComponentMapModel) -> bool {
        self.expected
            .iter()
            .all(|expected| actual.get(expected.ty) == Some(expected.value))
    }

    fn always_matches(&self) -> bool {
        self.expected.is_empty()
    }

    fn as_patch(&self) -> ComponentPatchModel {
        ComponentPatchModel::builder()
            .set_all(&self.expected)
            .build()
    }
}

#[derive(Debug, Default)]
struct ExactPredicateBuilder {
    expected: Vec<TypedComponent>,
}

impl ExactPredicateBuilder {
    fn expect(mut self, ty: &'static str, value: &'static str) -> Result<Self, String> {
        if self.expected.iter().any(|component| component.ty == ty) {
            return Err(format!("Predicate already has component of type: '{}'", ty));
        }
        self.expected.push(TypedComponent { ty, value });
        Ok(self)
    }

    fn build(self) -> ExactPredicateModel {
        ExactPredicateModel {
            expected: self.expected,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderModel {
    key: &'static str,
    components: ComponentMapModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComponentLookupModel {
    elements: Vec<HolderModel>,
    scanned: BTreeSet<&'static str>,
}

impl ComponentLookupModel {
    fn new(elements: Vec<HolderModel>) -> Self {
        Self {
            elements,
            scanned: BTreeSet::new(),
        }
    }

    fn find_all(&mut self, ty: &'static str, value: Option<&'static str>) -> Vec<&'static str> {
        self.scanned.insert(ty);
        self.elements
            .iter()
            .filter_map(|holder| {
                holder.components.get(ty).and_then(|actual| {
                    value
                        .map(|expected| expected == actual)
                        .unwrap_or(true)
                        .then_some(holder.key)
                })
            })
            .collect()
    }

    fn find_matching(
        &mut self,
        ty: &'static str,
        predicate: impl Fn(&str) -> bool,
    ) -> Vec<&'static str> {
        self.scanned.insert(ty);
        self.elements
            .iter()
            .filter_map(|holder| {
                holder
                    .components
                    .get(ty)
                    .filter(|value| predicate(value))
                    .map(|_| holder.key)
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingComponentsModel {
    registry: &'static str,
    entries: Vec<(&'static str, ComponentMapModel)>,
}

#[derive(Default)]
struct ComponentInitializersModel {
    initializers: Vec<(&'static str, &'static str, &'static str, &'static str)>,
}

impl ComponentInitializersModel {
    fn add(
        &mut self,
        registry: &'static str,
        key: &'static str,
        ty: &'static str,
        value: &'static str,
    ) {
        self.initializers.push((registry, key, ty, value));
    }

    fn build(
        &self,
        registries: &[(&'static str, Vec<&'static str>)],
    ) -> Vec<PendingComponentsModel> {
        registries
            .iter()
            .map(|(registry, keys)| {
                let entries =
                    keys.iter()
                        .map(|key| {
                            let mut builder = ComponentMapModel::builder();
                            for (_, _, ty, value) in self.initializers.iter().filter(
                                |(entry_registry, entry_key, _, _)| {
                                    entry_registry == registry && entry_key == key
                                },
                            ) {
                                builder = builder.set(ty, Some(value));
                            }
                            (*key, builder.build().expect("initializer values are valid"))
                        })
                        .collect();
                PendingComponentsModel { registry, entries }
            })
            .collect()
    }
}

const COMPONENTS: &[ComponentTypeDef] = &[
    ComponentTypeDef::new("custom_data").persistent(),
    ComponentTypeDef::new("max_stack_size")
        .persistent()
        .network(),
    ComponentTypeDef::new("max_damage").persistent().network(),
    ComponentTypeDef::new("damage")
        .persistent()
        .network()
        .ignore_swap_animation(),
    ComponentTypeDef::new("unbreakable").persistent().network(),
    ComponentTypeDef::new("use_effects").persistent().network(),
    ComponentTypeDef::new("custom_name")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("minimum_attack_charge")
        .persistent()
        .network(),
    ComponentTypeDef::new("damage_type").persistent().network(),
    ComponentTypeDef::new("item_name")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("item_model")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("lore")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("rarity").persistent().network(),
    ComponentTypeDef::new("enchantments")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("can_place_on")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("can_break")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("attribute_modifiers")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("custom_model_data")
        .persistent()
        .network(),
    ComponentTypeDef::new("tooltip_display")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("repair_cost").persistent().network(),
    ComponentTypeDef::new("creative_slot_lock").network(),
    ComponentTypeDef::new("enchantment_glint_override")
        .persistent()
        .network(),
    ComponentTypeDef::new("intangible_projectile").persistent(),
    ComponentTypeDef::new("food")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("consumable")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("use_remainder")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("use_cooldown")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("damage_resistant")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("tool")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("weapon")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("attack_range")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("enchantable")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("equippable")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("repairable")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("glider").persistent().network(),
    ComponentTypeDef::new("tooltip_style")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("death_protection")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("blocks_attacks")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("piercing_weapon")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("kinetic_weapon")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("swing_animation")
        .persistent()
        .network(),
    ComponentTypeDef::new("additional_trade_cost").network(),
    ComponentTypeDef::new("stored_enchantments")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("dye").persistent().network(),
    ComponentTypeDef::new("dyed_color").persistent().network(),
    ComponentTypeDef::new("map_color").persistent().network(),
    ComponentTypeDef::new("map_id").persistent().network(),
    ComponentTypeDef::new("map_decorations")
        .persistent()
        .cache_encoding(),
    ComponentTypeDef::new("map_post_processing").network(),
    ComponentTypeDef::new("charged_projectiles")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("bundle_contents")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("potion_contents")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("potion_duration_scale")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("suspicious_stew_effects")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("writable_book_content")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("written_book_content")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("trim")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("debug_stick_state")
        .persistent()
        .cache_encoding(),
    ComponentTypeDef::new("entity_data").persistent().network(),
    ComponentTypeDef::new("bucket_entity_data")
        .persistent()
        .network(),
    ComponentTypeDef::new("block_entity_data")
        .persistent()
        .network(),
    ComponentTypeDef::new("instrument")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("provides_trim_material")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("ominous_bottle_amplifier")
        .persistent()
        .network(),
    ComponentTypeDef::new("jukebox_playable")
        .persistent()
        .network(),
    ComponentTypeDef::new("provides_banner_patterns")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("recipes")
        .persistent()
        .cache_encoding(),
    ComponentTypeDef::new("lodestone_tracker")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("firework_explosion")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("fireworks")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("profile")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("note_block_sound")
        .persistent()
        .network(),
    ComponentTypeDef::new("banner_patterns")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("base_color").persistent().network(),
    ComponentTypeDef::new("pot_decorations")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("container")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("block_state")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("bees")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("lock").persistent(),
    ComponentTypeDef::new("container_loot").persistent(),
    ComponentTypeDef::new("break_sound")
        .persistent()
        .network()
        .cache_encoding(),
    ComponentTypeDef::new("villager/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("wolf/variant").persistent().network(),
    ComponentTypeDef::new("wolf/sound_variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("wolf/collar").persistent().network(),
    ComponentTypeDef::new("fox/variant").persistent().network(),
    ComponentTypeDef::new("salmon/size").persistent().network(),
    ComponentTypeDef::new("parrot/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("tropical_fish/pattern")
        .persistent()
        .network(),
    ComponentTypeDef::new("tropical_fish/base_color")
        .persistent()
        .network(),
    ComponentTypeDef::new("tropical_fish/pattern_color")
        .persistent()
        .network(),
    ComponentTypeDef::new("mooshroom/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("rabbit/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("pig/variant").persistent().network(),
    ComponentTypeDef::new("pig/sound_variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("cow/variant").persistent().network(),
    ComponentTypeDef::new("cow/sound_variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("chicken/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("chicken/sound_variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("zombie_nautilus/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("frog/variant").persistent().network(),
    ComponentTypeDef::new("horse/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("painting/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("llama/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("axolotl/variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("cat/variant").persistent().network(),
    ComponentTypeDef::new("cat/sound_variant")
        .persistent()
        .network(),
    ComponentTypeDef::new("cat/collar").persistent().network(),
    ComponentTypeDef::new("sheep/color").persistent().network(),
    ComponentTypeDef::new("shulker/color")
        .persistent()
        .network(),
];

const COMMON_ITEM_COMPONENTS: &[(&str, &str)] = &[
    ("max_stack_size", "64"),
    ("lore", "ItemLore.EMPTY"),
    ("enchantments", "ItemEnchantments.EMPTY"),
    ("repair_cost", "0"),
    ("use_effects", "UseEffects.DEFAULT"),
    ("attribute_modifiers", "ItemAttributeModifiers.EMPTY"),
    ("rarity", "Rarity.COMMON"),
    ("break_sound", "SoundEvents.ITEM_BREAK"),
    ("tooltip_display", "TooltipDisplay.DEFAULT"),
    ("swing_animation", "SwingAnimation.DEFAULT"),
];

const CORE_COMPONENT_PACKAGE_NULL_MARKED: bool = true;
const DATA_COMPONENTS_BOOTSTRAP_RETURN: &str = "custom_data";
const ENCODER_CACHE_SIZE: usize = 512;

fn component_type(id: &str) -> Option<ComponentTypeDef> {
    COMPONENTS
        .iter()
        .copied()
        .find(|component| component.id == id)
}

fn common_item_components() -> ComponentMapModel {
    ComponentMapModel {
        values: COMMON_ITEM_COMPONENTS.iter().copied().collect(),
    }
}

fn core_component_package_is_null_marked() -> bool {
    CORE_COMPONENT_PACKAGE_NULL_MARKED
}

#[cfg(test)]
mod tests;
