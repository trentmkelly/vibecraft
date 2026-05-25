use super::*;

mod mid_slice;

// Lookup tables mirroring OverworldBiomeBuilder's instance fields in Java.
// First index = temperature tier (0=coldest, 4=hottest).
// Second index = humidity tier (0=driest, 4=wettest).
// Source: decompiled-server-26.1.2/net/minecraft/world/level/biome/OverworldBiomeBuilder.java

const OCEANS: [[&str; 5]; 2] = [
    // Deep oceans (deepOceanContinentalness range)
    [
        "minecraft:deep_frozen_ocean",
        "minecraft:deep_cold_ocean",
        "minecraft:deep_ocean",
        "minecraft:deep_lukewarm_ocean",
        "minecraft:warm_ocean",
    ],
    // Shallow oceans (oceanContinentalness range)
    [
        "minecraft:frozen_ocean",
        "minecraft:cold_ocean",
        "minecraft:ocean",
        "minecraft:lukewarm_ocean",
        "minecraft:warm_ocean",
    ],
];

#[derive(Debug, Clone, Copy)]
pub(super) struct ClimateEntryParameters {
    temperature: ClimateParameter,
    humidity: ClimateParameter,
    continentalness: ClimateParameter,
    erosion: ClimateParameter,
    weirdness: ClimateParameter,
    offset: f32,
}

impl ClimateEntryParameters {
    pub(super) fn new(
        temperature: ClimateParameter,
        humidity: ClimateParameter,
        continentalness: ClimateParameter,
        erosion: ClimateParameter,
        weirdness: ClimateParameter,
        offset: f32,
    ) -> Self {
        Self {
            temperature,
            humidity,
            continentalness,
            erosion,
            weirdness,
            offset,
        }
    }
}

const MIDDLE_BIOMES: [[&str; 5]; 5] = [
    [
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_taiga",
        "minecraft:taiga",
    ],
    [
        "minecraft:plains",
        "minecraft:plains",
        "minecraft:forest",
        "minecraft:taiga",
        "minecraft:old_growth_spruce_taiga",
    ],
    [
        "minecraft:flower_forest",
        "minecraft:plains",
        "minecraft:forest",
        "minecraft:birch_forest",
        "minecraft:dark_forest",
    ],
    [
        "minecraft:savanna",
        "minecraft:savanna",
        "minecraft:forest",
        "minecraft:jungle",
        "minecraft:jungle",
    ],
    [
        "minecraft:desert",
        "minecraft:desert",
        "minecraft:desert",
        "minecraft:desert",
        "minecraft:desert",
    ],
];

const MIDDLE_BIOMES_VARIANT: [[Option<&str>; 5]; 5] = [
    [
        Some("minecraft:ice_spikes"),
        None,
        Some("minecraft:snowy_taiga"),
        None,
        None,
    ],
    [
        None,
        None,
        None,
        None,
        Some("minecraft:old_growth_pine_taiga"),
    ],
    [
        Some("minecraft:sunflower_plains"),
        None,
        None,
        Some("minecraft:old_growth_birch_forest"),
        None,
    ],
    [
        None,
        None,
        Some("minecraft:plains"),
        Some("minecraft:sparse_jungle"),
        Some("minecraft:bamboo_jungle"),
    ],
    [None, None, None, None, None],
];

const PLATEAU_BIOMES: [[&str; 5]; 5] = [
    [
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_plains",
        "minecraft:snowy_taiga",
        "minecraft:snowy_taiga",
    ],
    [
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:forest",
        "minecraft:taiga",
        "minecraft:old_growth_spruce_taiga",
    ],
    [
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:meadow",
        "minecraft:pale_garden",
    ],
    [
        "minecraft:savanna_plateau",
        "minecraft:savanna_plateau",
        "minecraft:forest",
        "minecraft:forest",
        "minecraft:jungle",
    ],
    [
        "minecraft:badlands",
        "minecraft:badlands",
        "minecraft:badlands",
        "minecraft:wooded_badlands",
        "minecraft:wooded_badlands",
    ],
];

