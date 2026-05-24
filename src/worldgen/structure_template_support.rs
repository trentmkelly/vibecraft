use super::*;

impl StructureProcessorTypeModel {
    pub const REGISTRY_ORDER: [Self; 11] = [
        Self::BlockIgnore,
        Self::BlockRot,
        Self::Gravity,
        Self::JigsawReplacement,
        Self::Rule,
        Self::Nop,
        Self::BlockAge,
        Self::BlackstoneReplace,
        Self::LavaSubmergedBlock,
        Self::ProtectedBlocks,
        Self::Capped,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::BlockIgnore => "minecraft:block_ignore",
            Self::BlockRot => "minecraft:block_rot",
            Self::Gravity => "minecraft:gravity",
            Self::JigsawReplacement => "minecraft:jigsaw_replacement",
            Self::Rule => "minecraft:rule",
            Self::Nop => "minecraft:nop",
            Self::BlockAge => "minecraft:block_age",
            Self::BlackstoneReplace => "minecraft:blackstone_replace",
            Self::LavaSubmergedBlock => "minecraft:lava_submerged_block",
            Self::ProtectedBlocks => "minecraft:protected_blocks",
            Self::Capped => "minecraft:capped",
        }
    }
}

impl StructureProcessorModel {
    pub fn processor_type(&self) -> StructureProcessorTypeModel {
        match self {
            Self::BlockIgnore { .. } => StructureProcessorTypeModel::BlockIgnore,
            Self::BlockRot { .. } => StructureProcessorTypeModel::BlockRot,
            Self::Gravity { .. } => StructureProcessorTypeModel::Gravity,
            Self::JigsawReplacement => StructureProcessorTypeModel::JigsawReplacement,
            Self::Rule { .. } => StructureProcessorTypeModel::Rule,
            Self::Nop => StructureProcessorTypeModel::Nop,
            Self::BlockAge { .. } => StructureProcessorTypeModel::BlockAge,
            Self::BlackstoneReplace => StructureProcessorTypeModel::BlackstoneReplace,
            Self::LavaSubmergedBlock => StructureProcessorTypeModel::LavaSubmergedBlock,
            Self::ProtectedBlocks { .. } => StructureProcessorTypeModel::ProtectedBlocks,
            Self::Capped { .. } => StructureProcessorTypeModel::Capped,
        }
    }

    pub fn codec_id(&self) -> &'static str {
        self.processor_type().id()
    }

    pub fn block_ignore_should_drop(&self, block: &str) -> bool {
        match self {
            Self::BlockIgnore { blocks } => blocks.contains(&block),
            _ => false,
        }
    }

    pub fn block_rot_keeps(
        &self,
        original_block_in_rottable_set: bool,
        random_next_float: f32,
    ) -> bool {
        match self {
            Self::BlockRot {
                rottable_blocks,
                integrity,
            } => {
                let applies = rottable_blocks.is_none() || original_block_in_rottable_set;
                !applies || random_next_float <= *integrity
            }
            _ => true,
        }
    }

    pub fn jigsaw_replacement_output(
        &self,
        input_block: &'static str,
        final_state: Option<&'static str>,
        debug_keep_jigsaws: bool,
    ) -> Option<&'static str> {
        if !matches!(self, Self::JigsawReplacement) {
            return Some(input_block);
        }
        if input_block != "minecraft:jigsaw" || debug_keep_jigsaws {
            return Some(input_block);
        }
        match final_state.unwrap_or("minecraft:air") {
            "minecraft:structure_void" => None,
            state => Some(state),
        }
    }

    pub fn gravity_adjusted_y(
        &self,
        level_height: i32,
        original_template_y: i32,
        server_level: bool,
    ) -> Option<(&'static str, i32)> {
        match self {
            Self::Gravity { heightmap, offset } => {
                let heightmap = match (*heightmap, server_level) {
                    ("WORLD_SURFACE_WG", true) => "WORLD_SURFACE",
                    ("OCEAN_FLOOR_WG", true) => "OCEAN_FLOOR",
                    _ => *heightmap,
                };
                Some((heightmap, level_height + offset + original_template_y))
            }
            _ => None,
        }
    }

    pub fn lava_submerged_output(
        &self,
        existing_block: &'static str,
        processed_shape_full_block: bool,
        processed_block: &'static str,
    ) -> &'static str {
        if matches!(self, Self::LavaSubmergedBlock)
            && existing_block == "minecraft:lava"
            && !processed_shape_full_block
        {
            "minecraft:lava"
        } else {
            processed_block
        }
    }

    pub fn capped_can_run(
        &self,
        original_len: usize,
        processed_len: usize,
        sampled_limit: i32,
    ) -> bool {
        matches!(self, Self::Capped { .. })
            && sampled_limit > 0
            && processed_len > 0
            && original_len == processed_len
    }
}

