use crate::presentation_data::{ParticleOptionShape, PARTICLES};

pub const CORE_PARTICLES_PACKAGE_NULL_MARKED: bool = true;

const SIMPLE_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[],
    stream_fields: &[],
};

const BLOCK_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[FieldSpec::required("block_state", FieldKind::BlockState)],
    stream_fields: &[FieldSpec::required(
        "block_state_id",
        FieldKind::BlockStateId,
    )],
};

const ITEM_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[FieldSpec::required("item", FieldKind::ItemStackTemplate)],
    stream_fields: &[FieldSpec::required("item", FieldKind::ItemStackTemplate)],
};

const COLOR_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[FieldSpec::required("color", FieldKind::ArgbColor)],
    stream_fields: &[FieldSpec::required("color", FieldKind::Int)],
};

const DUST_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[
        FieldSpec::required("color", FieldKind::RgbColor),
        FieldSpec::required("scale", FieldKind::Scale),
    ],
    stream_fields: &[
        FieldSpec::required("color", FieldKind::Int),
        FieldSpec::required("scale", FieldKind::Float),
    ],
};

const DUST_COLOR_TRANSITION_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[
        FieldSpec::required("from_color", FieldKind::RgbColor),
        FieldSpec::required("to_color", FieldKind::RgbColor),
        FieldSpec::required("scale", FieldKind::Scale),
    ],
    stream_fields: &[
        FieldSpec::required("from_color", FieldKind::Int),
        FieldSpec::required("to_color", FieldKind::Int),
        FieldSpec::required("scale", FieldKind::Float),
    ],
};

const POWER_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[FieldSpec::optional("power", FieldKind::Float, "1.0")],
    stream_fields: &[FieldSpec::required("power", FieldKind::Float)],
};

const SPELL_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[
        FieldSpec::optional("color", FieldKind::RgbColor, "-1"),
        FieldSpec::optional("power", FieldKind::Float, "1.0"),
    ],
    stream_fields: &[
        FieldSpec::required("color", FieldKind::Int),
        FieldSpec::required("power", FieldKind::Float),
    ],
};

const VIBRATION_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[
        FieldSpec::required("destination", FieldKind::SafePositionSource),
        FieldSpec::required("arrival_in_ticks", FieldKind::Int),
    ],
    stream_fields: &[
        FieldSpec::required("destination", FieldKind::PositionSource),
        FieldSpec::required("arrival_in_ticks", FieldKind::VarInt),
    ],
};

const TRAIL_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[
        FieldSpec::required("target", FieldKind::Vec3),
        FieldSpec::required("color", FieldKind::RgbColor),
        FieldSpec::required("duration", FieldKind::PositiveInt),
    ],
    stream_fields: &[
        FieldSpec::required("target", FieldKind::Vec3),
        FieldSpec::required("color", FieldKind::Int),
        FieldSpec::required("duration", FieldKind::VarInt),
    ],
};

const SCULK_CHARGE_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[FieldSpec::required("roll", FieldKind::Float)],
    stream_fields: &[FieldSpec::required("roll", FieldKind::Float)],
};

const SHRIEK_PARTICLE_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[FieldSpec::required("delay", FieldKind::Int)],
    stream_fields: &[FieldSpec::required("delay", FieldKind::VarInt)],
};

