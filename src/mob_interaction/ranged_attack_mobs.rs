#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangedAttackMobMethodSurface {
    pub method_name: &'static str,
    pub target_parameter_type: &'static str,
    pub power_parameter_type: &'static str,
    pub return_type: &'static str,
}

pub const RANGED_ATTACK_MOB_METHOD: RangedAttackMobMethodSurface = RangedAttackMobMethodSurface {
    method_name: "performRangedAttack",
    target_parameter_type: "LivingEntity",
    power_parameter_type: "float",
    return_type: "void",
};

pub fn ranged_attack_mob_method_surface() -> RangedAttackMobMethodSurface {
    RANGED_ATTACK_MOB_METHOD
}
