use super::*;

pub fn builtin_noise_generator_settings(id: &str) -> Option<&'static NoiseGeneratorSettings> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    BUILTIN_NOISE_GENERATOR_SETTINGS.iter().find(|settings| {
        settings
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|settings_name| settings_name == name)
    })
}

pub fn builtin_normal_noise_parameters(id: &str) -> Option<&'static NormalNoiseParameters> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    NORMAL_NOISE_PARAMETERS.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn synth_noise_source(id: &str) -> Option<&'static SynthNoiseSource> {
    let name = id.strip_prefix("minecraft:").unwrap_or(id);
    SYNTH_NOISE_SOURCES.iter().find(|entry| {
        entry
            .id
            .strip_prefix("minecraft:")
            .is_some_and(|entry_name| entry_name == name)
    })
}

pub fn normal_noise_expected_deviation(octave_span: i32) -> f64 {
    0.1 * (1.0 + 1.0 / (f64::from(octave_span) + 1.0))
}

pub fn normal_noise_value_factor(parameters: NormalNoiseParameters) -> f64 {
    let mut min_octave = i32::MAX;
    let mut max_octave = i32::MIN;
    for (index, amplitude) in parameters.amplitudes.iter().enumerate() {
        if *amplitude != 0.0 {
            min_octave = min_octave.min(index as i32);
            max_octave = max_octave.max(index as i32);
        }
    }
    NORMAL_NOISE_TARGET_DEVIATION / 2.0 / normal_noise_expected_deviation(max_octave - min_octave)
}

pub fn perlin_noise_edge_value_from_parameters(
    parameters: NormalNoiseParameters,
    noise_value: f64,
) -> f64 {
    let octave_count = parameters.amplitudes.len();
    let mut value = 0.0;
    let mut value_factor =
        2.0_f64.powi(octave_count as i32 - 1) / (2.0_f64.powi(octave_count as i32) - 1.0);
    for amplitude in parameters.amplitudes {
        if *amplitude != 0.0 {
            value += amplitude * noise_value * value_factor;
        }
        value_factor /= 2.0;
    }
    value
}

pub fn normal_noise_max_value(parameters: NormalNoiseParameters) -> f64 {
    perlin_noise_edge_value_from_parameters(parameters, 2.0)
        * 2.0
        * normal_noise_value_factor(parameters)
}

pub fn normal_noise_value_bounds(id: &str) -> Option<(f64, f64)> {
    builtin_normal_noise_parameters(id).map(|parameters| {
        let max_value = normal_noise_max_value(*parameters);
        (-max_value, max_value)
    })
}

pub fn density_shift_noise_sample(
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise: &'static str,
    local_x: f64,
    local_y: f64,
    local_z: f64,
) -> f64 {
    random_state_normal_noise_sample(
        seed,
        settings,
        noise,
        local_x * 0.25,
        local_y * 0.25,
        local_z * 0.25,
    ) * 4.0
}

pub fn normal_noise_non_zero_octaves(parameters: NormalNoiseParameters) -> Vec<i32> {
    parameters
        .amplitudes
        .iter()
        .enumerate()
        .filter_map(|(index, amplitude)| {
            (*amplitude != 0.0).then_some(parameters.first_octave + index as i32)
        })
        .collect()
}

pub fn perlin_noise_construction_plan(
    parameters: NormalNoiseParameters,
    use_new_initialization: bool,
) -> Result<PerlinNoiseConstructionPlan, &'static str> {
    let octave_count = parameters.amplitudes.len();
    let zero_octave_index = -parameters.first_octave;
    let mut levels = Vec::new();
    let mut legacy_skipped_octaves = Vec::new();
    let mut legacy_created_zero_octave_first = false;

    if use_new_initialization {
        for (index, amplitude) in parameters.amplitudes.iter().enumerate() {
            if *amplitude != 0.0 {
                levels.push(PerlinNoiseLevelPlan {
                    level_index: index,
                    octave: parameters.first_octave + index as i32,
                    source: PerlinNoiseLevelSource::HashedOctave,
                });
            }
        }
    } else {
        legacy_created_zero_octave_first = true;
        if zero_octave_index >= 0 && (zero_octave_index as usize) < octave_count {
            let zero_index = zero_octave_index as usize;
            if parameters.amplitudes[zero_index] != 0.0 {
                levels.push(PerlinNoiseLevelPlan {
                    level_index: zero_index,
                    octave: 0,
                    source: PerlinNoiseLevelSource::SequentialLegacyZero,
                });
            }
        }

        for index in (0..zero_octave_index).rev() {
            let octave = parameters.first_octave + index;
            if (index as usize) < octave_count && parameters.amplitudes[index as usize] != 0.0 {
                levels.push(PerlinNoiseLevelPlan {
                    level_index: index as usize,
                    octave,
                    source: PerlinNoiseLevelSource::SequentialLegacy,
                });
            } else {
                legacy_skipped_octaves.push(octave);
            }
        }

        if zero_octave_index < octave_count.saturating_sub(1) as i32 {
            return Err("positive octaves are temporarily disabled");
        }
    }

    Ok(PerlinNoiseConstructionPlan {
        first_octave: parameters.first_octave,
        octave_count,
        zero_octave_index,
        levels,
        legacy_created_zero_octave_first,
        legacy_skipped_octaves,
    })
}