const PLATEAU_BIOMES_VARIANT: [[Option<&str>; 5]; 5] = [
    [Some("minecraft:ice_spikes"), None, None, None, None],
    [
        Some("minecraft:cherry_grove"),
        None,
        Some("minecraft:meadow"),
        Some("minecraft:meadow"),
        Some("minecraft:old_growth_pine_taiga"),
    ],
    [
        Some("minecraft:cherry_grove"),
        Some("minecraft:cherry_grove"),
        Some("minecraft:forest"),
        Some("minecraft:birch_forest"),
        None,
    ],
    [None, None, None, None, None],
    [
        Some("minecraft:eroded_badlands"),
        Some("minecraft:eroded_badlands"),
        None,
        None,
        None,
    ],
];

const SHATTERED_BIOMES: [[Option<&str>; 5]; 5] = [
    [
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_forest"),
        Some("minecraft:windswept_forest"),
    ],
    [
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_gravelly_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_forest"),
        Some("minecraft:windswept_forest"),
    ],
    [
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_hills"),
        Some("minecraft:windswept_forest"),
        Some("minecraft:windswept_forest"),
    ],
    [None, None, None, None, None],
    [None, None, None, None, None],
];
pub(super) struct OverworldBiomeBuilder {
    temperatures: [ClimateParameter; 5],
    humidities: [ClimateParameter; 5],
    erosions: [ClimateParameter; 7],
    full_range: ClimateParameter,
    frozen_range: ClimateParameter,
    unfrozen_range: ClimateParameter,
    mushroom_fields_cont: ClimateParameter,
    deep_ocean_cont: ClimateParameter,
    ocean_cont: ClimateParameter,
    coast_cont: ClimateParameter,
    inland_cont: ClimateParameter,
    near_inland_cont: ClimateParameter,
    mid_inland_cont: ClimateParameter,
    far_inland_cont: ClimateParameter,
}

impl OverworldBiomeBuilder {
    pub(super) fn new() -> Self {
        let temperatures = [
            span(-1.0, -0.45),
            span(-0.45, -0.15),
            span(-0.15, 0.2),
            span(0.2, 0.55),
            span(0.55, 1.0),
        ];
        let frozen_range = temperatures[0];
        // Java: Climate.Parameter.span(temperatures[1], temperatures[4]) — min of [1], max of [4]
        let unfrozen_range = ClimateParameter {
            min: temperatures[1].min,
            max: temperatures[4].max,
        };
        let near_inland_cont = span(-0.11, 0.03);
        let mid_inland_cont = span(0.03, 0.3);
        let far_inland_cont = span(0.3, 1.0);
        Self {
            temperatures,
            humidities: [
                span(-1.0, -0.35),
                span(-0.35, -0.1),
                span(-0.1, 0.1),
                span(0.1, 0.3),
                span(0.3, 1.0),
            ],
            erosions: [
                span(-1.0, -0.78),
                span(-0.78, -0.375),
                span(-0.375, -0.2225),
                span(-0.2225, 0.05),
                span(0.05, 0.45),
                span(0.45, 0.55),
                span(0.55, 1.0),
            ],
            full_range: span(-1.0, 1.0),
            frozen_range,
            unfrozen_range,
            mushroom_fields_cont: span(-1.2, -1.05),
            deep_ocean_cont: span(-1.05, -0.455),
            ocean_cont: span(-0.455, -0.19),
            coast_cont: span(-0.19, -0.11),
            inland_cont: span(-0.11, 0.55),
            near_inland_cont,
            mid_inland_cont,
            far_inland_cont,
        }
    }

    pub(super) fn build(self) -> Vec<ClimateBiomeEntry> {
        let mut entries = Vec::new();
        self.add_off_coast_biomes(&mut entries);
        self.add_inland_biomes(&mut entries);
        self.add_underground_biomes(&mut entries);
        entries
    }

