#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerExperience {
    pub level: i32,
    pub progress: f32,
    pub total: i32,
    pub take_xp_delay: i32,
}

impl Default for PlayerExperience {
    fn default() -> Self {
        Self {
            level: 0,
            progress: 0.0,
            total: 0,
            take_xp_delay: 0,
        }
    }
}

impl PlayerExperience {
    pub fn give_points(&mut self, amount: i32) {
        self.progress += amount as f32 / xp_needed_for_next_level(self.level) as f32;
        self.total = self.total.saturating_add(amount).max(0);
        while self.progress < 0.0 {
            let remaining = self.progress * xp_needed_for_next_level(self.level) as f32;
            if self.level > 0 {
                self.give_levels(-1);
                self.progress = 1.0 + remaining / xp_needed_for_next_level(self.level) as f32;
            } else {
                self.give_levels(-1);
                self.progress = 0.0;
            }
        }
        while self.progress >= 1.0 {
            self.progress = (self.progress - 1.0) * xp_needed_for_next_level(self.level) as f32;
            self.give_levels(1);
            self.progress /= xp_needed_for_next_level(self.level) as f32;
        }
    }

    pub fn give_levels(&mut self, amount: i32) {
        self.level = self.level.saturating_add(amount);
        if self.level < 0 {
            self.level = 0;
            self.progress = 0.0;
            self.total = 0;
        }
    }