pub fn perlin_noise_snapshot(
    mut random: RandomSourceKind,
    parameters: NormalNoiseParameters,
    use_new_initialization: bool,
) -> Result<PerlinNoiseSnapshot, &'static str> {
    perlin_noise_snapshot_from_random(&mut random, parameters, use_new_initialization)
}

fn perlin_noise_snapshot_from_random(
    random: &mut RandomSourceKind,
    parameters: NormalNoiseParameters,
    use_new_initialization: bool,
) -> Result<PerlinNoiseSnapshot, &'static str> {
    perlin_noise_construction_plan(parameters, use_new_initialization)?;
    let octave_count = parameters.amplitudes.len();
    let zero_octave_index = -parameters.first_octave;
    let mut levels = vec![None; octave_count];

    if use_new_initialization {
        let positional = random.fork_positional();
        for (index, amplitude) in parameters.amplitudes.iter().enumerate() {
            if *amplitude != 0.0 {
                let octave = parameters.first_octave + index as i32;
                let mut octave_random = positional.at_hashed_name(&format!("octave_{octave}"));
                levels[index] = Some(improved_noise_snapshot(&mut octave_random));
            }
        }
    } else {
        let zero_octave = improved_noise_snapshot(random);
        if zero_octave_index >= 0 && (zero_octave_index as usize) < octave_count {
            let zero_index = zero_octave_index as usize;
            if parameters.amplitudes[zero_index] != 0.0 {
                levels[zero_index] = Some(zero_octave);
            }
        }

        for index in (0..zero_octave_index).rev() {
            if (index as usize) < octave_count && parameters.amplitudes[index as usize] != 0.0 {
                levels[index as usize] = Some(improved_noise_snapshot(random));
            } else {
                random.consume_count(262);
            }
        }
    }

    Ok(PerlinNoiseSnapshot {
        first_octave: parameters.first_octave,
        amplitudes: parameters.amplitudes,
        levels,
        lowest_freq_input_factor: 2.0_f64.powi(-zero_octave_index),
        lowest_freq_value_factor: 2.0_f64.powi(octave_count as i32 - 1)
            / (2.0_f64.powi(octave_count as i32) - 1.0),
    })
}

pub fn perlin_noise_wrap(x: f64) -> f64 {
    x - (x / 33_554_432.0 + 0.5).floor() * 33_554_432.0
}

pub fn perlin_noise_sample(
    snapshot: &PerlinNoiseSnapshot,
    x: f64,
    y: f64,
    z: f64,
    y_scale: f64,
    y_fudge: f64,
) -> f64 {
    let mut value = 0.0;
    let mut factor = snapshot.lowest_freq_input_factor;
    let mut value_factor = snapshot.lowest_freq_value_factor;

    for (index, noise) in snapshot.levels.iter().enumerate() {
        if let Some(noise) = noise {
            let noise_value = improved_noise_sample(
                noise,
                perlin_noise_wrap(x * factor),
                perlin_noise_wrap(y * factor),
                perlin_noise_wrap(z * factor),
                y_scale * factor,
                y_fudge * factor,
            );
            value += snapshot.amplitudes[index] * noise_value * value_factor;
        }
        factor *= 2.0;
        value_factor /= 2.0;
    }

    value
}

pub fn perlin_noise_sample_with_derivative(
    snapshot: &PerlinNoiseSnapshot,
    x: f64,
    y: f64,
    z: f64,
    derivative_out: &mut [f64; 3],
) -> f64 {
    let mut value = 0.0;
    let mut factor = snapshot.lowest_freq_input_factor;
    let mut value_factor = snapshot.lowest_freq_value_factor;

    for (index, noise) in snapshot.levels.iter().enumerate() {
        if let Some(noise) = noise {
            let mut level_derivative = [0.0; 3];
            let noise_value = improved_noise_sample_with_derivative(
                noise,
                perlin_noise_wrap(x * factor),
                perlin_noise_wrap(y * factor),
                perlin_noise_wrap(z * factor),
                &mut level_derivative,
            );
            let contribution_scale = snapshot.amplitudes[index] * value_factor;
            value += noise_value * contribution_scale;
            derivative_out[0] += level_derivative[0] * contribution_scale * factor;
            derivative_out[1] += level_derivative[1] * contribution_scale * factor;
            derivative_out[2] += level_derivative[2] * contribution_scale * factor;
        }
        factor *= 2.0;
        value_factor /= 2.0;
    }

    value
}

pub fn perlin_noise_edge_value(snapshot: &PerlinNoiseSnapshot, noise_value: f64) -> f64 {
    let mut value = 0.0;
    let mut value_factor = snapshot.lowest_freq_value_factor;
    for (index, noise) in snapshot.levels.iter().enumerate() {
        if noise.is_some() {
            value += snapshot.amplitudes[index] * noise_value * value_factor;
        }
        value_factor /= 2.0;
    }
    value
}

pub fn perlin_noise_max_broken_value(snapshot: &PerlinNoiseSnapshot, y_scale: f64) -> f64 {
    perlin_noise_edge_value(snapshot, y_scale + 2.0)
}

