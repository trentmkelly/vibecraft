use std::collections::BTreeMap;

use crate::registry::{feature_flags, FeatureFlagSet, Identifier};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSourceStackParity {
    pub source: CommandEndpoint,
    pub position: Vec3,
    pub rotation: Vec2,
    pub level: LevelModel,
    pub permissions: PermissionLevelModel,
    pub text_name: String,
    pub display_name: ComponentModel,
    pub server: ServerModel,
    pub entity: Option<EntityModel>,
    pub silent: bool,
    pub result_callback: CallbackModel,
    pub anchor: EntityAnchorModel,
    pub signing_context: &'static str,
    pub chat_message_chainer: &'static str,
}

impl CommandSourceStackParity {
    pub fn new(init: CommandSourceStackInit) -> Self {
        Self {
            source: init.source,
            position: init.position,
            rotation: init.rotation,
            level: init.level,
            permissions: init.permissions,
            text_name: init.text_name,
            display_name: init.display_name,
            server: init.server,
            entity: init.entity,
            silent: false,
            result_callback: CallbackModel::Empty,
            anchor: EntityAnchorModel::Feet,
            signing_context: "anonymous",
            chat_message_chainer: "immediate",
        }
    }

    pub fn with_source(&self, source: CommandEndpoint) -> Self {
        if self.source == source {
            self.clone()
        } else {
            Self {
                source,
                ..self.clone()
            }
        }
    }

    pub fn with_entity(&self, entity: EntityModel) -> Self {
        if self.entity.as_ref() == Some(&entity) {
            self.clone()
        } else {
            Self {
                text_name: entity.plain_text_name.clone(),
                display_name: entity.display_name.clone(),
                entity: Some(entity),
                ..self.clone()
            }
        }
    }

    pub fn with_position(&self, position: Vec3) -> Self {
        if self.position == position {
            self.clone()
        } else {
            Self {
                position,
                ..self.clone()
            }
        }
    }

    pub fn with_rotation(&self, rotation: Vec2) -> Self {
        if self.rotation == rotation {
            self.clone()
        } else {
            Self {
                rotation,
                ..self.clone()
            }
        }
    }

    pub fn with_callback(&self, result_callback: CallbackModel) -> Self {
        if self.result_callback == result_callback {
            self.clone()
        } else {
            Self {
                result_callback,
                ..self.clone()
            }
        }
    }

    pub fn with_callback_combined(
        &self,
        new_callback: CallbackModel,
        combiner: fn(CallbackModel, CallbackModel) -> CallbackModel,
    ) -> Self {
        self.with_callback(combiner(self.result_callback, new_callback))
    }

