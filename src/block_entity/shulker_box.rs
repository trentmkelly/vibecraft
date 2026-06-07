use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBoxBlockEvent {
    pub action: i32,
    pub open_count: i32,
    pub first_open_or_last_close: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShulkerBoxCollisionMove {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

impl ContainerBlockEntityModel {
    pub const SHULKER_COLUMNS: usize = 9;
    pub const SHULKER_ROWS: usize = 3;
    pub const SHULKER_CONTAINER_SIZE: usize = 27;
    pub const SHULKER_EVENT_SET_OPEN_COUNT: i32 = 1;

    pub fn trigger_shulker_event(&mut self, action: i32, open_count: i32) -> bool {
        if self.kind != ContainerBlockEntityKind::ShulkerBox
            || action != Self::SHULKER_EVENT_SET_OPEN_COUNT
        {
            return false;
        }
        self.viewer_count = open_count;
        match open_count {
            0 => self.shulker_status = ShulkerBoxAnimationStatus::Closing,
            1 => self.shulker_status = ShulkerBoxAnimationStatus::Opening,
            _ => {}
        }
        true
    }

    pub fn shulker_start_open_effects(
        &mut self,
        removed: bool,
        spectator: bool,
    ) -> Option<ShulkerBoxBlockEvent> {
        if self.kind != ContainerBlockEntityKind::ShulkerBox || removed || spectator {
            return None;
        }
        if self.viewer_count < 0 {
            self.viewer_count = 0;
        }
        self.viewer_count += 1;
        if self.viewer_count == 1 {
            self.shulker_status = ShulkerBoxAnimationStatus::Opening;
        }
        Some(ShulkerBoxBlockEvent {
            action: Self::SHULKER_EVENT_SET_OPEN_COUNT,
            open_count: self.viewer_count,
            first_open_or_last_close: self.viewer_count == 1,
        })
    }

    pub fn shulker_stop_open_effects(
        &mut self,
        removed: bool,
        spectator: bool,
    ) -> Option<ShulkerBoxBlockEvent> {
        if self.kind != ContainerBlockEntityKind::ShulkerBox || removed || spectator {
            return None;
        }
        self.viewer_count -= 1;
        if self.viewer_count <= 0 {
            self.shulker_status = ShulkerBoxAnimationStatus::Closing;
        }
        Some(ShulkerBoxBlockEvent {
            action: Self::SHULKER_EVENT_SET_OPEN_COUNT,
            open_count: self.viewer_count,
            first_open_or_last_close: self.viewer_count <= 0,
        })
    }

    pub fn shulker_slots_for_face(&self) -> Vec<usize> {
        if self.kind == ContainerBlockEntityKind::ShulkerBox {
            (0..Self::SHULKER_CONTAINER_SIZE).collect()
        } else {
            Vec::new()
        }
    }

    pub fn shulker_can_take_through_face(&self) -> bool {
        self.kind == ContainerBlockEntityKind::ShulkerBox
    }

    pub fn shulker_forces_solid_collision(&self) -> bool {
        self.shulker_is_closed()
    }

    pub fn shulker_pre_remove_has_side_effects(&self) -> bool {
        false
    }

    pub fn shulker_progress(&self, partial_tick: f32) -> f32 {
        self.shulker_progress_old + (self.lid_progress - self.shulker_progress_old) * partial_tick
    }

    pub fn shulker_bounding_lid_height(&self, partial_tick: f32) -> f32 {
        Self::SHULKER_MAX_LID_HEIGHT * self.shulker_progress(partial_tick)
    }

    pub fn shulker_collision_move_delta(
        &self,
        facing: Direction,
        previous_progress: f32,
        progress: f32,
    ) -> Option<ShulkerBoxCollisionMove> {
        if self.kind != ContainerBlockEntityKind::ShulkerBox || progress <= previous_progress {
            return None;
        }
        let delta = f64::from(Self::SHULKER_MAX_LID_HEIGHT * (progress - previous_progress)) + 0.01;
        let (dx, dy, dz) = match facing {
            Direction::West => (-delta, 0.0, 0.0),
            Direction::East => (delta, 0.0, 0.0),
            Direction::Down => (0.0, -delta, 0.0),
            Direction::Up => (0.0, delta, 0.0),
            Direction::North => (0.0, 0.0, -delta),
            Direction::South => (0.0, 0.0, delta),
        };
        Some(ShulkerBoxCollisionMove { dx, dy, dz })
    }

    pub fn shulker_color_from_block_id(block_id: &str) -> Option<DyeColor> {
        let color = block_id
            .strip_prefix("minecraft:")
            .unwrap_or(block_id)
            .strip_suffix("_shulker_box")?;
        match color {
            "white" => Some(DyeColor::White),
            "orange" => Some(DyeColor::Orange),
            "magenta" => Some(DyeColor::Magenta),
            "light_blue" => Some(DyeColor::LightBlue),
            "yellow" => Some(DyeColor::Yellow),
            "lime" => Some(DyeColor::Lime),
            "pink" => Some(DyeColor::Pink),
            "gray" => Some(DyeColor::Gray),
            "light_gray" => Some(DyeColor::LightGray),
            "cyan" => Some(DyeColor::Cyan),
            "purple" => Some(DyeColor::Purple),
            "blue" => Some(DyeColor::Blue),
            "brown" => Some(DyeColor::Brown),
            "green" => Some(DyeColor::Green),
            "red" => Some(DyeColor::Red),
            "black" => Some(DyeColor::Black),
            _ => None,
        }
    }
}
