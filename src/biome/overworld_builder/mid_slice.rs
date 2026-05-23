use super::super::*;
use super::OverworldBiomeBuilder;

impl OverworldBiomeBuilder {
    pub(super) fn add_mid_slice(&self, entries: &mut Vec<ClimateBiomeEntry>, weird: ClimateParameter) {
        let near_far = Self::p_span(self.near_inland_cont, self.far_inland_cont);
        let coast_near = Self::p_span(self.coast_cont, self.near_inland_cont);
        let near_mid = Self::p_span(self.near_inland_cont, self.mid_inland_cont);
        let mid_far = Self::p_span(self.mid_inland_cont, self.far_inland_cont);
        let coast_far = Self::p_span(self.coast_cont, self.far_inland_cont);
        let eros02 = Self::p_span(self.erosions[0], self.erosions[2]);
        let temp12 = Self::p_span(self.temperatures[1], self.temperatures[2]);
        let temp34 = Self::p_span(self.temperatures[3], self.temperatures[4]);

        self.add_surface_biome(
            entries,
            self.full_range,
            self.full_range,
            self.coast_cont,
            eros02,
            weird,
            0.0,
            "minecraft:stony_shore",
        );
        self.add_surface_biome(
            entries,
            temp12,
            self.full_range,
            near_far,
            self.erosions[6],
            weird,
            0.0,
            "minecraft:swamp",
        );
        self.add_surface_biome(
            entries,
            temp34,
            self.full_range,
            near_far,
            self.erosions[6],
            weird,
            0.0,
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
                let shattered = self.pick_shattered_biome(ti, hi, weird);
                let plateau = self.pick_plateau_biome(ti, hi, weird);
                let beach = self.pick_beach_biome(ti);
                let middle_or_windswept =
                    self.maybe_pick_windswept_savanna_biome(ti, hi, weird, middle);
                let shattered_coast = self.pick_shattered_coast_biome(ti, hi, weird);
                let slope = self.pick_slope_biome(ti, hi, weird);

                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    near_far,
                    self.erosions[0],
                    weird,
                    0.0,
                    slope,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    near_mid,
                    self.erosions[1],
                    weird,
                    0.0,
                    middle_or_badlands_or_slope,
                );
                let far_e1 = if ti == 0 { slope } else { plateau };
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.far_inland_cont,
                    self.erosions[1],
                    weird,
                    0.0,
                    far_e1,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[2],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.mid_inland_cont,
                    self.erosions[2],
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.far_inland_cont,
                    self.erosions[2],
                    weird,
                    0.0,
                    plateau,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    coast_near,
                    self.erosions[3],
                    weird,
                    0.0,
                    middle,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[3],
                    weird,
                    0.0,
                    middle_or_badlands,
                );
                if weird.max < 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[4],
                        weird,
                        0.0,
                        beach,
                    );
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        near_far,
                        self.erosions[4],
                        weird,
                        0.0,
                        middle,
                    );
                } else {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        coast_far,
                        self.erosions[4],
                        weird,
                        0.0,
                        middle,
                    );
                }
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.coast_cont,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered_coast,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    self.near_inland_cont,
                    self.erosions[5],
                    weird,
                    0.0,
                    middle_or_windswept,
                );
                self.add_surface_biome(
                    entries,
                    temp,
                    hum,
                    mid_far,
                    self.erosions[5],
                    weird,
                    0.0,
                    shattered,
                );
                if weird.max < 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[6],
                        weird,
                        0.0,
                        beach,
                    );
                } else {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        self.coast_cont,
                        self.erosions[6],
                        weird,
                        0.0,
                        middle,
                    );
                }
                if ti == 0 {
                    self.add_surface_biome(
                        entries,
                        temp,
                        hum,
                        near_far,
                        self.erosions[6],
                        weird,
                        0.0,
                        middle,
                    );
                }
            }
        }
    }

}