impl StructureRuleTestTypeModel {
    pub const REGISTRY_ORDER: [Self; 6] = [
        Self::AlwaysTrue,
        Self::BlockMatch,
        Self::BlockStateMatch,
        Self::TagMatch,
        Self::RandomBlockMatch,
        Self::RandomBlockStateMatch,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::AlwaysTrue => "minecraft:always_true",
            Self::BlockMatch => "minecraft:block_match",
            Self::BlockStateMatch => "minecraft:blockstate_match",
            Self::TagMatch => "minecraft:tag_match",
            Self::RandomBlockMatch => "minecraft:random_block_match",
            Self::RandomBlockStateMatch => "minecraft:random_blockstate_match",
        }
    }
}

impl StructurePosRuleTestTypeModel {
    pub const REGISTRY_ORDER: [Self; 3] = [
        Self::AlwaysTrue,
        Self::LinearPos,
        Self::AxisAlignedLinearPos,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::AlwaysTrue => "minecraft:always_true",
            Self::LinearPos => "minecraft:linear_pos",
            Self::AxisAlignedLinearPos => "minecraft:axis_aligned_linear_pos",
        }
    }
}

impl RuleBlockEntityModifierTypeModel {
    pub const REGISTRY_ORDER: [Self; 4] = [
        Self::Clear,
        Self::Passthrough,
        Self::AppendStatic,
        Self::AppendLoot,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Clear => "minecraft:clear",
            Self::Passthrough => "minecraft:passthrough",
            Self::AppendStatic => "minecraft:append_static",
            Self::AppendLoot => "minecraft:append_loot",
        }
    }
}

impl TemplateCompoundTagModel {
    pub fn with_string(mut self, key: &'static str, value: &'static str) -> Self {
        self.values
            .insert(key, TemplateNbtValueModel::String(value));
        self
    }

    pub fn with_long(mut self, key: &'static str, value: i64) -> Self {
        self.values.insert(key, TemplateNbtValueModel::Long(value));
        self
    }

    pub fn merge(&mut self, other: &Self) {
        for (key, value) in &other.values {
            self.values.insert(*key, value.clone());
        }
    }
}

impl RuleBlockEntityModifierModel {
    pub fn modifier_type(&self) -> RuleBlockEntityModifierTypeModel {
        match self {
            Self::Clear => RuleBlockEntityModifierTypeModel::Clear,
            Self::Passthrough => RuleBlockEntityModifierTypeModel::Passthrough,
            Self::AppendStatic { .. } => RuleBlockEntityModifierTypeModel::AppendStatic,
            Self::AppendLoot { .. } => RuleBlockEntityModifierTypeModel::AppendLoot,
        }
    }

