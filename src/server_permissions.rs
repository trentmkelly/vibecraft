#![allow(dead_code)]

use crate::registry::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevelModel {
    All = 0,
    Moderators = 1,
    Gamemasters = 2,
    Admins = 3,
    Owners = 4,
}

impl PermissionLevelModel {
    pub const VALUES: [Self; 5] = [
        Self::All,
        Self::Moderators,
        Self::Gamemasters,
        Self::Admins,
        Self::Owners,
    ];

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Moderators => "moderators",
            Self::Gamemasters => "gamemasters",
            Self::Admins => "admins",
            Self::Owners => "owners",
        }
    }

    pub fn id(self) -> i32 {
        self as i32
    }

    pub fn by_id(level: i32) -> Self {
        if level <= Self::All.id() {
            Self::All
        } else if level >= Self::Owners.id() {
            Self::Owners
        } else {
            Self::VALUES[level as usize]
        }
    }

    pub fn is_equal_or_higher_than(self, other: Self) -> bool {
        self.id() >= other.id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionModel {
    Atom(Identifier),
    HasCommandLevel(PermissionLevelModel),
}

impl PermissionModel {
    pub fn atom(name: &str) -> Result<Self, String> {
        Ok(Self::Atom(Identifier::with_default_namespace(name)?))
    }

    pub fn command_level(level: PermissionLevelModel) -> Self {
        Self::HasCommandLevel(level)
    }

    pub fn codec_name(&self) -> &'static str {
        match self {
            Self::Atom(_) => "atom",
            Self::HasCommandLevel(_) => "command_level",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionCheckModel {
    AlwaysPass,
    Require(PermissionModel),
}

impl PermissionCheckModel {
    pub fn check(&self, source: &PermissionSetModel) -> bool {
        match self {
            Self::AlwaysPass => true,
            Self::Require(permission) => source.has_permission(permission),
        }
    }

    pub fn codec_name(&self) -> &'static str {
        match self {
            Self::AlwaysPass => "always_pass",
            Self::Require(_) => "require",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionSetModel {
    NoPermissions,
    AllPermissions,
    LevelBased(PermissionLevelModel),
    Explicit(Vec<PermissionModel>),
    Union(Vec<PermissionSetModel>),
}

impl PermissionSetModel {
    pub fn has_permission(&self, permission: &PermissionModel) -> bool {
        match self {
            Self::NoPermissions => false,
            Self::AllPermissions => true,
            Self::LevelBased(level) => level_based_has_permission(*level, permission),
            Self::Explicit(permissions) => permissions.iter().any(|candidate| candidate == permission),
            Self::Union(sets) => sets.iter().any(|set| set.has_permission(permission)),
        }
    }

    pub fn union(self, other: Self) -> Result<Self, String> {
        match (&self, &other) {
            (Self::LevelBased(left), Self::LevelBased(right)) => {
                if left.is_equal_or_higher_than(*right) {
                    Ok(other)
                } else {
                    Ok(self)
                }
            }
            (_, Self::Union(sets)) => {
                let mut flattened = sets.clone();
                flattened.push(self);
                Self::union_from_sets(flattened)
            }
            (Self::Union(sets), _) => {
                let mut flattened = sets.clone();
                flattened.push(other);
                Self::union_from_sets(flattened)
            }
            _ => Self::union_from_sets(vec![self, other]),
        }
    }

    pub fn union_members(&self) -> Option<&[PermissionSetModel]> {
        match self {
            Self::Union(sets) => Some(sets),
            _ => None,
        }
    }

    fn union_from_sets(sets: Vec<Self>) -> Result<Self, String> {
        if sets.iter().any(|set| matches!(set, Self::Union(_))) {
            return Err("Cannot have PermissionSetUnion within another PermissionSetUnion".to_string());
        }

        let mut unique = Vec::new();
        for set in sets {
            if !unique.contains(&set) {
                unique.push(set);
            }
        }
        Ok(Self::Union(unique))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionSetSupplierModel {
    permissions: PermissionSetModel,
}

impl PermissionSetSupplierModel {
    pub fn new(permissions: PermissionSetModel) -> Self {
        Self { permissions }
    }

    pub fn permissions(&self) -> &PermissionSetModel {
        &self.permissions
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionProviderCheckModel {
    test: PermissionCheckModel,
}

impl PermissionProviderCheckModel {
    pub fn new(test: PermissionCheckModel) -> Self {
        Self { test }
    }

    pub fn test(&self, supplier: &PermissionSetSupplierModel) -> bool {
        self.test.check(supplier.permissions())
    }
}

pub fn level_based_permission_set(level: PermissionLevelModel) -> PermissionSetModel {
    PermissionSetModel::LevelBased(level)
}

pub fn permission_types_bootstrap_order() -> Vec<&'static str> {
    vec!["atom", "command_level"]
}

pub fn permission_check_types_bootstrap_order() -> Vec<&'static str> {
    vec!["always_pass", "require"]
}

pub mod permissions {
    use super::{PermissionLevelModel, PermissionModel};

    pub fn commands_moderator() -> PermissionModel {
        PermissionModel::command_level(PermissionLevelModel::Moderators)
    }

    pub fn commands_gamemaster() -> PermissionModel {
        PermissionModel::command_level(PermissionLevelModel::Gamemasters)
    }

    pub fn commands_admin() -> PermissionModel {
        PermissionModel::command_level(PermissionLevelModel::Admins)
    }

    pub fn commands_owner() -> PermissionModel {
        PermissionModel::command_level(PermissionLevelModel::Owners)
    }

    pub fn commands_entity_selectors() -> PermissionModel {
        atom("commands/entity_selectors")
    }

    pub fn chat_send_messages() -> PermissionModel {
        atom("chat/send_messages")
    }

    pub fn chat_send_commands() -> PermissionModel {
        atom("chat/send_commands")
    }

    pub fn chat_receive_player_messages() -> PermissionModel {
        atom("chat/receive_player_messages")
    }

    pub fn chat_receive_system_messages() -> PermissionModel {
        atom("chat/receive_system_messages")
    }

    pub fn chat_permissions() -> Vec<PermissionModel> {
        vec![
            chat_send_messages(),
            chat_send_commands(),
            chat_receive_player_messages(),
            chat_receive_system_messages(),
        ]
    }

    fn atom(name: &str) -> PermissionModel {
        match PermissionModel::atom(name) {
            Ok(permission) => permission,
            Err(error) => panic!("built-in permission id is invalid: {error}"),
        }
    }
}

fn level_based_has_permission(level: PermissionLevelModel, permission: &PermissionModel) -> bool {
    match permission {
        PermissionModel::HasCommandLevel(required) => level.is_equal_or_higher_than(*required),
        other => {
            *other == permissions::commands_entity_selectors()
                && level.is_equal_or_higher_than(PermissionLevelModel::Gamemasters)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEVEL_BASED_PERMISSION_SET_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/LevelBasedPermissionSet.java");
    const PERMISSION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/Permission.java");
    const PERMISSION_CHECK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionCheck.java");
    const PERMISSION_CHECK_TYPES_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionCheckTypes.java");
    const PERMISSION_LEVEL_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionLevel.java");
    const PERMISSION_PROVIDER_CHECK_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionProviderCheck.java");
    const PERMISSION_SET_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionSet.java");
    const PERMISSION_SET_SUPPLIER_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionSetSupplier.java");
    const PERMISSION_SET_UNION_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionSetUnion.java");
    const PERMISSION_TYPES_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/PermissionTypes.java");
    const PERMISSIONS_JAVA: &str =
        vibecraft_java_source!("/net/minecraft/server/permissions/Permissions.java");

    #[test]
    fn permission_levels_are_clamped_ordered_and_named_like_java() {
        assert_eq!(
            PermissionLevelModel::VALUES.map(PermissionLevelModel::serialized_name),
            ["all", "moderators", "gamemasters", "admins", "owners"]
        );
        assert_eq!(PermissionLevelModel::by_id(-1), PermissionLevelModel::All);
        assert_eq!(PermissionLevelModel::by_id(0), PermissionLevelModel::All);
        assert_eq!(PermissionLevelModel::by_id(2), PermissionLevelModel::Gamemasters);
        assert_eq!(PermissionLevelModel::by_id(99), PermissionLevelModel::Owners);
        assert!(PermissionLevelModel::Admins.is_equal_or_higher_than(PermissionLevelModel::Gamemasters));
        assert!(!PermissionLevelModel::Moderators.is_equal_or_higher_than(PermissionLevelModel::Admins));
    }

    #[test]
    fn level_based_permission_sets_match_java_special_cases() {
        let moderator = level_based_permission_set(PermissionLevelModel::Moderators);
        let gamemaster = level_based_permission_set(PermissionLevelModel::Gamemasters);
        let admin = level_based_permission_set(PermissionLevelModel::Admins);

        assert!(admin.has_permission(&permissions::commands_moderator()));
        assert!(admin.has_permission(&permissions::commands_admin()));
        assert!(!gamemaster.has_permission(&permissions::commands_admin()));
        assert!(!moderator.has_permission(&permissions::commands_entity_selectors()));
        assert!(gamemaster.has_permission(&permissions::commands_entity_selectors()));
        assert!(!admin.has_permission(&permissions::chat_send_messages()));

        assert_eq!(
            level_based_permission_set(PermissionLevelModel::Owners)
                .union(level_based_permission_set(PermissionLevelModel::Gamemasters)),
            Ok(level_based_permission_set(PermissionLevelModel::Gamemasters))
        );
    }

    #[test]
    fn permission_checks_provider_checks_and_suppliers_delegate_like_java() {
        let supplier = PermissionSetSupplierModel::new(level_based_permission_set(
            PermissionLevelModel::Gamemasters,
        ));
        let require_gamemaster = PermissionCheckModel::Require(permissions::commands_gamemaster());
        let require_admin = PermissionCheckModel::Require(permissions::commands_admin());

        assert!(PermissionCheckModel::AlwaysPass.check(supplier.permissions()));
        assert!(require_gamemaster.check(supplier.permissions()));
        assert!(!require_admin.check(supplier.permissions()));
        assert!(PermissionProviderCheckModel::new(require_gamemaster).test(&supplier));
        assert!(!PermissionProviderCheckModel::new(require_admin).test(&supplier));
    }

    #[test]
    fn permission_set_union_flattens_unions_and_rejects_nested_unions_like_java() {
        let chat = PermissionSetModel::Explicit(vec![permissions::chat_send_messages()]);
        let commands = PermissionSetModel::Explicit(vec![permissions::commands_owner()]);
        let selectors = PermissionSetModel::Explicit(vec![permissions::commands_entity_selectors()]);

        let union = match chat.clone().union(commands.clone()) {
            Ok(union) => union,
            Err(error) => panic!("union should succeed: {error}"),
        };
        assert!(union.has_permission(&permissions::chat_send_messages()));
        assert!(union.has_permission(&permissions::commands_owner()));
        assert!(!union.has_permission(&permissions::commands_entity_selectors()));

        let extended = match union.clone().union(selectors.clone()) {
            Ok(union) => union,
            Err(error) => panic!("union extension should succeed: {error}"),
        };
        assert!(extended.has_permission(&permissions::commands_entity_selectors()));
        assert_eq!(extended.union_members().map(<[PermissionSetModel]>::len), Some(3));
        assert!(PermissionSetModel::union_from_sets(vec![union]).is_err());
    }

    #[test]
    fn built_in_permissions_and_bootstrap_orders_match_java() {
        assert_eq!(permissions::commands_moderator().codec_name(), "command_level");
        assert_eq!(permissions::commands_entity_selectors().codec_name(), "atom");
        assert_eq!(
            permissions::chat_permissions(),
            vec![
                permissions::chat_send_messages(),
                permissions::chat_send_commands(),
                permissions::chat_receive_player_messages(),
                permissions::chat_receive_system_messages(),
            ]
        );
        assert_eq!(permission_types_bootstrap_order(), vec!["atom", "command_level"]);
        assert_eq!(permission_check_types_bootstrap_order(), vec!["always_pass", "require"]);
        assert_eq!(PermissionCheckModel::AlwaysPass.codec_name(), "always_pass");
        assert_eq!(
            PermissionCheckModel::Require(permissions::commands_owner()).codec_name(),
            "require"
        );
    }

    #[test]
    fn permission_sources_match_java_26_1_2() {
        assert_contains_all(
            LEVEL_BASED_PERMISSION_SET_JAVA,
            &[
                "LevelBasedPermissionSet ALL = create(PermissionLevel.ALL);",
                "LevelBasedPermissionSet MODERATOR = create(PermissionLevel.MODERATORS);",
                "LevelBasedPermissionSet GAMEMASTER = create(PermissionLevel.GAMEMASTERS);",
                "LevelBasedPermissionSet ADMIN = create(PermissionLevel.ADMINS);",
                "LevelBasedPermissionSet OWNER = create(PermissionLevel.OWNERS);",
                "permission instanceof Permission.HasCommandLevel levelCheck",
                "permission.equals(Permissions.COMMANDS_ENTITY_SELECTORS)",
                "case OWNERS -> OWNER",
                "return \"permission level: \" + level.name();",
            ],
        );
        assert_contains_all(
            PERMISSION_JAVA,
            &[
                "Codec<Permission> FULL_CODEC = BuiltInRegistries.PERMISSION_TYPE.byNameCodec().dispatch",
                "Codec.either(FULL_CODEC, Identifier.CODEC)",
                "record Atom(Identifier id) implements Permission",
                "Identifier.withDefaultNamespace(name)",
                "record HasCommandLevel(PermissionLevel level) implements Permission",
            ],
        );
        assert_contains_all(
            PERMISSION_CHECK_JAVA,
            &[
                "class AlwaysPass implements PermissionCheck",
                "public static final PermissionCheck.AlwaysPass INSTANCE",
                "return true;",
                "record Require(Permission permission) implements PermissionCheck",
                "return source.hasPermission(this.permission);",
            ],
        );
        assert_contains_all(
            PERMISSION_LEVEL_JAVA,
            &[
                "ALL(\"all\", 0)",
                "MODERATORS(\"moderators\", 1)",
                "GAMEMASTERS(\"gamemasters\", 2)",
                "ADMINS(\"admins\", 3)",
                "OWNERS(\"owners\", 4)",
                "ByIdMap.OutOfBoundsStrategy.CLAMP",
                "return this.id >= other.id;",
                "return this.name;",
            ],
        );
    }

    #[test]
    fn permission_registry_sources_match_java_26_1_2() {
        assert_contains_all(
            PERMISSION_CHECK_TYPES_JAVA,
            &[
                "Identifier.withDefaultNamespace(\"always_pass\")",
                "PermissionCheck.AlwaysPass.MAP_CODEC",
                "Identifier.withDefaultNamespace(\"require\")",
                "PermissionCheck.Require.MAP_CODEC",
            ],
        );
        assert_contains_all(
            PERMISSION_TYPES_JAVA,
            &[
                "Identifier.withDefaultNamespace(\"atom\")",
                "Permission.Atom.MAP_CODEC",
                "Identifier.withDefaultNamespace(\"command_level\")",
                "Permission.HasCommandLevel.MAP_CODEC",
            ],
        );
        assert_contains_all(
            PERMISSION_PROVIDER_CHECK_JAVA,
            &["implements Predicate<T>", "return this.test.check(t.permissions());"],
        );
        assert_contains_all(PERMISSION_SET_SUPPLIER_JAVA, &["PermissionSet permissions();"]);
    }

    #[test]
    fn permission_set_and_constants_sources_match_java_26_1_2() {
        assert_contains_all(
            PERMISSION_SET_JAVA,
            &[
                "PermissionSet NO_PERMISSIONS = permission -> false;",
                "PermissionSet ALL_PERMISSIONS = permission -> true;",
                "boolean hasPermission(Permission permission);",
                "return other instanceof PermissionSetUnion ? other.union(this) : new PermissionSetUnion(this, other);",
            ],
        );
        assert_contains_all(
            PERMISSION_SET_UNION_JAVA,
            &[
                "private final ReferenceSet<PermissionSet> permissions = new ReferenceArraySet();",
                "if (set.hasPermission(permission))",
                "new PermissionSetUnion(this.permissions, otherUnion.permissions)",
                "throw new IllegalArgumentException(\"Cannot have PermissionSetUnion within another PermissionSetUnion\")",
            ],
        );
        assert_contains_all(
            PERMISSIONS_JAVA,
            &[
                "COMMANDS_MODERATOR = new Permission.HasCommandLevel(PermissionLevel.MODERATORS)",
                "COMMANDS_GAMEMASTER = new Permission.HasCommandLevel(PermissionLevel.GAMEMASTERS)",
                "COMMANDS_ADMIN = new Permission.HasCommandLevel(PermissionLevel.ADMINS)",
                "COMMANDS_OWNER = new Permission.HasCommandLevel(PermissionLevel.OWNERS)",
                "COMMANDS_ENTITY_SELECTORS = Permission.Atom.create(\"commands/entity_selectors\")",
                "CHAT_SEND_MESSAGES = Permission.Atom.create(\"chat/send_messages\")",
                "CHAT_SEND_COMMANDS = Permission.Atom.create(\"chat/send_commands\")",
                "CHAT_RECEIVE_PLAYER_MESSAGES = Permission.Atom.create(\"chat/receive_player_messages\")",
                "CHAT_RECEIVE_SYSTEM_MESSAGES = Permission.Atom.create(\"chat/receive_system_messages\")",
            ],
        );
    }

    fn assert_contains_all(source: &str, sentinels: &[&str]) {
        for sentinel in sentinels {
            assert!(source.contains(sentinel), "missing permission sentinel {sentinel}");
        }
    }
}