    pub fn with_suppressed_output(&self) -> Self {
        if !self.silent && !self.source.always_accepts {
            Self {
                silent: true,
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }

    pub fn with_permission(&self, permissions: PermissionLevelModel) -> Self {
        if self.permissions == permissions {
            self.clone()
        } else {
            Self {
                permissions,
                ..self.clone()
            }
        }
    }

    pub fn with_maximum_permission(&self, new_permissions: PermissionLevelModel) -> Self {
        self.with_permission(self.permissions.union(new_permissions))
    }

    pub fn with_anchor(&self, anchor: EntityAnchorModel) -> Self {
        if self.anchor == anchor {
            self.clone()
        } else {
            Self {
                anchor,
                ..self.clone()
            }
        }
    }

    pub fn with_level(&self, level: LevelModel) -> Self {
        if self.level.name == level.name {
            self.clone()
        } else {
            let scale = self.level.coordinate_scale / level.coordinate_scale;
            Self {
                position: Vec3::new(
                    self.position.x * scale,
                    self.position.y,
                    self.position.z * scale,
                ),
                level,
                ..self.clone()
            }
        }
    }

    pub fn facing_entity(&self, entity: &EntityModel, anchor: EntityAnchorModel) -> Self {
        self.facing(anchor.apply_entity(entity))
    }

    pub fn facing(&self, target: Vec3) -> Self {
        let from = self.anchor.apply_stack(self);
        let xd = target.x - from.x;
        let yd = target.y - from.y;
        let zd = target.z - from.z;
        let sd = (xd * xd + zd * zd).sqrt();
        let x_rot = wrap_degrees((-(yd.atan2(sd).to_degrees())) as f32);
        let y_rot = wrap_degrees((zd.atan2(xd).to_degrees() - 90.0) as f32);
        self.with_rotation(Vec2::new(x_rot, y_rot))
    }

    pub fn with_signing_context(
        &self,
        signing_context: &'static str,
        chat_message_chainer: &'static str,
    ) -> Self {
        if self.signing_context == signing_context
            && self.chat_message_chainer == chat_message_chainer
        {
            self.clone()
        } else {
            Self {
                signing_context,
                chat_message_chainer,
                ..self.clone()
            }
        }
    }

    pub fn get_entity_or_exception(&self) -> Result<&EntityModel, CommandStackError> {
        self.entity.as_ref().ok_or(CommandStackError::NotEntity)
    }

    pub fn get_player_or_exception(&self) -> Result<&EntityModel, CommandStackError> {
        self.get_player().ok_or(CommandStackError::NotPlayer)
    }

    pub fn get_player(&self) -> Option<&EntityModel> {
        self.entity
            .as_ref()
            .filter(|entity| entity.kind == EntityKind::ServerPlayer)
    }

    pub fn is_player(&self) -> bool {
        self.get_player().is_some()
    }

    pub fn should_filter_message_to(&self, receiver: &EntityModel) -> bool {
        if self.get_player() == Some(receiver) {
            false
        } else {
            self.get_player()
                .is_some_and(|player| player.text_filtering_enabled)
                || receiver.text_filtering_enabled
        }
    }

    pub fn send_chat_message(
        &self,
        message: ComponentModel,
        filtered: bool,
        chat_type: &'static str,
    ) -> Vec<Delivery> {
        if self.silent {
            return Vec::new();
        }

        if let Some(player) = self.get_player() {
            vec![Delivery::PlayerChat {
                player: player.plain_text_name.clone(),
                message,
                filtered,
                chat_type,
            }]
        } else {
            vec![Delivery::SourceSystem {
                source: self.source.id,
                message: ComponentModel::literal(format!("{chat_type}: {}", message.text)),
            }]
        }
    }

    pub fn send_system_message(&self, message: ComponentModel) -> Vec<Delivery> {
        if self.silent {
            return Vec::new();
        }

        if let Some(player) = self.get_player() {
            vec![Delivery::PlayerSystem {
                player: player.plain_text_name.clone(),
                message,
            }]
        } else {
            vec![Delivery::SourceSystem {
                source: self.source.id,
                message,
            }]
        }
    }

    pub fn send_success(
        &self,
        message_supplier: impl FnOnce() -> ComponentModel,
        broadcast: bool,
    ) -> (Vec<Delivery>, bool) {
        let should_send_system_message = self.source.accepts_success && !self.silent;
        let should_broadcast = broadcast && self.source.should_inform_admins && !self.silent;
        if !should_send_system_message && !should_broadcast {
            return (Vec::new(), false);
        }

        let message = message_supplier();
        let mut deliveries = Vec::new();
        if should_send_system_message {
            deliveries.push(Delivery::SourceSystem {
                source: self.source.id,
                message: message.clone(),
            });
        }
        if should_broadcast {
            deliveries.extend(self.broadcast_to_admins(message));
        }
        (deliveries, true)
    }

    fn broadcast_to_admins(&self, message: ComponentModel) -> Vec<Delivery> {
        let broadcast = ComponentModel::literal(format!(
            "chat.type.admin({}, {})[gray,italic]",
            self.display_name.text, message.text
        ));
        let mut deliveries = Vec::new();
        if self.level.send_command_feedback {
            for player in &self.server.players {
                if player.command_source_id != self.source.id && player.operator {
                    deliveries.push(Delivery::PlayerSystem {
                        player: player.name.clone(),
                        message: broadcast.clone(),
                    });
                }
            }
        }

        if self.source.id != self.server.source_id && self.level.log_admin_commands {
            deliveries.push(Delivery::ServerSystem { message: broadcast });
        }
        deliveries
    }

    pub fn send_failure(&self, message: ComponentModel) -> Vec<Delivery> {
        if self.source.accepts_failure && !self.silent {
            vec![Delivery::SourceSystem {
                source: self.source.id,
                message: ComponentModel::literal(format!("{}[red]", message.text)),
            }]
        } else {
            Vec::new()
        }
    }

    pub fn suggest_registry_elements(&self, key: &str) -> Vec<Identifier> {
        if key == "minecraft:recipe" {
            return self.server.recipes.clone();
        }
        if key == "minecraft:advancement" {
            return self.server.advancements.clone();
        }
        self.server
            .registry_access
            .get(key)
            .or_else(|| self.server.reloadable_registries.get(key))
            .cloned()
            .unwrap_or_default()
    }

    pub fn levels(&self) -> &[String] {
        &self.server.levels
    }

    pub fn enabled_features(&self) -> FeatureFlagSet {
        self.level.enabled_features
    }

    pub fn handle_error(
        &self,
        message: ComponentModel,
        forked: bool,
        tracer: Option<&mut Vec<String>>,
    ) -> Vec<Delivery> {
        if let Some(tracer) = tracer {
            tracer.push(message.text.clone());
        }
        if forked {
            Vec::new()
        } else {
            self.send_failure(message)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandSourceStackInit {
    pub source: CommandEndpoint,
    pub position: Vec3,
    pub rotation: Vec2,
    pub level: LevelModel,
    pub permissions: PermissionLevelModel,
    pub text_name: String,
    pub display_name: ComponentModel,
    pub server: ServerModel,
    pub entity: Option<EntityModel>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentModel {
    pub text: String,
}

impl ComponentModel {
    pub fn literal(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandEndpoint {
    pub id: &'static str,
    pub accepts_success: bool,
    pub accepts_failure: bool,
    pub should_inform_admins: bool,
    pub always_accepts: bool,
}

impl CommandEndpoint {
    pub fn new(id: &'static str) -> Self {
        Self {
            id,
            accepts_success: true,
            accepts_failure: true,
            should_inform_admins: true,
            always_accepts: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LevelModel {
    pub name: String,
    pub coordinate_scale: f64,
    pub send_command_feedback: bool,
    pub log_admin_commands: bool,
    pub enabled_features: FeatureFlagSet,
}

impl LevelModel {
    pub fn new(name: impl Into<String>, coordinate_scale: f64) -> Self {
        Self {
            name: name.into(),
            coordinate_scale,
            send_command_feedback: true,
            log_admin_commands: true,
            enabled_features: FeatureFlagSet::of(&[feature_flags::VANILLA]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerModel {
    pub source_id: &'static str,
    pub players: Vec<PlayerConnectionModel>,
    pub recipes: Vec<Identifier>,
    pub advancements: Vec<Identifier>,
    pub registry_access: BTreeMap<String, Vec<Identifier>>,
    pub reloadable_registries: BTreeMap<String, Vec<Identifier>>,
    pub levels: Vec<String>,
}

impl ServerModel {
    pub fn new() -> Self {
        Self {
            source_id: "server",
            players: Vec::new(),
            recipes: Vec::new(),
            advancements: Vec::new(),
            registry_access: BTreeMap::new(),
            reloadable_registries: BTreeMap::new(),
            levels: vec!["overworld".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerConnectionModel {
    pub name: String,
    pub command_source_id: &'static str,
    pub operator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevelModel {
    All = 0,
    Moderators = 1,
    GameMasters = 2,
    Admins = 3,
    Owners = 4,
}

impl PermissionLevelModel {
    pub fn union(self, other: Self) -> Self {
        self.max(other)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityModel {
    pub plain_text_name: String,
    pub display_name: ComponentModel,
    pub kind: EntityKind,
    pub position: Vec3,
    pub eye_height: f64,
    pub text_filtering_enabled: bool,
}

impl EntityModel {
    pub fn player(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            plain_text_name: name.clone(),
            display_name: ComponentModel::literal(name.clone()),
            kind: EntityKind::ServerPlayer,
            position: Vec3::new(0.0, 64.0, 0.0),
            eye_height: 1.62,
            text_filtering_enabled: false,
        }
    }

    pub fn entity(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            plain_text_name: name.clone(),
            display_name: ComponentModel::literal(name.clone()),
            kind: EntityKind::Other,
            position: Vec3::new(0.0, 64.0, 0.0),
            eye_height: 1.0,
            text_filtering_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    ServerPlayer,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackModel {
    Empty,
    StoreResult,
    ReturnFrame,
}

impl CallbackModel {
    pub fn chain(first: Self, second: Self) -> Self {
        match (first, second) {
            (Self::Empty, callback) | (callback, Self::Empty) => callback,
            (_first, second) => second,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchorModel {
    Feet,
    Eyes,
}

impl EntityAnchorModel {
    fn apply_entity(self, entity: &EntityModel) -> Vec3 {
        match self {
            Self::Feet => entity.position,
            Self::Eyes => Vec3::new(
                entity.position.x,
                entity.position.y + entity.eye_height,
                entity.position.z,
            ),
        }
    }

    fn apply_stack(self, stack: &CommandSourceStackParity) -> Vec3 {
        match (self, &stack.entity) {
            (Self::Eyes, Some(entity)) => Vec3::new(
                stack.position.x,
                stack.position.y + entity.eye_height,
                stack.position.z,
            ),
            _ => stack.position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    SourceSystem {
        source: &'static str,
        message: ComponentModel,
    },
    PlayerSystem {
        player: String,
        message: ComponentModel,
    },
    PlayerChat {
        player: String,
        message: ComponentModel,
        filtered: bool,
        chat_type: &'static str,
    },
    ServerSystem {
        message: ComponentModel,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStackError {
    NotPlayer,
    NotEntity,
}

fn wrap_degrees(mut degrees: f32) -> f32 {
    degrees %= 360.0;
    if degrees >= 180.0 {
        degrees -= 360.0;
    }
    if degrees < -180.0 {
        degrees += 360.0;
    }
    degrees
}

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_stack() -> CommandSourceStackParity {
        CommandSourceStackParity::new(CommandSourceStackInit {
            source: CommandEndpoint::new("console"),
            position: Vec3::new(8.0, 70.0, -4.0),
            rotation: Vec2::new(0.0, 0.0),
            level: LevelModel::new("overworld", 1.0),
            permissions: PermissionLevelModel::GameMasters,
            text_name: "Server".to_string(),
            display_name: ComponentModel::literal("Server"),
            server: ServerModel::new(),
            entity: None,
        })
    }

    #[test]
    fn constructor_sets_java_defaults_for_optional_state() {
        let stack = base_stack();

        assert!(!stack.silent);
        assert_eq!(stack.result_callback, CallbackModel::Empty);
        assert_eq!(stack.anchor, EntityAnchorModel::Feet);
        assert_eq!(stack.signing_context, "anonymous");
        assert_eq!(stack.chat_message_chainer, "immediate");
    }

    #[test]
    fn with_entity_updates_entity_and_display_identity() {
        let entity = EntityModel::player("Alex");
        let stack = base_stack().with_entity(entity.clone());

        assert_eq!(stack.entity, Some(entity));
        assert_eq!(stack.text_name, "Alex");
        assert_eq!(stack.display_name, ComponentModel::literal("Alex"));
        assert!(stack.is_player());
    }

    #[test]
    fn with_methods_preserve_other_fields_and_update_only_target_field() {
        let stack = base_stack()
            .with_source(CommandEndpoint {
                id: "command_block",
                ..CommandEndpoint::new("command_block")
            })
            .with_position(Vec3::new(1.0, 2.0, 3.0))
            .with_rotation(Vec2::new(4.0, 5.0))
            .with_anchor(EntityAnchorModel::Eyes)
            .with_signing_context("signed", "queued");

        assert_eq!(stack.source.id, "command_block");
        assert_eq!(stack.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(stack.rotation, Vec2::new(4.0, 5.0));
        assert_eq!(stack.anchor, EntityAnchorModel::Eyes);
        assert_eq!(stack.signing_context, "signed");
        assert_eq!(stack.chat_message_chainer, "queued");
    }

    #[test]
    fn suppressed_output_sets_silent_unless_source_always_accepts() {
        let suppressed = base_stack().with_suppressed_output();
        let always = base_stack().with_source(CommandEndpoint {
            always_accepts: true,
            ..CommandEndpoint::new("always")
        });

        assert!(suppressed.silent);
        assert!(!always.with_suppressed_output().silent);
    }

    #[test]
    fn maximum_permission_uses_highest_level() {
        let stack = base_stack().with_maximum_permission(PermissionLevelModel::Owners);

        assert_eq!(stack.permissions, PermissionLevelModel::Owners);
        assert_eq!(
            base_stack()
                .with_maximum_permission(PermissionLevelModel::All)
                .permissions,
            PermissionLevelModel::GameMasters
        );
        assert_eq!(
            base_stack()
                .with_permission(PermissionLevelModel::Moderators)
                .with_maximum_permission(PermissionLevelModel::Admins)
                .permissions,
            PermissionLevelModel::Admins
        );
    }

    #[test]
    fn with_level_scales_x_and_z_by_dimension_coordinate_scale_ratio() {
        let nether = LevelModel::new("nether", 8.0);
        let stack = base_stack().with_position(Vec3::new(80.0, 65.0, -40.0));

        assert_eq!(
            stack.with_level(nether).position,
            Vec3::new(10.0, 65.0, -5.0)
        );
    }

    #[test]
    fn facing_uses_anchor_position_and_java_rotation_formula() {
        let entity = EntityModel {
            eye_height: 1.0,
            ..EntityModel::player("Alex")
        };
        let stack = base_stack()
            .with_position(Vec3::new(0.0, 64.0, 0.0))
            .with_entity(entity)
            .with_anchor(EntityAnchorModel::Eyes)
            .facing(Vec3::new(0.0, 65.0, 10.0));

        assert_eq!(stack.rotation, Vec2::new(-0.0, 0.0));
        assert_eq!(
            base_stack().facing(Vec3::new(10.0, 70.0, -4.0)).rotation,
            Vec2::new(-0.0, -90.0)
        );

        let target = EntityModel {
            position: Vec3::new(8.0, 70.0, 6.0),
            ..EntityModel::entity("Target")
        };
        assert_eq!(
            base_stack()
                .facing_entity(&target, EntityAnchorModel::Feet)
                .rotation,
            Vec2::new(-0.0, 0.0)
        );
    }

    #[test]
    fn player_accessors_match_java_exception_paths() {
        let non_player = base_stack().with_entity(EntityModel::entity("Zombie"));
        let player = base_stack().with_entity(EntityModel::player("Alex"));

        assert_eq!(
            base_stack().get_entity_or_exception(),
            Err(CommandStackError::NotEntity)
        );
        assert_eq!(
            non_player.get_player_or_exception(),
            Err(CommandStackError::NotPlayer)
        );
        assert!(player.get_player_or_exception().is_ok());
    }

    #[test]
    fn should_filter_message_to_matches_player_and_receiver_filter_flags() {
        let mut sender = EntityModel::player("Alex");
        let receiver = EntityModel::player("Steve");
        let stack = base_stack().with_entity(sender.clone());

        assert!(!stack.should_filter_message_to(&sender));
        assert!(!stack.should_filter_message_to(&receiver));

        sender.text_filtering_enabled = true;
        assert!(base_stack()
            .with_entity(sender)
            .should_filter_message_to(&receiver));

        let mut filtered_receiver = EntityModel::player("Steve");
        filtered_receiver.text_filtering_enabled = true;
        assert!(stack.should_filter_message_to(&filtered_receiver));
    }

    #[test]
    fn system_and_chat_messages_route_to_player_or_source_and_respect_silent() {
        let source_delivery = base_stack().send_system_message(ComponentModel::literal("hello"));
        let player_stack = base_stack().with_entity(EntityModel::player("Alex"));
        let chat_delivery =
            player_stack.send_chat_message(ComponentModel::literal("hi"), true, "chat");

        assert_eq!(
            source_delivery,
            [Delivery::SourceSystem {
                source: "console",
                message: ComponentModel::literal("hello")
            }]
        );
        assert_eq!(
            chat_delivery,
            [Delivery::PlayerChat {
                player: "Alex".to_string(),
                message: ComponentModel::literal("hi"),
                filtered: true,
                chat_type: "chat"
            }]
        );
        assert!(base_stack()
            .with_suppressed_output()
            .send_system_message(ComponentModel::literal("hidden"))
            .is_empty());
    }

    #[test]
    fn send_success_lazily_builds_message_and_broadcasts_to_admins_and_server() {
        let mut server = ServerModel::new();
        server.players = vec![
            PlayerConnectionModel {
                name: "Op".to_string(),
                command_source_id: "op_source",
                operator: true,
            },
            PlayerConnectionModel {
                name: "Self".to_string(),
                command_source_id: "console",
                operator: true,
            },
            PlayerConnectionModel {
                name: "User".to_string(),
                command_source_id: "user",
                operator: false,
            },
        ];
        let stack = CommandSourceStackParity {
            server,
            ..base_stack()
        };

        let (deliveries, built) = stack.send_success(|| ComponentModel::literal("done"), true);

        assert!(built);
        assert_eq!(
            deliveries,
            [
                Delivery::SourceSystem {
                    source: "console",
                    message: ComponentModel::literal("done")
                },
                Delivery::PlayerSystem {
                    player: "Op".to_string(),
                    message: ComponentModel::literal("chat.type.admin(Server, done)[gray,italic]")
                },
                Delivery::ServerSystem {
                    message: ComponentModel::literal("chat.type.admin(Server, done)[gray,italic]")
                },
            ]
        );
    }

    #[test]
    fn send_success_does_not_build_message_when_no_delivery_is_allowed() {
        let stack = base_stack()
            .with_source(CommandEndpoint {
                accepts_success: false,
                should_inform_admins: false,
                ..CommandEndpoint::new("quiet")
            })
            .with_suppressed_output();

        let (deliveries, built) =
            stack.send_success(|| panic!("message supplier should not run"), true);

        assert!(!built);
        assert!(deliveries.is_empty());
    }

    #[test]
    fn send_failure_wraps_message_red_when_failure_feedback_is_accepted() {
        let deliveries = base_stack().send_failure(ComponentModel::literal("bad"));

        assert_eq!(
            deliveries,
            [Delivery::SourceSystem {
                source: "console",
                message: ComponentModel::literal("bad[red]")
            }]
        );
    }

    #[test]
    fn registry_suggestions_prefer_recipe_advancement_and_then_access_fallbacks() {
        let mut server = ServerModel::new();
        server.recipes = vec![id("minecraft:crafting_table")];
        server.advancements = vec![id("minecraft:story/root")];
        server
            .registry_access
            .insert("minecraft:item".to_string(), vec![id("minecraft:stick")]);
        server.reloadable_registries.insert(
            "minecraft:worldgen/biome".to_string(),
            vec![id("minecraft:plains")],
        );
        let stack = CommandSourceStackParity {
            server,
            ..base_stack()
        };

        assert_eq!(
            stack.suggest_registry_elements("minecraft:recipe"),
            [id("minecraft:crafting_table")]
        );
        assert_eq!(
            stack.suggest_registry_elements("minecraft:advancement"),
            [id("minecraft:story/root")]
        );
        assert_eq!(
            stack.suggest_registry_elements("minecraft:item"),
            [id("minecraft:stick")]
        );
        assert_eq!(
            stack.suggest_registry_elements("minecraft:worldgen/biome"),
            [id("minecraft:plains")]
        );
        assert!(stack
            .suggest_registry_elements("minecraft:missing")
            .is_empty());
    }

    #[test]
    fn levels_enabled_features_callback_and_error_handling_are_exposed() {
        let mut level = LevelModel::new("overworld", 1.0);
        level.enabled_features =
            FeatureFlagSet::of(&[feature_flags::VANILLA, feature_flags::REDSTONE_EXPERIMENTS]);
        let mut server = ServerModel::new();
        server.levels = vec!["overworld".to_string(), "nether".to_string()];
        let stack = CommandSourceStackParity {
            level,
            server,
            ..base_stack()
        }
        .with_callback(CallbackModel::ReturnFrame)
        .with_callback_combined(CallbackModel::StoreResult, CallbackModel::chain);
        let mut trace = Vec::new();

        assert_eq!(stack.levels(), ["overworld", "nether"]);
        assert_eq!(
            stack.enabled_features(),
            FeatureFlagSet::of(&[feature_flags::VANILLA, feature_flags::REDSTONE_EXPERIMENTS])
        );
        assert_eq!(stack.result_callback, CallbackModel::StoreResult);
        assert_eq!(
            stack.handle_error(ComponentModel::literal("failed"), false, Some(&mut trace)),
            [Delivery::SourceSystem {
                source: "console",
                message: ComponentModel::literal("failed[red]")
            }]
        );
        assert_eq!(trace, ["failed"]);
        assert!(stack
            .handle_error(ComponentModel::literal("forked"), true, None)
            .is_empty());
    }
}