    // Java Climate.Parameter.span(Parameter a, Parameter b): { min: a.min, max: b.max }
    fn p_span(a: ClimateParameter, b: ClimateParameter) -> ClimateParameter {
        ClimateParameter {
            min: a.min,
            max: b.max,
        }
    }

    // Java addSurfaceBiome emits two entries: depth=point(0.0) and depth=point(1.0)
    fn add_surface_biome(
        &self,
        entries: &mut Vec<ClimateBiomeEntry>,
        climate: ClimateEntryParameters,
        biome: &'static str,
    ) {
        for depth_val in [0.0_f32, 1.0_f32] {
            entries.push(ClimateBiomeEntry {
                parameters: ClimateParameterPoint {
                    temperature: climate.temperature,
                    humidity: climate.humidity,
                    continentalness: climate.continentalness,
                    erosion: climate.erosion,
                    depth: point(depth_val),
                    weirdness: climate.weirdness,
                    offset: quantize_coord(climate.offset),
                },
                biome,
            });
        }
    }

    // Java addUndergroundBiome emits one entry with depth=span(0.2, 0.9)
    fn add_underground_biome(
        &self,
        entries: &mut Vec<ClimateBiomeEntry>,
        climate: ClimateEntryParameters,
        biome: &'static str,
    ) {
        entries.push(ClimateBiomeEntry {
            parameters: ClimateParameterPoint {
                temperature: climate.temperature,
                humidity: climate.humidity,
                continentalness: climate.continentalness,
                erosion: climate.erosion,
                depth: span(0.2, 0.9),
                weirdness: climate.weirdness,
                offset: quantize_coord(climate.offset),
            },
            biome,
        });
    }

    // Java addBottomBiome emits one entry with depth=point(1.1)
    fn add_bottom_biome(
        &self,
        entries: &mut Vec<ClimateBiomeEntry>,
        climate: ClimateEntryParameters,
        biome: &'static str,
    ) {
        entries.push(ClimateBiomeEntry {
            parameters: ClimateParameterPoint {
                temperature: climate.temperature,
                humidity: climate.humidity,
                continentalness: climate.continentalness,
                erosion: climate.erosion,
                depth: point(1.1),
                weirdness: climate.weirdness,
                offset: quantize_coord(climate.offset),
            },
            biome,
        });
    }

    // -- Biome picker functions --

