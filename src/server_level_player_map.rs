#![allow(dead_code)]

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlayerMapPlayer {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PlayerMapModel {
    players: BTreeMap<PlayerMapPlayer, bool>,
}

impl PlayerMapPlayer {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

impl PlayerMapModel {
    pub fn get_all_players(&self) -> Vec<PlayerMapPlayer> {
        self.players.keys().cloned().collect()
    }

    pub fn add_player(&mut self, player: PlayerMapPlayer, ignored: bool) {
        self.players.insert(player, ignored);
    }

    pub fn remove_player(&mut self, player: &PlayerMapPlayer) {
        self.players.remove(player);
    }

    pub fn ignore_player(&mut self, player: &PlayerMapPlayer) {
        if let Some(ignored) = self.players.get_mut(player) {
            *ignored = true;
        }
    }

    pub fn un_ignore_player(&mut self, player: &PlayerMapPlayer) {
        if let Some(ignored) = self.players.get_mut(player) {
            *ignored = false;
        }
    }

    pub fn ignored_or_unknown(&self, player: &PlayerMapPlayer) -> bool {
        self.players.get(player).copied().is_none_or(|ignored| ignored)
    }

    pub fn ignored(&self, player: &PlayerMapPlayer) -> bool {
        self.players.get(player).copied().is_some_and(|ignored| ignored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_map_adds_removes_and_lists_players() {
        let steve = PlayerMapPlayer::new("Steve");
        let alex = PlayerMapPlayer::new("Alex");
        let mut map = PlayerMapModel::default();

        map.add_player(steve.clone(), false);
        map.add_player(alex.clone(), true);

        assert_eq!(map.get_all_players(), vec![alex.clone(), steve.clone()]);
        assert!(!map.ignored(&steve));
        assert!(map.ignored(&alex));

        map.add_player(steve.clone(), true);
        assert!(map.ignored(&steve));

        map.remove_player(&alex);
        assert_eq!(map.get_all_players(), vec![steve]);
    }

    #[test]
    fn ignore_and_unignore_replace_only_existing_players() {
        let steve = PlayerMapPlayer::new("Steve");
        let missing = PlayerMapPlayer::new("Missing");
        let mut map = PlayerMapModel::default();

        map.ignore_player(&missing);
        assert!(map.get_all_players().is_empty());
        assert!(map.ignored_or_unknown(&missing));
        assert!(!map.ignored(&missing));

        map.add_player(steve.clone(), false);
        map.ignore_player(&steve);
        assert!(map.ignored_or_unknown(&steve));
        assert!(map.ignored(&steve));

        map.un_ignore_player(&steve);
        assert!(!map.ignored_or_unknown(&steve));
        assert!(!map.ignored(&steve));

        map.un_ignore_player(&missing);
        assert_eq!(map.get_all_players(), vec![steve]);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn player_map_source_matches_java_26_1_2() {
        const PLAYER_MAP: &str =
            vibecraft_java_source!("/net/minecraft/server/level/PlayerMap.java");

        for sentinel in [
            "public final class PlayerMap",
            "private final Object2BooleanMap<ServerPlayer> players = new Object2BooleanOpenHashMap();",
            "public Set<ServerPlayer> getAllPlayers()",
            "return this.players.keySet();",
            "public void addPlayer(final ServerPlayer player, final boolean ignored)",
            "this.players.put(player, ignored);",
            "public void removePlayer(final ServerPlayer player)",
            "this.players.removeBoolean(player);",
            "public void ignorePlayer(final ServerPlayer player)",
            "this.players.replace(player, true);",
            "public void unIgnorePlayer(final ServerPlayer player)",
            "this.players.replace(player, false);",
            "return this.players.getOrDefault(player, true);",
            "return this.players.getBoolean(player);",
        ] {
            assert!(
                PLAYER_MAP.contains(sentinel),
                "PlayerMap.java is missing sentinel: {sentinel}"
            );
        }
    }
}