const BLENDED_NOISE_LIMIT_AMPLITUDES: [f64; 16] = [1.0; 16];
const BLENDED_NOISE_MAIN_AMPLITUDES: [f64; 8] = [1.0; 8];
pub fn blended_noise_max_value(y_scale: f64) -> f64 {
    perlin_noise_edge_value_from_parameters(
        NormalNoiseParameters {
            id: "minecraft:blended_noise_limit",
            first_octave: -15,
            amplitudes: &BLENDED_NOISE_LIMIT_AMPLITUDES,
        },
        684.412 * y_scale + 2.0,
    )
}

pub(super) fn random_state_terrain_random(
    seed: i64,
    settings: NoiseGeneratorSettings,
) -> RandomSourceKind {
    let algorithm = if settings.legacy_random_source {
        RandomAlgorithm::Legacy
    } else {
        RandomAlgorithm::Xoroshiro
    };
    random_state_seed_factories(seed, algorithm).terrain
}

pub fn blended_noise_snapshot(
    mut random: RandomSourceKind,
    xz_scale: f64,
    y_scale: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
) -> Result<BlendedNoiseSnapshot, &'static str> {
    let limit_parameters = NormalNoiseParameters {
        id: "minecraft:blended_noise_limit",
        first_octave: -15,
        amplitudes: &BLENDED_NOISE_LIMIT_AMPLITUDES,
    };
    let main_parameters = NormalNoiseParameters {
        id: "minecraft:blended_noise_main",
        first_octave: -7,
        amplitudes: &BLENDED_NOISE_MAIN_AMPLITUDES,
    };
    let min_limit_noise = perlin_noise_snapshot_from_random(&mut random, limit_parameters, false)?;
    let max_limit_noise = perlin_noise_snapshot_from_random(&mut random, limit_parameters, false)?;
    let main_noise = perlin_noise_snapshot_from_random(&mut random, main_parameters, false)?;
    let xz_multiplier = 684.412 * xz_scale;
    let y_multiplier = 684.412 * y_scale;
    let max_value = perlin_noise_max_broken_value(&min_limit_noise, y_multiplier);

    Ok(BlendedNoiseSnapshot {
        min_limit_noise,
        max_limit_noise,
        main_noise,
        xz_multiplier,
        y_multiplier,
        xz_factor,
        y_factor,
        smear_scale_multiplier,
        max_value,
    })
}

static GLOBAL_BLENDED_NOISE_SNAPSHOT_CACHE: OnceLock<
    Mutex<HashMap<BlendedNoiseSnapshotCacheKey, BlendedNoiseSnapshot>>,
> = OnceLock::new();

fn blended_noise_snapshot_cache(
) -> std::sync::MutexGuard<'static, HashMap<BlendedNoiseSnapshotCacheKey, BlendedNoiseSnapshot>> {
    match GLOBAL_BLENDED_NOISE_SNAPSHOT_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BlendedNoiseSnapshotCacheKey {
    seed: i64,
    settings_id: &'static str,
    xz_scale: u64,
    y_scale: u64,
    xz_factor: u64,
    y_factor: u64,
    smear_scale_multiplier: u64,
}

pub(super) fn random_state_blended_noise_snapshot(
    seed: i64,
    settings: NoiseGeneratorSettings,
    xz_scale: f64,
    y_scale: f64,
    xz_factor: f64,
    y_factor: f64,
    smear_scale_multiplier: f64,
) -> Result<BlendedNoiseSnapshot, &'static str> {
    let key = BlendedNoiseSnapshotCacheKey {
        seed,
        settings_id: settings.id,
        xz_scale: xz_scale.to_bits(),
        y_scale: y_scale.to_bits(),
        xz_factor: xz_factor.to_bits(),
        y_factor: y_factor.to_bits(),
        smear_scale_multiplier: smear_scale_multiplier.to_bits(),
    };
    if let Some(snapshot) = blended_noise_snapshot_cache().get(&key).cloned() {
        return Ok(snapshot);
    }

    let snapshot = blended_noise_snapshot(
        random_state_terrain_random(seed, settings),
        xz_scale,
        y_scale,
        xz_factor,
        y_factor,
        smear_scale_multiplier,
    )?;
    blended_noise_snapshot_cache().insert(key, snapshot.clone());
    Ok(snapshot)
}

