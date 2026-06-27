use std::collections::{BTreeMap, BTreeSet};

use crate::jsonrpc_api::schema_registry;
use crate::jsonrpc_api::{MethodInfo, ParamInfo, ResultInfo};
use crate::jsonrpc_methods::{
    DiscoverResponse, DiscoverableJsonRpcMethod, DiscoveryService,
};
use crate::management_server::{INCOMING_RPC_METHOD_DEFS, OUTGOING_RPC_METHOD_DEFS};
use crate::registry::Identifier;

pub const DATA_PACKAGE_NULL_MARKED: bool = true;
const DATA_PACKAGE_INFO_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/package-info.java");

const ATLAS_IDS: &[&str] = &[
    "minecraft:armor_trims",
    "minecraft:banner_patterns",
    "minecraft:beds",
    "minecraft:blocks",
    "minecraft:items",
    "minecraft:chests",
    "minecraft:decorated_pot",
    "minecraft:gui",
    "minecraft:map_decorations",
    "minecraft:paintings",
    "minecraft:particles",
    "minecraft:shield_patterns",
    "minecraft:shulker_boxes",
    "minecraft:signs",
    "minecraft:celestials",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BlockFamilyVariant {
    Button,
    Chiseled,
    Cracked,
    Cut,
    Door,
    CustomFence,
    Fence,
    CustomFenceGate,
    FenceGate,
    Mosaic,
    Sign,
    Slab,
    Stairs,
    PressurePlate,
    Polished,
    Trapdoor,
    Wall,
    WallSign,
    Bricks,
    Cobbled,
    Tiles,
}

impl BlockFamilyVariant {
    const ALL: [Self; 21] = [
        Self::Button,
        Self::Chiseled,
        Self::Cracked,
        Self::Cut,
        Self::Door,
        Self::CustomFence,
        Self::Fence,
        Self::CustomFenceGate,
        Self::FenceGate,
        Self::Mosaic,
        Self::Sign,
        Self::Slab,
        Self::Stairs,
        Self::PressurePlate,
        Self::Polished,
        Self::Trapdoor,
        Self::Wall,
        Self::WallSign,
        Self::Bricks,
        Self::Cobbled,
        Self::Tiles,
    ];

    fn recipe_group(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Chiseled => "chiseled",
            Self::Cracked => "cracked",
            Self::Cut => "cut",
            Self::Door => "door",
            Self::CustomFence | Self::Fence => "fence",
            Self::CustomFenceGate | Self::FenceGate => "fence_gate",
            Self::Mosaic => "mosaic",
            Self::Sign => "sign",
            Self::Slab => "slab",
            Self::Stairs => "stairs",
            Self::PressurePlate => "pressure_plate",
            Self::Polished => "polished",
            Self::Trapdoor => "trapdoor",
            Self::Wall => "wall",
            Self::WallSign => "wall_sign",
            Self::Bricks => "bricks",
            Self::Cobbled => "cobbled",
            Self::Tiles => "tiles",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BlockFamilyModel {
    base_block: &'static str,
    variants: BTreeMap<BlockFamilyVariant, &'static str>,
    generate_model: bool,
    generate_crafting_recipe: bool,
    generate_stonecutter_recipe: bool,
    recipe_group_prefix: Option<&'static str>,
    recipe_unlocked_by: Option<&'static str>,
}

impl BlockFamilyModel {
    fn get(&self, variant: BlockFamilyVariant) -> Option<&'static str> {
        self.variants.get(&variant).copied()
    }

    fn optional_text(value: Option<&'static str>) -> Option<&'static str> {
        value.filter(|text| !text.trim().is_empty())
    }
}

#[derive(Debug)]
struct BlockFamilyBuilder {
    family: BlockFamilyModel,
}

impl BlockFamilyBuilder {
    fn new(base_block: &'static str) -> Self {
        Self {
            family: BlockFamilyModel {
                base_block,
                variants: BTreeMap::new(),
                generate_model: true,
                generate_crafting_recipe: true,
                generate_stonecutter_recipe: false,
                recipe_group_prefix: None,
                recipe_unlocked_by: None,
            },
        }
    }

    fn variant(mut self, variant: BlockFamilyVariant, block: &'static str) -> Self {
        self.family.variants.insert(variant, block);
        self
    }

    fn sign(mut self, sign: &'static str, wall_sign: &'static str) -> Self {
        self.family.variants.insert(BlockFamilyVariant::Sign, sign);
        self.family
            .variants
            .insert(BlockFamilyVariant::WallSign, wall_sign);
        self
    }

    fn dont_generate_model(mut self) -> Self {
        self.family.generate_model = false;
        self
    }

    fn dont_generate_crafting_recipe(mut self) -> Self {
        self.family.generate_crafting_recipe = false;
        self
    }

    fn generate_stonecutter_recipe(mut self) -> Self {
        self.family.generate_stonecutter_recipe = true;
        self
    }

    fn recipe_group_prefix(mut self, prefix: &'static str) -> Self {
        self.family.recipe_group_prefix = Some(prefix);
        self
    }

    fn recipe_unlocked_by(mut self, criterion: &'static str) -> Self {
        self.family.recipe_unlocked_by = Some(criterion);
        self
    }

    fn get_family(self) -> BlockFamilyModel {
        self.family
    }
}

#[derive(Debug, Default)]
struct BlockFamiliesModel {
    families: BTreeMap<&'static str, BlockFamilyModel>,
}

impl BlockFamiliesModel {
    fn family_builder(&mut self, base: &'static str) -> Result<BlockFamilyBuilder, String> {
        if self.families.contains_key(base) {
            return Err(format!("Duplicate family definition for minecraft:{base}"));
        }
        Ok(BlockFamilyBuilder::new(base))
    }

    fn insert(&mut self, family: BlockFamilyModel) {
        self.families.insert(family.base_block, family);
    }

    fn get_family(&self, base: &str) -> Option<&BlockFamilyModel> {
        self.families.get(base)
    }

    fn get_all_families(&self) -> impl Iterator<Item = &BlockFamilyModel> {
        self.families.values()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackOutputTarget {
    DataPack,
    ResourcePack,
    Reports,
}

impl PackOutputTarget {
    fn directory(self) -> &'static str {
        match self {
            Self::DataPack => "data",
            Self::ResourcePack => "assets",
            Self::Reports => "reports",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackOutputModel {
    output_folder: String,
}

impl PackOutputModel {
    fn new(output_folder: &str) -> Self {
        Self {
            output_folder: output_folder.to_string(),
        }
    }

    fn get_output_folder(&self, target: PackOutputTarget) -> String {
        format!("{}/{}", self.output_folder, target.directory())
    }

    fn create_path_provider(&self, target: PackOutputTarget, kind: &str) -> PathProviderModel {
        PathProviderModel {
            root: self.get_output_folder(target),
            kind: kind.to_string(),
        }
    }

    fn create_registry_elements_path_provider(&self, registry_path: &str) -> PathProviderModel {
        self.create_path_provider(PackOutputTarget::DataPack, registry_path)
    }

    fn create_registry_tags_path_provider(&self, registry_path: &str) -> PathProviderModel {
        self.create_path_provider(PackOutputTarget::DataPack, &format!("tags/{registry_path}"))
    }

    fn create_registry_component_path_provider(&self, registry_path: &str) -> PathProviderModel {
        self.create_path_provider(
            PackOutputTarget::Reports,
            &format!("components/{registry_path}"),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PathProviderModel {
    root: String,
    kind: String,
}

impl PathProviderModel {
    fn file(&self, element: &str, extension: &str) -> String {
        let (namespace, path) = split_identifier(element);
        format!(
            "{}/{}/{}/{}.{}",
            self.root, namespace, self.kind, path, extension
        )
    }

    fn json(&self, element: &str) -> String {
        self.file(element, "json")
    }
}

#[derive(Debug, Default)]
struct CachedOutputModel {
    writes: BTreeMap<String, Vec<u8>>,
}

impl CachedOutputModel {
    fn no_cache_write(&mut self, path: &str, input: &[u8]) {
        self.writes.insert(path.to_string(), input.to_vec());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct JsonRpcApiSchemaProviderModel {
    path: String,
}

impl JsonRpcApiSchemaProviderModel {
    const FILE_NAME: &'static str = "json-rpc-api-schema.json";
    const NAME: &'static str = "Json RPC API schema";

    fn new(pack_output: &PackOutputModel) -> Self {
        Self {
            path: format!(
                "{}/{}",
                pack_output.get_output_folder(PackOutputTarget::Reports),
                Self::FILE_NAME
            ),
        }
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }

    fn discover(&self) -> DiscoverResponse {
        DiscoveryService::discover(
            &schema_registry(),
            &incoming_discoverable_methods(),
            &outgoing_discoverable_methods(),
        )
    }

    fn run(&self, cache: &mut CachedOutputModel) -> DiscoverResponse {
        let discover = self.discover();
        cache.no_cache_write(&self.path, discover_summary(&discover).as_bytes());
        discover
    }
}

fn incoming_discoverable_methods() -> Vec<DiscoverableJsonRpcMethod> {
    INCOMING_RPC_METHOD_DEFS
        .iter()
        .filter_map(|definition| {
            let param = definition.param.map(|(name, schema)| {
                ParamInfo::new(name, crate::jsonrpc_api::JsonRpcSchema::of_ref(schema, schema))
            });
            let result = Some(ResultInfo::new(
                definition.result.0,
                crate::jsonrpc_api::JsonRpcSchema::of_ref(definition.result.1, definition.result.1),
            ));
            Some(DiscoverableJsonRpcMethod::new(
                Identifier::with_default_namespace(definition.method).ok()?,
                MethodInfo::new(definition.description, param, result),
                definition.discoverable,
            ))
        })
        .collect()
}

fn outgoing_discoverable_methods() -> Vec<DiscoverableJsonRpcMethod> {
    OUTGOING_RPC_METHOD_DEFS
        .iter()
        .filter_map(|definition| {
            let param = definition.param.map(|(name, schema)| {
                ParamInfo::new(name, crate::jsonrpc_api::JsonRpcSchema::of_ref(schema, schema))
            });
            Some(DiscoverableJsonRpcMethod::new(
                Identifier::with_default_namespace(&definition.registry_key()).ok()?,
                MethodInfo::new(definition.description, param, None),
                definition.discoverable,
            ))
        })
        .collect()
}

fn discover_summary(discover: &DiscoverResponse) -> String {
    format!(
        "openrpc={};title={};version={};methods={};schemas={}",
        discover.json_rpc_protocol_version,
        discover.discover_info.title,
        discover.discover_info.version,
        discover.methods.len(),
        discover.components.schemas.len()
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderCacheModel {
    version: String,
    data: BTreeMap<String, String>,
}

impl ProviderCacheModel {
    fn load(root: &str, raw: &str) -> Result<Self, String> {
        let mut lines = raw.lines();
        let header = lines.next().ok_or("Missing cache file header")?;
        let Some(rest) = header.strip_prefix("// ") else {
            return Err("Missing cache file header".to_string());
        };
        let version = rest.split('\t').next().unwrap_or(rest).to_string();
        let mut data = BTreeMap::new();
        for line in lines {
            let (hash, path) = line
                .split_once(' ')
                .ok_or_else(|| format!("invalid cache row: {line}"))?;
            data.insert(format!("{root}/{path}"), hash.to_string());
        }
        Ok(Self { version, data })
    }

    fn save(&self, root: &str, extra_header_info: &str) -> String {
        let mut output = format!("// {}\t{}\n", self.version, extra_header_info);
        for (path, hash) in &self.data {
            let relative = path.strip_prefix(&format!("{root}/")).unwrap_or(path);
            output.push_str(hash);
            output.push(' ');
            output.push_str(relative);
            output.push('\n');
        }
        output
    }
}

#[derive(Debug)]
struct CacheUpdaterModel {
    provider: String,
    old_cache: ProviderCacheModel,
    new_cache: ProviderCacheModel,
    writes: usize,
    closed: bool,
    files: BTreeSet<String>,
}

impl CacheUpdaterModel {
    fn new(provider: &str, version: &str, old_cache: ProviderCacheModel) -> Self {
        Self {
            provider: provider.to_string(),
            old_cache,
            new_cache: ProviderCacheModel {
                version: version.to_string(),
                data: BTreeMap::new(),
            },
            writes: 0,
            closed: false,
            files: BTreeSet::new(),
        }
    }

    fn write_if_needed(&mut self, path: &str, hash: &str) -> Result<(), String> {
        if self.closed {
            return Err("Cannot write to cache as it has already been closed".to_string());
        }
        let old_hash = self.old_cache.data.get(path).map(String::as_str);
        if old_hash != Some(hash) || !self.files.contains(path) {
            self.writes += 1;
            self.files.insert(path.to_string());
        }
        self.new_cache
            .data
            .insert(path.to_string(), hash.to_string());
        Ok(())
    }

    fn close(mut self) -> UpdateResultModel {
        self.closed = true;
        UpdateResultModel {
            provider_id: self.provider,
            cache: self.new_cache,
            writes: self.writes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UpdateResultModel {
    provider_id: String,
    cache: ProviderCacheModel,
    writes: usize,
}

#[derive(Debug)]
struct HashCacheModel {
    root_dir: String,
    version_id: String,
    caches: BTreeMap<String, ProviderCacheModel>,
    caches_to_write: BTreeSet<String>,
    writes: usize,
}

impl HashCacheModel {
    fn new(root_dir: &str, provider_ids: &[&str], version_id: &str) -> Self {
        let caches = provider_ids
            .iter()
            .map(|provider| {
                (
                    (*provider).to_string(),
                    ProviderCacheModel {
                        version: "unknown".to_string(),
                        data: BTreeMap::new(),
                    },
                )
            })
            .collect();
        Self {
            root_dir: root_dir.to_string(),
            version_id: version_id.to_string(),
            caches,
            caches_to_write: BTreeSet::new(),
            writes: 0,
        }
    }

    fn should_run_in_this_version(&self, provider_id: &str) -> bool {
        self.caches
            .get(provider_id)
            .is_none_or(|cache| cache.version != self.version_id)
    }

    fn generate_update(&self, provider_id: &str) -> Result<CacheUpdaterModel, String> {
        let existing = self
            .caches
            .get(provider_id)
            .ok_or_else(|| format!("Provider not registered: {provider_id}"))?;
        Ok(CacheUpdaterModel::new(
            provider_id,
            &self.version_id,
            existing.clone(),
        ))
    }

    fn apply_update(&mut self, result: UpdateResultModel) {
        self.writes += result.writes;
        self.caches_to_write.insert(result.provider_id.clone());
        self.caches.insert(result.provider_id, result.cache);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderModel {
    name: &'static str,
}

#[derive(Debug)]
struct DataGeneratorModel {
    vanilla_pack_output: PackOutputModel,
    all_provider_ids: BTreeSet<String>,
    providers_to_run: BTreeMap<String, ProviderModel>,
}

impl DataGeneratorModel {
    fn new(output: &str) -> Self {
        Self {
            vanilla_pack_output: PackOutputModel::new(output),
            all_provider_ids: BTreeSet::new(),
            providers_to_run: BTreeMap::new(),
        }
    }

    fn get_vanilla_pack(&mut self, to_run: bool) -> PackGeneratorModel<'_> {
        PackGeneratorModel {
            generator: self,
            to_run,
            provider_prefix: "vanilla".to_string(),
            output: PackOutputModel::new("generated"),
        }
    }

    fn get_builtin_datapack(&mut self, to_run: bool, pack_id: &str) -> PackGeneratorModel<'_> {
        let output = format!(
            "{}/data/minecraft/datapacks/{}",
            self.vanilla_pack_output.output_folder, pack_id
        );
        PackGeneratorModel {
            generator: self,
            to_run,
            provider_prefix: pack_id.to_string(),
            output: PackOutputModel::new(&output),
        }
    }
}

#[derive(Debug)]
struct PackGeneratorModel<'a> {
    generator: &'a mut DataGeneratorModel,
    to_run: bool,
    provider_prefix: String,
    output: PackOutputModel,
}

impl PackGeneratorModel<'_> {
    fn add_provider(&mut self, provider: ProviderModel) -> Result<ProviderModel, String> {
        let provider_id = format!("{}/{}", self.provider_prefix, provider.name);
        if !self.generator.all_provider_ids.insert(provider_id.clone()) {
            return Err(format!("Duplicate provider: {provider_id}"));
        }
        if self.to_run {
            self.generator
                .providers_to_run
                .insert(provider_id, provider.clone());
        }
        Ok(provider)
    }
}

fn split_identifier(id: &str) -> (&str, &str) {
    id.split_once(':').unwrap_or(("minecraft", id))
}

fn parse_atlas_ids(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            line.split("Identifier.withDefaultNamespace(\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .map(|path| format!("minecraft:{path}"))
        })
        .collect()
}

fn parse_block_family_fields(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with("public static final BlockFamily ") {
                return None;
            }
            trimmed
                .split(" = ")
                .next()
                .and_then(|lhs| lhs.split_whitespace().last())
                .map(str::to_string)
        })
        .collect()
}

fn parse_main_provider_mentions(source: &str) -> BTreeSet<&str> {
    [
        "RegistriesDatapackGenerator",
        "VanillaAdvancementProvider",
        "VanillaLootTableProvider",
        "VanillaRecipeProvider",
        "VanillaBlockTagsProvider",
        "VanillaItemTagsProvider",
        "BiomeTagsProvider",
        "RegistryDumpReport",
        "PacketReport",
        "DatapackStructureReport",
        "TradeRebalanceLootTableProvider",
        "TradeRebalanceEnchantmentTagsProvider",
        "TradeRebalanceTradeTagsProvider",
        "JsonRpcApiSchema",
    ]
    .into_iter()
    .filter(|name| source.contains(name))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atlas_ids_match_java_order() {
        let source =
            vibecraft_java_source!("/net/minecraft/data/AtlasIds.java");
        let parsed = parse_atlas_ids(source);
        assert_eq!(parsed, ATLAS_IDS);
    }

    #[test]
    fn block_family_variants_and_builder_match_java() {
        let family = BlockFamilyBuilder::new("oak_planks")
            .variant(BlockFamilyVariant::Button, "oak_button")
            .variant(BlockFamilyVariant::Fence, "oak_fence")
            .variant(BlockFamilyVariant::FenceGate, "oak_fence_gate")
            .sign("oak_sign", "oak_wall_sign")
            .variant(BlockFamilyVariant::Slab, "oak_slab")
            .variant(BlockFamilyVariant::Stairs, "oak_stairs")
            .variant(BlockFamilyVariant::PressurePlate, "oak_pressure_plate")
            .variant(BlockFamilyVariant::Door, "oak_door")
            .variant(BlockFamilyVariant::Trapdoor, "oak_trapdoor")
            .recipe_group_prefix("wooden")
            .recipe_unlocked_by("has_planks")
            .get_family();

        assert_eq!(family.base_block, "oak_planks");
        assert_eq!(family.get(BlockFamilyVariant::Sign), Some("oak_sign"));
        assert_eq!(
            family.get(BlockFamilyVariant::WallSign),
            Some("oak_wall_sign")
        );
        assert!(family.generate_model);
        assert!(family.generate_crafting_recipe);
        assert!(!family.generate_stonecutter_recipe);
        assert_eq!(
            BlockFamilyModel::optional_text(family.recipe_group_prefix),
            Some("wooden")
        );
        assert_eq!(BlockFamilyModel::optional_text(Some("   ")), None);
        assert_eq!(BlockFamilyVariant::CustomFence.recipe_group(), "fence");
        assert_eq!(BlockFamilyVariant::Fence.recipe_group(), "fence");
        assert_eq!(
            BlockFamilyVariant::CustomFenceGate.recipe_group(),
            "fence_gate"
        );
        assert_eq!(BlockFamilyVariant::ALL.len(), 21);
        assert_eq!(BlockFamilyVariant::ALL[0], BlockFamilyVariant::Button);
        assert_eq!(BlockFamilyVariant::Tiles.recipe_group(), "tiles");
    }

    #[test]
    fn block_family_flags_and_duplicate_registration_match_java() {
        let mut families = BlockFamiliesModel::default();
        let copper = families
            .family_builder("copper_block")
            .unwrap()
            .variant(BlockFamilyVariant::Cut, "cut_copper")
            .dont_generate_model()
            .get_family();
        families.insert(copper);
        assert!(families.family_builder("copper_block").is_err());

        let quartz = BlockFamilyBuilder::new("quartz_block")
            .variant(BlockFamilyVariant::Slab, "quartz_slab")
            .dont_generate_crafting_recipe()
            .generate_stonecutter_recipe()
            .get_family();
        assert!(!quartz.generate_crafting_recipe);
        assert!(quartz.generate_stonecutter_recipe);
        assert_eq!(
            families
                .get_family("copper_block")
                .unwrap()
                .get(BlockFamilyVariant::Cut),
            Some("cut_copper")
        );
        assert_eq!(families.get_all_families().count(), 1);
    }

    #[test]
    fn block_families_java_table_has_expected_entries_and_helpers() {
        let source =
            vibecraft_java_source!("/net/minecraft/data/BlockFamilies.java");
        let fields = parse_block_family_fields(source);
        assert_eq!(fields.len(), 70);
        assert_eq!(fields.first().unwrap(), "ACACIA_PLANKS");
        assert!(fields.contains(&"BAMBOO_MOSAIC".to_string()));
        assert!(fields.contains(&"WAXED_OXIDIZED_CUT_COPPER".to_string()));
        assert_eq!(fields.last().unwrap(), "DEEPSLATE_TILES");
        assert!(source.contains("private static final Map<Block, BlockFamily> MAP"));
        assert!(source.contains("Duplicate family definition for "));
        assert!(source.contains("public static Stream<BlockFamily> getAllFamilies()"));
        assert!(source.contains("public static @Nullable BlockFamily getFamily"));
    }

    #[test]
    fn pack_output_targets_and_path_providers_match_java() {
        let output = PackOutputModel::new("generated");
        assert_eq!(
            output.get_output_folder(PackOutputTarget::ResourcePack),
            "generated/assets"
        );
        assert_eq!(
            output
                .create_registry_elements_path_provider("worldgen/biome")
                .json("minecraft:plains"),
            "generated/data/minecraft/worldgen/biome/plains.json"
        );
        assert_eq!(
            output
                .create_registry_tags_path_provider("item")
                .json("minecraft:logs"),
            "generated/data/minecraft/tags/item/logs.json"
        );
        assert_eq!(
            output
                .create_registry_component_path_provider("item")
                .file("example:stick", "json"),
            "generated/reports/example/components/item/stick.json"
        );
    }

    #[test]
    fn cached_output_and_data_provider_save_contracts_match_java() {
        let mut no_cache = CachedOutputModel::default();
        no_cache.no_cache_write("generated/data/test.json", b"{}");
        assert_eq!(no_cache.writes["generated/data/test.json"], b"{}".to_vec());

        let mut keys = vec!["parent", "zeta", "type", "alpha"];
        keys.sort_by_key(|key| match *key {
            "type" => (0, *key),
            "parent" => (1, *key),
            _ => (2, *key),
        });
        assert_eq!(keys, vec!["type", "parent", "alpha", "zeta"]);
    }

    #[test]
    fn json_rpc_api_schema_provider_matches_java_dataprovider() {
        const JSON_RPC_API_SCHEMA: &str = vibecraft_java_source!(
            "/net/minecraft/server/jsonrpc/dataprovider/JsonRpcApiSchema.java"
        );
        for sentinel in [
            "public class JsonRpcApiSchema implements DataProvider",
            "private final Path path;",
            "packOutput.getOutputFolder(PackOutput.Target.REPORTS).resolve(\"json-rpc-api-schema.json\")",
            "public CompletableFuture<?> run(final CachedOutput cache)",
            "DiscoveryService.DiscoverResponse discover = DiscoveryService.discover(Schema.getSchemaRegistry());",
            "DiscoveryService.DiscoverResponse.CODEC.codec().encodeStart(JsonOps.INSTANCE, discover).getOrThrow()",
            "DataProvider.saveStable(",
            "this.path",
            "public String getName()",
            "return \"Json RPC API schema\";",
        ] {
            assert!(
                JSON_RPC_API_SCHEMA.contains(sentinel),
                "JsonRpcApiSchema.java missing sentinel: {sentinel}"
            );
        }

        let output = PackOutputModel::new("generated");
        let provider = JsonRpcApiSchemaProviderModel::new(&output);
        assert_eq!(
            provider.path,
            "generated/reports/json-rpc-api-schema.json"
        );
        assert_eq!(provider.get_name(), "Json RPC API schema");

        let mut cache = CachedOutputModel::default();
        let discover = provider.run(&mut cache);
        assert_eq!(discover.json_rpc_protocol_version, "1.3.2");
        assert_eq!(discover.discover_info.title, "Minecraft Server JSON-RPC");
        assert_eq!(discover.discover_info.version, "2.0.0");
        assert!(discover
            .methods
            .iter()
            .any(|method| method.name.to_string() == "minecraft:players"));
        assert!(discover
            .methods
            .iter()
            .all(|method| method.name.to_string() != "minecraft:rpc.discover"));
        assert_eq!(
            cache.writes["generated/reports/json-rpc-api-schema.json"],
            discover_summary(&discover).into_bytes()
        );
    }

    #[test]
    fn hash_cache_provider_cache_and_updater_match_java() {
        let cache =
            ProviderCacheModel::load("root", "// version-1\tprovider\nabcd data/a.json\n").unwrap();
        assert_eq!(cache.version, "version-1");
        assert_eq!(cache.data["root/data/a.json"], "abcd");
        assert_eq!(
            cache.save("root", "extra"),
            "// version-1\textra\nabcd data/a.json\n"
        );
        assert!(ProviderCacheModel::load("root", "bad\n").is_err());

        let mut hash_cache = HashCacheModel::new("root", &["vanilla/tags"], "version-2");
        assert!(hash_cache.should_run_in_this_version("vanilla/tags"));
        let mut updater = hash_cache.generate_update("vanilla/tags").unwrap();
        updater.write_if_needed("root/data/a.json", "abcd").unwrap();
        let result = updater.close();
        assert_eq!(result.writes, 1);
        hash_cache.apply_update(result);
        assert_eq!(hash_cache.root_dir, "root");
        assert_eq!(hash_cache.writes, 1);
        assert!(hash_cache.caches_to_write.contains("vanilla/tags"));
        assert!(hash_cache.generate_update("missing").is_err());
    }

    #[test]
    fn cache_updater_rejects_writes_after_close_and_skips_unchanged_existing_files() {
        let mut old_data = BTreeMap::new();
        old_data.insert("root/data/a.json".to_string(), "abcd".to_string());
        let old_cache = ProviderCacheModel {
            version: "version-2".to_string(),
            data: old_data,
        };
        let mut updater = CacheUpdaterModel::new("vanilla/tags", "version-2", old_cache);
        updater.files.insert("root/data/a.json".to_string());
        updater.write_if_needed("root/data/a.json", "abcd").unwrap();
        let result = updater.close();
        assert_eq!(result.writes, 0);
    }

    #[test]
    fn data_generator_pack_registration_matches_java() {
        let mut generator = DataGeneratorModel::new("generated");
        {
            let mut vanilla = generator.get_vanilla_pack(true);
            assert_eq!(vanilla.output.output_folder, "generated");
            vanilla
                .add_provider(ProviderModel { name: "tags" })
                .unwrap();
        }
        {
            let mut skipped = generator.get_vanilla_pack(false);
            skipped
                .add_provider(ProviderModel { name: "reports" })
                .unwrap();
        }
        assert!(generator.all_provider_ids.contains("vanilla/tags"));
        assert!(generator.all_provider_ids.contains("vanilla/reports"));
        assert!(generator.providers_to_run.contains_key("vanilla/tags"));
        assert!(!generator.providers_to_run.contains_key("vanilla/reports"));
        assert!(generator
            .get_vanilla_pack(true)
            .add_provider(ProviderModel { name: "tags" })
            .is_err());

        let trade = generator.get_builtin_datapack(true, "trade_rebalance");
        assert_eq!(
            trade.output.output_folder,
            "generated/data/minecraft/datapacks/trade_rebalance"
        );
    }

    #[test]
    fn main_options_and_provider_wiring_match_java_sentinels() {
        let source = vibecraft_java_source!("/net/minecraft/data/Main.java");
        for option in [
            "help", "server", "dev", "reports", "validate", "all", "output", "input",
        ] {
            assert!(source.contains(&format!("accepts(\"{option}\"")));
        }
        assert!(source.contains("defaultsTo(\"generated\""));
        assert!(source.contains(
            "new DataGenerator.Cached(output, SharedConstants.getCurrentVersion(), true)"
        ));
        assert!(source.contains("Util.shutdownExecutors();"));
        assert_eq!(parse_main_provider_mentions(source).len(), 14);
    }

    #[test]
    fn package_is_null_marked() {
        const {
            assert!(DATA_PACKAGE_NULL_MARKED);
        }
        assert_eq!(
            DATA_PACKAGE_INFO_JAVA.match_indices("@NullMarked").count(),
            1
        );
        assert!(DATA_PACKAGE_INFO_JAVA.contains("package net.minecraft.data;"));
        assert!(DATA_PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
    }
}