    pub fn codec_id(&self) -> &'static str {
        self.modifier_type().id()
    }

    pub fn apply(
        &self,
        random: &mut RandomSourceKind,
        existing_tag: Option<TemplateCompoundTagModel>,
    ) -> Option<TemplateCompoundTagModel> {
        match self {
            Self::Clear => Some(TemplateCompoundTagModel::default()),
            Self::Passthrough => existing_tag,
            Self::AppendStatic { data } => {
                let mut result = existing_tag.unwrap_or_default();
                result.merge(data);
                Some(result)
            }
            Self::AppendLoot { loot_table } => {
                let mut result = existing_tag.unwrap_or_default();
                result
                    .values
                    .insert("LootTable", TemplateNbtValueModel::String(loot_table));
                result.values.insert(
                    "LootTableSeed",
                    TemplateNbtValueModel::Long(random_next_i64(random)),
                );
                Some(result)
            }
        }
    }
}

impl TemplatePathFactoryModel {
    pub fn new(source_dir: impl Into<String>) -> Self {
        Self {
            source_dir: source_dir.into(),
        }
    }

    pub fn for_pack_type(source_dir: &str, pack_type_dir: &str) -> Self {
        Self::new(format!(
            "{}/{}",
            source_dir.trim_end_matches('/'),
            pack_type_dir.trim_matches('/')
        ))
    }

    pub fn create_and_validate_path_to_structure(
        &self,
        id: &Identifier,
        kind: StructureTemplateFileKind,
    ) -> Result<String, String> {
        let relative_path = match kind {
            StructureTemplateFileKind::Nbt => format!("structure/{}.nbt", id.path()),
            StructureTemplateFileKind::Snbt => format!("structure/{}.snbt", id.path()),
        };
        let file_id = Identifier::new(id.namespace(), &relative_path)?;
        self.create_and_validate_path_to_resource(&file_id)
    }

    pub fn create_and_validate_path_to_resource(
        &self,
        resource_location: &Identifier,
    ) -> Result<String, String> {
        let mut parts = Vec::new();
        for part in resource_location.path().split('/') {
            if part.is_empty() || part == "." || part == ".." {
                return Err(format!(
                    "Invalid file path '{}': invalid path segment '{}'",
                    resource_location, part
                ));
            }
            if !is_template_path_part_portable(part) {
                return Err(format!(
                    "Resource path '{}' is not portable",
                    resource_location
                ));
            }
            parts.push(part);
        }

        Ok(format!(
            "{}/{}/{}",
            self.source_dir.trim_end_matches('/'),
            resource_location.namespace(),
            parts.join("/")
        ))
    }
}

impl StructureTemplateManagerModel {
    pub const STRUCTURE_DIRECTORY_NAME: &'static str = "structure";
    pub const STRUCTURE_FILE_EXTENSION: &'static str = ".nbt";
    pub const STRUCTURE_TEXT_FILE_EXTENSION: &'static str = ".snbt";

    pub fn get_or_create(&mut self, id: Identifier) -> &'static str {
        self.cache.entry(id).or_insert(Some("runtime_template"));
        "runtime_template"
    }

    pub fn get_or_try_load(
        &mut self,
        id: Identifier,
        loader: impl FnOnce(&Identifier) -> Option<&'static str>,
    ) -> Option<&'static str> {
        if let Some(cached) = self.cache.get(&id) {
            return *cached;
        }
        let loaded = loader(&id);
        self.cache.insert(id, loaded);
        loaded
    }

    pub fn remove(&mut self, id: &Identifier) {
        self.cache.remove(id);
    }

    pub fn on_resource_manager_reload(&mut self) {
        self.cache.clear();
    }

    pub fn save_kind(debug_save_as_snbt: bool) -> StructureTemplateFileKind {
        if debug_save_as_snbt {
            StructureTemplateFileKind::Snbt
        } else {
            StructureTemplateFileKind::Nbt
        }
    }

    pub fn try_load_from_sources(
        &mut self,
        id: Identifier,
        sources: &[TemplateSourceModel],
    ) -> (Option<&'static str>, Vec<TemplateLoadAttemptModel>) {
        if let Some(cached) = self.cache.get(&id) {
            return (*cached, Vec::new());
        }

        let mut attempts = Vec::new();
        for source in sources {
            let (loaded, result) = source.load(&id);
            attempts.push(TemplateLoadAttemptModel {
                source_kind: source.kind,
                id: id.clone(),
                result,
            });
            if loaded.is_some() {
                self.cache.insert(id, loaded);
                return (loaded, attempts);
            }
        }

        self.cache.insert(id, None);
        (None, attempts)
    }

    pub fn list_templates_from_sources(sources: &[TemplateSourceModel]) -> Vec<Identifier> {
        let mut listed = Vec::new();
        for source in sources {
            for id in source.list() {
                if !listed.contains(&id) {
                    listed.push(id);
                }
            }
        }
        listed
    }
}

