use super::*;

impl SkullBlockEntity {
    pub fn new() -> Self {
        Self {
            profile: None,
            note_block_sound: None,
            custom_name: None,
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn save_additional(&self) -> Tag {
        let mut fields = Vec::new();
        if let Some(profile) = &self.profile {
            fields.push(("profile".to_string(), profile.clone()));
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            fields.push((
                "note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            ));
        }
        if let Some(custom_name) = &self.custom_name {
            fields.push(("custom_name".to_string(), Tag::String(custom_name.clone())));
        }
        Tag::Compound(fields)
    }

    pub fn load_additional(tag: &Tag) -> Self {
        let Tag::Compound(entries) = tag else {
            return Self::new();
        };
        Self {
            profile: entries
                .iter()
                .find(|(name, _)| name == "profile")
                .map(|(_, tag)| tag.clone()),
            note_block_sound: get_string(entries, "note_block_sound").map(ToString::to_string),
            custom_name: get_string(entries, "custom_name").map(ToString::to_string),
            animation_tick_count: 0,
            is_animating: false,
        }
    }

    pub fn apply_implicit_components(&mut self, components: &BTreeMap<String, Tag>) {
        self.profile = components.get("minecraft:profile").cloned();
        self.note_block_sound = components
            .get("minecraft:note_block_sound")
            .and_then(|tag| match tag {
                Tag::String(id) => Some(id.clone()),
                _ => None,
            });
        self.custom_name = components
            .get("minecraft:custom_name")
            .and_then(|tag| match tag {
                Tag::String(name) => Some(name.clone()),
                _ => None,
            });
    }

    pub fn collect_implicit_components(&self) -> BTreeMap<String, Tag> {
        let mut components = BTreeMap::new();
        if let Some(profile) = &self.profile {
            components.insert("minecraft:profile".to_string(), profile.clone());
        }
        if let Some(note_block_sound) = &self.note_block_sound {
            components.insert(
                "minecraft:note_block_sound".to_string(),
                Tag::String(note_block_sound.clone()),
            );
        }
        if let Some(custom_name) = &self.custom_name {
            components.insert(
                "minecraft:custom_name".to_string(),
                Tag::String(custom_name.clone()),
            );
        }
        components
    }

    pub fn remove_components_from_tag(tag: &mut Tag) {
        if let Tag::Compound(entries) = tag {
            entries.retain(|(name, _)| {
                name != "profile" && name != "note_block_sound" && name != "custom_name"
            });
        }
    }

    pub fn animation_tick(&mut self, powered: bool) {
        if powered {
            self.is_animating = true;
            self.animation_tick_count += 1;
        } else {
            self.is_animating = false;
        }
    }

    pub fn animation(&self, partial_tick: f32) -> f32 {
        if self.is_animating {
            self.animation_tick_count as f32 + partial_tick
        } else {
            self.animation_tick_count as f32
        }
    }

    pub fn get_update_tag(&self) -> Tag {
        self.save_additional()
    }
}
