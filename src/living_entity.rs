#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionHand {
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LivingAnimation {
    SwingMainHand,
    SwingOffHand,
    Hurt,
    Death,
    CriticalHit,
    MagicCriticalHit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeValue {
    pub id: &'static str,
    pub base: f32,
    pub modifier: f32,
    pub dirty: bool,
}

impl AttributeValue {
    pub fn value(&self) -> f32 {
        self.base + self.modifier
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobEffectState {
    pub id: &'static str,
    pub amplifier: u8,
    pub duration: i32,
    pub ambient: bool,
    pub visible: bool,
    pub show_icon: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackRef {
    pub item: &'static str,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageReport {
    pub original_damage: f32,
    pub armor_reduced_damage: f32,
    pub absorbed_damage: f32,
    pub health_damage: f32,
    pub died: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivingEntityState {
    pub health: f32,
    pub max_health: f32,
    pub armor: f32,
    pub toughness: f32,
    pub absorption: f32,
    pub effects: Vec<MobEffectState>,
    pub attributes: Vec<AttributeValue>,
    pub equipment: Vec<(EquipmentSlot, ItemStackRef)>,
    pub using_item: Option<(InteractionHand, ItemStackRef, i32)>,
    pub hurt_time: i32,
    pub death_time: i32,
    pub drops: Vec<ItemStackRef>,
    pub experience_reward: i32,
    pub knockback: (f32, f32, f32),
    pub last_animation: Option<LivingAnimation>,
    pub dead: bool,
}

impl LivingEntityState {
    pub fn new(max_health: f32) -> Self {
        Self {
            health: max_health,
            max_health,
            armor: 0.0,
            toughness: 0.0,
            absorption: 0.0,
            effects: Vec::new(),
            attributes: vec![AttributeValue {
                id: "minecraft:max_health",
                base: max_health,
                modifier: 0.0,
                dirty: false,
            }],
            equipment: Vec::new(),
            using_item: None,
            hurt_time: 0,
            death_time: 0,
            drops: Vec::new(),
            experience_reward: 0,
            knockback: (0.0, 0.0, 0.0),
            last_animation: None,
            dead: false,
        }
    }

    pub fn set_health(&mut self, health: f32) {
        self.health = health.clamp(0.0, self.max_health);
        self.dead = self.health <= 0.0;
    }

    pub fn heal(&mut self, amount: f32) {
        if self.health > 0.0 {
            self.set_health(self.health + amount);
        }
    }

    pub fn hurt(&mut self, damage: f32) -> DamageReport {
        let original_damage = damage.max(0.0);
        let armor_reduced_damage =
            reduce_damage_by_armor(original_damage, self.armor, self.toughness);
        let absorbed_damage = armor_reduced_damage.min(self.absorption);
        self.absorption -= absorbed_damage;
        let health_damage = armor_reduced_damage - absorbed_damage;
        self.set_health(self.health - health_damage);
        self.hurt_time = 10;
        self.last_animation = Some(if self.dead {
            LivingAnimation::Death
        } else {
            LivingAnimation::Hurt
        });
        if self.dead {
            self.death_time = 1;
        }

        DamageReport {
            original_damage,
            armor_reduced_damage,
            absorbed_damage,
            health_damage,
            died: self.dead,
        }
    }

    pub fn add_effect(&mut self, effect: MobEffectState) -> EffectChange {
        if let Some(existing) = self
            .effects
            .iter_mut()
            .find(|existing| existing.id == effect.id)
        {
            let refreshed_attributes = existing.amplifier != effect.amplifier;
            *existing = effect;
            EffectChange::Updated {
                refresh_attributes: refreshed_attributes,
            }
        } else {
            self.effects.push(effect);
            EffectChange::Added
        }
    }

    pub fn tick_effects(&mut self) -> Vec<EffectChange> {
        let mut changes = Vec::new();
        for effect in &mut self.effects {
            effect.duration -= 1;
            if effect.duration > 0 && effect.duration % 600 == 0 {
                changes.push(EffectChange::Updated {
                    refresh_attributes: false,
                });
            }
        }
        let before = self.effects.len();
        self.effects.retain(|effect| effect.duration > 0);
        if self.effects.len() != before {
            changes.push(EffectChange::Removed);
        }
        changes
    }

    pub fn set_attribute_modifier(&mut self, id: &'static str, modifier: f32) {
        if let Some(attribute) = self
            .attributes
            .iter_mut()
            .find(|attribute| attribute.id == id)
        {
            attribute.modifier = modifier;
            attribute.dirty = true;
            if id == "minecraft:max_health" {
                self.max_health = attribute.value();
                self.set_health(self.health);
            }
            return;
        }
        self.attributes.push(AttributeValue {
            id,
            base: 0.0,
            modifier,
            dirty: true,
        });
    }

    pub fn dirty_attributes(&mut self) -> Vec<AttributeValue> {
        let dirty = self
            .attributes
            .iter()
            .filter(|attribute| attribute.dirty)
            .cloned()
            .collect();
        for attribute in &mut self.attributes {
            attribute.dirty = false;
        }
        dirty
    }

    pub fn equip(&mut self, slot: EquipmentSlot, item: ItemStackRef) {
        if let Some((_, existing)) = self
            .equipment
            .iter_mut()
            .find(|(existing_slot, _)| *existing_slot == slot)
        {
            *existing = item;
        } else {
            self.equipment.push((slot, item));
        }
    }

    pub fn start_using_item(&mut self, hand: InteractionHand, item: ItemStackRef, duration: i32) {
        self.using_item = Some((hand, item, duration.max(0)));
    }

    pub fn stop_using_item(&mut self) -> Option<ItemStackRef> {
        self.using_item.take().map(|(_, stack, _)| stack)
    }

    pub fn swing(&mut self, hand: InteractionHand) {
        self.last_animation = Some(match hand {
            InteractionHand::MainHand => LivingAnimation::SwingMainHand,
            InteractionHand::OffHand => LivingAnimation::SwingOffHand,
        });
    }

    pub fn apply_knockback(&mut self, strength: f32, x_ratio: f32, z_ratio: f32) {
        self.knockback = (x_ratio * strength, 0.4, z_ratio * strength);
    }

    pub fn die_with_drops(&mut self, drops: Vec<ItemStackRef>, experience: i32) {
        self.set_health(0.0);
        self.drops = drops;
        self.experience_reward = experience.max(0);
        self.last_animation = Some(LivingAnimation::Death);
        self.death_time = self.death_time.max(1);
    }

    pub fn save(&self) -> SavedLivingEntity {
        SavedLivingEntity {
            health: self.health,
            absorption: self.absorption,
            hurt_time: self.hurt_time as i16,
            death_time: self.death_time as i16,
            active_effects: self.effects.clone(),
            attributes: self.attributes.clone(),
            equipment: self.equipment.clone(),
        }
    }

    pub fn load(&mut self, saved: SavedLivingEntity) {
        self.set_health(saved.health);
        self.absorption = saved.absorption.max(0.0);
        self.hurt_time = i32::from(saved.hurt_time);
        self.death_time = i32::from(saved.death_time);
        self.effects = saved.active_effects;
        self.attributes = saved.attributes;
        self.equipment = saved.equipment;
    }

    pub fn sync_plan(&mut self) -> LivingSyncPlan {
        LivingSyncPlan {
            set_health: self.health,
            set_absorption: self.absorption,
            update_effects: self.effects.clone(),
            update_attributes: self.dirty_attributes(),
            set_equipment: self.equipment.clone(),
            animation: self.last_animation,
            drops: self.drops.clone(),
            experience: self.experience_reward,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectChange {
    Added,
    Updated { refresh_attributes: bool },
    Removed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SavedLivingEntity {
    pub health: f32,
    pub absorption: f32,
    pub hurt_time: i16,
    pub death_time: i16,
    pub active_effects: Vec<MobEffectState>,
    pub attributes: Vec<AttributeValue>,
    pub equipment: Vec<(EquipmentSlot, ItemStackRef)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivingSyncPlan {
    pub set_health: f32,
    pub set_absorption: f32,
    pub update_effects: Vec<MobEffectState>,
    pub update_attributes: Vec<AttributeValue>,
    pub set_equipment: Vec<(EquipmentSlot, ItemStackRef)>,
    pub animation: Option<LivingAnimation>,
    pub drops: Vec<ItemStackRef>,
    pub experience: i32,
}

fn reduce_damage_by_armor(damage: f32, armor: f32, toughness: f32) -> f32 {
    let armor_factor = (armor - damage / (2.0 + toughness / 4.0)).clamp(armor * 0.2, 20.0);
    damage * (1.0 - armor_factor / 25.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &'static str, count: u32) -> ItemStackRef {
        ItemStackRef { item, count }
    }

    #[test]
    fn health_damage_armor_absorption_death_and_healing_follow_living_rules() {
        let mut entity = LivingEntityState::new(20.0);
        entity.armor = 10.0;
        entity.toughness = 2.0;
        entity.absorption = 3.0;

        let report = entity.hurt(10.0);
        assert_eq!(report.original_damage, 10.0);
        assert!(report.armor_reduced_damage < 10.0);
        assert_eq!(report.absorbed_damage, 3.0);
        assert_eq!(entity.hurt_time, 10);
        assert_eq!(entity.last_animation, Some(LivingAnimation::Hurt));
        assert!(!report.died);

        entity.heal(100.0);
        assert_eq!(entity.health, 20.0);
        entity.hurt(1000.0);
        assert!(entity.dead);
        assert_eq!(entity.death_time, 1);
        assert_eq!(entity.last_animation, Some(LivingAnimation::Death));
    }

    #[test]
    fn effects_add_update_tick_remove_and_attribute_dirty_sync_are_tracked() {
        let mut entity = LivingEntityState::new(20.0);
        assert_eq!(
            entity.add_effect(MobEffectState {
                id: "minecraft:speed",
                amplifier: 0,
                duration: 601,
                ambient: false,
                visible: true,
                show_icon: true,
            }),
            EffectChange::Added
        );
        assert_eq!(
            entity.add_effect(MobEffectState {
                id: "minecraft:speed",
                amplifier: 1,
                duration: 2,
                ambient: false,
                visible: true,
                show_icon: true,
            }),
            EffectChange::Updated {
                refresh_attributes: true
            }
        );
        assert_eq!(entity.tick_effects(), Vec::<EffectChange>::new());
        assert_eq!(entity.tick_effects(), vec![EffectChange::Removed]);

        entity.set_attribute_modifier("minecraft:max_health", 10.0);
        assert_eq!(entity.max_health, 30.0);
        let dirty = entity.dirty_attributes();
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].id, "minecraft:max_health");
        assert!(entity.dirty_attributes().is_empty());
    }

    #[test]
    fn equipment_hands_use_item_knockback_and_animation_are_modeled() {
        let mut entity = LivingEntityState::new(20.0);
        entity.equip(EquipmentSlot::MainHand, stack("minecraft:diamond_sword", 1));
        entity.equip(EquipmentSlot::OffHand, stack("minecraft:shield", 1));
        entity.equip(EquipmentSlot::Head, stack("minecraft:diamond_helmet", 1));
        entity.start_using_item(
            InteractionHand::OffHand,
            stack("minecraft:shield", 1),
            72000,
        );
        assert_eq!(entity.stop_using_item(), Some(stack("minecraft:shield", 1)));
        entity.swing(InteractionHand::MainHand);
        assert_eq!(entity.last_animation, Some(LivingAnimation::SwingMainHand));
        entity.swing(InteractionHand::OffHand);
        assert_eq!(entity.last_animation, Some(LivingAnimation::SwingOffHand));
        entity.apply_knockback(0.5, -1.0, 0.25);
        assert_eq!(entity.knockback, (-0.5, 0.4, 0.125));
        assert_eq!(entity.equipment.len(), 3);
    }

    #[test]
    fn death_drops_experience_save_load_and_sync_plan_cover_living_packet_surface() {
        let mut entity = LivingEntityState::new(20.0);
        entity.equip(EquipmentSlot::Chest, stack("minecraft:iron_chestplate", 1));
        entity.set_attribute_modifier("minecraft:attack_damage", 3.0);
        entity.add_effect(MobEffectState {
            id: "minecraft:strength",
            amplifier: 0,
            duration: 200,
            ambient: false,
            visible: true,
            show_icon: true,
        });
        entity.die_with_drops(vec![stack("minecraft:bone", 2)], 5);

        let sync = entity.sync_plan();
        assert_eq!(sync.set_health, 0.0);
        assert_eq!(sync.update_effects[0].id, "minecraft:strength");
        assert_eq!(sync.update_attributes[0].id, "minecraft:attack_damage");
        assert_eq!(sync.set_equipment[0].0, EquipmentSlot::Chest);
        assert_eq!(sync.drops, vec![stack("minecraft:bone", 2)]);
        assert_eq!(sync.experience, 5);
        assert_eq!(sync.animation, Some(LivingAnimation::Death));

        let saved = entity.save();
        let mut loaded = LivingEntityState::new(1.0);
        loaded.load(saved);
        assert_eq!(loaded.health, 0.0);
        assert_eq!(loaded.effects[0].id, "minecraft:strength");
        assert_eq!(loaded.equipment[0].1.item, "minecraft:iron_chestplate");
    }

    #[test]
    fn all_equipment_slots_and_animations_are_represented() {
        let slots = [
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Feet,
            EquipmentSlot::Legs,
            EquipmentSlot::Chest,
            EquipmentSlot::Head,
            EquipmentSlot::Body,
            EquipmentSlot::Saddle,
        ];
        assert_eq!(slots.len(), 8);

        let animations = [
            LivingAnimation::SwingMainHand,
            LivingAnimation::SwingOffHand,
            LivingAnimation::Hurt,
            LivingAnimation::Death,
            LivingAnimation::CriticalHit,
            LivingAnimation::MagicCriticalHit,
        ];
        assert_eq!(animations.len(), 6);
    }
}
