#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishingHookPredicateModel {
    pub in_open_water: Option<bool>,
}

impl FishingHookPredicateModel {
    pub const ANY: Self = Self {
        in_open_water: None,
    };

    pub fn in_open_water(requirement: bool) -> Self {
        Self {
            in_open_water: Some(requirement),
        }
    }

    pub fn codec(&self) -> EntitySubPredicateTypeModel {
        EntitySubPredicateTypeModel::FishingHook
    }

    pub fn matches(
        &self,
        entity: &EntityContextModel,
        _level: &LevelContextModel,
        _position: Option<Vec3Model>,
    ) -> bool {
        let Some(requirement) = self.in_open_water else {
            return true;
        };

        entity
            .fishing_hook
            .is_some_and(|hook| hook.open_water == requirement)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySubPredicateTypeModel {
    FishingHook,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EntityContextModel {
    fishing_hook: Option<FishingHookEntityModel>,
}

impl EntityContextModel {
    pub fn generic() -> Self {
        Self { fishing_hook: None }
    }

    pub fn fishing_hook(open_water: bool) -> Self {
        Self {
            fishing_hook: Some(FishingHookEntityModel { open_water }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FishingHookEntityModel {
    open_water: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LevelContextModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec3Model {
    x: i32,
    y: i32,
    z: i32,
}

impl Vec3Model {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_matches_fishing_hooks_and_non_hook_entities() {
        assert!(FishingHookPredicateModel::ANY.matches(
            &EntityContextModel::generic(),
            &LevelContextModel,
            None
        ));
        assert!(FishingHookPredicateModel::ANY.matches(
            &EntityContextModel::fishing_hook(false),
            &LevelContextModel,
            Some(Vec3Model::new(0, 64, 0))
        ));
    }

    #[test]
    fn open_water_requirement_only_matches_fishing_hook_entities() {
        let in_open_water = FishingHookPredicateModel::in_open_water(true);
        let not_in_open_water = FishingHookPredicateModel::in_open_water(false);

        assert!(in_open_water.matches(
            &EntityContextModel::fishing_hook(true),
            &LevelContextModel,
            None
        ));
        assert!(!in_open_water.matches(
            &EntityContextModel::fishing_hook(false),
            &LevelContextModel,
            None
        ));
        assert!(!in_open_water.matches(&EntityContextModel::generic(), &LevelContextModel, None));
        assert!(not_in_open_water.matches(
            &EntityContextModel::fishing_hook(false),
            &LevelContextModel,
            None
        ));
    }

    #[test]
    fn codec_returns_registered_fishing_hook_type() {
        assert_eq!(
            FishingHookPredicateModel::in_open_water(true).codec(),
            EntitySubPredicateTypeModel::FishingHook
        );
    }
}
