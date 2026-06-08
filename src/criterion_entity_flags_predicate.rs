#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityFlagsPredicateModel {
    pub is_on_ground: Option<bool>,
    pub is_on_fire: Option<bool>,
    pub is_crouching: Option<bool>,
    pub is_sprinting: Option<bool>,
    pub is_swimming: Option<bool>,
    pub is_flying: Option<bool>,
    pub is_baby: Option<bool>,
    pub is_in_water: Option<bool>,
    pub is_fall_flying: Option<bool>,
}

impl EntityFlagsPredicateModel {
    pub fn builder() -> EntityFlagsPredicateBuilder {
        EntityFlagsPredicateBuilder::flags()
    }

    pub fn matches(&self, entity: &EntityFlagStateModel) -> bool {
        if self
            .is_on_ground
            .is_some_and(|expected| entity.on_ground != expected)
        {
            return false;
        }

        if self
            .is_on_fire
            .is_some_and(|expected| entity.on_fire != expected)
        {
            return false;
        }

        if self
            .is_crouching
            .is_some_and(|expected| entity.crouching != expected)
        {
            return false;
        }

        if self
            .is_sprinting
            .is_some_and(|expected| entity.sprinting != expected)
        {
            return false;
        }

        if self
            .is_swimming
            .is_some_and(|expected| entity.swimming != expected)
        {
            return false;
        }

        if let Some(expected_flying) = self.is_flying {
            let entity_is_flying = entity.living
                && (entity.fall_flying || (entity.player && entity.player_ability_flying));
            if entity_is_flying != expected_flying {
                return false;
            }
        }

        if self
            .is_in_water
            .is_some_and(|expected| entity.in_water != expected)
        {
            return false;
        }

        if self
            .is_fall_flying
            .is_some_and(|expected| entity.living && entity.fall_flying != expected)
        {
            return false;
        }

        !(self.is_baby.is_some() && entity.living)
            || entity.is_baby == self.is_baby.unwrap_or(entity.is_baby)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityFlagsPredicateBuilder {
    is_on_ground: Option<bool>,
    is_on_fire: Option<bool>,
    is_crouching: Option<bool>,
    is_sprinting: Option<bool>,
    is_swimming: Option<bool>,
    is_flying: Option<bool>,
    is_baby: Option<bool>,
    is_in_water: Option<bool>,
    is_fall_flying: Option<bool>,
}

impl EntityFlagsPredicateBuilder {
    pub fn flags() -> Self {
        Self::default()
    }

    pub fn set_on_ground(mut self, on_ground: bool) -> Self {
        self.is_on_ground = Some(on_ground);
        self
    }

    pub fn set_on_fire(mut self, on_fire: bool) -> Self {
        self.is_on_fire = Some(on_fire);
        self
    }

    pub fn set_crouching(mut self, crouching: bool) -> Self {
        self.is_crouching = Some(crouching);
        self
    }

    pub fn set_sprinting(mut self, sprinting: bool) -> Self {
        self.is_sprinting = Some(sprinting);
        self
    }

    pub fn set_swimming(mut self, swimming: bool) -> Self {
        self.is_swimming = Some(swimming);
        self
    }

    pub fn set_is_flying(mut self, flying: bool) -> Self {
        self.is_flying = Some(flying);
        self
    }

    pub fn set_is_baby(mut self, baby: bool) -> Self {
        self.is_baby = Some(baby);
        self
    }

    pub fn set_is_in_water(mut self, in_water: bool) -> Self {
        self.is_in_water = Some(in_water);
        self
    }

    pub fn set_is_fall_flying(mut self, fall_flying: bool) -> Self {
        self.is_fall_flying = Some(fall_flying);
        self
    }

    pub fn build(self) -> EntityFlagsPredicateModel {
        EntityFlagsPredicateModel {
            is_on_ground: self.is_on_ground,
            is_on_fire: self.is_on_fire,
            is_crouching: self.is_crouching,
            is_sprinting: self.is_sprinting,
            is_swimming: self.is_swimming,
            is_flying: self.is_flying,
            is_baby: self.is_baby,
            is_in_water: self.is_in_water,
            is_fall_flying: self.is_fall_flying,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityFlagStateModel {
    living: bool,
    player: bool,
    on_ground: bool,
    on_fire: bool,
    crouching: bool,
    sprinting: bool,
    swimming: bool,
    player_ability_flying: bool,
    is_baby: bool,
    in_water: bool,
    fall_flying: bool,
}

impl EntityFlagStateModel {
    pub fn entity() -> Self {
        Self::default()
    }

    pub fn living() -> Self {
        Self {
            living: true,
            ..Self::default()
        }
    }

    pub fn player() -> Self {
        Self {
            living: true,
            player: true,
            ..Self::default()
        }
    }

    pub fn on_ground(mut self, on_ground: bool) -> Self {
        self.on_ground = on_ground;
        self
    }

    pub fn on_fire(mut self, on_fire: bool) -> Self {
        self.on_fire = on_fire;
        self
    }

    pub fn crouching(mut self, crouching: bool) -> Self {
        self.crouching = crouching;
        self
    }

    pub fn sprinting(mut self, sprinting: bool) -> Self {
        self.sprinting = sprinting;
        self
    }

    pub fn swimming(mut self, swimming: bool) -> Self {
        self.swimming = swimming;
        self
    }

    pub fn player_ability_flying(mut self, flying: bool) -> Self {
        self.player_ability_flying = flying;
        self
    }

    pub fn baby(mut self, baby: bool) -> Self {
        self.is_baby = baby;
        self
    }

    pub fn in_water(mut self, in_water: bool) -> Self {
        self.in_water = in_water;
        self
    }

    pub fn fall_flying(mut self, fall_flying: bool) -> Self {
        self.fall_flying = fall_flying;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_flags_predicate_matches_any_entity() {
        let predicate = EntityFlagsPredicateModel::builder().build();

        assert!(predicate.matches(&EntityFlagStateModel::entity()));
        assert!(predicate.matches(&EntityFlagStateModel::living().baby(true)));
    }

    #[test]
    fn basic_entity_flags_match_in_java_order() {
        let entity = EntityFlagStateModel::living()
            .on_ground(true)
            .on_fire(false)
            .crouching(true)
            .sprinting(false)
            .swimming(true)
            .in_water(true);
        let predicate = EntityFlagsPredicateModel::builder()
            .set_on_ground(true)
            .set_on_fire(false)
            .set_crouching(true)
            .set_sprinting(false)
            .set_swimming(true)
            .set_is_in_water(true)
            .build();

        assert!(predicate.matches(&entity));
        assert!(!EntityFlagsPredicateModel::builder()
            .set_on_ground(false)
            .set_on_fire(false)
            .build()
            .matches(&entity));
    }

    #[test]
    fn flying_flag_combines_living_fall_flying_and_player_ability_flying() {
        let flying = EntityFlagsPredicateModel::builder()
            .set_is_flying(true)
            .build();
        let not_flying = EntityFlagsPredicateModel::builder()
            .set_is_flying(false)
            .build();

        assert!(flying.matches(&EntityFlagStateModel::living().fall_flying(true)));
        assert!(flying.matches(&EntityFlagStateModel::player().player_ability_flying(true)));
        assert!(!flying.matches(&EntityFlagStateModel::living()));
        assert!(!flying.matches(&EntityFlagStateModel::entity().fall_flying(true)));
        assert!(not_flying.matches(&EntityFlagStateModel::entity().fall_flying(true)));
    }

    #[test]
    fn fall_flying_and_baby_predicates_are_ignored_for_non_living_entities() {
        let predicate = EntityFlagsPredicateModel::builder()
            .set_is_fall_flying(true)
            .set_is_baby(true)
            .build();

        assert!(predicate.matches(&EntityFlagStateModel::entity()));
        assert!(!predicate.matches(&EntityFlagStateModel::living()));
        assert!(predicate.matches(&EntityFlagStateModel::living().fall_flying(true).baby(true)));
    }

    #[test]
    fn fall_flying_is_checked_before_baby_for_living_entities() {
        let predicate = EntityFlagsPredicateModel::builder()
            .set_is_fall_flying(false)
            .set_is_baby(true)
            .build();

        assert!(!predicate.matches(&EntityFlagStateModel::living().fall_flying(true).baby(true)));
    }

    #[test]
    fn builder_preserves_all_optional_flag_fields() {
        let predicate = EntityFlagsPredicateModel::builder()
            .set_on_ground(true)
            .set_on_fire(true)
            .set_crouching(true)
            .set_sprinting(true)
            .set_swimming(true)
            .set_is_flying(true)
            .set_is_baby(false)
            .set_is_in_water(true)
            .set_is_fall_flying(false)
            .build();

        assert_eq!(
            predicate,
            EntityFlagsPredicateModel {
                is_on_ground: Some(true),
                is_on_fire: Some(true),
                is_crouching: Some(true),
                is_sprinting: Some(true),
                is_swimming: Some(true),
                is_flying: Some(true),
                is_baby: Some(false),
                is_in_water: Some(true),
                is_fall_flying: Some(false),
            }
        );
    }
}
