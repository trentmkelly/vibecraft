use super::*;

pub fn builtin_surface_rule_preset(id: &str) -> Option<&'static SurfaceRulePresetData> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_SURFACE_RULE_PRESETS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn surface_rule_type(id: &str) -> Option<&'static SurfaceRuleType> {
    SURFACE_RULE_TYPES.iter().find(|entry| entry.id == id)
}

pub fn surface_condition_type(id: &str) -> Option<&'static SurfaceConditionType> {
    SURFACE_CONDITION_TYPES.iter().find(|entry| entry.id == id)
}

fn random_source_next_f32(random: &mut RandomSourceKind) -> f32 {
    match random {
        RandomSourceKind::Legacy(random) => random.next_f32(),
        RandomSourceKind::Xoroshiro(random) => {
            ((random.next_i64() as u64 >> 40) as f32) / ((1_u32 << 24) as f32)
        }
    }
}

pub(super) fn surface_positional_random_float(
    seed: i64,
    algorithm: RandomAlgorithm,
    random_name: &str,
    x: i32,
    y: i32,
    z: i32,
) -> f32 {
    let base = random_state_seed_factories(seed, algorithm).base;
    let factory = random_state_named_factory(base, random_name);
    let mut random = factory.at(x, y, z);
    random_source_next_f32(&mut random)
}

pub fn surface_condition_test(
    condition: &SurfaceConditionSource,
    context: &SurfaceMaterialContext,
    height_context: &WorldGenerationHeightContext,
) -> bool {
    match condition {
        SurfaceConditionSource::Biome(targets) => targets.contains(&context.biome),
        SurfaceConditionSource::NoiseThreshold { min, max } => {
            context.noise >= *min && context.noise <= *max
        }
        SurfaceConditionSource::VerticalGradient {
            random_name,
            true_at_and_below,
            false_at_and_above,
        } => {
            let true_y = true_at_and_below.resolve_y(*height_context);
            let false_y = false_at_and_above.resolve_y(*height_context);
            if context.y <= true_y {
                true
            } else if context.y >= false_y {
                false
            } else {
                let probability = 1.0 - f64::from(context.y - true_y) / f64::from(false_y - true_y);
                surface_positional_random_float(
                    context.seed,
                    context.random_algorithm,
                    random_name,
                    context.x,
                    context.y,
                    context.z,
                ) < probability as f32
            }
        }
        SurfaceConditionSource::YAbove {
            anchor,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            let threshold = anchor.resolve_y(*height_context)
                + context.surface_depth * *surface_depth_multiplier;
            let block_y = context.y
                + if *add_stone_depth {
                    context.stone_depth_above
                } else {
                    0
                };
            block_y >= threshold
        }
        SurfaceConditionSource::Water {
            offset,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            let threshold =
                context.water_height + *offset + context.surface_depth * *surface_depth_multiplier;
            let block_y = context.y
                + if *add_stone_depth {
                    context.stone_depth_above
                } else {
                    0
                };
            context.water_height == i32::MIN || block_y >= threshold
        }
        SurfaceConditionSource::StoneDepth {
            offset,
            add_surface_depth,
            secondary_depth_range,
            surface,
        } => {
            let mut threshold = 1 + *offset;
            if *add_surface_depth {
                threshold += context.surface_depth;
            }
            if *secondary_depth_range != 0 {
                threshold +=
                    (((context.noise + 1.0) * 0.5) * f64::from(*secondary_depth_range)) as i32;
            }
            let depth = match surface {
                CaveSurface::Floor => context.stone_depth_above,
                CaveSurface::Ceiling => context.stone_depth_below,
            };
            depth <= threshold
        }
        SurfaceConditionSource::Not(target) => {
            !surface_condition_test(target, context, height_context)
        }
        SurfaceConditionSource::Steep => context.steep,
        SurfaceConditionSource::Hole => context.hole,
        SurfaceConditionSource::AbovePreliminarySurface => {
            context.y >= context.preliminary_surface_y
        }
        SurfaceConditionSource::Temperature => context.temperature < 0.15,
    }
}

