#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Medium {
    Air,
    Water,
    Lava,
    BubbleColumn { drag_down: bool },
    PowderSnow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactBlock {
    None,
    Honey,
    Slime,
    Web,
    Ladder,
    Vine,
    Scaffolding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoseIntent {
    Standing,
    Sneaking,
    Sprinting,
    Swimming,
    Crawling,
    FallFlying,
    Riding(RideKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RideKind {
    Boat,
    Minecart,
    LivingMount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalContact {
    None,
    Nether,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementInput {
    pub forward: f64,
    pub sideways: f64,
    pub jumping: bool,
    pub sneaking: bool,
    pub sprinting: bool,
}

impl MovementInput {
    pub const fn idle() -> Self {
        Self {
            forward: 0.0,
            sideways: 0.0,
            jumping: false,
            sneaking: false,
            sprinting: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementContext {
    pub medium: Medium,
    pub contact: ContactBlock,
    pub intent: PoseIntent,
    pub input: MovementInput,
    pub on_ground: bool,
    pub horizontal_collision: bool,
    pub has_elytra: bool,
    pub elytra_usable: bool,
    pub can_stand_on_powder_snow: bool,
    pub portal: PortalContact,
    pub portal_cooldown: i32,
    pub step_height_attribute: f64,
}

impl Default for MovementContext {
    fn default() -> Self {
        Self {
            medium: Medium::Air,
            contact: ContactBlock::None,
            intent: PoseIntent::Standing,
            input: MovementInput::idle(),
            on_ground: true,
            horizontal_collision: false,
            has_elytra: false,
            elytra_usable: false,
            can_stand_on_powder_snow: false,
            portal: PortalContact::None,
            portal_cooldown: 0,
            step_height_attribute: 0.6,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovementPlan {
    pub horizontal_multiplier: f64,
    pub vertical_multiplier: f64,
    pub gravity: f64,
    pub jump_velocity: Option<f64>,
    pub fluid_push: Option<f64>,
    pub pose: PoseIntent,
    pub step_height: f64,
    pub climbing: bool,
    pub riding: Option<RideKind>,
    pub portal_action: Option<PortalAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortalAction {
    CooldownTick,
    EnterNetherPortal,
    EnterEndPortal,
}

pub fn plan_movement(ctx: MovementContext) -> MovementPlan {
    let mut horizontal = base_horizontal_multiplier(ctx);
    let mut vertical = base_vertical_multiplier(ctx);
    let mut gravity = match ctx.medium {
        Medium::Water => 0.02,
        Medium::Lava => 0.02,
        _ => 0.08,
    };
    let climbing = is_climbable(ctx.contact)
        || (matches!(ctx.medium, Medium::PowderSnow) && ctx.can_stand_on_powder_snow);

    if ctx.input.sneaking || matches!(ctx.intent, PoseIntent::Sneaking) {
        horizontal *= 0.3;
    }
    if ctx.input.sprinting || matches!(ctx.intent, PoseIntent::Sprinting) {
        horizontal *= 1.3;
    }
    if matches!(ctx.intent, PoseIntent::Swimming) && matches!(ctx.medium, Medium::Water) {
        horizontal *= 1.3;
    }
    if matches!(ctx.intent, PoseIntent::Crawling) {
        horizontal *= 0.4;
    }
    if matches!(ctx.intent, PoseIntent::FallFlying) && ctx.has_elytra && ctx.elytra_usable {
        gravity = 0.01;
        vertical = vertical.max(0.98);
        horizontal = horizontal.max(0.99);
    }

    let fluid_push = match ctx.medium {
        Medium::Water => Some(0.014),
        Medium::Lava => Some(0.007),
        Medium::BubbleColumn { drag_down } => Some(if drag_down { -0.03 } else { 0.06 }),
        _ => None,
    };

    let jump_velocity = if ctx.input.jumping {
        jump_velocity(ctx.medium, ctx.contact, climbing, ctx.on_ground)
    } else {
        None
    };

    MovementPlan {
        horizontal_multiplier: horizontal,
        vertical_multiplier: vertical,
        gravity,
        jump_velocity,
        fluid_push,
        pose: resolved_pose(ctx),
        step_height: step_height(ctx),
        climbing,
        riding: match ctx.intent {
            PoseIntent::Riding(kind) => Some(kind),
            _ => None,
        },
        portal_action: portal_action(ctx.portal, ctx.portal_cooldown),
    }
}

pub fn block_speed_multiplier(contact: ContactBlock) -> (f64, f64, f64) {
    match contact {
        ContactBlock::Honey => (0.4, 0.05, 0.4),
        ContactBlock::Web => (0.25, 0.05, 0.25),
        ContactBlock::Scaffolding => (0.5, 0.15, 0.5),
        _ => (1.0, 1.0, 1.0),
    }
}

pub fn bounce_velocity(
    contact: ContactBlock,
    vertical_velocity: f64,
    bypasses_landing: bool,
) -> f64 {
    if contact == ContactBlock::Slime && vertical_velocity < 0.0 && !bypasses_landing {
        -vertical_velocity
    } else {
        vertical_velocity
    }
}

pub fn boat_paddle_acceleration(left: bool, right: bool) -> (f64, f64) {
    match (left, right) {
        (true, true) => (0.0, 0.04),
        (true, false) => (-1.0, 0.005),
        (false, true) => (1.0, 0.005),
        (false, false) => (0.0, 0.0),
    }
}

pub fn minecart_rail_speed_multiplier(powered: bool, ascending: bool) -> f64 {
    match (powered, ascending) {
        (true, true) => 0.06,
        (true, false) => 0.04,
        (false, true) => 0.0078125,
        (false, false) => 0.0,
    }
}

fn base_horizontal_multiplier(ctx: MovementContext) -> f64 {
    let (x, _, z) = block_speed_multiplier(ctx.contact);
    let block = x.min(z);
    let medium = match ctx.medium {
        Medium::Water => 0.8,
        Medium::Lava => 0.5,
        Medium::BubbleColumn { .. } => 0.8,
        Medium::PowderSnow => 0.9,
        Medium::Air => 1.0,
    };
    block * medium
}

fn base_vertical_multiplier(ctx: MovementContext) -> f64 {
    let (_, y, _) = block_speed_multiplier(ctx.contact);
    let medium = match ctx.medium {
        Medium::Water | Medium::BubbleColumn { .. } => 0.8,
        Medium::Lava => 0.5,
        Medium::PowderSnow => 1.5,
        Medium::Air => 1.0,
    };
    y * medium
}

fn is_climbable(contact: ContactBlock) -> bool {
    matches!(
        contact,
        ContactBlock::Ladder | ContactBlock::Vine | ContactBlock::Scaffolding
    )
}

fn jump_velocity(
    medium: Medium,
    contact: ContactBlock,
    climbing: bool,
    on_ground: bool,
) -> Option<f64> {
    if matches!(medium, Medium::Water | Medium::BubbleColumn { .. }) {
        Some(0.04)
    } else if medium == Medium::Lava {
        Some(0.04)
    } else if climbing {
        Some(0.2)
    } else if contact == ContactBlock::Slime && on_ground {
        Some(0.42)
    } else if on_ground {
        Some(0.42)
    } else {
        None
    }
}

fn resolved_pose(ctx: MovementContext) -> PoseIntent {
    match ctx.intent {
        PoseIntent::FallFlying if !(ctx.has_elytra && ctx.elytra_usable) => PoseIntent::Standing,
        PoseIntent::Swimming if !matches!(ctx.medium, Medium::Water) => PoseIntent::Crawling,
        PoseIntent::Sneaking if matches!(ctx.medium, Medium::Water) => PoseIntent::Swimming,
        other => other,
    }
}

fn step_height(ctx: MovementContext) -> f64 {
    if matches!(ctx.intent, PoseIntent::Sneaking | PoseIntent::Crawling)
        || matches!(ctx.medium, Medium::PowderSnow)
    {
        0.0
    } else {
        ctx.step_height_attribute.max(0.0)
    }
}

fn portal_action(portal: PortalContact, cooldown: i32) -> Option<PortalAction> {
    if cooldown > 0 {
        return Some(PortalAction::CooldownTick);
    }
    match portal {
        PortalContact::None => None,
        PortalContact::Nether => Some(PortalAction::EnterNetherPortal),
        PortalContact::End => Some(PortalAction::EnterEndPortal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fluid_mediums_apply_vanilla_drag_jump_and_bubble_push_rules() {
        let mut ctx = MovementContext {
            medium: Medium::Water,
            input: MovementInput {
                jumping: true,
                ..MovementInput::idle()
            },
            ..MovementContext::default()
        };
        let water = plan_movement(ctx);
        assert_eq!(water.horizontal_multiplier, 0.8);
        assert_eq!(water.vertical_multiplier, 0.8);
        assert_eq!(water.jump_velocity, Some(0.04));
        assert_eq!(water.fluid_push, Some(0.014));

        ctx.medium = Medium::Lava;
        let lava = plan_movement(ctx);
        assert_eq!(lava.horizontal_multiplier, 0.5);
        assert_eq!(lava.vertical_multiplier, 0.5);
        assert_eq!(lava.fluid_push, Some(0.007));

        ctx.medium = Medium::BubbleColumn { drag_down: false };
        assert_eq!(plan_movement(ctx).fluid_push, Some(0.06));
        ctx.medium = Medium::BubbleColumn { drag_down: true };
        assert_eq!(plan_movement(ctx).fluid_push, Some(-0.03));
    }

    #[test]
    fn special_blocks_apply_stuck_climb_bounce_and_powder_snow_rules() {
        let honey = plan_movement(MovementContext {
            contact: ContactBlock::Honey,
            ..MovementContext::default()
        });
        assert_eq!(honey.horizontal_multiplier, 0.4);
        assert_eq!(honey.vertical_multiplier, 0.05);

        assert_eq!(bounce_velocity(ContactBlock::Slime, -0.6, false), 0.6);
        assert_eq!(bounce_velocity(ContactBlock::Slime, -0.6, true), -0.6);

        let ladder = plan_movement(MovementContext {
            contact: ContactBlock::Ladder,
            input: MovementInput {
                jumping: true,
                ..MovementInput::idle()
            },
            on_ground: false,
            ..MovementContext::default()
        });
        assert!(ladder.climbing);
        assert_eq!(ladder.jump_velocity, Some(0.2));

        assert!(
            plan_movement(MovementContext {
                contact: ContactBlock::Vine,
                ..MovementContext::default()
            })
            .climbing
        );
        assert!(
            plan_movement(MovementContext {
                contact: ContactBlock::Scaffolding,
                ..MovementContext::default()
            })
            .climbing
        );
        assert_eq!(
            block_speed_multiplier(ContactBlock::Web),
            (0.25, 0.05, 0.25)
        );

        let powder = plan_movement(MovementContext {
            medium: Medium::PowderSnow,
            can_stand_on_powder_snow: true,
            ..MovementContext::default()
        });
        assert!(powder.climbing);
        assert_eq!(powder.step_height, 0.0);
    }

    #[test]
    fn pose_resolution_covers_sneak_sprint_swim_crawl_elytra_and_step_height() {
        let sneaking = plan_movement(MovementContext {
            intent: PoseIntent::Sneaking,
            step_height_attribute: 0.6,
            ..MovementContext::default()
        });
        assert_eq!(sneaking.horizontal_multiplier, 0.3);
        assert_eq!(sneaking.step_height, 0.0);

        let sprinting = plan_movement(MovementContext {
            intent: PoseIntent::Sprinting,
            ..MovementContext::default()
        });
        assert_eq!(sprinting.horizontal_multiplier, 1.3);

        let swimming = plan_movement(MovementContext {
            medium: Medium::Water,
            intent: PoseIntent::Swimming,
            ..MovementContext::default()
        });
        assert_eq!(swimming.pose, PoseIntent::Swimming);
        assert_eq!(swimming.horizontal_multiplier, 1.04);

        let crawl = plan_movement(MovementContext {
            intent: PoseIntent::Swimming,
            medium: Medium::Air,
            ..MovementContext::default()
        });
        assert_eq!(crawl.pose, PoseIntent::Crawling);

        let elytra = plan_movement(MovementContext {
            intent: PoseIntent::FallFlying,
            has_elytra: true,
            elytra_usable: true,
            on_ground: false,
            ..MovementContext::default()
        });
        assert_eq!(elytra.gravity, 0.01);
        assert_eq!(elytra.horizontal_multiplier, 1.0);
    }

    #[test]
    fn riding_and_portal_rules_surface_boats_minecarts_mounts_and_cooldowns() {
        assert_eq!(boat_paddle_acceleration(true, true), (0.0, 0.04));
        assert_eq!(boat_paddle_acceleration(true, false), (-1.0, 0.005));
        assert_eq!(minecart_rail_speed_multiplier(true, true), 0.06);
        assert_eq!(minecart_rail_speed_multiplier(false, true), 0.0078125);

        let riding = plan_movement(MovementContext {
            intent: PoseIntent::Riding(RideKind::Minecart),
            ..MovementContext::default()
        });
        assert_eq!(riding.riding, Some(RideKind::Minecart));
        assert_eq!(
            plan_movement(MovementContext {
                intent: PoseIntent::Riding(RideKind::Boat),
                ..MovementContext::default()
            })
            .riding,
            Some(RideKind::Boat)
        );
        assert_eq!(
            plan_movement(MovementContext {
                intent: PoseIntent::Riding(RideKind::LivingMount),
                ..MovementContext::default()
            })
            .riding,
            Some(RideKind::LivingMount)
        );

        assert_eq!(
            plan_movement(MovementContext {
                portal: PortalContact::Nether,
                ..MovementContext::default()
            })
            .portal_action,
            Some(PortalAction::EnterNetherPortal)
        );
        assert_eq!(
            plan_movement(MovementContext {
                portal: PortalContact::End,
                ..MovementContext::default()
            })
            .portal_action,
            Some(PortalAction::EnterEndPortal)
        );
        assert_eq!(
            plan_movement(MovementContext {
                portal: PortalContact::End,
                portal_cooldown: 4,
                ..MovementContext::default()
            })
            .portal_action,
            Some(PortalAction::CooldownTick)
        );
    }
}
