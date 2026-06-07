use super::*;

impl TestBlockMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "start" => Some(Self::Start),
            "log" => Some(Self::Log),
            "fail" => Some(Self::Fail),
            "accept" => Some(Self::Accept),
            _ => None,
        }
    }
}

impl Default for TestBlockEntityState {
    fn default() -> Self {
        Self {
            mode: TestBlockMode::Fail,
            message: String::new(),
            powered: false,
            triggered: false,
        }
    }
}

impl TestBlockEntityState {
    pub const SET_MODE_UPDATE_FLAGS: i32 = 2;

    pub fn save_additional(&self) -> Tag {
        Tag::Compound(vec![
            (
                "mode".to_string(),
                Tag::String(self.mode.as_str().to_string()),
            ),
            ("message".to_string(), Tag::String(self.message.clone())),
            ("powered".to_string(), Tag::Byte(i8::from(self.powered))),
        ])
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        Self {
            mode: entries
                .and_then(|entries| get_string(entries, "mode"))
                .and_then(TestBlockMode::from_str)
                .unwrap_or(TestBlockMode::Fail),
            message: entries
                .and_then(|entries| get_string(entries, "message"))
                .unwrap_or("")
                .to_string(),
            powered: entries
                .and_then(|entries| get_byte(entries, "powered"))
                .unwrap_or(0)
                != 0,
            triggered: false,
        }
    }

    pub fn reset(&mut self) {
        self.triggered = false;
        if self.mode == TestBlockMode::Start {
            self.powered = false;
        }
    }

    pub fn trigger(&mut self) {
        if self.mode == TestBlockMode::Start {
            self.powered = true;
        } else {
            self.triggered = true;
        }
    }

    pub fn set_mode_with_block_update(&mut self, mode: TestBlockMode) -> i32 {
        self.mode = mode;
        Self::SET_MODE_UPDATE_FLAGS
    }

    pub fn reset_updates_neighbors(&mut self, has_level: bool) -> bool {
        self.triggered = false;
        if self.mode == TestBlockMode::Start && has_level {
            self.powered = false;
            true
        } else {
            false
        }
    }

    pub fn trigger_updates_neighbors(&mut self, has_level: bool) -> bool {
        if self.mode == TestBlockMode::Start && has_level {
            self.powered = true;
            true
        } else {
            if self.mode != TestBlockMode::Start {
                self.triggered = true;
            }
            false
        }
    }

    pub fn should_log(&self) -> bool {
        !self.message.trim().is_empty()
    }
}

impl TestInstanceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cleared => "cleared",
            Self::Running => "running",
            Self::Finished => "finished",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "cleared" => Some(Self::Cleared),
            "running" => Some(Self::Running),
            "finished" => Some(Self::Finished),
            _ => None,
        }
    }
}

impl Default for TestInstanceBlockEntityData {
    fn default() -> Self {
        Self {
            test: None,
            size: (0, 0, 0),
            rotation: "none".to_string(),
            ignore_entities: false,
            status: TestInstanceStatus::Cleared,
            error_message: None,
        }
    }
}

impl TestInstanceBlockEntityData {
    pub fn with_status(&self, status: TestInstanceStatus) -> Self {
        Self {
            status,
            error_message: None,
            ..self.clone()
        }
    }

    pub fn with_error(&self, error: impl Into<String>) -> Self {
        Self {
            status: TestInstanceStatus::Finished,
            error_message: Some(error.into()),
            ..self.clone()
        }
    }

    pub(super) fn to_tag(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(test) = &self.test {
            fields.push(("test".to_string(), Tag::String(test.clone())));
        }
        fields.push((
            "size".to_string(),
            Tag::List(vec![
                Tag::Int(self.size.0),
                Tag::Int(self.size.1),
                Tag::Int(self.size.2),
            ]),
        ));
        fields.push(("rotation".to_string(), Tag::String(self.rotation.clone())));
        fields.push((
            "ignore_entities".to_string(),
            Tag::Byte(i8::from(self.ignore_entities)),
        ));
        fields.push((
            "status".to_string(),
            Tag::String(self.status.as_str().to_string()),
        ));
        if let Some(error_message) = &self.error_message {
            fields.push((
                "error_message".to_string(),
                Tag::String(error_message.clone()),
            ));
        }
        Tag::Compound(fields)
    }

    pub(super) fn from_tag(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        let size = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "size"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) if values.len() == 3 => Some((
                    tag_int_or_zero(&values[0]),
                    tag_int_or_zero(&values[1]),
                    tag_int_or_zero(&values[2]),
                )),
                _ => None,
            })
            .unwrap_or((0, 0, 0));
        Self {
            test: entries
                .and_then(|entries| get_string(entries, "test"))
                .map(ToString::to_string),
            size,
            rotation: entries
                .and_then(|entries| get_string(entries, "rotation"))
                .unwrap_or("none")
                .to_string(),
            ignore_entities: entries
                .and_then(|entries| get_byte(entries, "ignore_entities"))
                .unwrap_or(0)
                != 0,
            status: entries
                .and_then(|entries| get_string(entries, "status"))
                .and_then(TestInstanceStatus::from_str)
                .unwrap_or(TestInstanceStatus::Cleared),
            error_message: entries
                .and_then(|entries| get_string(entries, "error_message"))
                .map(ToString::to_string),
        }
    }
}

impl TestInstanceBlockEntityState {
    pub const STRUCTURE_OFFSET: BlockPos = BlockPos { x: 0, y: 1, z: 1 };
    pub const PLACE_UPDATE_FLAGS: i32 = 818;
    pub const BEAM_RUNNING: i32 = 0xFF80_8080u32 as i32;
    pub const BEAM_SUCCESS: i32 = 0xFF00_FF00u32 as i32;
    pub const BEAM_REQUIRED_FAILED: i32 = 0xFFFF_0000u32 as i32;
    pub const BEAM_OPTIONAL_FAILED: i32 = 0xFFFF_8000u32 as i32;