pub fn surface_rule_apply(
    rule: &SurfaceRuleSource,
    context: &SurfaceMaterialContext,
    height_context: &WorldGenerationHeightContext,
) -> Option<&'static str> {
    match rule {
        SurfaceRuleSource::Bandlands => Some(match (context.x + context.z).rem_euclid(5) {
            0 => "minecraft:white_terracotta",
            1 => "minecraft:orange_terracotta",
            2 => "minecraft:terracotta",
            3 => "minecraft:red_sand",
            _ => "minecraft:red_sandstone",
        }),
        SurfaceRuleSource::Block(block) => Some(block),
        SurfaceRuleSource::Sequence(rules) => rules
            .iter()
            .find_map(|rule| surface_rule_apply(rule, context, height_context)),
        SurfaceRuleSource::Condition { condition, rule } => {
            surface_condition_test(condition, context, height_context)
                .then(|| surface_rule_apply(rule, context, height_context))
                .flatten()
        }
    }
}

// ── DynSurfaceRule JSON parsing ───────────────────────────────────────────────

/// Parse a `VerticalAnchor` from a JSON object that contains exactly one of
/// the three anchor-kind keys: `"absolute"`, `"above_bottom"`, or `"below_top"`.
///
/// Mirrors the Java `VerticalAnchor` codec.
pub fn parse_vertical_anchor_from_json(v: &serde_json::Value) -> Result<VerticalAnchor, String> {
    if let Some(n) = v.get("absolute").and_then(|x| x.as_i64()) {
        return Ok(VerticalAnchor::Absolute(n as i32));
    }
    if let Some(n) = v.get("above_bottom").and_then(|x| x.as_i64()) {
        return Ok(VerticalAnchor::AboveBottom(n as i32));
    }
    if let Some(n) = v.get("below_top").and_then(|x| x.as_i64()) {
        return Ok(VerticalAnchor::BelowTop(n as i32));
    }
    Err(format!("unrecognised vertical anchor: {v}"))
}

