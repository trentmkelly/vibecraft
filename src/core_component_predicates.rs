use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
enum ComponentValue {
    Text(&'static str),
    CustomData(&'static str),
    Damage {
        damage: i32,
        max_damage: i32,
    },
    Enchantments(BTreeMap<&'static str, i32>),
    Items(Vec<ItemInstance>),
    FireworkExplosion(FireworkExplosionModel),
    Fireworks(FireworksModel),
    JukeboxPlayable {
        song_key: Option<&'static str>,
    },
    Potion {
        potion_key: Option<&'static str>,
    },
    Trim {
        material: &'static str,
        pattern: &'static str,
    },
    Holder(&'static str),
    AttributeModifiers(Vec<AttributeModifierEntry>),
    WritableBook(Vec<FilterableText>),
    WrittenBook(WrittenBookModel),
}

#[derive(Debug, Clone, PartialEq, Default)]
struct ComponentGetterModel {
    values: BTreeMap<&'static str, ComponentValue>,
}

impl ComponentGetterModel {
    fn with(mut self, component: &'static str, value: ComponentValue) -> Self {
        self.values.insert(component, value);
        self
    }

    fn get(&self, component: &'static str) -> Option<&ComponentValue> {
        self.values.get(component)
    }

    fn has(&self, component: &'static str) -> bool {
        self.get(component).is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IntBounds {
    min: Option<i32>,
    max: Option<i32>,
}

impl IntBounds {
    const ANY: Self = Self {
        min: None,
        max: None,
    };

    const fn exactly(value: i32) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    const fn at_least(value: i32) -> Self {
        Self {
            min: Some(value),
            max: None,
        }
    }

    fn matches(self, value: i32) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DoubleBounds {
    min: Option<f64>,
    max: Option<f64>,
}

impl DoubleBounds {
    const ANY: Self = Self {
        min: None,
        max: None,
    };

    const fn exactly(value: f64) -> Self {
        Self {
            min: Some(value),
            max: Some(value),
        }
    }

    fn matches(self, value: f64) -> bool {
        self.min.is_none_or(|min| value >= min) && self.max.is_none_or(|max| value <= max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemInstance {
    id: &'static str,
    empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilterableText {
    raw: &'static str,
    filtered: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FireworkExplosionModel {
    shape: &'static str,
    twinkle: bool,
    trail: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FireworksModel {
    explosions: Vec<FireworkExplosionModel>,
    flight_duration: i32,
}

#[derive(Debug, Clone, PartialEq)]
struct AttributeModifierEntry {
    attribute: &'static str,
    id: &'static str,
    amount: f64,
    operation: &'static str,
    slot: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WrittenBookModel {
    pages: Vec<FilterableText>,
    author: &'static str,
    title_raw: &'static str,
    generation: i32,
    resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PredicateTypeModel {
    AnyValue { component_type: &'static str },
    Concrete { predicate_type: &'static str },
}

impl PredicateTypeModel {
    fn key(&self) -> &'static str {
        match self {
            Self::AnyValue { component_type } => component_type,
            Self::Concrete { predicate_type } => predicate_type,
        }
    }

    fn unpack_type(&self) -> EitherType {
        match self {
            Self::AnyValue { component_type } => EitherType::Component(component_type),
            Self::Concrete { predicate_type } => EitherType::Predicate(predicate_type),
        }
    }

    fn copy_or_create_type(either: EitherType) -> Self {
        match either {
            EitherType::Component(component_type) => Self::AnyValue { component_type },
            EitherType::Predicate(predicate_type) => Self::Concrete { predicate_type },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EitherType {
    Predicate(&'static str),
    Component(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
enum DataComponentPredicateModel {
    AnyValue { component_type: &'static str },
    Damage(DamagePredicateModel),
    Enchantments(EnchantmentsPredicateModel),
    StoredEnchantments(EnchantmentsPredicateModel),
    Potions(HolderSetPredicate),
    CustomData(CustomDataPredicateModel),
    Container(ItemCollectionPredicate),
    BundleContents(ItemCollectionPredicate),
    FireworkExplosion(FireworkExplosionPredicate),
    Fireworks(FireworksPredicate),
    WritableBook(WritableBookPredicate),
    WrittenBook(WrittenBookPredicate),
    AttributeModifiers(AttributeModifiersPredicate),
    Trim(TrimPredicate),
    JukeboxPlayable(JukeboxPlayablePredicate),
    VillagerVariant(HolderSetPredicate),
}

impl DataComponentPredicateModel {
    fn matches(&self, components: &ComponentGetterModel) -> bool {
        match self {
            Self::AnyValue { component_type } => components.has(component_type),
            Self::Damage(predicate) => predicate.matches(components),
            Self::CustomData(predicate) => predicate.matches(components),
            Self::Enchantments(predicate) => {
                predicate.matches_component(components, "enchantments")
            }
            Self::StoredEnchantments(predicate) => {
                predicate.matches_component(components, "stored_enchantments")
            }
            Self::Potions(predicate) => matches!(
                components.get("potion_contents"),
                Some(ComponentValue::Potion {
                    potion_key: Some(key)
                }) if predicate.contains(key)
            ),
            Self::Container(predicate) => matches!(
                components.get("container"),
                Some(ComponentValue::Items(items)) if predicate.matches(&items.iter().filter(|item| !item.empty).cloned().collect::<Vec<_>>())
            ),
            Self::BundleContents(predicate) => matches!(
                components.get("bundle_contents"),
                Some(ComponentValue::Items(items)) if predicate.matches(items)
            ),
            Self::FireworkExplosion(predicate) => matches!(
                components.get("firework_explosion"),
                Some(ComponentValue::FireworkExplosion(value)) if predicate.matches(value)
            ),
            Self::Fireworks(predicate) => matches!(
                components.get("fireworks"),
                Some(ComponentValue::Fireworks(value)) if predicate.matches(value)
            ),
            Self::WritableBook(predicate) => matches!(
                components.get("writable_book_content"),
                Some(ComponentValue::WritableBook(pages)) if predicate.matches(pages)
            ),
            Self::WrittenBook(predicate) => matches!(
                components.get("written_book_content"),
                Some(ComponentValue::WrittenBook(book)) if predicate.matches(book)
            ),
            Self::AttributeModifiers(predicate) => matches!(
                components.get("attribute_modifiers"),
                Some(ComponentValue::AttributeModifiers(entries)) if predicate.matches(entries)
            ),
            Self::Trim(predicate) => matches!(
                components.get("trim"),
                Some(ComponentValue::Trim { material, pattern }) if predicate.matches(material, pattern)
            ),
            Self::JukeboxPlayable(predicate) => matches!(
                components.get("jukebox_playable"),
                Some(ComponentValue::JukeboxPlayable { song_key }) if predicate.matches(*song_key)
            ),
            Self::VillagerVariant(predicate) => matches!(
                components.get("villager/variant"),
                Some(ComponentValue::Holder(key)) if predicate.contains(key)
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct SinglePredicateModel {
    predicate_type: PredicateTypeModel,
    predicate: DataComponentPredicateModel,
}

fn predicate_map_from_singles(
    singles: Vec<SinglePredicateModel>,
) -> Result<BTreeMap<&'static str, DataComponentPredicateModel>, String> {
    let mut result = BTreeMap::new();
    for single in singles {
        if result
            .insert(single.predicate_type.key(), single.predicate)
            .is_some()
        {
            return Err(format!(
                "Duplicate predicate type '{}'",
                single.predicate_type.key()
            ));
        }
    }
    Ok(result)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomDataPredicateModel {
    required_fragment: &'static str,
}

impl CustomDataPredicateModel {
    fn matches(&self, components: &ComponentGetterModel) -> bool {
        matches!(
            components.get("custom_data"),
            Some(ComponentValue::CustomData(value)) if value.contains(self.required_fragment)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DamagePredicateModel {
    durability: IntBounds,
    damage: IntBounds,
}

impl DamagePredicateModel {
    fn durability(range: IntBounds) -> Self {
        Self {
            durability: range,
            damage: IntBounds::ANY,
        }
    }

    fn matches(&self, components: &ComponentGetterModel) -> bool {
        match components.get("damage") {
            Some(ComponentValue::Damage { damage, max_damage }) => {
                self.durability.matches(max_damage - damage) && self.damage.matches(*damage)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HolderSetPredicate {
    allowed: BTreeSet<&'static str>,
}

impl HolderSetPredicate {
    fn new(values: &[&'static str]) -> Self {
        Self {
            allowed: values.iter().copied().collect(),
        }
    }

    fn contains(&self, key: &str) -> bool {
        self.allowed.contains(key)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnchantmentPredicateModel {
    id: &'static str,
    level: IntBounds,
}

impl EnchantmentPredicateModel {
    fn contained_in(&self, enchantments: &BTreeMap<&'static str, i32>) -> bool {
        enchantments
            .get(self.id)
            .is_some_and(|level| self.level.matches(*level))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnchantmentsPredicateModel {
    enchantments: Vec<EnchantmentPredicateModel>,
}

impl EnchantmentsPredicateModel {
    fn matches_component(
        &self,
        components: &ComponentGetterModel,
        component: &'static str,
    ) -> bool {
        match components.get(component) {
            Some(ComponentValue::Enchantments(enchantments)) => self
                .enchantments
                .iter()
                .all(|predicate| predicate.contained_in(enchantments)),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemCollectionPredicate {
    required_item: Option<&'static str>,
    min_count: usize,
}

impl ItemCollectionPredicate {
    fn any() -> Self {
        Self {
            required_item: None,
            min_count: 0,
        }
    }

    fn matches(&self, items: &[ItemInstance]) -> bool {
        items.len() >= self.min_count
            && self
                .required_item
                .is_none_or(|id| items.iter().any(|item| item.id == id))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FireworkExplosionPredicate {
    shape: Option<&'static str>,
    twinkle: Option<bool>,
    trail: Option<bool>,
}

impl FireworkExplosionPredicate {
    fn matches(&self, value: &FireworkExplosionModel) -> bool {
        self.shape.is_none_or(|shape| shape == value.shape)
            && self.twinkle.is_none_or(|twinkle| twinkle == value.twinkle)
            && self.trail.is_none_or(|trail| trail == value.trail)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FireworksPredicate {
    explosion: Option<FireworkExplosionPredicate>,
    flight_duration: IntBounds,
}

impl FireworksPredicate {
    fn matches(&self, value: &FireworksModel) -> bool {
        self.explosion.as_ref().is_none_or(|predicate| {
            value
                .explosions
                .iter()
                .any(|explosion| predicate.matches(explosion))
        }) && self.flight_duration.matches(value.flight_duration)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WritableBookPredicate {
    required_page: Option<&'static str>,
}

impl WritableBookPredicate {
    fn matches(&self, pages: &[FilterableText]) -> bool {
        self.required_page
            .is_none_or(|contents| pages.iter().any(|page| page.raw == contents))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WrittenBookPredicate {
    required_page: Option<&'static str>,
    author: Option<&'static str>,
    title: Option<&'static str>,
    generation: IntBounds,
    resolved: Option<bool>,
}

impl WrittenBookPredicate {
    fn matches(&self, value: &WrittenBookModel) -> bool {
        self.author.is_none_or(|author| author == value.author)
            && self.title.is_none_or(|title| title == value.title_raw)
            && self.generation.matches(value.generation)
            && self
                .resolved
                .is_none_or(|resolved| resolved == value.resolved)
            && self
                .required_page
                .is_none_or(|contents| value.pages.iter().any(|page| page.raw == contents))
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AttributeEntryPredicate {
    attribute: Option<HolderSetPredicate>,
    id: Option<&'static str>,
    amount: DoubleBounds,
    operation: Option<&'static str>,
    slot: Option<&'static str>,
}

impl AttributeEntryPredicate {
    fn test(&self, value: &AttributeModifierEntry) -> bool {
        self.attribute
            .as_ref()
            .is_none_or(|attributes| attributes.contains(value.attribute))
            && self.id.is_none_or(|id| id == value.id)
            && self.amount.matches(value.amount)
            && self
                .operation
                .is_none_or(|operation| operation == value.operation)
            && self.slot.is_none_or(|slot| slot == value.slot)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct AttributeModifiersPredicate {
    entry: Option<AttributeEntryPredicate>,
}

impl AttributeModifiersPredicate {
    fn matches(&self, entries: &[AttributeModifierEntry]) -> bool {
        self.entry
            .as_ref()
            .is_none_or(|entry| entries.iter().any(|value| entry.test(value)))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TrimPredicate {
    material: Option<HolderSetPredicate>,
    pattern: Option<HolderSetPredicate>,
}

impl TrimPredicate {
    fn matches(&self, material: &str, pattern: &str) -> bool {
        self.material
            .as_ref()
            .is_none_or(|allowed| allowed.contains(material))
            && self
                .pattern
                .as_ref()
                .is_none_or(|allowed| allowed.contains(pattern))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JukeboxPlayablePredicate {
    song: Option<HolderSetPredicate>,
}

impl JukeboxPlayablePredicate {
    fn any() -> Self {
        Self { song: None }
    }

    fn matches(&self, song_key: Option<&str>) -> bool {
        self.song
            .as_ref()
            .is_none_or(|songs| song_key.is_some_and(|key| songs.contains(key)))
    }
}

const PREDICATE_REGISTRY: &[&str] = &[
    "damage",
    "enchantments",
    "stored_enchantments",
    "potion_contents",
    "custom_data",
    "container",
    "bundle_contents",
    "firework_explosion",
    "fireworks",
    "writable_book_content",
    "written_book_content",
    "attribute_modifiers",
    "trim",
    "jukebox_playable",
    "villager/variant",
];

const DATA_COMPONENT_PREDICATES_BOOTSTRAP_RETURN: &str = "damage";
const COMPONENT_PREDICATES_PACKAGE_NULL_MARKED: bool = true;

fn component_predicates_package_is_null_marked() -> bool {
    COMPONENT_PREDICATES_PACKAGE_NULL_MARKED
}

#[cfg(test)]
mod tests {
    use super::*;

    fn firework(shape: &'static str, twinkle: bool, trail: bool) -> FireworkExplosionModel {
        FireworkExplosionModel {
            shape,
            twinkle,
            trail,
        }
    }

    #[test]
    fn predicate_type_codec_unpacks_any_value_as_component_type() {
        let any = PredicateTypeModel::copy_or_create_type(EitherType::Component("custom_name"));
        let concrete = PredicateTypeModel::copy_or_create_type(EitherType::Predicate("damage"));

        assert_eq!(
            any,
            PredicateTypeModel::AnyValue {
                component_type: "custom_name"
            }
        );
        assert_eq!(any.unpack_type(), EitherType::Component("custom_name"));
        assert_eq!(
            concrete,
            PredicateTypeModel::Concrete {
                predicate_type: "damage"
            }
        );
        assert_eq!(concrete.unpack_type(), EitherType::Predicate("damage"));
    }

    #[test]
    fn predicate_registry_matches_java_bootstrap_order() {
        assert_eq!(PREDICATE_REGISTRY.len(), 15);
        assert_eq!(
            PREDICATE_REGISTRY[0],
            DATA_COMPONENT_PREDICATES_BOOTSTRAP_RETURN
        );
        assert_eq!(PREDICATE_REGISTRY[14], "villager/variant");
        assert!(component_predicates_package_is_null_marked());
    }

    #[test]
    fn stream_codec_single_list_rejects_duplicate_predicate_types() {
        let singles = vec![
            SinglePredicateModel {
                predicate_type: PredicateTypeModel::Concrete {
                    predicate_type: "damage",
                },
                predicate: DataComponentPredicateModel::Damage(DamagePredicateModel {
                    durability: IntBounds::ANY,
                    damage: IntBounds::exactly(4),
                }),
            },
            SinglePredicateModel {
                predicate_type: PredicateTypeModel::Concrete {
                    predicate_type: "damage",
                },
                predicate: DataComponentPredicateModel::Damage(DamagePredicateModel {
                    durability: IntBounds::ANY,
                    damage: IntBounds::exactly(5),
                }),
            },
        ];

        assert_eq!(
            predicate_map_from_singles(singles).unwrap_err(),
            "Duplicate predicate type 'damage'"
        );
    }

    #[test]
    fn any_custom_data_and_damage_predicates_match_java_component_getter_behavior() {
        let components = ComponentGetterModel::default()
            .with("custom_name", ComponentValue::Text("Sword"))
            .with("custom_data", ComponentValue::CustomData("{Charged:1b}"))
            .with(
                "damage",
                ComponentValue::Damage {
                    damage: 7,
                    max_damage: 20,
                },
            );

        assert!(DataComponentPredicateModel::AnyValue {
            component_type: "custom_name"
        }
        .matches(&components));
        assert!(!DataComponentPredicateModel::AnyValue {
            component_type: "lore"
        }
        .matches(&components));
        assert!(
            DataComponentPredicateModel::CustomData(CustomDataPredicateModel {
                required_fragment: "Charged"
            })
            .matches(&components)
        );
        assert!(
            DataComponentPredicateModel::Damage(DamagePredicateModel::durability(
                IntBounds::at_least(13)
            ))
            .matches(&components)
        );
        assert!(!DataComponentPredicateModel::Damage(DamagePredicateModel {
            durability: IntBounds::ANY,
            damage: IntBounds::exactly(8),
        })
        .matches(&components));
        assert!(!DataComponentPredicateModel::Damage(DamagePredicateModel {
            durability: IntBounds::ANY,
            damage: IntBounds::ANY,
        })
        .matches(&ComponentGetterModel::default()));
    }

    #[test]
    fn enchantment_potion_villager_trim_and_jukebox_predicates_match_holder_sets() {
        let enchantments = BTreeMap::from([("sharpness", 5), ("mending", 1)]);
        let stored_enchantments = BTreeMap::from([("silk_touch", 1)]);
        let components = ComponentGetterModel::default()
            .with("enchantments", ComponentValue::Enchantments(enchantments))
            .with(
                "stored_enchantments",
                ComponentValue::Enchantments(stored_enchantments),
            )
            .with(
                "potion_contents",
                ComponentValue::Potion {
                    potion_key: Some("minecraft:water"),
                },
            )
            .with(
                "villager/variant",
                ComponentValue::Holder("minecraft:plains"),
            )
            .with(
                "trim",
                ComponentValue::Trim {
                    material: "minecraft:gold",
                    pattern: "minecraft:sentry",
                },
            )
            .with(
                "jukebox_playable",
                ComponentValue::JukeboxPlayable {
                    song_key: Some("minecraft:13"),
                },
            );

        assert!(
            DataComponentPredicateModel::Enchantments(EnchantmentsPredicateModel {
                enchantments: vec![EnchantmentPredicateModel {
                    id: "sharpness",
                    level: IntBounds::exactly(5),
                }]
            })
            .matches(&components)
        );
        assert!(
            DataComponentPredicateModel::StoredEnchantments(EnchantmentsPredicateModel {
                enchantments: vec![EnchantmentPredicateModel {
                    id: "silk_touch",
                    level: IntBounds::exactly(1),
                }]
            })
            .matches(&components)
        );
        assert!(
            DataComponentPredicateModel::Potions(HolderSetPredicate::new(&["minecraft:water"]))
                .matches(&components)
        );
        assert!(
            DataComponentPredicateModel::VillagerVariant(HolderSetPredicate::new(&[
                "minecraft:plains"
            ]))
            .matches(&components)
        );
        assert!(DataComponentPredicateModel::Trim(TrimPredicate {
            material: Some(HolderSetPredicate::new(&["minecraft:gold"])),
            pattern: Some(HolderSetPredicate::new(&["minecraft:sentry"])),
        })
        .matches(&components));
        assert!(
            DataComponentPredicateModel::JukeboxPlayable(JukeboxPlayablePredicate::any())
                .matches(&components)
        );
        assert!(!JukeboxPlayablePredicate {
            song: Some(HolderSetPredicate::new(&["minecraft:cat"]))
        }
        .matches(Some("minecraft:13")));
    }

    #[test]
    fn container_uses_non_empty_items_but_bundle_uses_all_items() {
        let components = ComponentGetterModel::default()
            .with(
                "container",
                ComponentValue::Items(vec![
                    ItemInstance {
                        id: "minecraft:air",
                        empty: true,
                    },
                    ItemInstance {
                        id: "minecraft:apple",
                        empty: false,
                    },
                ]),
            )
            .with(
                "bundle_contents",
                ComponentValue::Items(vec![ItemInstance {
                    id: "minecraft:air",
                    empty: true,
                }]),
            );

        assert!(
            DataComponentPredicateModel::Container(ItemCollectionPredicate {
                required_item: Some("minecraft:apple"),
                min_count: 1,
            })
            .matches(&components)
        );
        assert!(
            !DataComponentPredicateModel::Container(ItemCollectionPredicate {
                required_item: Some("minecraft:air"),
                min_count: 1,
            })
            .matches(&components)
        );
        assert!(
            DataComponentPredicateModel::BundleContents(ItemCollectionPredicate {
                required_item: Some("minecraft:air"),
                min_count: 1,
            })
            .matches(&components)
        );
        assert!(ItemCollectionPredicate::any().matches(&[]));
    }

    #[test]
    fn firework_explosion_and_fireworks_predicates_use_optional_fields() {
        let components = ComponentGetterModel::default()
            .with(
                "firework_explosion",
                ComponentValue::FireworkExplosion(firework("star", true, false)),
            )
            .with(
                "fireworks",
                ComponentValue::Fireworks(FireworksModel {
                    explosions: vec![
                        firework("small_ball", false, true),
                        firework("star", true, false),
                    ],
                    flight_duration: 2,
                }),
            );

        assert!(
            DataComponentPredicateModel::FireworkExplosion(FireworkExplosionPredicate {
                shape: Some("star"),
                twinkle: Some(true),
                trail: Some(false),
            })
            .matches(&components)
        );
        assert!(
            !DataComponentPredicateModel::FireworkExplosion(FireworkExplosionPredicate {
                shape: Some("star"),
                twinkle: None,
                trail: Some(true),
            })
            .matches(&components)
        );
        assert!(DataComponentPredicateModel::Fireworks(FireworksPredicate {
            explosion: Some(FireworkExplosionPredicate {
                shape: Some("small_ball"),
                twinkle: None,
                trail: Some(true),
            }),
            flight_duration: IntBounds::exactly(2),
        })
        .matches(&components));
    }

    #[test]
    fn attribute_modifier_entry_predicate_checks_java_field_order() {
        let components = ComponentGetterModel::default().with(
            "attribute_modifiers",
            ComponentValue::AttributeModifiers(vec![AttributeModifierEntry {
                attribute: "minecraft:generic.attack_damage",
                id: "minecraft:sharp",
                amount: 3.0,
                operation: "add_value",
                slot: "mainhand",
            }]),
        );

        assert!(
            DataComponentPredicateModel::AttributeModifiers(AttributeModifiersPredicate {
                entry: Some(AttributeEntryPredicate {
                    attribute: Some(HolderSetPredicate::new(&[
                        "minecraft:generic.attack_damage"
                    ])),
                    id: Some("minecraft:sharp"),
                    amount: DoubleBounds::exactly(3.0),
                    operation: Some("add_value"),
                    slot: Some("mainhand"),
                })
            })
            .matches(&components)
        );
        assert!(!AttributeEntryPredicate {
            attribute: None,
            id: Some("minecraft:sharp"),
            amount: DoubleBounds::ANY,
            operation: Some("multiply_base"),
            slot: None,
        }
        .test(&AttributeModifierEntry {
            attribute: "minecraft:generic.attack_damage",
            id: "minecraft:sharp",
            amount: 3.0,
            operation: "add_value",
            slot: "mainhand",
        }));
    }

    #[test]
    fn writable_and_written_book_predicates_compare_raw_page_title_author_generation_and_resolved()
    {
        let components = ComponentGetterModel::default()
            .with(
                "writable_book_content",
                ComponentValue::WritableBook(vec![FilterableText {
                    raw: "draft",
                    filtered: Some("filtered"),
                }]),
            )
            .with(
                "written_book_content",
                ComponentValue::WrittenBook(WrittenBookModel {
                    pages: vec![FilterableText {
                        raw: "{\"text\":\"page\"}",
                        filtered: None,
                    }],
                    author: "Alex",
                    title_raw: "Manual",
                    generation: 1,
                    resolved: true,
                }),
            );

        assert!(
            DataComponentPredicateModel::WritableBook(WritableBookPredicate {
                required_page: Some("draft")
            })
            .matches(&components)
        );
        assert!(
            DataComponentPredicateModel::WrittenBook(WrittenBookPredicate {
                required_page: Some("{\"text\":\"page\"}"),
                author: Some("Alex"),
                title: Some("Manual"),
                generation: IntBounds::exactly(1),
                resolved: Some(true),
            })
            .matches(&components)
        );
        assert!(
            !DataComponentPredicateModel::WrittenBook(WrittenBookPredicate {
                required_page: None,
                author: Some("Steve"),
                title: None,
                generation: IntBounds::ANY,
                resolved: None,
            })
            .matches(&components)
        );
    }
}
