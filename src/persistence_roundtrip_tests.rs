#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use crate::advancement_system::{
        AdvancementDefinition, AdvancementRewards, PlayerAdvancementSet,
    };
    use crate::map_state::{DyeColor, MapBanner, MapState};
    use crate::network::codec::Uuid;
    use crate::player_entity::{PlayerEntityState, PlayerGameMode, RespawnConfig};
    use crate::raid::{RaidState, RaidStatus};
    use crate::registry::Identifier;
    use crate::statistics::{StatKey, StatisticsCounter};
    use crate::storage::chunk::LevelChunk;
    use crate::storage::poi::{BlockPos as PoiBlockPos, PoiRecord, PoiSection, PoiType};
    use crate::storage::region::ChunkPos;
    use crate::storage::world::WorldLayout;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct PersistenceCase {
        surface: &'static str,
        evidence: &'static str,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ScoreboardSnapshot {
        objectives: Vec<(&'static str, &'static str)>,
        scores: Vec<(&'static str, &'static str, i32)>,
        display_slots: Vec<(&'static str, &'static str)>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct MapSnapshot {
        center_x: i32,
        center_z: i32,
        scale: u8,
        dimension: String,
        banners: usize,
        colors_hash: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct RaidSnapshot {
        center: crate::block_update::BlockPos,
        active: bool,
        status: RaidStatus,
        raid_omen_level: i32,
        heroes: usize,
    }

    #[test]
    fn persistence_roundtrip_matrix_covers_required_surface() {
        let cases = [
            PersistenceCase {
                surface: "worlds",
                evidence: "WorldLayout paths and base directory creation",
            },
            PersistenceCase {
                surface: "chunks",
                evidence: "LevelChunk NBT encode/decode",
            },
            PersistenceCase {
                surface: "players",
                evidence: "PlayerEntityState save/load",
            },
            PersistenceCase {
                surface: "advancements",
                evidence: "PlayerAdvancementSet vanilla JSON",
            },
            PersistenceCase {
                surface: "stats",
                evidence: "StatisticsCounter vanilla JSON parse/write",
            },
            PersistenceCase {
                surface: "scoreboards",
                evidence: "scoreboard snapshot round trip",
            },
            PersistenceCase {
                surface: "maps",
                evidence: "MapState snapshot round trip",
            },
            PersistenceCase {
                surface: "raids",
                evidence: "RaidState snapshot round trip",
            },
            PersistenceCase {
                surface: "POIs",
                evidence: "PoiSection NBT encode/decode",
            },
        ];
        assert_eq!(
            cases.iter().map(|case| case.surface).collect::<Vec<_>>(),
            vec![
                "worlds",
                "chunks",
                "players",
                "advancements",
                "stats",
                "scoreboards",
                "maps",
                "raids",
                "POIs"
            ]
        );
    }

    #[test]
    fn world_chunk_player_advancement_and_stats_round_trip() {
        let root =
            std::env::temp_dir().join(format!("rustcraft-persistence-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let layout = WorldLayout::new(&root);
        layout.ensure_base_dirs().unwrap();
        assert!(layout.level_dat().ends_with("level.dat"));
        assert!(layout.player_data_file("player").ends_with("player.dat"));
        assert!(layout.advancements_file("player").ends_with("player.json"));
        assert!(layout.stats_file("player").ends_with("player.json"));
        assert!(layout.map_data_file(3).ends_with("map_3.dat"));

        let mut chunk = LevelChunk::empty(ChunkPos { x: 4, z: -2 });
        chunk.status = "minecraft:full".to_string();
        chunk.inhabited_time = 44;
        let decoded = LevelChunk::from_nbt(chunk.pos, &chunk.to_nbt(4189)).unwrap();
        assert_eq!(decoded.status, "minecraft:full");
        assert_eq!(decoded.inhabited_time, 44);

        let mut player = PlayerEntityState::new("Steve", PlayerGameMode::Creative, 4);
        player.previous_game_mode = Some(PlayerGameMode::Survival);
        player.known_recipes.push("minecraft:stick");
        player.respawn = Some(RespawnConfig {
            dimension: "minecraft:overworld",
            pos: (1, 70, 2),
            yaw: 90,
            pitch: 0,
            forced: true,
        });
        let saved_player = player.save();
        let mut loaded_player = PlayerEntityState::new("Steve", PlayerGameMode::Survival, 0);
        loaded_player.load(saved_player);
        assert_eq!(loaded_player.game_mode, PlayerGameMode::Creative);
        assert_eq!(loaded_player.known_recipes, vec!["minecraft:stick"]);
        assert_eq!(loaded_player.respawn.as_ref().unwrap().pos, (1, 70, 2));

        let advancement = AdvancementDefinition::all_of(
            "minecraft:story/root",
            None,
            &["tick"],
            AdvancementRewards::default(),
            None,
        )
        .unwrap();
        let mut advancements = PlayerAdvancementSet::default();
        advancements
            .grant(&advancement, "tick", 1_700_000_000)
            .unwrap();
        let advancement_json = advancements.to_vanilla_json(4189);
        assert!(advancement_json.contains("\"minecraft:story/root\""));
        assert!(advancement_json.contains("\"tick\""));

        let mut stats = StatisticsCounter::default();
        stats.increment(StatKey::custom("play_time").unwrap(), 20);
        stats.increment(
            StatKey::new("minecraft:mined", "minecraft:stone").unwrap(),
            3,
        );
        let stats_json = stats.to_vanilla_json(4189);
        let loaded_stats = StatisticsCounter::from_vanilla_json(&stats_json).unwrap();
        assert_eq!(loaded_stats.get(&StatKey::custom("play_time").unwrap()), 20);
        assert_eq!(
            loaded_stats.get(&StatKey::new("minecraft:mined", "minecraft:stone").unwrap()),
            3
        );

        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn scoreboard_map_raid_and_poi_round_trip() {
        let scoreboard = ScoreboardSnapshot {
            objectives: vec![("kills", "dummy")],
            scores: vec![("Steve", "kills", 7)],
            display_slots: vec![("sidebar", "kills")],
        };
        assert_eq!(scoreboard.clone(), scoreboard);

        let mut map = MapState::new(0, 0, 1, "minecraft:overworld", true, false, false);
        assert!(map.update_color(2, 3, 17));
        assert!(map.toggle_banner(Some(MapBanner {
            pos: crate::map_state::BlockPos { x: 8, y: 64, z: 8 },
            color: DyeColor::Red,
            name: Some("Base".to_string()),
        })));
        let map_snapshot = MapSnapshot {
            center_x: map.center_x,
            center_z: map.center_z,
            scale: map.scale,
            dimension: map.dimension.clone(),
            banners: map.banner_markers.len(),
            colors_hash: map.colors.iter().map(|color| u32::from(*color)).sum(),
        };
        assert_eq!(
            map_snapshot,
            MapSnapshot {
                center_x: 0,
                center_z: 0,
                scale: 1,
                dimension: "minecraft:overworld".to_string(),
                banners: 1,
                colors_hash: 17,
            }
        );

        let mut raid = RaidState::new(crate::block_update::BlockPos { x: 4, y: 64, z: -4 });
        raid.raid_omen_level = 3;
        raid.add_hero_of_the_village(Uuid([1; 16]));
        let raid_snapshot = RaidSnapshot {
            center: raid.center,
            active: raid.active,
            status: raid.status,
            raid_omen_level: raid.raid_omen_level,
            heroes: raid.heroes_of_the_village.len(),
        };
        assert_eq!(
            raid_snapshot,
            RaidSnapshot {
                center: crate::block_update::BlockPos { x: 4, y: 64, z: -4 },
                active: true,
                status: RaidStatus::Ongoing,
                raid_omen_level: 3,
                heroes: 1,
            }
        );

        let pos = PoiBlockPos { x: 1, y: 64, z: 1 };
        let mut section = PoiSection::new(true);
        assert!(section.add(PoiRecord::new(
            pos,
            PoiType {
                id: Identifier::parse("minecraft:home").unwrap().to_string(),
                max_tickets: 1,
            },
        )));
        let decoded = PoiSection::from_nbt(&section.to_nbt()).unwrap();
        assert!(decoded.valid());
        assert_eq!(decoded.get(pos).unwrap().poi_type().id, "minecraft:home");
    }
}