const EXPLOSION_PARTICLE_INFO_CODEC_SHAPE: CodecShape = CodecShape {
    fields: &[
        FieldSpec::required("particle", FieldKind::ParticleOptions),
        FieldSpec::optional("scaling", FieldKind::Float, "1.0"),
        FieldSpec::optional("speed", FieldKind::Float, "1.0"),
    ],
    stream_fields: &[
        FieldSpec::required("particle", FieldKind::ParticleOptions),
        FieldSpec::required("scaling", FieldKind::Float),
        FieldSpec::required("speed", FieldKind::Float),
    ],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CodecShape {
    fields: &'static [FieldSpec],
    stream_fields: &'static [FieldSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FieldSpec {
    name: &'static str,
    kind: FieldKind,
    default: Option<&'static str>,
}

impl FieldSpec {
    const fn required(name: &'static str, kind: FieldKind) -> Self {
        Self {
            name,
            kind,
            default: None,
        }
    }

    const fn optional(name: &'static str, kind: FieldKind, default: &'static str) -> Self {
        Self {
            name,
            kind,
            default: Some(default),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    ArgbColor,
    BlockState,
    BlockStateId,
    Float,
    Int,
    ItemStackTemplate,
    ParticleOptions,
    PositionSource,
    PositiveInt,
    RgbColor,
    SafePositionSource,
    Scale,
    VarInt,
    Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ParticleTypeModel {
    id: &'static str,
    override_limiter: bool,
    option_shape: ParticleOptionShape,
}

impl ParticleTypeModel {
    fn from_registry(index: usize) -> Self {
        let def = PARTICLES[index];
        Self {
            id: def.id,
            override_limiter: def.override_limiter,
            option_shape: def.option_shape,
        }
    }

    fn codec(self) -> CodecShape {
        codec_shape_for(self.option_shape)
    }

    fn stream_codec(self) -> CodecShape {
        codec_shape_for(self.option_shape)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SimpleParticleTypeModel {
    particle_type: ParticleTypeModel,
}

impl SimpleParticleTypeModel {
    fn get_type(self) -> ParticleTypeModel {
        self.particle_type
    }

    fn codec(self) -> CodecShape {
        SIMPLE_PARTICLE_CODEC_SHAPE
    }

    fn stream_codec(self) -> CodecShape {
        SIMPLE_PARTICLE_CODEC_SHAPE
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BlockParticleOptionModel {
    particle_type: ParticleTypeModel,
    block_state_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ItemParticleOptionModel {
    particle_type: ParticleTypeModel,
    item_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ColorParticleOptionModel {
    particle_type: ParticleTypeModel,
    color: u32,
}

impl ColorParticleOptionModel {
    fn create(particle_type: ParticleTypeModel, color: u32) -> Self {
        Self {
            particle_type,
            color,
        }
    }

    fn create_from_rgb(particle_type: ParticleTypeModel, red: f32, green: f32, blue: f32) -> Self {
        Self::create(particle_type, argb_color_from_float(1.0, red, green, blue))
    }

    fn red(self) -> f32 {
        argb_red(self.color) as f32 / 255.0
    }

    fn green(self) -> f32 {
        argb_green(self.color) as f32 / 255.0
    }

    fn blue(self) -> f32 {
        argb_blue(self.color) as f32 / 255.0
    }

    fn alpha(self) -> f32 {
        argb_alpha(self.color) as f32 / 255.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ScalableParticleOptionsBaseModel {
    scale: f32,
}

impl ScalableParticleOptionsBaseModel {
    const MIN_SCALE: f32 = 0.01;
    const MAX_SCALE: f32 = 4.0;

    fn new(scale: f32) -> Self {
        Self {
            scale: scale.clamp(Self::MIN_SCALE, Self::MAX_SCALE),
        }
    }

    fn validate_codec_scale(scale: f32) -> Result<f32, String> {
        if (Self::MIN_SCALE..=Self::MAX_SCALE).contains(&scale) {
            Ok(scale)
        } else {
            Err(format!("Value must be within range [0.01;4.0]: {scale}"))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DustParticleOptionsModel {
    base: ScalableParticleOptionsBaseModel,
    color: u32,
}

impl DustParticleOptionsModel {
    const REDSTONE_PARTICLE_COLOR: u32 = 0xFF0000;
    const REDSTONE: Self = Self {
        base: ScalableParticleOptionsBaseModel { scale: 1.0 },
        color: Self::REDSTONE_PARTICLE_COLOR,
    };

    fn new(color: u32, scale: f32) -> Self {
        Self {
            base: ScalableParticleOptionsBaseModel::new(scale),
            color,
        }
    }

    fn get_type(self) -> &'static str {
        "minecraft:dust"
    }

    fn color_vector(self) -> [f32; 3] {
        rgb_vector_from_24_bit(self.color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct DustColorTransitionOptionsModel {
    base: ScalableParticleOptionsBaseModel,
    from_color: u32,
    to_color: u32,
}

impl DustColorTransitionOptionsModel {
    const SCULK_PARTICLE_COLOR: u32 = 0x39D6E0;
    const SCULK_TO_REDSTONE: Self = Self {
        base: ScalableParticleOptionsBaseModel { scale: 1.0 },
        from_color: Self::SCULK_PARTICLE_COLOR,
        to_color: DustParticleOptionsModel::REDSTONE_PARTICLE_COLOR,
    };

    fn new(from_color: u32, to_color: u32, scale: f32) -> Self {
        Self {
            base: ScalableParticleOptionsBaseModel::new(scale),
            from_color,
            to_color,
        }
    }

    fn get_type(self) -> &'static str {
        "minecraft:dust_color_transition"
    }

    fn source_color_vector(self) -> [f32; 3] {
        rgb_vector_from_24_bit(self.from_color)
    }

    fn to_color_vector(self) -> [f32; 3] {
        rgb_vector_from_24_bit(self.to_color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PowerParticleOptionModel {
    particle_type: ParticleTypeModel,
    power: f32,
}

impl PowerParticleOptionModel {
    fn create(particle_type: ParticleTypeModel, power: f32) -> Self {
        Self {
            particle_type,
            power,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SpellParticleOptionModel {
    particle_type: ParticleTypeModel,
    color: u32,
    power: f32,
}

impl SpellParticleOptionModel {
    fn create(particle_type: ParticleTypeModel, color: u32, power: f32) -> Self {
        Self {
            particle_type,
            color,
            power,
        }
    }

    fn create_from_rgb(
        particle_type: ParticleTypeModel,
        red: f32,
        green: f32,
        blue: f32,
        power: f32,
    ) -> Self {
        Self::create(
            particle_type,
            argb_color_from_float(1.0, red, green, blue),
            power,
        )
    }

    fn red(self) -> f32 {
        argb_red(self.color) as f32 / 255.0
    }

    fn green(self) -> f32 {
        argb_green(self.color) as f32 / 255.0
    }

    fn blue(self) -> f32 {
        argb_blue(self.color) as f32 / 255.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SculkChargeParticleOptionsModel {
    roll: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShriekParticleOptionModel {
    delay: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TrailParticleOptionModel {
    target: [i32; 3],
    color: u32,
    duration: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionSourceKind {
    Block,
    Entity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VibrationParticleOptionModel {
    destination: PositionSourceKind,
    arrival_in_ticks: i32,
}

impl VibrationParticleOptionModel {
    fn codec_accepts_destination(destination: PositionSourceKind) -> bool {
        !matches!(destination, PositionSourceKind::Entity)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ExplosionParticleInfoModel {
    particle: ParticleTypeModel,
    scaling: f32,
    speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParticleLimitModel {
    limit: i32,
}

impl ParticleLimitModel {
    const SPORE_BLOSSOM: Self = Self { limit: 1000 };
}

fn codec_shape_for(shape: ParticleOptionShape) -> CodecShape {
    match shape {
        ParticleOptionShape::Simple => SIMPLE_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Block => BLOCK_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Dust => DUST_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::DustColorTransition => DUST_COLOR_TRANSITION_CODEC_SHAPE,
        ParticleOptionShape::Color => COLOR_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Power => POWER_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Item => ITEM_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Vibration => VIBRATION_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Trail => TRAIL_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::SculkCharge => SCULK_CHARGE_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Shriek => SHRIEK_PARTICLE_CODEC_SHAPE,
        ParticleOptionShape::Spell => SPELL_PARTICLE_CODEC_SHAPE,
    }
}

fn argb_color_from_float(alpha: f32, red: f32, green: f32, blue: f32) -> u32 {
    ((alpha * 255.0) as u32) << 24
        | ((red * 255.0) as u32) << 16
        | ((green * 255.0) as u32) << 8
        | (blue * 255.0) as u32
}

fn argb_alpha(color: u32) -> u8 {
    (color >> 24) as u8
}

fn argb_red(color: u32) -> u8 {
    (color >> 16) as u8
}

fn argb_green(color: u32) -> u8 {
    (color >> 8) as u8
}

fn argb_blue(color: u32) -> u8 {
    color as u8
}

fn rgb_vector_from_24_bit(color: u32) -> [f32; 3] {
    [
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
    ]
}

fn find_particle(id: &str) -> ParticleTypeModel {
    let index = PARTICLES
        .iter()
        .position(|particle| particle.id == id)
        .expect("particle id should be registered");
    ParticleTypeModel::from_registry(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_types_registry_matches_java_order_and_shapes() {
        assert_eq!(PARTICLES.len(), 117);
        assert_eq!(PARTICLES.first().unwrap().id, "minecraft:angry_villager");
        assert_eq!(PARTICLES.last().unwrap().id, "minecraft:firefly");

        let expected = [
            (
                "minecraft:angry_villager",
                false,
                ParticleOptionShape::Simple,
            ),
            ("minecraft:block", false, ParticleOptionShape::Block),
            ("minecraft:block_marker", true, ParticleOptionShape::Block),
            (
                "minecraft:damage_indicator",
                true,
                ParticleOptionShape::Simple,
            ),
            ("minecraft:dust", false, ParticleOptionShape::Dust),
            (
                "minecraft:dust_color_transition",
                false,
                ParticleOptionShape::DustColorTransition,
            ),
            ("minecraft:effect", false, ParticleOptionShape::Spell),
            ("minecraft:entity_effect", false, ParticleOptionShape::Color),
            ("minecraft:dragon_breath", false, ParticleOptionShape::Power),
            ("minecraft:item", false, ParticleOptionShape::Item),
            ("minecraft:vibration", true, ParticleOptionShape::Vibration),
            ("minecraft:trail", false, ParticleOptionShape::Trail),
            (
                "minecraft:sculk_charge",
                true,
                ParticleOptionShape::SculkCharge,
            ),
            ("minecraft:shriek", false, ParticleOptionShape::Shriek),
            ("minecraft:block_crumble", false, ParticleOptionShape::Block),
            ("minecraft:firefly", false, ParticleOptionShape::Simple),
        ];

        for (id, override_limiter, option_shape) in expected {
            let particle = find_particle(id);
            assert_eq!(particle.id, id);
            assert_eq!(particle.override_limiter, override_limiter);
            assert_eq!(particle.option_shape, option_shape);
        }
    }

    #[test]
    fn particle_types_registry_shape_distribution_matches_java() {
        assert_eq!(count_shape(ParticleOptionShape::Simple), 99);
        assert_eq!(count_shape(ParticleOptionShape::Block), 5);
        assert_eq!(count_shape(ParticleOptionShape::Color), 3);
        assert_eq!(count_shape(ParticleOptionShape::Spell), 2);
        assert_eq!(count_shape(ParticleOptionShape::Power), 1);
        assert_eq!(count_shape(ParticleOptionShape::Item), 1);
        assert_eq!(count_shape(ParticleOptionShape::Dust), 1);
        assert_eq!(count_shape(ParticleOptionShape::DustColorTransition), 1);
        assert_eq!(count_shape(ParticleOptionShape::Vibration), 1);
        assert_eq!(count_shape(ParticleOptionShape::Trail), 1);
        assert_eq!(count_shape(ParticleOptionShape::SculkCharge), 1);
        assert_eq!(count_shape(ParticleOptionShape::Shriek), 1);
    }

    #[test]
    fn particle_options_dispatch_to_type_codecs() {
        let dust = find_particle("minecraft:dust");
        assert_eq!(dust.codec(), DUST_PARTICLE_CODEC_SHAPE);
        assert_eq!(dust.stream_codec(), DUST_PARTICLE_CODEC_SHAPE);

        let entity_effect = find_particle("minecraft:entity_effect");
        assert_eq!(entity_effect.codec(), COLOR_PARTICLE_CODEC_SHAPE);
        assert_eq!(entity_effect.stream_codec(), COLOR_PARTICLE_CODEC_SHAPE);

        let simple = SimpleParticleTypeModel {
            particle_type: find_particle("minecraft:cloud"),
        };
        assert_eq!(simple.get_type().id, "minecraft:cloud");
        assert_eq!(simple.codec(), SIMPLE_PARTICLE_CODEC_SHAPE);
        assert_eq!(simple.stream_codec(), SIMPLE_PARTICLE_CODEC_SHAPE);
    }

    #[test]
    fn block_and_item_options_retain_type_and_payload() {
        let block = BlockParticleOptionModel {
            particle_type: find_particle("minecraft:block_marker"),
            block_state_id: 42,
        };
        assert_eq!(block.particle_type.id, "minecraft:block_marker");
        assert_eq!(block.block_state_id, 42);
        assert_eq!(
            codec_shape_for(block.particle_type.option_shape),
            BLOCK_PARTICLE_CODEC_SHAPE
        );

        let item = ItemParticleOptionModel {
            particle_type: find_particle("minecraft:item"),
            item_id: "minecraft:diamond",
        };
        assert_eq!(item.particle_type.id, "minecraft:item");
        assert_eq!(item.item_id, "minecraft:diamond");
        assert_eq!(
            codec_shape_for(item.particle_type.option_shape),
            ITEM_PARTICLE_CODEC_SHAPE
        );
    }

    #[test]
    fn color_options_unpack_argb_channels_and_rgb_factory_sets_opaque_alpha() {
        let option = ColorParticleOptionModel::create(find_particle("minecraft:flash"), 0x80402010);
        assert_eq!(option.particle_type.id, "minecraft:flash");
        assert_eq!(option.red(), 64.0 / 255.0);
        assert_eq!(option.green(), 32.0 / 255.0);
        assert_eq!(option.blue(), 16.0 / 255.0);
        assert_eq!(option.alpha(), 128.0 / 255.0);

        let rgb = ColorParticleOptionModel::create_from_rgb(
            find_particle("minecraft:entity_effect"),
            1.0,
            0.5,
            0.0,
        );
        assert_eq!(rgb.color, 0xFFFF7F00);
        assert_eq!(rgb.alpha(), 1.0);
    }

    #[test]
    fn scalable_particle_options_clamp_constructors_and_validate_codec_range() {
        assert_eq!(
            ScalableParticleOptionsBaseModel::new(0.0).scale,
            ScalableParticleOptionsBaseModel::MIN_SCALE
        );
        assert_eq!(
            ScalableParticleOptionsBaseModel::new(8.0).scale,
            ScalableParticleOptionsBaseModel::MAX_SCALE
        );
        assert_eq!(ScalableParticleOptionsBaseModel::new(2.5).scale, 2.5);
        assert_eq!(
            ScalableParticleOptionsBaseModel::validate_codec_scale(0.01),
            Ok(0.01)
        );
        assert_eq!(
            ScalableParticleOptionsBaseModel::validate_codec_scale(4.0),
            Ok(4.0)
        );
        assert_eq!(
            ScalableParticleOptionsBaseModel::validate_codec_scale(0.0).unwrap_err(),
            "Value must be within range [0.01;4.0]: 0"
        );
    }

    #[test]
    fn dust_options_match_constants_type_and_rgb_vectors() {
        assert_eq!(
            DustParticleOptionsModel::REDSTONE_PARTICLE_COLOR,
            16_711_680
        );
        assert_eq!(DustParticleOptionsModel::REDSTONE.color, 0xFF0000);
        assert_eq!(DustParticleOptionsModel::REDSTONE.base.scale, 1.0);

        let option = DustParticleOptionsModel::new(0x336699, 99.0);
        assert_eq!(option.get_type(), "minecraft:dust");
        assert_eq!(option.base.scale, 4.0);
        assert_eq!(
            option.color_vector(),
            [51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0]
        );
    }

    #[test]
    fn dust_transition_options_match_constants_type_and_rgb_vectors() {
        assert_eq!(
            DustColorTransitionOptionsModel::SCULK_PARTICLE_COLOR,
            3_790_560
        );
        assert_eq!(
            DustColorTransitionOptionsModel::SCULK_TO_REDSTONE.from_color,
            0x39D6E0
        );
        assert_eq!(
            DustColorTransitionOptionsModel::SCULK_TO_REDSTONE.to_color,
            0xFF0000
        );

        let option = DustColorTransitionOptionsModel::new(0x010203, 0xA0B0C0, -5.0);
        assert_eq!(option.get_type(), "minecraft:dust_color_transition");
        assert_eq!(option.base.scale, 0.01);
        assert_eq!(
            option.source_color_vector(),
            [1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0]
        );
        assert_eq!(
            option.to_color_vector(),
            [160.0 / 255.0, 176.0 / 255.0, 192.0 / 255.0]
        );
    }

    #[test]
    fn power_and_spell_options_match_defaults_factories_and_channels() {
        assert_eq!(
            POWER_PARTICLE_CODEC_SHAPE.fields,
            &[FieldSpec::optional("power", FieldKind::Float, "1.0")]
        );
        let power =
            PowerParticleOptionModel::create(find_particle("minecraft:dragon_breath"), 2.25);
        assert_eq!(power.particle_type.id, "minecraft:dragon_breath");
        assert_eq!(power.power, 2.25);

        assert_eq!(
            SPELL_PARTICLE_CODEC_SHAPE.fields,
            &[
                FieldSpec::optional("color", FieldKind::RgbColor, "-1"),
                FieldSpec::optional("power", FieldKind::Float, "1.0")
            ]
        );
        let spell = SpellParticleOptionModel::create_from_rgb(
            find_particle("minecraft:instant_effect"),
            0.25,
            0.5,
            1.0,
            0.75,
        );
        assert_eq!(spell.color, 0xFF3F7FFF);
        assert_eq!(spell.red(), 63.0 / 255.0);
        assert_eq!(spell.green(), 127.0 / 255.0);
        assert_eq!(spell.blue(), 1.0);
        assert_eq!(spell.power, 0.75);
    }

    #[test]
    fn record_like_options_match_java_fields_and_type_targets() {
        let sculk = SculkChargeParticleOptionsModel { roll: 1.5 };
        assert_eq!(sculk.roll, 1.5);
        assert_eq!(SCULK_CHARGE_PARTICLE_CODEC_SHAPE.fields[0].name, "roll");

        let shriek = ShriekParticleOptionModel { delay: 40 };
        assert_eq!(shriek.delay, 40);
        assert_eq!(
            SHRIEK_PARTICLE_CODEC_SHAPE.stream_fields[0].kind,
            FieldKind::VarInt
        );

        let trail = TrailParticleOptionModel {
            target: [1, 2, 3],
            color: 0x123456,
            duration: 20,
        };
        assert_eq!(trail.target, [1, 2, 3]);
        assert_eq!(
            TRAIL_PARTICLE_CODEC_SHAPE.fields[2].kind,
            FieldKind::PositiveInt
        );

        let vibration = VibrationParticleOptionModel {
            destination: PositionSourceKind::Block,
            arrival_in_ticks: 7,
        };
        assert_eq!(vibration.destination, PositionSourceKind::Block);
        assert_eq!(vibration.arrival_in_ticks, 7);
        assert!(VibrationParticleOptionModel::codec_accepts_destination(
            PositionSourceKind::Block
        ));
        assert!(!VibrationParticleOptionModel::codec_accepts_destination(
            PositionSourceKind::Entity
        ));
    }

    #[test]
    fn explosion_info_and_particle_limit_match_java_records() {
        assert_eq!(
            EXPLOSION_PARTICLE_INFO_CODEC_SHAPE.fields,
            &[
                FieldSpec::required("particle", FieldKind::ParticleOptions),
                FieldSpec::optional("scaling", FieldKind::Float, "1.0"),
                FieldSpec::optional("speed", FieldKind::Float, "1.0")
            ]
        );
        let info = ExplosionParticleInfoModel {
            particle: find_particle("minecraft:explosion"),
            scaling: 1.25,
            speed: 0.5,
        };
        assert_eq!(info.particle.id, "minecraft:explosion");
        assert_eq!(info.scaling, 1.25);
        assert_eq!(info.speed, 0.5);

        assert_eq!(ParticleLimitModel::SPORE_BLOSSOM.limit, 1000);
    }

    #[test]
    fn package_is_null_marked() {
        const {
            assert!(CORE_PARTICLES_PACKAGE_NULL_MARKED);
        }
    }

    fn count_shape(shape: ParticleOptionShape) -> usize {
        PARTICLES
            .iter()
            .filter(|particle| particle.option_shape == shape)
            .count()
    }
}