    pub fn set_points(&mut self, amount: i32) -> bool {
        let needed = xp_needed_for_next_level(self.level);
        if amount >= needed {
            return false;
        }
        let max = (needed - 1) as f32 / needed as f32;
        self.progress = (amount as f32 / needed as f32).clamp(0.0, max);
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExperienceOrb {
    pub id: i32,
    pub value: i32,
    pub count: i32,
    pub age: i32,
    pub health: i32,
    pub removed: bool,
}

impl ExperienceOrb {
    pub fn new(id: i32, value: i32) -> Self {
        Self {
            id,
            value,
            count: 1,
            age: 0,
            health: 5,
            removed: false,
        }
    }

    pub fn icon(&self) -> i32 {
        experience_icon(self.value)
    }

    pub fn tick_age(&mut self) {
        self.age += 1;
        if self.age >= 6000 {
            self.removed = true;
        }
    }

    pub fn hurt(&mut self, damage: i32) -> bool {
        if self.removed {
            return false;
        }
        self.health -= damage;
        if self.health <= 0 {
            self.removed = true;
        }
        true
    }

    pub fn can_merge(&self, id: i32, value: i32) -> bool {
        !self.removed && (self.id - id) % 40 == 0 && self.value == value
    }

    pub fn merge(&mut self, other: &mut ExperienceOrb) -> bool {
        if other.id == self.id || !self.can_merge(other.id, other.value) {
            return false;
        }
        self.count += other.count;
        self.age = self.age.min(other.age);
        other.removed = true;
        true
    }

    pub fn player_touch(
        &mut self,
        player: &mut PlayerExperience,
        repair_targets: &mut [RepairableItem],
    ) -> bool {
        if self.removed || player.take_xp_delay != 0 {
            return false;
        }
        player.take_xp_delay = 2;
        let remaining = repair_player_items(repair_targets, self.value);
        if remaining > 0 {
            player.give_points(remaining);
        }
        self.count -= 1;
        if self.count == 0 {
            self.removed = true;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairableItem {
    pub damage: i32,
    pub repair_per_xp: i32,
}

pub fn repair_player_items(items: &mut [RepairableItem], amount: i32) -> i32 {
    let Some(item) = items.iter_mut().find(|item| item.damage > 0) else {
        return amount;
    };
    let repair_per_xp = item.repair_per_xp.max(1);
    let possible_repair = amount * repair_per_xp;
    let repaired = possible_repair.min(item.damage);
    item.damage -= repaired;
    let spent_xp = (repaired + repair_per_xp - 1) / repair_per_xp;
    let remaining = amount.saturating_sub(spent_xp);
    if remaining > 0 {
        repair_player_items(items, remaining)
    } else {
        0
    }
}

pub fn award_orbs(total: i32, first_id: i32) -> Vec<ExperienceOrb> {
    let mut remaining = total.max(0);
    let mut next_id = first_id;
    let mut orbs = Vec::new();
    while remaining > 0 {
        let value = experience_value(remaining);
        remaining -= value;
        orbs.push(ExperienceOrb::new(next_id, value));
        next_id += 1;
    }
    orbs
}

pub fn merge_awarded_orb(existing: &mut [ExperienceOrb], id_roll: i32, value: i32) -> bool {
    if let Some(orb) = existing
        .iter_mut()
        .find(|orb| orb.can_merge(id_roll, value))
    {
        orb.count += 1;
        orb.age = 0;
        true
    } else {
        false
    }
}

pub fn xp_needed_for_next_level(level: i32) -> i32 {
    if level >= 30 {
        112 + (level - 30) * 9
    } else if level >= 15 {
        37 + (level - 15) * 5
    } else {
        7 + level * 2
    }
}

pub fn experience_value(max_value: i32) -> i32 {
    if max_value >= 2477 {
        2477
    } else if max_value >= 1237 {
        1237
    } else if max_value >= 617 {
        617
    } else if max_value >= 307 {
        307
    } else if max_value >= 149 {
        149
    } else if max_value >= 73 {
        73
    } else if max_value >= 37 {
        37
    } else if max_value >= 17 {
        17
    } else if max_value >= 7 {
        7
    } else if max_value >= 3 {
        3
    } else {
        1
    }
}

pub fn experience_icon(value: i32) -> i32 {
    if value >= 2477 {
        10
    } else if value >= 1237 {
        9
    } else if value >= 617 {
        8
    } else if value >= 307 {
        7
    } else if value >= 149 {
        6
    } else if value >= 73 {
        5
    } else if value >= 37 {
        4
    } else if value >= 17 {
        3
    } else if value >= 7 {
        2
    } else if value >= 3 {
        1
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperienceRewardSource {
    MobKill,
    PlayerKill,
    Mining,
    Smelting,
    Breeding,
    Fishing,
    Trading,
    Advancement,
    Command,
    ExperienceBottle,
}

pub fn reward_amount(source: ExperienceRewardSource, base: i32) -> i32 {
    match source {
        ExperienceRewardSource::ExperienceBottle => base.clamp(3, 11),
        ExperienceRewardSource::Command => base,
        _ => base.max(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orb_value_splitting_and_icons_match_vanilla_thresholds() {
        assert_eq!(experience_value(3000), 2477);
        assert_eq!(experience_value(16), 7);
        assert_eq!(experience_value(6), 3);
        assert_eq!(experience_value(2), 1);
        assert_eq!(experience_icon(2477), 10);
        assert_eq!(experience_icon(73), 5);
        assert_eq!(experience_icon(3), 1);
        assert_eq!(experience_icon(1), 0);

        let orbs = award_orbs(40, 100);
        assert_eq!(
            orbs.iter().map(|orb| orb.value).collect::<Vec<_>>(),
            vec![37, 3]
        );
        assert_eq!(orbs[0].id, 100);
        assert_eq!(orbs[1].id, 101);
    }

    #[test]
    fn orb_merge_age_health_and_lifetime_follow_entity_rules() {
        let mut first = ExperienceOrb::new(80, 7);
        let mut second = ExperienceOrb::new(120, 7);
        second.count = 3;
        second.age = 10;

        assert!(first.merge(&mut second));
        assert_eq!(first.count, 4);
        assert_eq!(first.age, 0);
        assert!(second.removed);
        assert!(first.hurt(2));
        assert_eq!(first.health, 3);
        assert!(first.hurt(3));
        assert!(first.removed);

        let mut lifetime = ExperienceOrb::new(1, 1);
        lifetime.age = 5999;
        lifetime.tick_age();
        assert!(lifetime.removed);
    }

    #[test]
    fn player_pickup_repairs_items_before_awarding_remaining_xp_and_sets_delay() {
        let mut player = PlayerExperience::default();
        let mut orb = ExperienceOrb::new(1, 7);
        let mut repair = [RepairableItem {
            damage: 10,
            repair_per_xp: 2,
        }];

        assert!(orb.player_touch(&mut player, &mut repair));
        assert_eq!(repair[0].damage, 0);
        assert_eq!(player.take_xp_delay, 2);
        assert_eq!(player.total, 2);
        assert!(orb.removed);

        let mut blocked = ExperienceOrb::new(2, 3);
        assert!(!blocked.player_touch(&mut player, &mut []));
        assert!(!blocked.removed);
    }

    #[test]
    fn player_experience_math_matches_command_level_thresholds() {
        let mut player = PlayerExperience::default();
        player.give_points(16);
        assert_eq!(player.level, 2);
        assert_eq!(player.total, 16);
        assert!(player.progress.abs() < 0.0001);

        player.give_levels(28);
        assert_eq!(xp_needed_for_next_level(player.level), 112);
        assert!(player.set_points(111));
        assert!(!player.set_points(112));
        player.give_points(-10_000);
        assert_eq!(player, PlayerExperience::default());
    }

    #[test]
    fn reward_sources_clamp_bottles_and_preserve_normal_positive_rewards() {
        assert_eq!(
            reward_amount(ExperienceRewardSource::ExperienceBottle, 1),
            3
        );
        assert_eq!(
            reward_amount(ExperienceRewardSource::ExperienceBottle, 20),
            11
        );
        assert_eq!(reward_amount(ExperienceRewardSource::Mining, -5), 0);
        assert_eq!(reward_amount(ExperienceRewardSource::Command, -5), -5);
    }
}
