#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionResultKind {
    Success,
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionKind {
    Water,
    Poison,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickFace {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUseOutcome {
    Pass,
    Fail,
    Success {
        returned_item: &'static str,
        stat: &'static str,
        sound: &'static str,
        game_event: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Air,
    ConvertableToMud,
    Mud,
    WaterSource,
    LavaSource,
    PowderSnowSource,
    Waterloggable,
    SolidReplaceable,
    Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidKind {
    Empty,
    Water,
    Lava,
    PowderSnow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Miss,
    Entity,
    Block {
        block: BlockKind,
        face: ClickFace,
        may_interact: bool,
        may_use_item_at_offset: bool,
        water_evaporates: bool,
        user_shift_down: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotionUseOnOutcome {
    pub result: InteractionResultKind,
    pub new_block: Option<BlockKind>,
    pub returned_item: Option<&'static str>,
    pub sounds: Vec<&'static str>,
    pub particles: usize,
    pub game_event: Option<&'static str>,
}

pub fn potion_use_on(
    potion: PotionKind,
    clicked_face: ClickFace,
    clicked_block: BlockKind,
) -> PotionUseOnOutcome {
    if clicked_face != ClickFace::Down
        && clicked_block == BlockKind::ConvertableToMud
        && potion == PotionKind::Water
    {
        PotionUseOnOutcome {
            result: InteractionResultKind::Success,
            new_block: Some(BlockKind::Mud),
            returned_item: Some("minecraft:glass_bottle"),
            sounds: vec!["generic_splash", "bottle_empty"],
            particles: 5,
            game_event: Some("fluid_place"),
        }
    } else {
        PotionUseOnOutcome {
            result: InteractionResultKind::Pass,
            new_block: None,
            returned_item: None,
            sounds: Vec::new(),
            particles: 0,
            game_event: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThrowablePotionUse {
    pub projectile_entity: &'static str,
    pub sound: &'static str,
    pub sound_source: &'static str,
    pub pitch_degrees: f32,
    pub shoot_power: f32,
    pub uncertainty: f32,
    pub consumed: u32,
    pub stat: &'static str,
    pub result: InteractionResultKind,
}

pub fn throwable_potion_use(item: &'static str) -> Option<ThrowablePotionUse> {
    match item {
        "minecraft:splash_potion" => Some(ThrowablePotionUse {
            projectile_entity: "minecraft:splash_potion",
            sound: "splash_potion_throw",
            sound_source: "players",
            pitch_degrees: -20.0,
            shoot_power: 0.5,
            uncertainty: 1.0,
            consumed: 1,
            stat: "item_used",
            result: InteractionResultKind::Success,
        }),
        "minecraft:lingering_potion" => Some(ThrowablePotionUse {
            projectile_entity: "minecraft:lingering_potion",
            sound: "lingering_potion_throw",
            sound_source: "neutral",
            pitch_degrees: -20.0,
            shoot_power: 0.5,
            uncertainty: 1.0,
            consumed: 1,
            stat: "item_used",
            result: InteractionResultKind::Success,
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectileDispenseConfig {
    pub power_multiplier: f32,
    pub uncertainty_multiplier: f32,
}

pub fn throwable_potion_dispense_config() -> ProjectileDispenseConfig {
    ProjectileDispenseConfig {
        power_multiplier: 1.25,
        uncertainty_multiplier: 0.5,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleTarget {
    DragonBreathCloud { owner_is_dragon: bool, alive: bool },
    WaterSource,
    Miss,
    Blocked,
}

pub fn bottle_use(target: BottleTarget, may_interact: bool) -> ItemUseOutcome {
    match target {
        BottleTarget::DragonBreathCloud {
            owner_is_dragon: true,
            alive: true,
        } => ItemUseOutcome::Success {
            returned_item: "minecraft:dragon_breath",
            stat: "item_used",
            sound: "bottle_fill_dragonbreath",
            game_event: "fluid_pickup",
        },
        BottleTarget::WaterSource if may_interact => ItemUseOutcome::Success {
            returned_item: "minecraft:potion{water}",
            stat: "item_used",
            sound: "bottle_fill",
            game_event: "fluid_pickup",
        },
        _ => ItemUseOutcome::Pass,
    }
}

pub fn bucket_use(content: FluidKind, hit: HitKind, infinite_materials: bool) -> ItemUseOutcome {
    let HitKind::Block {
        block,
        may_interact,
        may_use_item_at_offset,
        water_evaporates,
        user_shift_down,
        ..
    } = hit
    else {
        return ItemUseOutcome::Pass;
    };

    if !may_interact || !may_use_item_at_offset {
        return ItemUseOutcome::Fail;
    }

    if content == FluidKind::Empty {
        return pickup_with_bucket(block);
    }

    if content == FluidKind::Water && water_evaporates {
        return ItemUseOutcome::Success {
            returned_item: empty_bucket_result(content, infinite_materials),
            stat: "item_used",
            sound: "fire_extinguish",
            game_event: "fluid_place",
        };
    }

    let can_place = matches!(
        (content, block, user_shift_down),
        (FluidKind::Water, BlockKind::Waterloggable, false)
            | (_, BlockKind::Air, _)
            | (_, BlockKind::SolidReplaceable, _)
    );

    if !can_place {
        return ItemUseOutcome::Fail;
    }

    ItemUseOutcome::Success {
        returned_item: empty_bucket_result(content, infinite_materials),
        stat: "item_used",
        sound: match content {
            FluidKind::Lava => "bucket_empty_lava",
            FluidKind::PowderSnow => "bucket_empty_powder_snow",
            FluidKind::Water => "bucket_empty",
            FluidKind::Empty => unreachable!(),
        },
        game_event: "fluid_place",
    }
}

fn pickup_with_bucket(block: BlockKind) -> ItemUseOutcome {
    match block {
        BlockKind::WaterSource => ItemUseOutcome::Success {
            returned_item: "minecraft:water_bucket",
            stat: "item_used",
            sound: "bucket_fill",
            game_event: "fluid_pickup",
        },
        BlockKind::LavaSource => ItemUseOutcome::Success {
            returned_item: "minecraft:lava_bucket",
            stat: "item_used",
            sound: "bucket_fill_lava",
            game_event: "fluid_pickup",
        },
        BlockKind::PowderSnowSource => ItemUseOutcome::Success {
            returned_item: "minecraft:powder_snow_bucket",
            stat: "item_used",
            sound: "bucket_fill_powder_snow",
            game_event: "fluid_pickup",
        },
        _ => ItemUseOutcome::Fail,
    }
}

fn empty_bucket_result(content: FluidKind, infinite_materials: bool) -> &'static str {
    if infinite_materials {
        match content {
            FluidKind::Water => "minecraft:water_bucket",
            FluidKind::Lava => "minecraft:lava_bucket",
            FluidKind::PowderSnow => "minecraft:powder_snow_bucket",
            FluidKind::Empty => "minecraft:bucket",
        }
    } else {
        "minecraft:bucket"
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrowDefaults {
    pub item: &'static str,
    pub entity: &'static str,
    pub pickup_allowed: bool,
    pub potion: Option<PotionKind>,
}

pub fn arrow_defaults(item: &'static str) -> Option<ArrowDefaults> {
    match item {
        "minecraft:arrow" => Some(ArrowDefaults {
            item,
            entity: "minecraft:arrow",
            pickup_allowed: true,
            potion: None,
        }),
        "minecraft:tipped_arrow" => Some(ArrowDefaults {
            item,
            entity: "minecraft:arrow",
            pickup_allowed: true,
            potion: Some(PotionKind::Poison),
        }),
        "minecraft:spectral_arrow" => Some(ArrowDefaults {
            item,
            entity: "minecraft:spectral_arrow",
            pickup_allowed: true,
            potion: None,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_hit(block: BlockKind) -> HitKind {
        HitKind::Block {
            block,
            face: ClickFace::Up,
            may_interact: true,
            may_use_item_at_offset: true,
            water_evaporates: false,
            user_shift_down: false,
        }
    }

    #[test]
    fn water_potion_converts_mud_blocks_only_from_non_down_faces() {
        assert_eq!(
            potion_use_on(
                PotionKind::Water,
                ClickFace::Up,
                BlockKind::ConvertableToMud
            ),
            PotionUseOnOutcome {
                result: InteractionResultKind::Success,
                new_block: Some(BlockKind::Mud),
                returned_item: Some("minecraft:glass_bottle"),
                sounds: vec!["generic_splash", "bottle_empty"],
                particles: 5,
                game_event: Some("fluid_place")
            }
        );
        assert_eq!(
            potion_use_on(
                PotionKind::Water,
                ClickFace::Down,
                BlockKind::ConvertableToMud
            )
            .result,
            InteractionResultKind::Pass
        );
        assert_eq!(
            potion_use_on(
                PotionKind::Other,
                ClickFace::Up,
                BlockKind::ConvertableToMud
            )
            .result,
            InteractionResultKind::Pass
        );
    }

    #[test]
    fn throwable_potions_spawn_consume_and_use_vanilla_projectile_settings() {
        let splash = throwable_potion_use("minecraft:splash_potion").unwrap();
        assert_eq!(splash.projectile_entity, "minecraft:splash_potion");
        assert_eq!(splash.sound, "splash_potion_throw");
        assert_eq!(splash.sound_source, "players");
        assert_eq!(splash.pitch_degrees, -20.0);
        assert_eq!(splash.shoot_power, 0.5);
        assert_eq!(splash.consumed, 1);

        let lingering = throwable_potion_use("minecraft:lingering_potion").unwrap();
        assert_eq!(lingering.projectile_entity, "minecraft:lingering_potion");
        assert_eq!(lingering.sound_source, "neutral");

        assert_eq!(
            throwable_potion_dispense_config(),
            ProjectileDispenseConfig {
                power_multiplier: 1.25,
                uncertainty_multiplier: 0.5
            }
        );
        assert_eq!(throwable_potion_use("minecraft:potion"), None);
    }

    #[test]
    fn bottles_fill_dragon_breath_before_water_sources() {
        assert_eq!(
            bottle_use(
                BottleTarget::DragonBreathCloud {
                    owner_is_dragon: true,
                    alive: true
                },
                true
            ),
            ItemUseOutcome::Success {
                returned_item: "minecraft:dragon_breath",
                stat: "item_used",
                sound: "bottle_fill_dragonbreath",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            bottle_use(BottleTarget::WaterSource, true),
            ItemUseOutcome::Success {
                returned_item: "minecraft:potion{water}",
                stat: "item_used",
                sound: "bottle_fill",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            bottle_use(BottleTarget::WaterSource, false),
            ItemUseOutcome::Pass
        );
        assert_eq!(
            bottle_use(
                BottleTarget::DragonBreathCloud {
                    owner_is_dragon: false,
                    alive: true
                },
                true
            ),
            ItemUseOutcome::Pass
        );
    }

    #[test]
    fn buckets_pick_up_sources_place_fluids_and_respect_denials() {
        assert_eq!(
            bucket_use(FluidKind::Empty, block_hit(BlockKind::WaterSource), false),
            ItemUseOutcome::Success {
                returned_item: "minecraft:water_bucket",
                stat: "item_used",
                sound: "bucket_fill",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            bucket_use(FluidKind::Water, block_hit(BlockKind::Waterloggable), false),
            ItemUseOutcome::Success {
                returned_item: "minecraft:bucket",
                stat: "item_used",
                sound: "bucket_empty",
                game_event: "fluid_place"
            }
        );
        assert_eq!(
            bucket_use(FluidKind::Water, block_hit(BlockKind::Waterloggable), true),
            ItemUseOutcome::Success {
                returned_item: "minecraft:water_bucket",
                stat: "item_used",
                sound: "bucket_empty",
                game_event: "fluid_place"
            }
        );
        assert_eq!(
            bucket_use(
                FluidKind::Water,
                HitKind::Block {
                    block: BlockKind::Air,
                    face: ClickFace::Up,
                    may_interact: false,
                    may_use_item_at_offset: true,
                    water_evaporates: false,
                    user_shift_down: false
                },
                false
            ),
            ItemUseOutcome::Fail
        );
        assert_eq!(
            bucket_use(FluidKind::Empty, HitKind::Miss, false),
            ItemUseOutcome::Pass
        );
    }

    #[test]
    fn water_bucket_evaporates_in_hot_dimensions_without_emptying_block() {
        assert_eq!(
            bucket_use(
                FluidKind::Water,
                HitKind::Block {
                    block: BlockKind::Air,
                    face: ClickFace::Up,
                    may_interact: true,
                    may_use_item_at_offset: true,
                    water_evaporates: true,
                    user_shift_down: false
                },
                false
            ),
            ItemUseOutcome::Success {
                returned_item: "minecraft:bucket",
                stat: "item_used",
                sound: "fire_extinguish",
                game_event: "fluid_place"
            }
        );
    }

    #[test]
    fn arrow_and_tipped_arrow_defaults_match_item_classes() {
        assert_eq!(
            arrow_defaults("minecraft:arrow"),
            Some(ArrowDefaults {
                item: "minecraft:arrow",
                entity: "minecraft:arrow",
                pickup_allowed: true,
                potion: None
            })
        );
        assert_eq!(
            arrow_defaults("minecraft:tipped_arrow").unwrap().potion,
            Some(PotionKind::Poison)
        );
        assert_eq!(
            arrow_defaults("minecraft:spectral_arrow").unwrap().entity,
            "minecraft:spectral_arrow"
        );
    }

    #[test]
    fn fluid_container_model_covers_remaining_denial_and_container_variants() {
        assert_eq!(InteractionResultKind::Fail, InteractionResultKind::Fail);
        for face in [
            ClickFace::North,
            ClickFace::South,
            ClickFace::West,
            ClickFace::East,
        ] {
            assert_eq!(
                potion_use_on(PotionKind::Water, face, BlockKind::ConvertableToMud).result,
                InteractionResultKind::Success
            );
        }

        assert_eq!(
            bucket_use(FluidKind::Empty, block_hit(BlockKind::LavaSource), false),
            ItemUseOutcome::Success {
                returned_item: "minecraft:lava_bucket",
                stat: "item_used",
                sound: "bucket_fill_lava",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            bucket_use(
                FluidKind::Empty,
                block_hit(BlockKind::PowderSnowSource),
                false
            ),
            ItemUseOutcome::Success {
                returned_item: "minecraft:powder_snow_bucket",
                stat: "item_used",
                sound: "bucket_fill_powder_snow",
                game_event: "fluid_pickup"
            }
        );
        assert_eq!(
            bucket_use(
                FluidKind::Lava,
                block_hit(BlockKind::SolidReplaceable),
                false
            ),
            ItemUseOutcome::Success {
                returned_item: "minecraft:bucket",
                stat: "item_used",
                sound: "bucket_empty_lava",
                game_event: "fluid_place"
            }
        );
        assert_eq!(
            bucket_use(
                FluidKind::PowderSnow,
                block_hit(BlockKind::SolidReplaceable),
                false
            ),
            ItemUseOutcome::Success {
                returned_item: "minecraft:bucket",
                stat: "item_used",
                sound: "bucket_empty_powder_snow",
                game_event: "fluid_place"
            }
        );
        assert_eq!(
            bucket_use(FluidKind::Water, block_hit(BlockKind::Solid), false),
            ItemUseOutcome::Fail
        );
        assert_eq!(
            bucket_use(FluidKind::Water, HitKind::Entity, false),
            ItemUseOutcome::Pass
        );
        assert_eq!(bottle_use(BottleTarget::Miss, true), ItemUseOutcome::Pass);
        assert_eq!(
            bottle_use(BottleTarget::Blocked, true),
            ItemUseOutcome::Pass
        );
    }
}