/// Parse a `DynSurfaceCondition` from a JSON object.
///
/// Mirrors Java's `SurfaceRules.ConditionSource` codec dispatch on `"type"`.
pub fn parse_dyn_surface_condition(v: &serde_json::Value) -> Result<DynSurfaceCondition, String> {
    let ty = v["type"].as_str().ok_or("surface condition missing type")?;
    let name = ty.strip_prefix("minecraft:").unwrap_or(ty);
    match name {
        "biome" => {
            let arr = v["biome_is"]
                .as_array()
                .ok_or("biome condition missing biome_is")?;
            let biomes = arr
                .iter()
                .map(|b| {
                    b.as_str()
                        .ok_or("biome_is entry not a string")
                        .map(|s| s.to_string())
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DynSurfaceCondition::Biome(biomes))
        }
        "noise_threshold" => {
            let noise = v["noise"]
                .as_str()
                .ok_or("noise_threshold missing noise")?
                .to_string();
            let min = v["min_threshold"]
                .as_f64()
                .ok_or("noise_threshold missing min_threshold")?;
            let max = v["max_threshold"]
                .as_f64()
                .ok_or("noise_threshold missing max_threshold")?;
            Ok(DynSurfaceCondition::NoiseThreshold { noise, min, max })
        }
        "vertical_gradient" => {
            let random_name = v["random_name"]
                .as_str()
                .ok_or("vertical_gradient missing random_name")?
                .to_string();
            let true_at = parse_vertical_anchor_from_json(&v["true_at_and_below"])?;
            let false_at = parse_vertical_anchor_from_json(&v["false_at_and_above"])?;
            Ok(DynSurfaceCondition::VerticalGradient {
                random_name,
                true_at_and_below: true_at,
                false_at_and_above: false_at,
            })
        }
        "y_above" => {
            let anchor = parse_vertical_anchor_from_json(&v["anchor"])?;
            let multiplier = v["surface_depth_multiplier"]
                .as_i64()
                .ok_or("y_above missing surface_depth_multiplier")?
                as i32;
            let add_stone = v["add_stone_depth"]
                .as_bool()
                .ok_or("y_above missing add_stone_depth")?;
            Ok(DynSurfaceCondition::YAbove {
                anchor,
                surface_depth_multiplier: multiplier,
                add_stone_depth: add_stone,
            })
        }
        "water" => {
            let offset = v["offset"].as_i64().ok_or("water missing offset")? as i32;
            let multiplier = v["surface_depth_multiplier"]
                .as_i64()
                .ok_or("water missing surface_depth_multiplier")?
                as i32;
            let add_stone = v["add_stone_depth"]
                .as_bool()
                .ok_or("water missing add_stone_depth")?;
            Ok(DynSurfaceCondition::Water {
                offset,
                surface_depth_multiplier: multiplier,
                add_stone_depth: add_stone,
            })
        }
        "stone_depth" => {
            let offset = v["offset"].as_i64().ok_or("stone_depth missing offset")? as i32;
            let add_depth = v["add_surface_depth"]
                .as_bool()
                .ok_or("stone_depth missing add_surface_depth")?;
            let secondary = v["secondary_depth_range"]
                .as_i64()
                .ok_or("stone_depth missing secondary_depth_range")?
                as i32;
            let surface = match v["surface_type"]
                .as_str()
                .ok_or("stone_depth missing surface_type")?
            {
                "floor" => CaveSurface::Floor,
                "ceiling" => CaveSurface::Ceiling,
                s => return Err(format!("unknown stone_depth surface_type: {s}")),
            };
            Ok(DynSurfaceCondition::StoneDepth {
                offset,
                add_surface_depth: add_depth,
                secondary_depth_range: secondary,
                surface,
            })
        }
        "not" => {
            let inner = parse_dyn_surface_condition(&v["invert"])?;
            Ok(DynSurfaceCondition::Not(Box::new(inner)))
        }
        "steep" => Ok(DynSurfaceCondition::Steep),
        "hole" => Ok(DynSurfaceCondition::Hole),
        "above_preliminary_surface" => Ok(DynSurfaceCondition::AbovePreliminarySurface),
        "temperature" => Ok(DynSurfaceCondition::Temperature),
        _ => Err(format!("unknown surface condition type: {ty}")),
    }
}

/// Parse a `DynSurfaceRule` from a JSON object.
///
/// Mirrors Java's `SurfaceRules.RuleSource` codec dispatch on `"type"`.
pub fn parse_dyn_surface_rule(v: &serde_json::Value) -> Result<DynSurfaceRule, String> {
    let ty = v["type"].as_str().ok_or("surface rule missing type")?;
    let name = ty.strip_prefix("minecraft:").unwrap_or(ty);
    match name {
        "sequence" => {
            let arr = v["sequence"]
                .as_array()
                .ok_or("sequence rule missing sequence array")?;
            let rules = arr
                .iter()
                .map(parse_dyn_surface_rule)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DynSurfaceRule::Sequence(rules))
        }
        "condition" => {
            let condition = parse_dyn_surface_condition(&v["if_true"])?;
            let rule = parse_dyn_surface_rule(&v["then_run"])?;
            Ok(DynSurfaceRule::Condition {
                condition: Box::new(condition),
                rule: Box::new(rule),
            })
        }
        "block" => {
            let block_name = v["result_state"]["Name"]
                .as_str()
                .ok_or("block rule missing result_state.Name")?
                .to_string();
            Ok(DynSurfaceRule::Block(block_name))
        }
        "bandlands" => Ok(DynSurfaceRule::Bandlands),
        _ => Err(format!("unknown surface rule type: {ty}")),
    }
}

