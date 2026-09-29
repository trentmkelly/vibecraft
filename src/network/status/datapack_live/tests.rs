//! `/datapack` and `/reload` through the command model, the pack repository and the
//! world packet bus.

use std::fs;

use super::*;
use crate::command::{
    execute_builtin_command, CommandError, LevelBasedPermissionSet, ServerCommandState,
};
use crate::network::compression::CompressionState;
use crate::network::varint::read_var_i32;
use crate::resources::{
    configure_pack_repository, DataPackRepository, PackConfigureOptions, WorldDataConfiguration,
};

/// A world whose `datapacks` folder holds one pack that adds a tag.
struct World {
    root: std::path::PathBuf,
}

impl World {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "vibecraft-datapack-live-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let pack = root.join("datapacks/added");
        fs::create_dir_all(pack.join("data/minecraft/tags/item")).unwrap_or_else(|e| panic!("{e}"));
        fs::write(
            pack.join("pack.mcmeta"),
            r#"{"pack":{"description":"added","min_format":[101,1],"max_format":101}}"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        fs::write(
            pack.join("data/minecraft/tags/item/added_by_pack.json"),
            r#"{"values":["minecraft:stick"]}"#,
        )
        .unwrap_or_else(|e| panic!("{e}"));
        Self { root }
    }

    /// A server with only `vanilla` selected, like one that had `added` disabled.
    fn resources(&self) -> ServerResources {
        let mut repository = DataPackRepository::server_repository(&self.root.join("datapacks"))
            .unwrap_or_else(|e| panic!("{e}"));
        let mut config = WorldDataConfiguration::default_26_1_2();
        config.data_packs = crate::resources::DataPackConfig::new(["vanilla"], ["file/added"]);
        // `MinecraftServer.configurePackRepository` also fills the disabled list.
        let configured = configure_pack_repository(
            &mut repository,
            &config,
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
        );
        ServerResources::new(repository, configured, &self.root).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn run(resources: &ServerResources, line: &str) -> (ServerCommandState, Result<(), CommandError>) {
    let mut state = ServerCommandState::default();
    seed_from(resources, &mut state);
    let result =
        execute_builtin_command(&mut state, LevelBasedPermissionSet::OWNER, line).map(|_| ());
    (state, result)
}

fn has_tag(resources: &ServerResources) -> bool {
    let current = resources.current();
    let items = current
        .registries
        .lookup(
            &crate::registry::Identifier::new("minecraft", "item")
                .unwrap_or_else(|e| panic!("{e}")),
        )
        .unwrap_or_else(|| panic!("item registry"));
    items
        .tags()
        .keys()
        .any(|tag| tag.to_string() == "minecraft:added_by_pack")
}

#[test]
fn seeding_reflects_the_repository_and_datapack_enable_reloads_with_a_broadcast() {
    let world = World::new("enable");
    let resources = world.resources();
    assert!(!has_tag(&resources));

    let (state, result) = run(&resources, "datapack enable file/added");
    assert_eq!(result, Ok(()));
    assert_eq!(state.selected_data_packs, vec!["vanilla", "file/added"]);
    assert!(state
        .available_data_packs
        .contains(&"trade_rebalance".to_string()));
    // Feature packs cannot be enabled while their features are off.
    assert!(state
        .unavailable_feature_data_packs
        .contains(&"trade_rebalance".to_string()));

    let bus = WorldPacketBus::default();
    let subscription = bus.subscribe(1);
    assert_eq!(
        apply_requests(&resources, &state, &bus),
        DataPackOutcome::Reloaded
    );
    assert!(has_tag(&resources));
    assert_eq!(
        resources.list_packs().selected,
        vec!["vanilla", "file/added"]
    );

    // `PlayerList.reloadResources` sends the tags and then the recipes to every player.
    let mut frames = Vec::new();
    let count = subscription
        .drain_into(&mut frames, CompressionState::disabled())
        .unwrap_or_else(|e| panic!("drain: {e}"));
    assert_eq!(count, 2);
    let mut reader = &frames[..];
    let mut ids = Vec::new();
    while !reader.is_empty() {
        let payload = CompressionState::disabled()
            .decode_packet(&mut reader)
            .unwrap_or_else(|e| panic!("frame: {e}"));
        ids.push(read_var_i32(&mut &payload[..]).unwrap_or_else(|e| panic!("id: {e}")));
    }
    assert_eq!(ids, vec![134, 133]);
}

#[test]
fn datapack_disable_and_plain_reload_follow_the_repository() {
    let world = World::new("disable");
    let resources = world.resources();
    let bus = WorldPacketBus::default();

    // `/reload` discovers the not-disabled new packs; `added` is in the disabled list.
    let (state, result) = run(&resources, "reload");
    assert_eq!(result, Ok(()));
    assert_eq!(state.selected_data_packs, vec!["vanilla"]);
    assert_eq!(
        apply_requests(&resources, &state, &bus),
        DataPackOutcome::Reloaded
    );
    assert!(!has_tag(&resources));

    let (state, _) = run(&resources, "datapack enable file/added first");
    assert_eq!(state.selected_data_packs, vec!["file/added", "vanilla"]);
    apply_requests(&resources, &state, &bus);
    assert!(has_tag(&resources));
    assert_eq!(
        resources.list_packs().selected,
        vec!["file/added", "vanilla"]
    );

    let (state, result) = run(&resources, "datapack disable file/added");
    assert_eq!(result, Ok(()));
    apply_requests(&resources, &state, &bus);
    assert!(!has_tag(&resources));
    assert_eq!(resources.list_packs().selected, vec!["vanilla"]);
    // Disabling twice / an unknown pack are command errors, not reloads.
    let (state, result) = run(&resources, "datapack disable file/added");
    assert_eq!(result, Err(CommandError::DataPackAlreadyDisabled));
    assert_eq!(
        apply_requests(&resources, &state, &bus),
        DataPackOutcome::Untouched
    );
    let (_, result) = run(&resources, "datapack enable file/nope");
    assert_eq!(result, Err(CommandError::DataPackUnknown));
}

#[test]
fn a_failed_reload_reports_failure_and_keeps_the_previous_packs() {
    let world = World::new("failure");
    let resources = world.resources();
    let bus = WorldPacketBus::default();
    // The pack is discovered, then its folder disappears before the archive-less
    // reload opens it: directory packs open lazily and cannot fail, so simulate a
    // broken archive instead.
    let archive = world.root.join("datapacks/broken.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&archive).unwrap_or_else(|e| panic!("{e}")));
    zip.start_file("pack.mcmeta", zip::write::SimpleFileOptions::default())
        .unwrap_or_else(|e| panic!("{e}"));
    std::io::Write::write_all(
        &mut zip,
        br#"{"pack":{"description":"b","min_format":[101,1],"max_format":101}}"#,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    zip.finish().unwrap_or_else(|e| panic!("{e}"));
    let (state, result) = run(&resources, "datapack enable file/broken.zip");
    assert_eq!(result, Ok(()));
    fs::remove_file(&archive).unwrap_or_else(|e| panic!("{e}"));

    assert_eq!(
        apply_requests(&resources, &state, &bus),
        DataPackOutcome::Failed
    );
    assert_eq!(resources.list_packs().selected, vec!["vanilla"]);
}

#[test]
fn datapack_create_uses_the_world_datapacks_folder() {
    let world = World::new("create");
    let resources = world.resources();
    let (state, result) = run(&resources, "datapack create fresh Fresh pack");
    assert_eq!(result, Ok(()));
    assert!(world.root.join("datapacks/fresh/pack.mcmeta").is_file());
    assert_eq!(state.created_data_packs.len(), 1);
    assert!(resources
        .list_packs()
        .available
        .contains(&"file/fresh".to_string()));
}

#[test]
fn only_datapack_and_reload_commands_read_the_repository() {
    assert!(touches_data_packs("datapack list"));
    assert!(touches_data_packs("/reload"));
    assert!(touches_data_packs("  /datapack enable x"));
    assert!(!touches_data_packs("say datapack"));
    assert!(!touches_data_packs("reloadx"));
    assert!(!touches_data_packs(""));
}
