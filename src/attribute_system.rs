#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeSentiment {
    Positive,
    Neutral,
    Negative,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttributeDef {
    pub id: &'static str,
    pub description_id: &'static str,
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub syncable: bool,
    pub sentiment: AttributeSentiment,
}

const fn attr(
    id: &'static str,
    default_value: f64,
    min_value: f64,
    max_value: f64,
    syncable: bool,
) -> AttributeDef {
    AttributeDef {
        id,
        description_id: id,
        default_value,
        min_value,
        max_value,
        syncable,
        sentiment: AttributeSentiment::Positive,
    }
}

const fn sentiment(mut base: AttributeDef, sentiment: AttributeSentiment) -> AttributeDef {
    base.sentiment = sentiment;
    base
}

pub const ATTRIBUTES: &[AttributeDef] = &[
    attr("minecraft:armor", 0.0, 0.0, 30.0, true),
    attr("minecraft:armor_toughness", 0.0, 0.0, 20.0, true),
    attr("minecraft:attack_damage", 2.0, 0.0, 2048.0, false),
    attr("minecraft:attack_knockback", 0.0, 0.0, 5.0, false),
    attr("minecraft:attack_speed", 4.0, 0.0, 1024.0, true),
    attr("minecraft:block_break_speed", 1.0, 0.0, 1024.0, true),
    attr("minecraft:block_interaction_range", 4.5, 0.0, 64.0, true),
    sentiment(
        attr("minecraft:burning_time", 1.0, 0.0, 1024.0, true),
        AttributeSentiment::Negative,
    ),
    attr("minecraft:camera_distance", 4.0, 0.0, 32.0, true),
    attr(
        "minecraft:explosion_knockback_resistance",
        0.0,
        0.0,
        1.0,
        true,
    ),
    attr("minecraft:entity_interaction_range", 3.0, 0.0, 64.0, true),
    sentiment(
        attr("minecraft:fall_damage_multiplier", 1.0, 0.0, 100.0, true),
        AttributeSentiment::Negative,
    ),
    attr("minecraft:flying_speed", 0.4, 0.0, 1024.0, true),
    attr("minecraft:follow_range", 32.0, 0.0, 2048.0, false),
    sentiment(
        attr("minecraft:gravity", 0.08, -1.0, 1.0, true),
        AttributeSentiment::Neutral,
    ),
    attr("minecraft:jump_strength", 0.42, 0.0, 32.0, true),
    attr("minecraft:knockback_resistance", 0.0, 0.0, 1.0, false),
    attr("minecraft:luck", 0.0, -1024.0, 1024.0, true),
    attr("minecraft:max_absorption", 0.0, 0.0, 2048.0, true),
    attr("minecraft:max_health", 20.0, 1.0, 1024.0, true),
    attr("minecraft:mining_efficiency", 0.0, 0.0, 1024.0, true),
    attr("minecraft:movement_efficiency", 0.0, 0.0, 1.0, true),
    attr("minecraft:movement_speed", 0.7, 0.0, 1024.0, true),
    attr("minecraft:oxygen_bonus", 0.0, 0.0, 1024.0, true),
    attr("minecraft:safe_fall_distance", 3.0, -1024.0, 1024.0, true),
    sentiment(
        attr("minecraft:scale", 1.0, 0.0625, 16.0, true),
        AttributeSentiment::Neutral,
    ),
    attr("minecraft:sneaking_speed", 0.3, 0.0, 1.0, true),
    attr("minecraft:spawn_reinforcements", 0.0, 0.0, 1.0, false),
    attr("minecraft:step_height", 0.6, 0.0, 10.0, true),
    attr("minecraft:submerged_mining_speed", 0.2, 0.0, 20.0, true),
    attr("minecraft:sweeping_damage_ratio", 0.0, 0.0, 1.0, true),
    attr("minecraft:tempt_range", 10.0, 0.0, 2048.0, false),
    attr("minecraft:water_movement_efficiency", 0.0, 0.0, 1.0, true),
    sentiment(
        attr(
            "minecraft:waypoint_transmit_range",
            0.0,
            0.0,
            60_000_000.0,
            false,
        ),
        AttributeSentiment::Neutral,
    ),
    sentiment(
        attr(
            "minecraft:waypoint_receive_range",
            0.0,
            0.0,
            60_000_000.0,
            false,
        ),
        AttributeSentiment::Neutral,
    ),
];