pub fn blended_noise_sample(snapshot: &BlendedNoiseSnapshot, x: f64, y: f64, z: f64) -> f64 {
    let limit_x = x * snapshot.xz_multiplier;
    let limit_y = y * snapshot.y_multiplier;
    let limit_z = z * snapshot.xz_multiplier;
    let main_x = limit_x / snapshot.xz_factor;
    let main_y = limit_y / snapshot.y_factor;
    let main_z = limit_z / snapshot.xz_factor;
    let limit_smear = snapshot.y_multiplier * snapshot.smear_scale_multiplier;
    let main_smear = limit_smear / snapshot.y_factor;
    let mut main_noise_value = 0.0;
    let mut pow = 1.0;

    for index in 0..8 {
        if let Some(noise) = snapshot
            .main_noise
            .levels
            .get(snapshot.main_noise.levels.len() - 1 - index)
            .and_then(Option::as_ref)
        {
            main_noise_value += improved_noise_sample(
                noise,
                perlin_noise_wrap(main_x * pow),
                perlin_noise_wrap(main_y * pow),
                perlin_noise_wrap(main_z * pow),
                main_smear * pow,
                main_y * pow,
            ) / pow;
        }
        pow /= 2.0;
    }

    let factor = (main_noise_value / 10.0 + 1.0) / 2.0;
    let is_max = factor >= 1.0;
    let is_min = factor <= 0.0;
    let mut blend_min = 0.0;
    let mut blend_max = 0.0;
    pow = 1.0;

    for index in 0..16 {
        let wx = perlin_noise_wrap(limit_x * pow);
        let wy = perlin_noise_wrap(limit_y * pow);
        let wz = perlin_noise_wrap(limit_z * pow);
        let y_scale_pow = limit_smear * pow;
        let level_index = snapshot.min_limit_noise.levels.len() - 1 - index;
        if !is_max {
            if let Some(noise) = &snapshot.min_limit_noise.levels[level_index] {
                blend_min +=
                    improved_noise_sample(noise, wx, wy, wz, y_scale_pow, limit_y * pow) / pow;
            }
        }
        if !is_min {
            if let Some(noise) = &snapshot.max_limit_noise.levels[level_index] {
                blend_max +=
                    improved_noise_sample(noise, wx, wy, wz, y_scale_pow, limit_y * pow) / pow;
            }
        }
        pow /= 2.0;
    }

    lerp(factor.clamp(0.0, 1.0), blend_min / 512.0, blend_max / 512.0) / 128.0
}

pub fn simplex_noise_snapshot(random: &mut RandomSourceKind) -> SimplexNoiseSnapshot {
    let xo = random_next_f64(random) * 256.0;
    let yo = random_next_f64(random) * 256.0;
    let zo = random_next_f64(random) * 256.0;
    let mut permutation = [0u8; 256];
    for (index, value) in permutation.iter_mut().enumerate() {
        *value = index as u8;
    }
    for index in 0..256 {
        let offset = random_next_i32_bound(random, 256 - index as i32) as usize;
        permutation.swap(index, index + offset);
    }
    SimplexNoiseSnapshot {
        xo,
        yo,
        zo,
        permutation,
    }
}

fn simplex_noise_permutation(snapshot: &SimplexNoiseSnapshot, x: i32) -> i32 {
    i32::from(snapshot.permutation[(x & 0xff) as usize])
}

fn simplex_corner_noise(index: i32, x: f64, y: f64, z: f64, base: f64) -> f64 {
    let mut t = base - x * x - y * y - z * z;
    if t < 0.0 {
        0.0
    } else {
        t *= t;
        t * t * gradient_dot(index as u8, x, y, z)
    }
}

pub fn simplex_noise_sample_2d(snapshot: &SimplexNoiseSnapshot, xin: f64, yin: f64) -> f64 {
    let sqrt_3 = 3.0_f64.sqrt();
    let f2 = 0.5 * (sqrt_3 - 1.0);
    let g2 = (3.0 - sqrt_3) / 6.0;
    let s = (xin + yin) * f2;
    let i = (xin + s).floor() as i32;
    let j = (yin + s).floor() as i32;
    let t = f64::from(i + j) * g2;
    let x0 = xin - (f64::from(i) - t);
    let y0 = yin - (f64::from(j) - t);
    let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };
    let x1 = x0 - f64::from(i1) + g2;
    let y1 = y0 - f64::from(j1) + g2;
    let x2 = x0 - 1.0 + 2.0 * g2;
    let y2 = y0 - 1.0 + 2.0 * g2;
    let ii = i & 0xff;
    let jj = j & 0xff;
    let gi0 =
        simplex_noise_permutation(snapshot, ii + simplex_noise_permutation(snapshot, jj)) % 12;
    let gi1 = simplex_noise_permutation(
        snapshot,
        ii + i1 + simplex_noise_permutation(snapshot, jj + j1),
    ) % 12;
    let gi2 = simplex_noise_permutation(
        snapshot,
        ii + 1 + simplex_noise_permutation(snapshot, jj + 1),
    ) % 12;
    70.0 * (simplex_corner_noise(gi0, x0, y0, 0.0, 0.5)
        + simplex_corner_noise(gi1, x1, y1, 0.0, 0.5)
        + simplex_corner_noise(gi2, x2, y2, 0.0, 0.5))
}

