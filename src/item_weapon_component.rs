#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponComponent {
    pub item_damage_per_attack: i32,
    pub disable_blocking_for_seconds: f32,
}

impl WeaponComponent {
    pub const AXE_DISABLES_BLOCKING_FOR_SECONDS: f32 = 5.0;
    pub const DEFAULT_ITEM_DAMAGE_PER_ATTACK: i32 = 1;
    pub const DEFAULT_DISABLE_BLOCKING_FOR_SECONDS: f32 = 0.0;

    pub fn new(item_damage_per_attack: i32, disable_blocking_for_seconds: f32) -> Self {
        Self {
            item_damage_per_attack,
            disable_blocking_for_seconds,
        }
    }

    pub fn with_damage_per_attack(item_damage_per_attack: i32) -> Self {
        Self::new(
            item_damage_per_attack,
            Self::DEFAULT_DISABLE_BLOCKING_FOR_SECONDS,
        )
    }

    pub fn default_component() -> Self {
        Self::with_damage_per_attack(Self::DEFAULT_ITEM_DAMAGE_PER_ATTACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEAPON_JAVA: &str = vibecraft_java_source!("/net/minecraft/world/item/component/Weapon.java");

    #[test]
    fn weapon_component_defaults_and_constant_match_java() {
        for sentinel in [
            "public record Weapon(int itemDamagePerAttack, float disableBlockingForSeconds)",
            "public static final float AXE_DISABLES_BLOCKING_FOR_SECONDS = 5.0F;",
            "ExtraCodecs.NON_NEGATIVE_INT.optionalFieldOf(\"item_damage_per_attack\", 1)",
            "ExtraCodecs.NON_NEGATIVE_FLOAT.optionalFieldOf(\"disable_blocking_for_seconds\", 0.0F)",
            "ByteBufCodecs.VAR_INT, Weapon::itemDamagePerAttack, ByteBufCodecs.FLOAT, Weapon::disableBlockingForSeconds",
            "public Weapon(final int damagePerAttack)",
            "this(damagePerAttack, 0.0F);",
        ] {
            assert!(
                WEAPON_JAVA.contains(sentinel),
                "missing Weapon sentinel {sentinel}"
            );
        }

        assert_eq!(WeaponComponent::AXE_DISABLES_BLOCKING_FOR_SECONDS, 5.0);
        assert_eq!(WeaponComponent::DEFAULT_ITEM_DAMAGE_PER_ATTACK, 1);
        assert_eq!(WeaponComponent::DEFAULT_DISABLE_BLOCKING_FOR_SECONDS, 0.0);
        assert_eq!(
            WeaponComponent::default_component(),
            WeaponComponent {
                item_damage_per_attack: 1,
                disable_blocking_for_seconds: 0.0
            }
        );
        assert_eq!(
            WeaponComponent::with_damage_per_attack(2),
            WeaponComponent {
                item_damage_per_attack: 2,
                disable_blocking_for_seconds: 0.0
            }
        );
        assert_eq!(
            WeaponComponent::new(1, WeaponComponent::AXE_DISABLES_BLOCKING_FOR_SECONDS),
            WeaponComponent {
                item_damage_per_attack: 1,
                disable_blocking_for_seconds: 5.0
            }
        );
    }
}