pub fn attribute(id: &str) -> Option<&'static AttributeDef> {
    let id = if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    };
    ATTRIBUTES.iter().find(|attribute| attribute.id == id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierOperation {
    AddValue,
    AddMultipliedBase,
    AddMultipliedTotal,
}

impl ModifierOperation {
    pub fn id(self) -> i32 {
        match self {
            Self::AddValue => 0,
            Self::AddMultipliedBase => 1,
            Self::AddMultipliedTotal => 2,
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::AddValue => "add_value",
            Self::AddMultipliedBase => "add_multiplied_base",
            Self::AddMultipliedTotal => "add_multiplied_total",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeModifier {
    pub id: String,
    pub amount: f64,
    pub operation: ModifierOperation,
    pub permanent: bool,
}

impl AttributeModifier {
    pub fn new(id: impl Into<String>, amount: f64, operation: ModifierOperation) -> Self {
        Self {
            id: id.into(),
            amount,
            operation,
            permanent: false,
        }
    }

    pub fn permanent(mut self) -> Self {
        self.permanent = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeInstance {
    pub attribute: &'static AttributeDef,
    base_value: f64,
    modifiers: Vec<AttributeModifier>,
    dirty: bool,
    cached_value: f64,
}

impl AttributeInstance {
    pub fn new(attribute: &'static AttributeDef) -> Self {
        Self {
            attribute,
            base_value: attribute.default_value,
            modifiers: Vec::new(),
            dirty: true,
            cached_value: attribute.default_value,
        }
    }

    pub fn base_value(&self) -> f64 {
        self.base_value
    }

    pub fn set_base_value(&mut self, base_value: f64) {
        if self.base_value != base_value {
            self.base_value = base_value;
            self.dirty = true;
        }
    }

    pub fn add_modifier(&mut self, modifier: AttributeModifier) -> Result<(), &'static str> {
        if self
            .modifiers
            .iter()
            .any(|existing| existing.id == modifier.id)
        {
            return Err("modifier already applied");
        }
        self.modifiers.push(modifier);
        self.dirty = true;
        Ok(())
    }

    pub fn add_or_update_transient(&mut self, mut modifier: AttributeModifier) {
        modifier.permanent = false;
        if let Some(existing) = self
            .modifiers
            .iter_mut()
            .find(|existing| existing.id == modifier.id)
        {
            *existing = modifier;
        } else {
            self.modifiers.push(modifier);
        }
        self.dirty = true;
    }

    pub fn add_or_replace_permanent(&mut self, modifier: AttributeModifier) {
        self.remove_modifier(&modifier.id);
        let mut modifier = modifier;
        modifier.permanent = true;
        self.modifiers.push(modifier);
        self.dirty = true;
    }

    pub fn remove_modifier(&mut self, id: &str) -> bool {
        let before = self.modifiers.len();
        self.modifiers.retain(|modifier| modifier.id != id);
        let removed = self.modifiers.len() != before;
        if removed {
            self.dirty = true;
        }
        removed
    }

    pub fn value(&mut self) -> f64 {
        if self.dirty {
            self.cached_value = self.calculate_value();
            self.dirty = false;
        }
        self.cached_value
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn pack(&self) -> PackedAttribute {
        PackedAttribute {
            attribute_id: self.attribute.id,
            base_value: self.base_value,
            modifiers: self
                .modifiers
                .iter()
                .filter(|modifier| modifier.permanent)
                .cloned()
                .collect(),
        }
    }

    pub fn sync_packet(&mut self, entity_id: i32) -> Option<AttributeSyncPacket> {
        if !self.attribute.syncable || !self.dirty {
            return None;
        }
        let value = self.value();
        Some(AttributeSyncPacket {
            entity_id,
            attribute_id: self.attribute.id,
            value,
            modifiers: self.modifiers.clone(),
        })
    }

    fn calculate_value(&self) -> f64 {
        let mut base = self.base_value;
        for modifier in self.modifiers_for(ModifierOperation::AddValue) {
            base += modifier.amount;
        }

        let mut result = base;
        for modifier in self.modifiers_for(ModifierOperation::AddMultipliedBase) {
            result += base * modifier.amount;
        }
        for modifier in self.modifiers_for(ModifierOperation::AddMultipliedTotal) {
            result *= 1.0 + modifier.amount;
        }

        self.sanitize(result)
    }

    fn modifiers_for(
        &self,
        operation: ModifierOperation,
    ) -> impl Iterator<Item = &AttributeModifier> {
        self.modifiers
            .iter()
            .filter(move |modifier| modifier.operation == operation)
    }

    fn sanitize(&self, value: f64) -> f64 {
        if value.is_nan() {
            self.attribute.min_value
        } else {
            value.clamp(self.attribute.min_value, self.attribute.max_value)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PackedAttribute {
    pub attribute_id: &'static str,
    pub base_value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSyncPacket {
    pub entity_id: i32,
    pub attribute_id: &'static str,
    pub value: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_registry_matches_26_1_2_builtin_surface() {
        assert_eq!(ATTRIBUTES.len(), 35);
        assert_eq!(ATTRIBUTES[0].id, "minecraft:armor");
        assert_eq!(
            ATTRIBUTES.last().unwrap().id,
            "minecraft:waypoint_receive_range"
        );
        assert_eq!(attribute("attack_speed").unwrap().default_value, 4.0);
        assert!(attribute("minecraft:max_health").unwrap().syncable);
        assert!(!attribute("minecraft:attack_damage").unwrap().syncable);
        assert_eq!(
            attribute("minecraft:burning_time").unwrap().sentiment,
            AttributeSentiment::Negative
        );
        assert_eq!(attribute("minecraft:gravity").unwrap().min_value, -1.0);
    }

    #[test]
    fn modifier_operation_order_matches_attribute_instance_calculation() {
        let mut instance = AttributeInstance::new(attribute("minecraft:attack_damage").unwrap());
        instance.set_base_value(10.0);
        instance
            .add_modifier(AttributeModifier::new(
                "flat",
                2.0,
                ModifierOperation::AddValue,
            ))
            .unwrap();
        instance
            .add_modifier(AttributeModifier::new(
                "base",
                0.5,
                ModifierOperation::AddMultipliedBase,
            ))
            .unwrap();
        instance
            .add_modifier(AttributeModifier::new(
                "total",
                0.25,
                ModifierOperation::AddMultipliedTotal,
            ))
            .unwrap();

        assert_eq!(instance.value(), 22.5);
        assert!(!instance.is_dirty());
        assert_eq!(ModifierOperation::AddMultipliedTotal.id(), 2);
        assert_eq!(
            ModifierOperation::AddMultipliedBase.serialized_name(),
            "add_multiplied_base"
        );
    }

    #[test]
    fn duplicate_update_remove_permanent_pack_and_sanitize_follow_vanilla_rules() {
        let mut instance = AttributeInstance::new(attribute("minecraft:movement_speed").unwrap());
        assert!(instance
            .add_modifier(AttributeModifier::new(
                "same",
                0.1,
                ModifierOperation::AddMultipliedTotal
            ))
            .is_ok());
        assert_eq!(
            instance.add_modifier(AttributeModifier::new(
                "same",
                0.2,
                ModifierOperation::AddMultipliedTotal
            )),
            Err("modifier already applied")
        );

        instance.add_or_update_transient(AttributeModifier::new(
            "same",
            0.2,
            ModifierOperation::AddMultipliedTotal,
        ));
        assert!(instance.value() > 0.7);
        instance.add_or_replace_permanent(AttributeModifier::new(
            "boots",
            1.0,
            ModifierOperation::AddValue,
        ));
        let packed = instance.pack();
        assert_eq!(packed.modifiers.len(), 1);
        assert_eq!(packed.modifiers[0].id, "boots");
        assert!(instance.remove_modifier("boots"));

        instance.set_base_value(f64::NAN);
        assert_eq!(instance.value(), 0.0);
    }

    #[test]
    fn sync_packets_emit_only_dirty_syncable_attributes_with_current_value() {
        let mut synced = AttributeInstance::new(attribute("minecraft:max_health").unwrap());
        synced.set_base_value(40.0);
        synced
            .add_modifier(AttributeModifier::new(
                "bonus",
                4.0,
                ModifierOperation::AddValue,
            ))
            .unwrap();
        let packet = synced.sync_packet(7).unwrap();
        assert_eq!(packet.entity_id, 7);
        assert_eq!(packet.attribute_id, "minecraft:max_health");
        assert_eq!(packet.value, 44.0);
        assert!(synced.sync_packet(7).is_none());

        let mut unsynced = AttributeInstance::new(attribute("minecraft:attack_damage").unwrap());
        unsynced.set_base_value(8.0);
        assert_eq!(unsynced.sync_packet(7), None);
    }
}