    fn pick_middle_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if weird.max < 0 {
            MIDDLE_BIOMES[ti][hi]
        } else {
            MIDDLE_BIOMES_VARIANT[ti][hi].unwrap_or(MIDDLE_BIOMES[ti][hi])
        }
    }

    fn pick_middle_biome_or_badlands_if_hot(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
    ) -> &'static str {
        if ti == 4 {
            self.pick_badlands_biome(hi, weird)
        } else {
            self.pick_middle_biome(ti, hi, weird)
        }
    }

    fn pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
    ) -> &'static str {
        if ti == 0 {
            self.pick_slope_biome(ti, hi, weird)
        } else {
            self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird)
        }
    }

    fn maybe_pick_windswept_savanna_biome(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
        fallback: &'static str,
    ) -> &'static str {
        if ti > 1 && hi < 4 && weird.max >= 0 {
            "minecraft:windswept_savanna"
        } else {
            fallback
        }
    }

    fn pick_shattered_coast_biome(
        &self,
        ti: usize,
        hi: usize,
        weird: ClimateParameter,
    ) -> &'static str {
        let base = if weird.max >= 0 {
            self.pick_middle_biome(ti, hi, weird)
        } else {
            self.pick_beach_biome(ti)
        };
        self.maybe_pick_windswept_savanna_biome(ti, hi, weird, base)
    }

    fn pick_beach_biome(&self, ti: usize) -> &'static str {
        match ti {
            0 => "minecraft:snowy_beach",
            4 => "minecraft:desert",
            _ => "minecraft:beach",
        }
    }

    fn pick_badlands_biome(&self, hi: usize, weird: ClimateParameter) -> &'static str {
        if hi < 2 {
            if weird.max < 0 {
                "minecraft:badlands"
            } else {
                "minecraft:eroded_badlands"
            }
        } else if hi < 3 {
            "minecraft:badlands"
        } else {
            "minecraft:wooded_badlands"
        }
    }

    fn pick_plateau_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if weird.max >= 0 {
            if let Some(variant) = PLATEAU_BIOMES_VARIANT[ti][hi] {
                return variant;
            }
        }
        PLATEAU_BIOMES[ti][hi]
    }

    fn pick_peak_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if ti <= 2 {
            if weird.max < 0 {
                "minecraft:jagged_peaks"
            } else {
                "minecraft:frozen_peaks"
            }
        } else if ti == 3 {
            "minecraft:stony_peaks"
        } else {
            self.pick_badlands_biome(hi, weird)
        }
    }

    fn pick_slope_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        if ti >= 3 {
            self.pick_plateau_biome(ti, hi, weird)
        } else if hi <= 1 {
            "minecraft:snowy_slopes"
        } else {
            "minecraft:grove"
        }
    }

    fn pick_shattered_biome(&self, ti: usize, hi: usize, weird: ClimateParameter) -> &'static str {
        SHATTERED_BIOMES[ti][hi].unwrap_or_else(|| self.pick_middle_biome(ti, hi, weird))
    }

    // -- Inland slice generators --

    fn add_peaks(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let plateau = self.pick_plateau_biome(ti, hi, weird);
                let shattered = self.pick_shattered_biome(ti, hi, weird);
                let shattered_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, shattered);
                let peak = self.pick_peak_biome(ti, hi, weird);

                let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
                let coast_near = Self::p_span(self.coast_cont, self.near_inland_cont);
                let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
                let eros23 = Self::p_span(self.erosions[2], self.erosions[3]);

                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_far, self.erosions[0], weird, 0.0),
                    peak,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        coast_near,
                        self.erosions[1],
                        weird,
                        0.0,
                    ),
                    middle_or_badlands_or_slope,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[1], weird, 0.0),
                    peak,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_near, eros23, weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[2], weird, 0.0),
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.mid_inland_cont,
                        self.erosions[3],
                        weird,
                        0.0,
                    ),
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.far_inland_cont,
                        self.erosions[3],
                        weird,
                        0.0,
                    ),
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_far, self.erosions[4], weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        coast_near,
                        self.erosions[5],
                        weird,
                        0.0,
                    ),
                    shattered_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[5], weird, 0.0),
                    shattered,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_far, self.erosions[6], weird, 0.0),
                    middle,
                );
            }
        }
    }

    fn add_high_slice(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let plateau = self.pick_plateau_biome(ti, hi, weird);
                let shattered = self.pick_shattered_biome(ti, hi, weird);
                let middle_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, middle);
                let slope = self.pick_slope_biome(ti, hi, weird);
                let peak = self.pick_peak_biome(ti, hi, weird);

                let coast_near = Self::p_span(self.coast_cont, self.near_inland_cont);
                let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
                let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
                let eros01 = Self::p_span(self.erosions[0], self.erosions[1]);
                let eros23 = Self::p_span(self.erosions[2], self.erosions[3]);

                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, self.coast_cont, eros01, weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.near_inland_cont,
                        self.erosions[0],
                        weird,
                        0.0,
                    ),
                    slope,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[0], weird, 0.0),
                    peak,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.near_inland_cont,
                        self.erosions[1],
                        weird,
                        0.0,
                    ),
                    middle_or_badlands_or_slope,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[1], weird, 0.0),
                    slope,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_near, eros23, weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[2], weird, 0.0),
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.mid_inland_cont,
                        self.erosions[3],
                        weird,
                        0.0,
                    ),
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.far_inland_cont,
                        self.erosions[3],
                        weird,
                        0.0,
                    ),
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_far, self.erosions[4], weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        coast_near,
                        self.erosions[5],
                        weird,
                        0.0,
                    ),
                    middle_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[5], weird, 0.0),
                    shattered,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, coast_far, self.erosions[6], weird, 0.0),
                    middle,
                );
            }
        }
    }

    fn add_low_slice(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        let near_far = Self::p_span(self.near_inland_cont, self.far_inland_cont);
        let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
        let eros02 = Self::p_span(self.erosions[0], self.erosions[2]);
        let eros01 = Self::p_span(self.erosions[0], self.erosions[1]);
        let eros23 = Self::p_span(self.erosions[2], self.erosions[3]);
        let eros34 = Self::p_span(self.erosions[3], self.erosions[4]);
        let temp12 = Self::p_span(self.temperatures[1], self.temperatures[2]);
        let temp34 = Self::p_span(self.temperatures[3], self.temperatures[4]);

        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.full_range,
                self.full_range,
                self.coast_cont,
                eros02,
                weird,
                0.0,
            ),
            "minecraft:stony_shore",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                temp12,
                self.full_range,
                near_far,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:swamp",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                temp34,
                self.full_range,
                near_far,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:mangrove_swamp",
        );

        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle = self.pick_middle_biome(ti, hi, weird);
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                let middle_or_badlands_or_slope =
                    self.pick_middle_biome_or_badlands_if_hot_or_slope_if_cold(ti, hi, weird);
                let beach = self.pick_beach_biome(ti);
                let middle_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, middle);
                let shattered_coast = self.pick_shattered_coast_biome(ti, hi, weird);

                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.near_inland_cont,
                        eros01,
                        weird,
                        0.0,
                    ),
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, eros01, weird, 0.0),
                    middle_or_badlands_or_slope,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.near_inland_cont,
                        eros23,
                        weird,
                        0.0,
                    ),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, eros23, weird, 0.0),
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, self.coast_cont, eros34, weird, 0.0),
                    beach,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, near_far, self.erosions[4], weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[5],
                        weird,
                        0.0,
                    ),
                    shattered_coast,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.near_inland_cont,
                        self.erosions[5],
                        weird,
                        0.0,
                    ),
                    middle_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, self.erosions[5], weird, 0.0),
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[6],
                        weird,
                        0.0,
                    ),
                    beach,
                );
                if ti == 0 {
                    self.add_surface_biome(
                        entries,
                        ClimateEntryParameters::new(
                            temp,
                            hum,
                            near_far,
                            self.erosions[6],
                            weird,
                            0.0,
                        ),
                        middle,
                    );
                }
            }
        }
    }

    fn add_valleys(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
        let eros01 = Self::p_span(self.erosions[0], self.erosions[1]);
        let eros25 = Self::p_span(self.erosions[2], self.erosions[5]);
        let inland_far = Self::p_span(self.inland_cont, self.far_inland_cont);
        let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
        let temp12 = Self::p_span(self.temperatures[1], self.temperatures[2]);
        let temp34 = Self::p_span(self.temperatures[3], self.temperatures[4]);

        let frozen_stony = if weird.max < 0 {
            "minecraft:stony_shore"
        } else {
            "minecraft:frozen_river"
        };
        let unfrozen_stony = if weird.max < 0 {
            "minecraft:stony_shore"
        } else {
            "minecraft:river"
        };

        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.frozen_range,
                self.full_range,
                self.coast_cont,
                eros01,
                weird,
                0.0,
            ),
            frozen_stony,
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.unfrozen_range,
                self.full_range,
                self.coast_cont,
                eros01,
                weird,
                0.0,
            ),
            unfrozen_stony,
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.frozen_range,
                self.full_range,
                self.near_inland_cont,
                eros01,
                weird,
                0.0,
            ),
            "minecraft:frozen_river",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.unfrozen_range,
                self.full_range,
                self.near_inland_cont,
                eros01,
                weird,
                0.0,
            ),
            "minecraft:river",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.frozen_range,
                self.full_range,
                coast_far,
                eros25,
                weird,
                0.0,
            ),
            "minecraft:frozen_river",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.unfrozen_range,
                self.full_range,
                coast_far,
                eros25,
                weird,
                0.0,
            ),
            "minecraft:river",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.frozen_range,
                self.full_range,
                self.coast_cont,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:frozen_river",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.unfrozen_range,
                self.full_range,
                self.coast_cont,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:river",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                temp12,
                self.full_range,
                inland_far,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:swamp",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                temp34,
                self.full_range,
                inland_far,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:mangrove_swamp",
        );
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.frozen_range,
                self.full_range,
                inland_far,
                self.erosions[6],
                weird,
                0.0,
            ),
            "minecraft:frozen_river",
        );

        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            for hi in 0..5_usize {
                let hum = self.humidities[hi];
                let middle_or_badlands = self.pick_middle_biome_or_badlands_if_hot(ti, hi, weird);
                self.add_surface_biome(
                    entries,
                    ClimateEntryParameters::new(temp, hum, mid_far, eros01, weird, 0.0),
                    middle_or_badlands,
                );
            }
        }
    }

    fn add_inland_biomes(&self, entries: &mut Vec<ClimateBiomeEntry>) {
        self.add_mid_slice(entries, span(-1.0, -0.93333334));
        self.add_high_slice(entries, span(-0.93333334, -0.7666667));
        self.add_peaks(entries, span(-0.7666667, -0.56666666));
        self.add_high_slice(entries, span(-0.56666666, -0.4));
        self.add_mid_slice(entries, span(-0.4, -0.26666668));
        self.add_low_slice(entries, span(-0.26666668, -0.05));
        self.add_valleys(entries, span(-0.05, 0.05));
        self.add_low_slice(entries, span(0.05, 0.26666668));
        self.add_mid_slice(entries, span(0.26666668, 0.4));
        self.add_high_slice(entries, span(0.4, 0.56666666));
        self.add_peaks(entries, span(0.56666666, 0.7666667));
        self.add_high_slice(entries, span(0.7666667, 0.93333334));
        self.add_mid_slice(entries, span(0.93333334, 1.0));
    }

    fn add_off_coast_biomes(&self, entries: &mut Vec<ClimateBiomeEntry>) {
        self.add_surface_biome(
            entries,
            ClimateEntryParameters::new(
                self.full_range,
                self.full_range,
                self.mushroom_fields_cont,
                self.full_range,
                self.full_range,
                0.0,
            ),
            "minecraft:mushroom_fields",
        );
        for ti in 0..5_usize {
            let temp = self.temperatures[ti];
            self.add_surface_biome(
                entries,
                ClimateEntryParameters::new(
                    temp,
                    self.full_range,
                    self.deep_ocean_cont,
                    self.full_range,
                    self.full_range,
                    0.0,
                ),
                OCEANS[0][ti],
            );
            self.add_surface_biome(
                entries,
                ClimateEntryParameters::new(
                    temp,
                    self.full_range,
                    self.ocean_cont,
                    self.full_range,
                    self.full_range,
                    0.0,
                ),
                OCEANS[1][ti],
            );
        }
    }

    fn add_underground_biomes(&self, entries: &mut Vec<ClimateBiomeEntry>) {
        self.add_underground_biome(
            entries,
            ClimateEntryParameters::new(
                self.full_range,
                self.full_range,
                span(0.8, 1.0),
                self.full_range,
                self.full_range,
                0.0,
            ),
            "minecraft:dripstone_caves",
        );
        self.add_underground_biome(
            entries,
            ClimateEntryParameters::new(
                self.full_range,
                span(0.7, 1.0),
                self.full_range,
                self.full_range,
                self.full_range,
                0.0,
            ),
            "minecraft:lush_caves",
        );
        self.add_bottom_biome(
            entries,
            ClimateEntryParameters::new(
                self.full_range,
                self.full_range,
                self.full_range,
                Self::p_span(self.erosions[0], self.erosions[1]),
                self.full_range,
                0.0,
            ),
            "minecraft:deep_dark",
        );
    }
}