pub fn simplex_noise_sample_3d(
    snapshot: &SimplexNoiseSnapshot,
    xin: f64,
    yin: f64,
    zin: f64,
) -> f64 {
    let s = (xin + yin + zin) / 3.0;
    let i = (xin + s).floor() as i32;
    let j = (yin + s).floor() as i32;
    let k = (zin + s).floor() as i32;
    let t = f64::from(i + j + k) / 6.0;
    let x0 = xin - (f64::from(i) - t);
    let y0 = yin - (f64::from(j) - t);
    let z0 = zin - (f64::from(k) - t);
    let (i1, j1, k1, i2, j2, k2) = if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else if y0 < z0 {
        (0, 0, 1, 0, 1, 1)
    } else if x0 < z0 {
        (0, 1, 0, 0, 1, 1)
    } else {
        (0, 1, 0, 1, 1, 0)
    };
    let x1 = x0 - f64::from(i1) + 1.0 / 6.0;
    let y1 = y0 - f64::from(j1) + 1.0 / 6.0;
    let z1 = z0 - f64::from(k1) + 1.0 / 6.0;
    let x2 = x0 - f64::from(i2) + 1.0 / 3.0;
    let y2 = y0 - f64::from(j2) + 1.0 / 3.0;
    let z2 = z0 - f64::from(k2) + 1.0 / 3.0;
    let x3 = x0 - 0.5;
    let y3 = y0 - 0.5;
    let z3 = z0 - 0.5;
    let ii = i & 0xff;
    let jj = j & 0xff;
    let kk = k & 0xff;
    let gi0 = simplex_noise_permutation(
        snapshot,
        ii + simplex_noise_permutation(snapshot, jj + simplex_noise_permutation(snapshot, kk)),
    ) % 12;
    let gi1 = simplex_noise_permutation(
        snapshot,
        ii + i1
            + simplex_noise_permutation(
                snapshot,
                jj + j1 + simplex_noise_permutation(snapshot, kk + k1),
            ),
    ) % 12;
    let gi2 = simplex_noise_permutation(
        snapshot,
        ii + i2
            + simplex_noise_permutation(
                snapshot,
                jj + j2 + simplex_noise_permutation(snapshot, kk + k2),
            ),
    ) % 12;
    let gi3 = simplex_noise_permutation(
        snapshot,
        ii + 1
            + simplex_noise_permutation(
                snapshot,
                jj + 1 + simplex_noise_permutation(snapshot, kk + 1),
            ),
    ) % 12;
    32.0 * (simplex_corner_noise(gi0, x0, y0, z0, 0.6)
        + simplex_corner_noise(gi1, x1, y1, z1, 0.6)
        + simplex_corner_noise(gi2, x2, y2, z2, 0.6)
        + simplex_corner_noise(gi3, x3, y3, z3, 0.6))
}

pub fn end_island_height_value(
    island_noise: &SimplexNoiseSnapshot,
    section_x: i32,
    section_z: i32,
) -> f32 {
    let chunk_x = section_x / 2;
    let chunk_z = section_z / 2;
    let sub_section_x = section_x % 2;
    let sub_section_z = section_z % 2;
    let mut doffs = 100.0 - ((section_x * section_x + section_z * section_z) as f32).sqrt() * 8.0;
    doffs = doffs.clamp(-100.0, 80.0);

    for xo in -12..=12 {
        for zo in -12..=12 {
            let total_chunk_x = i64::from(chunk_x + xo);
            let total_chunk_z = i64::from(chunk_z + zo);
            if total_chunk_x * total_chunk_x + total_chunk_z * total_chunk_z > 4096
                && simplex_noise_sample_2d(island_noise, total_chunk_x as f64, total_chunk_z as f64)
                    < -0.9
            {
                let island_size = ((total_chunk_x.unsigned_abs() as f32) * 3439.0
                    + (total_chunk_z.unsigned_abs() as f32) * 147.0)
                    % 13.0
                    + 9.0;
                let xd = (sub_section_x - xo * 2) as f32;
                let zd = (sub_section_z - zo * 2) as f32;
                let new_doffs =
                    (100.0 - (xd * xd + zd * zd).sqrt() * island_size).clamp(-100.0, 80.0);
                doffs = doffs.max(new_doffs);
            }
        }
    }

    doffs
}

pub fn end_island_density_sample(seed: i64, block_x: i32, block_z: i32) -> f64 {
    let mut random = RandomSourceKind::Legacy(LegacyRandom::new(seed));
    random.consume_count(17_292);
    let island_noise = simplex_noise_snapshot(&mut random);
    (f64::from(end_island_height_value(
        &island_noise,
        block_x / 8,
        block_z / 8,
    )) - 8.0)
        / 128.0
}