// ── DynSurfaceRule evaluation ─────────────────────────────────────────────────

/// Reusable per-chunk surface context, shaped after Java's
/// `SurfaceRules.Context`.
///
/// Java constructs this once from `SurfaceSystem.buildSurface`, applies the
/// rule source to it, then calls `updateXZ` per column and `updateY` per solid
/// block. RustCraft's dynamic rule tree is still interpreted, but its input
/// state now follows that same lifecycle.
pub(super) struct SurfaceRulesContext {
    pub(super) seed: i64,
    pub(super) algorithm: RandomAlgorithm,
    pub(super) heights: WorldGenerationHeightContext,
    pub(super) last_update_xz: u64,
    pub(super) last_update_y: u64,
    pub(super) condition_cache: RefCell<HashMap<usize, SurfaceConditionCacheEntry>>,
    pub(super) profile: Option<RefCell<SurfaceRuleProfile>>,
    // Per column (XZ) — set once per column
    pub(super) block_x: i32,
    pub(super) block_z: i32,
    pub(super) surface_depth: i32,
    pub(super) surface_secondary: f64,
    pub(super) steep: bool,
    pub(super) hole: bool,
    pub(super) min_surface_level: i32,
    // Per block (Y) — updated per block
    pub(super) block_y: i32,
    pub(super) water_height: i32,
    pub(super) stone_depth_above: i32,
    pub(super) stone_depth_below: i32,
    pub(super) biome: &'static str,
    pub(super) temperature: f32,
    pub(super) biome_needs_update: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SurfaceConditionCacheEntry {
    update_key: u64,
    result: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SurfaceConditionKindProfile {
    pub(super) tests: usize,
    pub(super) cache_hits: usize,
    pub(super) computes: usize,
    pub(super) compute_us: u128,
}

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct SurfaceRuleProfile {
    pub(super) rule_visits: usize,
    pub(super) block_rule_visits: usize,
    pub(super) condition_rule_visits: usize,
    pub(super) sequence_rule_visits: usize,
    pub(super) bandlands_rule_visits: usize,
    pub(super) condition_tests: usize,
    pub(super) condition_cache_hits: usize,
    pub(super) condition_computes: usize,
    pub(super) condition_compute_us: u128,
    pub(super) biome: SurfaceConditionKindProfile,
    pub(super) noise_threshold: SurfaceConditionKindProfile,
    pub(super) vertical_gradient: SurfaceConditionKindProfile,
    pub(super) y_above: SurfaceConditionKindProfile,
    pub(super) water: SurfaceConditionKindProfile,
    pub(super) stone_depth: SurfaceConditionKindProfile,
    pub(super) not: SurfaceConditionKindProfile,
    pub(super) steep: SurfaceConditionKindProfile,
    pub(super) hole: SurfaceConditionKindProfile,
    pub(super) above_preliminary_surface: SurfaceConditionKindProfile,
    pub(super) temperature: SurfaceConditionKindProfile,
}

impl SurfaceRuleProfile {
    fn for_condition_kind_mut(
        &mut self,
        condition: &DynSurfaceCondition,
    ) -> &mut SurfaceConditionKindProfile {
        match condition {
            DynSurfaceCondition::Biome(_) => &mut self.biome,
            DynSurfaceCondition::NoiseThreshold { .. } => &mut self.noise_threshold,
            DynSurfaceCondition::VerticalGradient { .. } => &mut self.vertical_gradient,
            DynSurfaceCondition::YAbove { .. } => &mut self.y_above,
            DynSurfaceCondition::Water { .. } => &mut self.water,
            DynSurfaceCondition::StoneDepth { .. } => &mut self.stone_depth,
            DynSurfaceCondition::Not(_) => &mut self.not,
            DynSurfaceCondition::Steep => &mut self.steep,
            DynSurfaceCondition::Hole => &mut self.hole,
            DynSurfaceCondition::AbovePreliminarySurface => &mut self.above_preliminary_surface,
            DynSurfaceCondition::Temperature => &mut self.temperature,
        }
    }

    fn record_condition_test(&mut self, condition: &DynSurfaceCondition) {
        self.condition_tests += 1;
        self.for_condition_kind_mut(condition).tests += 1;
    }

    fn record_condition_cache_hit(&mut self, condition: &DynSurfaceCondition) {
        self.condition_cache_hits += 1;
        self.for_condition_kind_mut(condition).cache_hits += 1;
    }

    fn record_condition_compute(&mut self, condition: &DynSurfaceCondition, elapsed_us: u128) {
        self.condition_computes += 1;
        self.condition_compute_us += elapsed_us;
        let kind = self.for_condition_kind_mut(condition);
        kind.computes += 1;
        kind.compute_us += elapsed_us;
    }
}

#[cfg(test)]
pub(super) type BuildSurfaceColumnState = SurfaceRulesContext;

impl SurfaceRulesContext {
    pub(super) fn new(
        seed: i64,
        algorithm: RandomAlgorithm,
        heights: WorldGenerationHeightContext,
    ) -> Self {
        Self {
            seed,
            algorithm,
            heights,
            last_update_xz: 0,
            last_update_y: 0,
            condition_cache: std::cell::RefCell::new(std::collections::HashMap::with_capacity(256)),
            profile: std::env::var_os("RUSTCRAFT_WORLDGEN_SURFACE_DEBUG")
                .map(|_| RefCell::new(SurfaceRuleProfile::default())),
            block_x: 0,
            block_z: 0,
            surface_depth: 0,
            surface_secondary: 0.0,
            steep: false,
            hole: false,
            min_surface_level: 0,
            block_y: 0,
            water_height: i32::MIN,
            stone_depth_above: 0,
            stone_depth_below: 0,
            biome: "minecraft:plains",
            temperature: 0.8,
            biome_needs_update: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn update_xz(
        &mut self,
        block_x: i32,
        block_z: i32,
        surface_depth: i32,
        surface_secondary: f64,
        steep: bool,
        hole: bool,
        min_surface_level: i32,
    ) {
        self.last_update_xz = self.last_update_xz.wrapping_add(1);
        self.last_update_y = self.last_update_y.wrapping_add(1);
        self.block_x = block_x;
        self.block_z = block_z;
        self.surface_depth = surface_depth;
        self.surface_secondary = surface_secondary;
        self.steep = steep;
        self.hole = hole;
        self.min_surface_level = min_surface_level;
    }

    pub(super) fn update_y(
        &mut self,
        stone_depth_above: i32,
        stone_depth_below: i32,
        water_height: i32,
        block_y: i32,
    ) {
        self.last_update_y = self.last_update_y.wrapping_add(1);
        self.block_y = block_y;
        self.water_height = water_height;
        self.stone_depth_above = stone_depth_above;
        self.stone_depth_below = stone_depth_below;
        self.biome_needs_update = true;
    }

    fn resolve_biome(&mut self, resolver: &mut impl FnMut(i32, i32, i32) -> (&'static str, f32)) {
        if !self.biome_needs_update {
            return;
        }
        let (biome, temperature) = resolver(self.block_x, self.block_y, self.block_z);
        self.biome = biome;
        self.temperature = temperature;
        self.biome_needs_update = false;
    }
}

#[derive(Debug, Clone, Copy)]
enum SurfaceConditionCacheGranularity {
    Xz,
    Y,
}

impl DynSurfaceCondition {
    fn cache_granularity(&self) -> SurfaceConditionCacheGranularity {
        match self {
            DynSurfaceCondition::NoiseThreshold { .. }
            | DynSurfaceCondition::Steep
            | DynSurfaceCondition::Hole => SurfaceConditionCacheGranularity::Xz,
            DynSurfaceCondition::Biome(_)
            | DynSurfaceCondition::VerticalGradient { .. }
            | DynSurfaceCondition::YAbove { .. }
            | DynSurfaceCondition::Water { .. }
            | DynSurfaceCondition::StoneDepth { .. }
            | DynSurfaceCondition::Not(_)
            | DynSurfaceCondition::AbovePreliminarySurface
            | DynSurfaceCondition::Temperature => SurfaceConditionCacheGranularity::Y,
        }
    }
}

/// Test a `DynSurfaceCondition` against the current column/block state.
///
/// Mirrors Java's `SurfaceRules.Condition::test()`.
pub(super) fn dyn_surface_condition_test(
    cond: &DynSurfaceCondition,
    state: &SurfaceRulesContext,
    settings: NoiseGeneratorSettings,
) -> bool {
    if state.profile.is_none() {
        return dyn_surface_condition_compute(cond, state, settings);
    }
    if let Some(profile) = &state.profile {
        profile.borrow_mut().record_condition_test(cond);
    }
    let update_key = match cond.cache_granularity() {
        SurfaceConditionCacheGranularity::Xz => state.last_update_xz,
        SurfaceConditionCacheGranularity::Y => state.last_update_y,
    };
    let cache_key = cond as *const DynSurfaceCondition as usize;
    if let Some(entry) = state.condition_cache.borrow().get(&cache_key).copied() {
        if entry.update_key == update_key {
            if let Some(profile) = &state.profile {
                profile.borrow_mut().record_condition_cache_hit(cond);
            }
            return entry.result;
        }
    }
    let compute_started = state.profile.as_ref().map(|_| Instant::now());
    let result = dyn_surface_condition_compute(cond, state, settings);
    if let Some(started) = compute_started {
        if let Some(profile) = &state.profile {
            profile
                .borrow_mut()
                .record_condition_compute(cond, started.elapsed().as_micros());
        }
    }
    state
        .condition_cache
        .borrow_mut()
        .insert(cache_key, SurfaceConditionCacheEntry { update_key, result });
    result
}

fn dyn_surface_condition_test_live(
    cond: &DynSurfaceCondition,
    state: &mut SurfaceRulesContext,
    settings: NoiseGeneratorSettings,
    biome_resolver: &mut impl FnMut(i32, i32, i32) -> (&'static str, f32),
) -> bool {
    if state.profile.is_none() {
        return dyn_surface_condition_compute_live(cond, state, settings, biome_resolver);
    }
    if let Some(profile) = &state.profile {
        profile.borrow_mut().record_condition_test(cond);
    }
    let update_key = match cond.cache_granularity() {
        SurfaceConditionCacheGranularity::Xz => state.last_update_xz,
        SurfaceConditionCacheGranularity::Y => state.last_update_y,
    };
    let cache_key = cond as *const DynSurfaceCondition as usize;
    if let Some(entry) = state.condition_cache.borrow().get(&cache_key).copied() {
        if entry.update_key == update_key {
            if let Some(profile) = &state.profile {
                profile.borrow_mut().record_condition_cache_hit(cond);
            }
            return entry.result;
        }
    }
    let compute_started = state.profile.as_ref().map(|_| Instant::now());
    let result = dyn_surface_condition_compute_live(cond, state, settings, biome_resolver);
    if let Some(started) = compute_started {
        if let Some(profile) = &state.profile {
            profile
                .borrow_mut()
                .record_condition_compute(cond, started.elapsed().as_micros());
        }
    }
    state
        .condition_cache
        .borrow_mut()
        .insert(cache_key, SurfaceConditionCacheEntry { update_key, result });
    result
}

fn dyn_surface_condition_compute(
    cond: &DynSurfaceCondition,
    state: &SurfaceRulesContext,
    settings: NoiseGeneratorSettings,
) -> bool {
    match cond {
        DynSurfaceCondition::Biome(biomes) => biomes.iter().any(|b| b == &state.biome),
        DynSurfaceCondition::NoiseThreshold { noise, min, max } => {
            // Sample the named noise at (blockX, 0, blockZ).
            // Uses the thread-local noise cache when active.
            let value = random_state_normal_noise_snapshot(state.seed, settings, noise)
                .map(|snap| {
                    normal_noise_sample(&snap, state.block_x as f64, 0.0, state.block_z as f64)
                })
                .unwrap_or(0.0);
            value >= *min && value <= *max
        }
        DynSurfaceCondition::VerticalGradient {
            random_name,
            true_at_and_below,
            false_at_and_above,
        } => {
            let true_y = true_at_and_below.resolve_y(state.heights);
            let false_y = false_at_and_above.resolve_y(state.heights);
            if state.block_y <= true_y {
                true
            } else if state.block_y >= false_y {
                false
            } else {
                let probability =
                    1.0 - f64::from(state.block_y - true_y) / f64::from(false_y - true_y);
                surface_positional_random_float(
                    state.seed,
                    state.algorithm,
                    random_name,
                    state.block_x,
                    state.block_y,
                    state.block_z,
                ) < probability as f32
            }
        }
        DynSurfaceCondition::YAbove {
            anchor,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            let threshold =
                anchor.resolve_y(state.heights) + state.surface_depth * surface_depth_multiplier;
            let block_y = state.block_y
                + if *add_stone_depth {
                    state.stone_depth_above
                } else {
                    0
                };
            block_y >= threshold
        }
        DynSurfaceCondition::Water {
            offset,
            surface_depth_multiplier,
            add_stone_depth,
        } => {
            if state.water_height == i32::MIN {
                return true; // no water column → block is above any water
            }
            let threshold =
                state.water_height + offset + state.surface_depth * surface_depth_multiplier;
            let block_y = state.block_y
                + if *add_stone_depth {
                    state.stone_depth_above
                } else {
                    0
                };
            block_y >= threshold
        }
        DynSurfaceCondition::StoneDepth {
            offset,
            add_surface_depth,
            secondary_depth_range,
            surface,
        } => {
            let mut threshold = 1 + offset;
            if *add_surface_depth {
                threshold += state.surface_depth;
            }
            if *secondary_depth_range != 0 {
                // Java: Mth.map(surfaceSecondary, -1.0, 1.0, 0.0, secondaryDepthRange)
                //      = (surfaceSecondary + 1.0) / 2.0 * secondaryDepthRange
                threshold += ((state.surface_secondary + 1.0)
                    * 0.5
                    * f64::from(*secondary_depth_range)) as i32;
            }
            let depth = match surface {
                CaveSurface::Floor => state.stone_depth_above,
                CaveSurface::Ceiling => state.stone_depth_below,
            };
            depth <= threshold
        }
        DynSurfaceCondition::Not(inner) => !dyn_surface_condition_test(inner, state, settings),
        DynSurfaceCondition::Steep => state.steep,
        DynSurfaceCondition::Hole => state.hole,
        DynSurfaceCondition::AbovePreliminarySurface => state.block_y >= state.min_surface_level,
        DynSurfaceCondition::Temperature => state.temperature < 0.15,
    }
}

fn dyn_surface_condition_compute_live(
    cond: &DynSurfaceCondition,
    state: &mut SurfaceRulesContext,
    settings: NoiseGeneratorSettings,
    biome_resolver: &mut impl FnMut(i32, i32, i32) -> (&'static str, f32),
) -> bool {
    match cond {
        DynSurfaceCondition::Biome(biomes) => {
            state.resolve_biome(biome_resolver);
            biomes.iter().any(|b| b == &state.biome)
        }
        DynSurfaceCondition::Temperature => {
            state.resolve_biome(biome_resolver);
            state.temperature < 0.15
        }
        DynSurfaceCondition::Not(inner) => {
            !dyn_surface_condition_test_live(inner, state, settings, biome_resolver)
        }
        _ => dyn_surface_condition_compute(cond, state, settings),
    }
}

/// Evaluate a `DynSurfaceRule` against the current column/block state.
///
/// Returns the block ID to place, or `None` if no rule matches.
/// Mirrors Java's `SurfaceRules.SurfaceRule::tryApply(blockX, blockY, blockZ)`.
pub(super) fn dyn_surface_rule_apply<'a>(
    rule: &'a DynSurfaceRule,
    state: &mut SurfaceRulesContext,
    settings: NoiseGeneratorSettings,
    band_fn: &impl Fn(i32, i32, i32) -> &'static str,
    biome_resolver: &mut impl FnMut(i32, i32, i32) -> (&'static str, f32),
) -> Option<&'a str> {
    if let Some(profile) = &state.profile {
        let mut profile = profile.borrow_mut();
        profile.rule_visits += 1;
        match rule {
            DynSurfaceRule::Bandlands => profile.bandlands_rule_visits += 1,
            DynSurfaceRule::Block(_) => profile.block_rule_visits += 1,
            DynSurfaceRule::Sequence(_) => profile.sequence_rule_visits += 1,
            DynSurfaceRule::Condition { .. } => profile.condition_rule_visits += 1,
        }
    }
    match rule {
        DynSurfaceRule::Bandlands => Some(band_fn(state.block_x, state.block_y, state.block_z)),
        DynSurfaceRule::Block(block) => Some(block.as_str()),
        DynSurfaceRule::Sequence(rules) => rules
            .iter()
            .find_map(|r| dyn_surface_rule_apply(r, state, settings, band_fn, biome_resolver)),
        DynSurfaceRule::Condition { condition, rule } => {
            if dyn_surface_condition_test_live(condition, state, settings, biome_resolver) {
                dyn_surface_rule_apply(rule, state, settings, band_fn, biome_resolver)
            } else {
                None
            }
        }
    }
}

// ── Surface rule loading from JSON ────────────────────────────────────────────

/// Cache of loaded `DynSurfaceRule` values keyed by noise settings ID.
/// Populated lazily on first access from the data directory.
static SURFACE_RULE_CACHE: std::sync::OnceLock<std::sync::Mutex<HashMap<String, DynSurfaceRule>>> =
    std::sync::OnceLock::new();

fn surface_rule_cache() -> &'static std::sync::Mutex<HashMap<String, DynSurfaceRule>> {
    SURFACE_RULE_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

/// Load (or return from cache) the `DynSurfaceRule` for the given noise settings ID.
///
/// Parse the `surface_rule` field from a noise settings JSON file.
///
/// Reads from the canonical data location used by the test suite:
/// `../decompiled-server-26.1.2/data/minecraft/worldgen/noise_settings/<name>.json`.
///
/// Results are cached so each settings ID is only loaded once.
///
/// Mirrors how Java deserialises `NoiseGeneratorSettings` via its Codec.
pub fn load_surface_rule(settings_id: &str) -> Option<DynSurfaceRule> {
    {
        let cache = surface_rule_cache().lock().ok()?;
        if let Some(rule) = cache.get(settings_id) {
            return Some(rule.clone());
        }
    }
    let name = settings_id
        .strip_prefix("minecraft:")
        .unwrap_or(settings_id);
    // Use the same data directory as the noise-settings parity tests.
    let path = format!(
        "../decompiled-server-26.1.2/data/minecraft/worldgen/noise_settings/{}.json",
        name
    );
    let raw = std::fs::read_to_string(&path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let rule = parse_dyn_surface_rule(&json["surface_rule"]).ok()?;
    let mut cache = surface_rule_cache().lock().ok()?;
    cache.insert(settings_id.to_string(), rule.clone());
    Some(rule)
}