fn is_template_path_part_portable(part: &str) -> bool {
    !part.is_empty()
        && part.chars().all(|ch| {
            ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '_' | '-' | '.')
        })
}

impl TemplateSourceModel {
    pub fn directory(
        source_dir: Option<&'static str>,
        load_as_text: bool,
        available: &[(&str, &'static str)],
        fail_on_load: &[&str],
    ) -> Self {
        Self::new(
            TemplateSourceKindModel::Directory,
            source_dir,
            load_as_text,
            available,
            fail_on_load,
        )
    }

    pub fn resource_manager(available: &[(&str, &'static str)], fail_on_load: &[&str]) -> Self {
        Self::new(
            TemplateSourceKindModel::ResourceManager,
            None,
            false,
            available,
            fail_on_load,
        )
    }

    fn new(
        kind: TemplateSourceKindModel,
        source_dir: Option<&'static str>,
        load_as_text: bool,
        available: &[(&str, &'static str)],
        fail_on_load: &[&str],
    ) -> Self {
        Self {
            kind,
            source_dir,
            load_as_text,
            available: available
                .iter()
                .map(|(id, template)| {
                    (
                        Identifier::parse(id).expect("valid template id fixture"),
                        *template,
                    )
                })
                .collect(),
            fail_on_load: fail_on_load
                .iter()
                .map(|id| Identifier::parse(id).expect("valid template id fixture"))
                .collect(),
        }
    }

    pub fn load(&self, id: &Identifier) -> (Option<&'static str>, TemplateLoadAttemptResultModel) {
        if self.kind == TemplateSourceKindModel::Directory && self.source_dir.is_none() {
            return (None, TemplateLoadAttemptResultModel::SourceUnavailable);
        }
        if self.fail_on_load.contains(id) {
            return (None, TemplateLoadAttemptResultModel::ErrorSuppressed);
        }
        match self.available.get(id).copied() {
            Some(template) => (Some(template), TemplateLoadAttemptResultModel::Loaded),
            None => (None, TemplateLoadAttemptResultModel::Missing),
        }
    }

    pub fn list(&self) -> Vec<Identifier> {
        if self.kind == TemplateSourceKindModel::Directory && self.source_dir.is_none() {
            return Vec::new();
        }
        self.available.keys().cloned().collect()
    }
}

pub fn structure_random_rule_test_matches(
    block_matches: bool,
    probability: f32,
    random_next_float: f32,
) -> bool {
    block_matches && random_next_float < probability
}

pub fn structure_linear_pos_chance(
    dist: i32,
    min_dist: i32,
    max_dist: i32,
    min_chance: f32,
    max_chance: f32,
) -> Result<f32, String> {
    if min_dist >= max_dist {
        return Err(format!("Invalid range: [{min_dist},{max_dist}]"));
    }
    let t = ((dist - min_dist) as f32 / (max_dist - min_dist) as f32).clamp(0.0, 1.0);
    Ok(min_chance + (max_chance - min_chance) * t)
}

pub fn structure_axis_aligned_distance(
    world_pos: (i32, i32, i32),
    reference: (i32, i32, i32),
    axis: char,
) -> i32 {
    match axis {
        'x' | 'X' => (world_pos.0 - reference.0).abs(),
        'y' | 'Y' => (world_pos.1 - reference.1).abs(),
        'z' | 'Z' => (world_pos.2 - reference.2).abs(),
        _ => 0,
    }
}

