use std::collections::BTreeMap;

use super::{Identifier, Lifecycle, LoadedTags, Registry, TagFile};

pub mod registries {
    pub const BLOCK: &str = "minecraft:block";
    pub const ITEM: &str = "minecraft:item";
    pub const ENTITY_TYPE: &str = "minecraft:entity_type";
    pub const DIMENSION_TYPE: &str = "minecraft:dimension_type";
    pub const BIOME: &str = "minecraft:worldgen/biome";
}

#[derive(Debug, Clone)]
pub struct BuiltInRegistries {
    pub blocks: Registry<String>,
    pub items: Registry<String>,
    pub entity_types: Registry<String>,
    pub dimension_types: Registry<String>,
    pub biomes: Registry<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataPackRegistryEntry {
    pub registry: Identifier,
    pub location: Identifier,
    pub value: String,
    pub lifecycle: Lifecycle,
}

#[derive(Debug, Clone)]
pub struct DynamicRegistryAccess {
    registries: BTreeMap<Identifier, Registry<String>>,
    order: Vec<Identifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltInRegistryDescriptor {
    pub java_field: &'static str,
    pub registry: &'static str,
    pub default_key: Option<&'static str>,
    pub intrusive_holders: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerReloadStage {
    BuiltInRegistries,
    DataPackRegistries,
    Tags,
    FrozenRegistries,
    Recipes,
    LootTables,
    Advancements,
    Functions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerResourceReloadRequest {
    pub registry_entries: Vec<DataPackRegistryEntry>,
    pub tag_files: Vec<TagFile>,
}

#[derive(Debug, Clone)]
pub struct ServerResourceReload {
    registries: DynamicRegistryAccess,
    tags: BTreeMap<Identifier, LoadedTags>,
    stages: Vec<ServerReloadStage>,
}

impl ServerResourceReload {
    pub fn registries(&self) -> &DynamicRegistryAccess {
        &self.registries
    }

    pub fn tags(&self, registry: &Identifier) -> Option<&LoadedTags> {
        self.tags.get(registry)
    }

    pub fn stages(&self) -> &[ServerReloadStage] {
        &self.stages
    }
}

#[derive(Debug, Clone)]
pub struct ReloadableServerRegistries {
    builtins: BuiltInRegistries,
    last_successful: DynamicRegistryAccess,
    stages: Vec<ServerReloadStage>,
}

impl ReloadableServerRegistries {
    pub fn new(builtins: BuiltInRegistries) -> Self {
        let last_successful = DynamicRegistryAccess::from_builtins(&builtins);
        Self {
            builtins,
            last_successful,
            stages: vanilla_reload_stages(),
        }
    }

    pub fn reload(
        &mut self,
        request: ServerResourceReloadRequest,
    ) -> Result<ServerResourceReload, String> {
        let mut registries = DynamicRegistryAccess::from_builtins(&self.builtins);
        registries.apply_data_pack_entries(request.registry_entries)?;

        let mut tags = BTreeMap::new();
        let mut tag_files_by_registry: BTreeMap<Identifier, Vec<TagFile>> = BTreeMap::new();
        for file in request.tag_files {
            tag_files_by_registry
                .entry(file.registry.clone())
                .or_default()
                .push(file);
        }

        for (registry_id, files) in tag_files_by_registry {
            let registry = registries
                .registry(&registry_id)
                .ok_or_else(|| format!("tag reload references unknown registry {registry_id}"))?;
            let loaded = LoadedTags::load(registry, files).map_err(|errors| errors.join("; "))?;
            tags.insert(registry_id, loaded);
        }

        registries.freeze_all();
        self.last_successful = registries.clone();

        Ok(ServerResourceReload {
            registries,
            tags,
            stages: self.stages.clone(),
        })
    }

    pub fn last_successful(&self) -> &DynamicRegistryAccess {
        &self.last_successful
    }

    pub fn stages(&self) -> &[ServerReloadStage] {
        &self.stages
    }
}

fn vanilla_reload_stages() -> Vec<ServerReloadStage> {
    vec![
        ServerReloadStage::BuiltInRegistries,
        ServerReloadStage::DataPackRegistries,
        ServerReloadStage::Tags,
        ServerReloadStage::FrozenRegistries,
        ServerReloadStage::Recipes,
        ServerReloadStage::LootTables,
        ServerReloadStage::Advancements,
        ServerReloadStage::Functions,
    ]
}

impl DynamicRegistryAccess {
    pub fn from_builtins(builtins: &BuiltInRegistries) -> Self {
        let mut registries = BTreeMap::new();
        let mut order = Vec::new();
        for registry in [
            builtins.blocks.clone(),
            builtins.items.clone(),
            builtins.entity_types.clone(),
            builtins.dimension_types.clone(),
            builtins.biomes.clone(),
        ] {
            order.push(registry.registry_id().clone());
            registries.insert(registry.registry_id().clone(), registry);
        }
        Self { registries, order }
    }

    pub fn apply_data_pack_entries(
        &mut self,
        entries: impl IntoIterator<Item = DataPackRegistryEntry>,
    ) -> Result<(), String> {
        for entry in entries {
            let registry_id = entry.registry.clone();
            if !self.registries.contains_key(&registry_id) {
                self.order.push(registry_id.clone());
            }
            let registry = self
                .registries
                .entry(registry_id.clone())
                .or_insert_with(|| Registry::new(registry_id));
            if registry.is_frozen() {
                let snapshot = registry.serialize_with(Clone::clone);
                *registry = Registry::deserialize_with(snapshot, |value| Ok(value.to_string()))?;
            }
            registry.apply_data_pack_overrides([(entry.location, entry.value, entry.lifecycle)])?;
        }
        Ok(())
    }

    pub fn freeze_all(&mut self) {
        for registry in self.registries.values_mut() {
            registry.freeze();
        }
    }

    pub fn registry(&self, id: &Identifier) -> Option<&Registry<String>> {
        self.registries.get(id)
    }

    pub fn registry_ids(&self) -> Vec<Identifier> {
        self.order.clone()
    }
}

pub fn builtin_registry_manifest_26_1_2() -> Vec<BuiltInRegistryDescriptor> {
    BUILTIN_REGISTRY_MANIFEST_26_1_2.to_vec()
}

static BUILTIN_REGISTRY_MANIFEST_26_1_2: &[BuiltInRegistryDescriptor] = &[
    descriptor("GAME_EVENT", "minecraft:game_event", Some("step"), false),
    descriptor("SOUND_EVENT", "minecraft:sound_event", None, false),
    descriptor("FLUID", "minecraft:fluid", Some("empty"), true),
    descriptor("MOB_EFFECT", "minecraft:mob_effect", None, false),
    descriptor("BLOCK", "minecraft:block", Some("air"), true),
    descriptor(
        "DEBUG_SUBSCRIPTION",
        "minecraft:debug_subscription",
        None,
        false,
    ),
    descriptor("ENTITY_TYPE", "minecraft:entity_type", Some("pig"), true),
    descriptor("ITEM", "minecraft:item", Some("air"), true),
    descriptor("POTION", "minecraft:potion", None, false),
    descriptor("PARTICLE_TYPE", "minecraft:particle_type", None, false),
    descriptor(
        "BLOCK_ENTITY_TYPE",
        "minecraft:block_entity_type",
        None,
        true,
    ),
    descriptor("CUSTOM_STAT", "minecraft:custom_stat", None, false),
    descriptor(
        "CHUNK_STATUS",
        "minecraft:chunk_status",
        Some("empty"),
        false,
    ),
    descriptor("RULE_TEST", "minecraft:rule_test", None, false),
    descriptor(
        "RULE_BLOCK_ENTITY_MODIFIER",
        "minecraft:rule_block_entity_modifier",
        None,
        false,
    ),
    descriptor("POS_RULE_TEST", "minecraft:pos_rule_test", None, false),
    descriptor("MENU", "minecraft:menu", None, false),
    descriptor("RECIPE_TYPE", "minecraft:recipe_type", None, false),
    descriptor(
        "RECIPE_SERIALIZER",
        "minecraft:recipe_serializer",
        None,
        false,
    ),
    descriptor("ATTRIBUTE", "minecraft:attribute", None, false),
    descriptor(
        "POSITION_SOURCE_TYPE",
        "minecraft:position_source_type",
        None,
        false,
    ),
    descriptor(
        "COMMAND_ARGUMENT_TYPE",
        "minecraft:command_argument_type",
        None,
        false,
    ),
    descriptor("STAT_TYPE", "minecraft:stat_type", None, false),
    descriptor(
        "VILLAGER_TYPE",
        "minecraft:villager_type",
        Some("plains"),
        false,
    ),
    descriptor(
        "VILLAGER_PROFESSION",
        "minecraft:villager_profession",
        Some("none"),
        false,
    ),
    descriptor(
        "POINT_OF_INTEREST_TYPE",
        "minecraft:point_of_interest_type",
        None,
        false,
    ),
    descriptor(
        "MEMORY_MODULE_TYPE",
        "minecraft:memory_module_type",
        Some("dummy"),
        false,
    ),
    descriptor("SENSOR_TYPE", "minecraft:sensor_type", Some("dummy"), false),
    descriptor("ACTIVITY", "minecraft:activity", None, false),
    descriptor(
        "LOOT_POOL_ENTRY_TYPE",
        "minecraft:loot_pool_entry_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_FUNCTION_TYPE",
        "minecraft:loot_function_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_CONDITION_TYPE",
        "minecraft:loot_condition_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_NUMBER_PROVIDER_TYPE",
        "minecraft:loot_number_provider_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_NBT_PROVIDER_TYPE",
        "minecraft:loot_nbt_provider_type",
        None,
        false,
    ),
    descriptor(
        "LOOT_SCORE_PROVIDER_TYPE",
        "minecraft:loot_score_provider_type",
        None,
        false,
    ),
    descriptor(
        "FLOAT_PROVIDER_TYPE",
        "minecraft:float_provider_type",
        None,
        false,
    ),
    descriptor(
        "INT_PROVIDER_TYPE",
        "minecraft:int_provider_type",
        None,
        false,
    ),
    descriptor(
        "HEIGHT_PROVIDER_TYPE",
        "minecraft:height_provider_type",
        None,
        false,
    ),
    descriptor(
        "BLOCK_PREDICATE_TYPE",
        "minecraft:block_predicate_type",
        None,
        false,
    ),
    descriptor("CARVER", "minecraft:worldgen/carver", None, false),
    descriptor("FEATURE", "minecraft:worldgen/feature", None, false),
    descriptor(
        "STRUCTURE_PLACEMENT",
        "minecraft:worldgen/structure_placement",
        None,
        false,
    ),
    descriptor(
        "STRUCTURE_PIECE",
        "minecraft:worldgen/structure_piece",
        None,
        false,
    ),
    descriptor(
        "STRUCTURE_TYPE",
        "minecraft:worldgen/structure_type",
        None,
        false,
    ),
    descriptor(
        "PLACEMENT_MODIFIER_TYPE",
        "minecraft:worldgen/placement_modifier_type",
        None,
        false,
    ),
    descriptor(
        "BLOCKSTATE_PROVIDER_TYPE",
        "minecraft:worldgen/block_state_provider_type",
        None,
        false,
    ),
    descriptor(
        "FOLIAGE_PLACER_TYPE",
        "minecraft:worldgen/foliage_placer_type",
        None,
        false,
    ),
    descriptor(
        "TRUNK_PLACER_TYPE",
        "minecraft:worldgen/trunk_placer_type",
        None,
        false,
    ),
    descriptor(
        "ROOT_PLACER_TYPE",
        "minecraft:worldgen/root_placer_type",
        None,
        false,
    ),
    descriptor(
        "TREE_DECORATOR_TYPE",
        "minecraft:worldgen/tree_decorator_type",
        None,
        false,
    ),
    descriptor(
        "FEATURE_SIZE_TYPE",
        "minecraft:worldgen/feature_size_type",
        None,
        false,
    ),
    descriptor(
        "BIOME_SOURCE",
        "minecraft:worldgen/biome_source",
        None,
        false,
    ),
    descriptor(
        "CHUNK_GENERATOR",
        "minecraft:worldgen/chunk_generator",
        None,
        false,
    ),
    descriptor(
        "MATERIAL_CONDITION",
        "minecraft:worldgen/material_condition",
        None,
        false,
    ),
    descriptor(
        "MATERIAL_RULE",
        "minecraft:worldgen/material_rule",
        None,
        false,
    ),
    descriptor(
        "DENSITY_FUNCTION_TYPE",
        "minecraft:worldgen/density_function_type",
        None,
        false,
    ),
    descriptor("BLOCK_TYPE", "minecraft:block_type", None, false),
    descriptor(
        "STRUCTURE_PROCESSOR",
        "minecraft:worldgen/structure_processor",
        None,
        false,
    ),
    descriptor(
        "STRUCTURE_POOL_ELEMENT",
        "minecraft:worldgen/structure_pool_element",
        None,
        false,
    ),
    descriptor(
        "POOL_ALIAS_BINDING_TYPE",
        "minecraft:worldgen/pool_alias_binding",
        None,
        false,
    ),
    descriptor(
        "DECORATED_POT_PATTERN",
        "minecraft:decorated_pot_pattern",
        None,
        false,
    ),
    descriptor(
        "CREATIVE_MODE_TAB",
        "minecraft:creative_mode_tab",
        None,
        false,
    ),
    descriptor("TRIGGER_TYPES", "minecraft:trigger_type", None, false),
    descriptor(
        "NUMBER_FORMAT_TYPE",
        "minecraft:number_format_type",
        None,
        false,
    ),
    descriptor(
        "DATA_COMPONENT_TYPE",
        "minecraft:data_component_type",
        None,
        false,
    ),
    descriptor("GAME_RULE", "minecraft:game_rule", None, false),
    descriptor(
        "ENTITY_SUB_PREDICATE_TYPE",
        "minecraft:entity_sub_predicate_type",
        None,
        false,
    ),
    descriptor(
        "DATA_COMPONENT_PREDICATE_TYPE",
        "minecraft:data_component_predicate_type",
        None,
        false,
    ),
    descriptor(
        "MAP_DECORATION_TYPE",
        "minecraft:map_decoration_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_EFFECT_COMPONENT_TYPE",
        "minecraft:enchantment_effect_component_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_LEVEL_BASED_VALUE_TYPE",
        "minecraft:enchantment_level_based_value_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_ENTITY_EFFECT_TYPE",
        "minecraft:enchantment_entity_effect_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE",
        "minecraft:enchantment_location_based_effect_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_VALUE_EFFECT_TYPE",
        "minecraft:enchantment_value_effect_type",
        None,
        false,
    ),
    descriptor(
        "ENCHANTMENT_PROVIDER_TYPE",
        "minecraft:enchantment_provider_type",
        None,
        false,
    ),
    descriptor(
        "CONSUME_EFFECT_TYPE",
        "minecraft:consume_effect_type",
        None,
        false,
    ),
    descriptor("RECIPE_DISPLAY", "minecraft:recipe_display", None, false),
    descriptor("SLOT_DISPLAY", "minecraft:slot_display", None, false),
    descriptor(
        "RECIPE_BOOK_CATEGORY",
        "minecraft:recipe_book_category",
        None,
        false,
    ),
    descriptor("TICKET_TYPE", "minecraft:ticket_type", None, false),
    descriptor(
        "INCOMING_RPC_METHOD",
        "minecraft:incoming_rpc_methods",
        None,
        false,
    ),
    descriptor(
        "OUTGOING_RPC_METHOD",
        "minecraft:outgoing_rpc_methods",
        None,
        false,
    ),
    descriptor(
        "TEST_ENVIRONMENT_DEFINITION_TYPE",
        "minecraft:test_environment_definition_type",
        None,
        false,
    ),
    descriptor(
        "TEST_INSTANCE_TYPE",
        "minecraft:test_instance_type",
        None,
        false,
    ),
    descriptor(
        "SPAWN_CONDITION_TYPE",
        "minecraft:spawn_condition_type",
        None,
        false,
    ),
    descriptor("DIALOG_TYPE", "minecraft:dialog_type", None, false),
    descriptor(
        "DIALOG_ACTION_TYPE",
        "minecraft:dialog_action_type",
        None,
        false,
    ),
    descriptor(
        "INPUT_CONTROL_TYPE",
        "minecraft:input_control_type",
        None,
        false,
    ),
    descriptor(
        "DIALOG_BODY_TYPE",
        "minecraft:dialog_body_type",
        None,
        false,
    ),
    descriptor("PERMISSION_TYPE", "minecraft:permission_type", None, false),
    descriptor(
        "PERMISSION_CHECK_TYPE",
        "minecraft:permission_check_type",
        None,
        false,
    ),
    descriptor(
        "ENVIRONMENT_ATTRIBUTE",
        "minecraft:environment_attribute",
        None,
        false,
    ),
    descriptor("ATTRIBUTE_TYPE", "minecraft:attribute_type", None, false),
    descriptor(
        "SLOT_SOURCE_TYPE",
        "minecraft:slot_source_type",
        None,
        false,
    ),
    descriptor("TEST_FUNCTION", "minecraft:test_function", None, false),
];

const fn descriptor(
    java_field: &'static str,
    registry: &'static str,
    default_key: Option<&'static str>,
    intrusive_holders: bool,
) -> BuiltInRegistryDescriptor {
    BuiltInRegistryDescriptor {
        java_field,
        registry,
        default_key,
        intrusive_holders,
    }
}

impl BuiltInRegistries {
    pub fn bootstrap_26_1_2() -> Result<Self, String> {
        let mut blocks = Registry::new(builtin_identifier(registries::BLOCK)?);
        register_stable_ids(
            &mut blocks,
            &[
                "minecraft:air",
                "minecraft:stone",
                "minecraft:dirt",
                "minecraft:grass_block",
                "minecraft:bedrock",
                "minecraft:water",
                "minecraft:lava",
            ],
        )?;

        let mut items = Registry::new(builtin_identifier(registries::ITEM)?);
        register_stable_ids(
            &mut items,
            &[
                "minecraft:air",
                "minecraft:stick",
                "minecraft:apple",
                "minecraft:stone",
                "minecraft:dirt",
                "minecraft:diamond_sword",
                "minecraft:netherite_chestplate",
            ],
        )?;

        let mut entity_types = Registry::new(builtin_identifier(registries::ENTITY_TYPE)?);
        register_stable_ids(
            &mut entity_types,
            &[
                "minecraft:player",
                "minecraft:pig",
                "minecraft:cow",
                "minecraft:armor_stand",
                "minecraft:item",
            ],
        )?;

        let mut dimension_types = Registry::new(builtin_identifier(registries::DIMENSION_TYPE)?);
        register_stable_ids(
            &mut dimension_types,
            &[
                "minecraft:overworld",
                "minecraft:the_nether",
                "minecraft:the_end",
                "minecraft:overworld_caves",
            ],
        )?;

        let mut biomes = Registry::new(builtin_identifier(registries::BIOME)?);
        register_stable_ids(
            &mut biomes,
            &[
                "minecraft:plains",
                "minecraft:forest",
                "minecraft:desert",
                "minecraft:nether_wastes",
                "minecraft:the_end",
            ],
        )?;

        blocks.freeze();
        items.freeze();
        entity_types.freeze();
        dimension_types.freeze();
        biomes.freeze();

        Ok(Self {
            blocks,
            items,
            entity_types,
            dimension_types,
            biomes,
        })
    }

    pub fn registry_ids(&self) -> Vec<Identifier> {
        vec![
            self.blocks.registry_id().clone(),
            self.items.registry_id().clone(),
            self.entity_types.registry_id().clone(),
            self.dimension_types.registry_id().clone(),
            self.biomes.registry_id().clone(),
        ]
    }
}

fn builtin_identifier(value: &str) -> Result<Identifier, String> {
    Identifier::parse(value)
        .map_err(|err| format!("built-in identifier {value:?} is invalid: {err}"))
}

fn register_stable_ids(registry: &mut Registry<String>, ids: &[&str]) -> Result<(), String> {
    for id in ids {
        registry.register(
            builtin_identifier(id)?,
            (*id).to_string(),
            Lifecycle::Stable,
        )?;
    }
    Ok(())
}