/// Constructs a `PerlinSimplexNoise` from a mutable random source and a sorted list of octave
/// numbers, matching Java's `PerlinSimplexNoise(RandomSource, IntSortedSet)` constructor exactly.
///
/// Octave numbers are signed integers (e.g. `[-2, -1, 0]`).  The lowest (most negative) octave
/// is the lowest frequency; the highest (most positive) is the highest frequency.  Positive
/// octaves (> 0) are initialised from a secondary random derived from the zero-octave state,
/// exactly as Java does with `LegacyRandomSource(positiveOctaveSeed)`.
pub fn perlin_simplex_noise_snapshot(
    random: &mut RandomSourceKind,
    octave_list: &[i32],
) -> PerlinSimplexNoiseSnapshot {
    let Some((&first_entry, remaining_entries)) = octave_list.split_first() else {
        panic!("Need some octaves!");
    };
    let mut first_octave = first_entry;
    let mut last_octave = first_entry;
    for &octave in remaining_entries {
        first_octave = first_octave.min(octave);
        last_octave = last_octave.max(octave);
    }
    let low_freq_octaves = -first_octave;
    let high_freq_octaves = last_octave;
    let octave_count = (low_freq_octaves + high_freq_octaves + 1) as usize;
    assert!(
        octave_count >= 1,
        "Total number of octaves needs to be >= 1"
    );

    // Java always constructs the zero-octave simplex first from the main random, regardless of
    // whether octave 0 is in the requested set, to keep the random state advancing correctly.
    let zero_octave_snapshot = simplex_noise_snapshot(random);
    let zero_octave_index = high_freq_octaves as usize;

    let mut levels: Vec<Option<SimplexNoiseSnapshot>> = vec![None; octave_count];

    if zero_octave_index < octave_count && octave_list.contains(&0) {
        levels[zero_octave_index] = Some(zero_octave_snapshot.clone());
    }

    // Negative octaves (lower frequency) come from the main random, placed at indices above
    // zero_octave_index.  Skipped octaves still consume 262 random values to stay in sync.
    for (i, level) in levels
        .iter_mut()
        .enumerate()
        .take(octave_count)
        .skip(zero_octave_index + 1)
    {
        let octave_num = zero_octave_index as i32 - i as i32;
        if octave_list.contains(&octave_num) {
            *level = Some(simplex_noise_snapshot(random));
        } else {
            random.consume_count(262);
        }
    }

    // Positive octaves (higher frequency, indices 0..zero_octave_index) come from a secondary
    // LegacyRandom seeded by evaluating the zero-octave simplex at its own origin offsets.
    // Java uses `(long)(value * 9.223372E18F)` — note the float literal forces f32 precision
    // before widening to f64, which we replicate exactly.
    if high_freq_octaves > 0 {
        let derived_seed = (simplex_noise_sample_3d(
            &zero_octave_snapshot,
            zero_octave_snapshot.xo,
            zero_octave_snapshot.yo,
            zero_octave_snapshot.zo,
        ) * 9.223_372E18_f32 as f64) as i64;
        let mut high_freq_random = RandomSourceKind::Legacy(LegacyRandom::new(derived_seed));

        for i in (0..zero_octave_index).rev() {
            let octave_num = zero_octave_index as i32 - i as i32;
            if i < octave_count && octave_list.contains(&octave_num) {
                levels[i] = Some(simplex_noise_snapshot(&mut high_freq_random));
            } else {
                high_freq_random.consume_count(262);
            }
        }
    }

    PerlinSimplexNoiseSnapshot {
        levels,
        highest_freq_input_factor: 2.0_f64.powi(high_freq_octaves),
        highest_freq_value_factor: 1.0 / (2.0_f64.powi(octave_count as i32) - 1.0),
    }
}

/// Evaluates a `PerlinSimplexNoise` at 2-D coordinates, matching Java's
/// `PerlinSimplexNoise.getValue(double x, double y, boolean useNoiseStart)`.
///
/// When `use_noise_start` is `true`, each octave's own `xo`/`yo` offsets are added to the
/// scaled coordinates before sampling — matching the Java flag behaviour.
pub fn perlin_simplex_noise_sample(
    snapshot: &PerlinSimplexNoiseSnapshot,
    x: f64,
    y: f64,
    use_noise_start: bool,
) -> f64 {
    let mut value = 0.0;
    let mut factor = snapshot.highest_freq_input_factor;
    let mut value_factor = snapshot.highest_freq_value_factor;

    for level in &snapshot.levels {
        if let Some(level) = level {
            let lx = x * factor + if use_noise_start { level.xo } else { 0.0 };
            let ly = y * factor + if use_noise_start { level.yo } else { 0.0 };
            value += simplex_noise_sample_2d(level, lx, ly) * value_factor;
        }
        factor /= 2.0;
        value_factor *= 2.0;
    }

    value
}

/// Returns the `Biome.TEMPERATURE_NOISE` singleton — a single-octave `PerlinSimplexNoise`
/// with `LegacyRandomSource(1234)`, used by `Biome.getTemperature()` for freeze/precipitation
/// checks.  Matches Java: `new PerlinSimplexNoise(new WorldgenRandom(new LegacyRandomSource(1234L)), ImmutableList.of(0))`.
pub fn biome_temperature_noise_snapshot() -> PerlinSimplexNoiseSnapshot {
    let mut random = RandomSourceKind::Legacy(LegacyRandom::new(1234));
    perlin_simplex_noise_snapshot(&mut random, &[0])
}

/// Returns the `Biome.FROZEN_TEMPERATURE_NOISE` singleton — a three-octave `PerlinSimplexNoise`
/// with `LegacyRandomSource(3456)`, used for frozen-ocean iceberg temperature blending.
/// Matches Java: `new PerlinSimplexNoise(new WorldgenRandom(new LegacyRandomSource(3456L)), ImmutableList.of(-2, -1, 0))`.
pub fn biome_frozen_temperature_noise_snapshot() -> PerlinSimplexNoiseSnapshot {
    let mut random = RandomSourceKind::Legacy(LegacyRandom::new(3456));
    perlin_simplex_noise_snapshot(&mut random, &[-2, -1, 0])
}

/// Returns the `Biome.BIOME_INFO_NOISE` singleton — a single-octave `PerlinSimplexNoise`
/// with `LegacyRandomSource(2345)`, used for ground-cover and iceberg surface variation.
/// Matches Java: `new PerlinSimplexNoise(new WorldgenRandom(new LegacyRandomSource(2345L)), ImmutableList.of(0))`.
pub fn biome_info_noise_snapshot() -> PerlinSimplexNoiseSnapshot {
    let mut random = RandomSourceKind::Legacy(LegacyRandom::new(2345));
    perlin_simplex_noise_snapshot(&mut random, &[0])
}