    pub fn save_additional(&self) -> Tag {
        let mut fields = vec![("data".to_string(), self.data.to_tag())];
        if !self.errors.is_empty() {
            fields.push((
                "errors".to_string(),
                Tag::List(
                    self.errors
                        .iter()
                        .map(TestInstanceErrorMarker::to_tag)
                        .collect(),
                ),
            ));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let entries = compound_entries(tag);
        let data = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "data"))
            .map(|(_, tag)| TestInstanceBlockEntityData::from_tag(tag))
            .unwrap_or_default();
        let errors = entries
            .and_then(|entries| entries.iter().find(|(name, _)| name == "errors"))
            .and_then(|(_, tag)| match tag {
                Tag::List(values) => Some(
                    values
                        .iter()
                        .filter_map(TestInstanceErrorMarker::from_tag)
                        .collect(),
                ),
                _ => None,
            })
            .unwrap_or_default();
        Self { data, errors }
    }

    pub fn set_running(&mut self) {
        self.data = self.data.with_status(TestInstanceStatus::Running);
    }

    pub fn set_success(&mut self) {
        self.data = self.data.with_status(TestInstanceStatus::Finished);
    }

    pub fn set_error_message(&mut self, message: impl Into<String>) {
        self.data = self.data.with_error(message);
    }

    pub fn mark_error(&mut self, pos: BlockPos, text: impl Into<String>) {
        self.errors.push(TestInstanceErrorMarker {
            pos,
            text: text.into(),
        });
    }

    pub fn clear_error_markers(&mut self) {
        self.errors.clear();
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }

    pub fn render_mode(&self) -> StructureRenderMode {
        StructureRenderMode::Box
    }

    pub fn beam_sections(&self, required: bool) -> Vec<i32> {
        match self.data.status {
            TestInstanceStatus::Cleared => Vec::new(),
            TestInstanceStatus::Running => vec![Self::BEAM_RUNNING],
            TestInstanceStatus::Finished if self.data.error_message.is_none() => {
                vec![Self::BEAM_SUCCESS]
            }
            TestInstanceStatus::Finished if required => vec![Self::BEAM_REQUIRED_FAILED],
            TestInstanceStatus::Finished => vec![Self::BEAM_OPTIONAL_FAILED],
        }
    }

    pub fn structure_pos(origin: BlockPos, padding: i32) -> BlockPos {
        BlockPos {
            x: origin.x + padding + Self::STRUCTURE_OFFSET.x,
            y: origin.y + padding + Self::STRUCTURE_OFFSET.y,
            z: origin.z + padding + Self::STRUCTURE_OFFSET.z,
        }
    }

    pub fn transformed_size(&self, resolved_rotation: &str) -> (i32, i32, i32) {
        if matches!(resolved_rotation, "clockwise_90" | "counterclockwise_90") {
            (self.data.size.2, self.data.size.1, self.data.size.0)
        } else {
            self.data.size
        }
    }

    pub fn start_corner(&self, origin: BlockPos, resolved_rotation: &str, padding: i32) -> BlockPos {
        let pos = Self::structure_pos(origin, padding);
        match resolved_rotation {
            "clockwise_90" => BlockPos {
                x: pos.x + self.data.size.2 - 1,
                ..pos
            },
            "clockwise_180" => BlockPos {
                x: pos.x + self.data.size.0 - 1,
                z: pos.z + self.data.size.2 - 1,
                ..pos
            },
            "counterclockwise_90" => BlockPos {
                z: pos.z + self.data.size.0 - 1,
                ..pos
            },
            _ => pos,
        }
    }

    pub fn renderable_box(&self, padding: i32, resolved_rotation: &str) -> StructureRenderableBox {
        let min = BlockPos {
            x: Self::STRUCTURE_OFFSET.x + padding,
            y: Self::STRUCTURE_OFFSET.y + padding,
            z: Self::STRUCTURE_OFFSET.z + padding,
        };
        let size = self.transformed_size(resolved_rotation);
        StructureRenderableBox {
            min,
            max: BlockPos {
                x: min.x + size.0,
                y: min.y + size.1,
                z: min.z + size.2,
            },
        }
    }

    // TODO(test-instance-world-ops): wire reset/save/export/run/place/barrier
    // operations through GameTestRunner, ServerLevel, and StructureTemplateManager.
    pub fn world_operations_supported(&self) -> bool {
        false
    }
}

impl TestInstanceErrorMarker {
    pub(super) fn to_tag(&self) -> Tag {
        Tag::Compound(vec![
            (
                "pos".to_string(),
                Tag::List(vec![
                    Tag::Int(self.pos.x),
                    Tag::Int(self.pos.y),
                    Tag::Int(self.pos.z),
                ]),
            ),
            ("text".to_string(), Tag::String(self.text.clone())),
        ])
    }

    pub(super) fn from_tag(tag: &Tag) -> Option<Self> {
        let entries = compound_entries(tag)?;
        let pos =
            entries
                .iter()
                .find(|(name, _)| name == "pos")
                .and_then(|(_, tag)| match tag {
                    Tag::List(values) if values.len() == 3 => Some(BlockPos {
                        x: tag_int_or_zero(&values[0]),
                        y: tag_int_or_zero(&values[1]),
                        z: tag_int_or_zero(&values[2]),
                    }),
                    _ => None,
                })?;
        let text = get_string(entries, "text")?.to_string();
        Some(Self { pos, text })
    }
}