pub fn normal_noise_snapshot(
    mut random: RandomSourceKind,
    parameters: NormalNoiseParameters,
    use_new_initialization: bool,
) -> Result<NormalNoiseSnapshot, &'static str> {
    let first = perlin_noise_snapshot_from_random(&mut random, parameters, use_new_initialization)?;
    let second =
        perlin_noise_snapshot_from_random(&mut random, parameters, use_new_initialization)?;
    let value_factor = normal_noise_value_factor(parameters);
    let max_value = (perlin_noise_edge_value(&first, 2.0) + perlin_noise_edge_value(&second, 2.0))
        * value_factor;

    Ok(NormalNoiseSnapshot {
        parameters,
        first,
        second,
        value_factor,
        max_value,
    })
}

pub fn normal_noise_sample(snapshot: &NormalNoiseSnapshot, x: f64, y: f64, z: f64) -> f64 {
    let first = perlin_noise_sample(&snapshot.first, x, y, z, 0.0, 0.0);
    let second = perlin_noise_sample(
        &snapshot.second,
        x * NORMAL_NOISE_INPUT_FACTOR,
        y * NORMAL_NOISE_INPUT_FACTOR,
        z * NORMAL_NOISE_INPUT_FACTOR,
        0.0,
        0.0,
    );
    (first + second) * snapshot.value_factor
}

// Thread-local cache for NormalNoise snapshots during chunk generation.
// Mirrors Java's `RandomState.noiseInstances` cache: the noise tables depend only
// on the seed and noise ID, not the sample position, so they can be reused across
// every block in a chunk.  Without this cache, `fill_slice` would re-initialize
// the full Perlin permutation tables on every cell-corner evaluation, making chunk
// generation ~1000× slower than needed.
//
// Set up via `with_noise_snapshot_cache` before calling `fill_from_noise_chunk`.
thread_local! {
    static NOISE_SNAPSHOT_CACHE: std::cell::RefCell<Option<HashMap<String, NormalNoiseSnapshot>>>
        = const { std::cell::RefCell::new(None) };
    static STATIC_NOISE_SNAPSHOT_CACHE: std::cell::RefCell<Option<HashMap<StaticNoiseSnapshotCacheKey, NormalNoiseSnapshot>>>
        = const { std::cell::RefCell::new(None) };
}
static GLOBAL_NOISE_SNAPSHOT_CACHE: OnceLock<Mutex<HashMap<String, NormalNoiseSnapshot>>> =
    OnceLock::new();

fn normal_noise_snapshot_cache(
) -> std::sync::MutexGuard<'static, HashMap<String, NormalNoiseSnapshot>> {
    match GLOBAL_NOISE_SNAPSHOT_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StaticNoiseSnapshotCacheKey {
    seed: i64,
    settings_id: &'static str,
    noise_id: &'static str,
}

fn normal_noise_snapshot_cache_key(
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_id: &str,
) -> String {
    format!("{seed}:{}:{noise_id}", settings.id)
}

/// Activate the thread-local noise cache, run `f`, then tear it down.
/// Any `random_state_normal_noise_snapshot` call inside `f` will use the cache.
pub(super) fn with_noise_snapshot_cache<T>(f: impl FnOnce() -> T) -> T {
    NOISE_SNAPSHOT_CACHE.with(|cell| {
        *cell.borrow_mut() = Some(HashMap::new());
    });
    STATIC_NOISE_SNAPSHOT_CACHE.with(|cell| {
        *cell.borrow_mut() = Some(HashMap::new());
    });
    let result = f();
    STATIC_NOISE_SNAPSHOT_CACHE.with(|cell| {
        *cell.borrow_mut() = None;
    });
    NOISE_SNAPSHOT_CACHE.with(|cell| {
        *cell.borrow_mut() = None;
    });
    result
}

pub fn random_state_normal_noise_snapshot(
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_id: &str,
) -> Option<NormalNoiseSnapshot> {
    let cache_key = normal_noise_snapshot_cache_key(seed, settings, noise_id);
    // Fast path: return a clone from the thread-local cache if active.
    let cached = NOISE_SNAPSHOT_CACHE.with(|cell| {
        cell.borrow()
            .as_ref()
            .and_then(|m| m.get(&cache_key).cloned())
    });
    if let Some(snapshot) = cached {
        return Some(snapshot);
    }

    if let Some(snapshot) = normal_noise_snapshot_cache().get(&cache_key).cloned() {
        NOISE_SNAPSHOT_CACHE.with(|cell| {
            if let Some(ref mut m) = *cell.borrow_mut() {
                m.insert(cache_key.clone(), snapshot.clone());
            }
        });
        return Some(snapshot);
    }

    let plan = random_state_normal_noise_instantiation_plan(seed, settings, noise_id)?;
    let parameters = builtin_normal_noise_parameters(plan.id)?;
    let snapshot =
        normal_noise_snapshot(plan.random, *parameters, plan.use_new_initialization).ok()?;
    normal_noise_snapshot_cache().insert(cache_key.clone(), snapshot.clone());

    // Store in cache if active.
    NOISE_SNAPSHOT_CACHE.with(|cell| {
        if let Some(ref mut m) = *cell.borrow_mut() {
            m.insert(cache_key, snapshot.clone());
        }
    });

    Some(snapshot)
}

fn with_random_state_normal_noise_snapshot<T>(
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_id: &'static str,
    f: impl FnOnce(&NormalNoiseSnapshot) -> T,
) -> Option<T> {
    STATIC_NOISE_SNAPSHOT_CACHE.with(|cell| {
        if cell.borrow().is_none() {
            let snapshot = random_state_normal_noise_snapshot(seed, settings, noise_id)?;
            return Some(f(&snapshot));
        }

        let cache_key = StaticNoiseSnapshotCacheKey {
            seed,
            settings_id: settings.id,
            noise_id,
        };
        if !cell
            .borrow()
            .as_ref()
            .is_some_and(|cache| cache.contains_key(&cache_key))
        {
            let snapshot = random_state_normal_noise_snapshot(seed, settings, noise_id)?;
            if let Some(ref mut cache) = *cell.borrow_mut() {
                cache.insert(cache_key, snapshot);
            }
        }

        let cache = cell.borrow();
        cache.as_ref()?.get(&cache_key).map(f)
    })
}

pub(super) fn random_state_normal_noise_sample(
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_id: &'static str,
    x: f64,
    y: f64,
    z: f64,
) -> f64 {
    with_random_state_normal_noise_snapshot(seed, settings, noise_id, |snapshot| {
        normal_noise_sample(snapshot, x, y, z)
    })
    .unwrap_or(0.0)
}

pub fn normal_noise_sample_with_derivative(
    snapshot: &NormalNoiseSnapshot,
    x: f64,
    y: f64,
    z: f64,
    derivative_out: &mut [f64; 3],
) -> f64 {
    let mut first_derivative = [0.0; 3];
    let first =
        perlin_noise_sample_with_derivative(&snapshot.first, x, y, z, &mut first_derivative);
    let mut second_derivative = [0.0; 3];
    let second = perlin_noise_sample_with_derivative(
        &snapshot.second,
        x * NORMAL_NOISE_INPUT_FACTOR,
        y * NORMAL_NOISE_INPUT_FACTOR,
        z * NORMAL_NOISE_INPUT_FACTOR,
        &mut second_derivative,
    );
    derivative_out[0] += (first_derivative[0] + second_derivative[0] * NORMAL_NOISE_INPUT_FACTOR)
        * snapshot.value_factor;
    derivative_out[1] += (first_derivative[1] + second_derivative[1] * NORMAL_NOISE_INPUT_FACTOR)
        * snapshot.value_factor;
    derivative_out[2] += (first_derivative[2] + second_derivative[2] * NORMAL_NOISE_INPUT_FACTOR)
        * snapshot.value_factor;
    (first + second) * snapshot.value_factor
}

pub fn random_state_normal_noise_instantiation_plan(
    seed: i64,
    settings: NoiseGeneratorSettings,
    noise_id: &str,
) -> Option<NormalNoiseInstantiationPlan> {
    let parameters = *builtin_normal_noise_parameters(noise_id)?;
    let use_legacy_nether_biome = matches!(
        parameters.id,
        "minecraft:nether/temperature" | "minecraft:nether/vegetation"
    );
    let random = if use_legacy_nether_biome {
        let offset = if parameters.id == "minecraft:nether/vegetation" {
            1
        } else {
            0
        };
        RandomSourceKind::Legacy(LegacyRandom::new(seed.wrapping_add(offset)))
    } else {
        let algorithm = if settings.legacy_random_source {
            RandomAlgorithm::Legacy
        } else {
            RandomAlgorithm::Xoroshiro
        };
        random_state_seed_factories(seed, algorithm)
            .base
            .at_hashed_name(parameters.id)
    };

    Some(NormalNoiseInstantiationPlan {
        id: parameters.id,
        first_octave: parameters.first_octave,
        non_zero_octaves: normal_noise_non_zero_octaves(parameters),
        use_new_initialization: !use_legacy_nether_biome,
        random,
    })
}

/// Analogous to the `noiseInstances` ConcurrentHashMap inside Java's `RandomState`.
///
/// `RandomState` is created once per world load and caches every `NormalNoise`
/// instance so that the same noise key always resolves to the same seeded object.
/// This struct mirrors that caching contract: the first call to
/// `get_or_create_noise` for a given id computes the snapshot; subsequent calls
/// return the cached copy.
pub struct RandomStateNoiseCache {
    seed: i64,
    settings: NoiseGeneratorSettings,
    cache: HashMap<String, NormalNoiseSnapshot>,
}

impl RandomStateNoiseCache {
    pub fn new(seed: i64, settings: NoiseGeneratorSettings) -> Self {
        Self {
            seed,
            settings,
            cache: HashMap::new(),
        }
    }

    /// Returns the NormalNoise snapshot for `noise_id`, computing and caching
    /// it on the first call — mirrors `RandomState.getOrCreateNoise()`.
    pub fn get_or_create_noise(&mut self, noise_id: &str) -> Option<&NormalNoiseSnapshot> {
        if !self.cache.contains_key(noise_id) {
            let snapshot = random_state_normal_noise_snapshot(self.seed, self.settings, noise_id)?;
            self.cache.insert(noise_id.to_string(), snapshot);
        }
        self.cache.get(noise_id)
    }
}
