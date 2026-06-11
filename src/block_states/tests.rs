use super::*;

#[test]
fn entries_cover_full_vanilla_block_registry_in_protocol_order() {
    let entries = block_state_entries();
    assert_eq!(
        entries.len(),
        crate::block_metadata::VANILLA_BLOCK_REGISTRY_COUNT
    );
    assert_eq!(entries[0].registry_id, "minecraft:air");
    assert_eq!(
        entries.last().unwrap().registry_id,
        "minecraft:firefly_bush"
    );

    // State-id ranges are contiguous across the whole registry, exactly like
    // Java's Block.BLOCK_STATE_REGISTRY incremental registration.
    let mut next_id = 0;
    for entry in entries {
        assert_eq!(entry.base_state_id, next_id, "{}", entry.registry_id);
        assert!(
            entry.default_state_id >= entry.base_state_id
                && entry.default_state_id < entry.base_state_id + entry.state_count(),
            "{}",
            entry.registry_id
        );
        next_id += entry.state_count();
    }
    assert_eq!(next_id as usize, VANILLA_BLOCK_STATE_COUNT_26_1_2);

    // The block-state table and the block registry agree entry-by-entry.
    for (entry, registry) in entries
        .iter()
        .zip(crate::block_metadata::BLOCK_REGISTRY.iter())
    {
        assert_eq!(entry.registry_id, registry.registry_id);
    }
}

#[test]
fn every_official_block_state_round_trips_against_the_vanilla_report() {
    // Full pin against the official server's data generator output: every one of
    // the 29873 states must resolve name->id and id->properties exactly.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("vanilla-data/reports/blocks_26_1_2.json");
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("vendored blocks report"))
            .expect("valid blocks report JSON");

    let mut checked_states = 0_usize;
    let mut checked_blocks = 0_usize;
    for (name, block) in report.as_object().expect("report object") {
        let entry = block_state_entry(name).unwrap_or_else(|| panic!("missing {name}"));
        checked_blocks += 1;
        for state in block["states"].as_array().expect("states array") {
            let id = state["id"].as_i64().expect("state id") as i32;
            if state["default"].as_bool().unwrap_or(false) {
                assert_eq!(entry.default_state_id, id, "{name} default");
                assert_eq!(network_id_for_block_state(name), Some(id), "{name} bare");
            }

            let empty = serde_json::Map::new();
            let properties = state["properties"].as_object().unwrap_or(&empty);
            if properties.is_empty() {
                assert_eq!(entry.base_state_id, id, "{name} singleton");
                assert_eq!(
                    block_state_name_for_network_id(id).as_deref(),
                    Some(name.as_str())
                );
            } else {
                let rendered = properties
                    .iter()
                    .map(|(property, value)| {
                        format!("{property}={}", value.as_str().expect("string value"))
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let state_name = format!("{name}[{rendered}]");
                assert_eq!(
                    network_id_for_block_state(&state_name),
                    Some(id),
                    "{state_name}"
                );
                let round_trip = block_state_name_for_network_id(id)
                    .unwrap_or_else(|| panic!("no name for {id}"));
                assert_eq!(
                    network_id_for_block_state(&round_trip),
                    Some(id),
                    "{round_trip}"
                );
            }
            checked_states += 1;
        }
    }
    assert_eq!(checked_blocks, block_state_entries().len());
    assert_eq!(checked_states, VANILLA_BLOCK_STATE_COUNT_26_1_2);
}

#[test]
fn known_vanilla_state_ids_resolve_like_java() {
    // Spot pins from the official report, including states past the copper
    // bars/chain/lantern registry insertions that the old registry data missed.
    assert_eq!(network_id_for_block_state("minecraft:air"), Some(0));
    assert_eq!(network_id_for_block_state("minecraft:stone"), Some(1));
    assert_eq!(network_id_for_block_state("minecraft:water"), Some(86));
    assert_eq!(
        network_id_for_block_state("minecraft:water[level=7]"),
        Some(93)
    );
    assert_eq!(
        network_id_for_block_state("minecraft:lava[level=15]"),
        Some(117)
    );
    assert_eq!(network_id_for_block_state("minecraft:obsidian"), Some(3369));
    assert_eq!(
        network_id_for_block_state("minecraft:sunflower"),
        Some(12916)
    );
    assert_eq!(
        network_id_for_block_state("minecraft:grass_block[snowy=true]"),
        Some(8)
    );
    assert_eq!(
        network_id_for_block_state("minecraft:grass_block[snowy=false]"),
        Some(9)
    );
    assert_eq!(
        network_id_for_block_state("minecraft:chest[facing=north,type=single,waterlogged=false]"),
        network_id_for_block_state("minecraft:chest"),
    );
}

#[test]
fn block_registry_network_ids_match_vanilla_block_registry_order() {
    assert_eq!(block_registry_network_id("minecraft:air"), Some(0));
    assert_eq!(block_registry_network_id("minecraft:stone"), Some(1));
    assert_eq!(block_registry_network_id("minecraft:diamond_ore"), Some(203));
    assert_eq!(block_registry_network_id("stone"), Some(1));
}

#[test]
fn name_parsing_matches_nbt_utils_read_block_state_leniency() {
    let default = network_id_for_block_state("minecraft:oak_stairs").expect("default");

    // Unknown property: ignored, default kept.
    assert_eq!(
        network_id_for_block_state("minecraft:oak_stairs[bogus=true]"),
        Some(default)
    );
    // Invalid value for a known property: ignored, default kept.
    assert_eq!(
        network_id_for_block_state("minecraft:oak_stairs[facing=up]"),
        Some(default)
    );
    // Missing properties keep the default-state values; order is irrelevant.
    assert_eq!(
        network_id_for_block_state("minecraft:oak_stairs[half=bottom,facing=north]"),
        Some(default)
    );
    assert_eq!(
        network_id_for_block_state("minecraft:oak_stairs[facing=north,half=bottom]"),
        Some(default)
    );
    // Whitespace around pairs is tolerated.
    assert_eq!(
        network_id_for_block_state("minecraft:oak_stairs[ facing = north , half = bottom ]"),
        Some(default)
    );
    // Bare path resolves in the minecraft namespace.
    assert_eq!(network_id_for_block_state("oak_stairs"), Some(default));
    // Unknown blocks fail instead of guessing.
    assert_eq!(network_id_for_block_state("minecraft:not_a_block"), None);
    assert_eq!(
        network_id_for_block_state("minecraft:not_a_block[facing=north]"),
        None
    );
}

#[test]
fn network_id_reverse_lookup_decodes_owning_block_and_properties() {
    assert_eq!(
        block_state_name_for_network_id(0).as_deref(),
        Some("minecraft:air")
    );
    let stairs_default = network_id_for_block_state("minecraft:oak_stairs").expect("default");
    assert_eq!(
        block_state_name_for_network_id(stairs_default).as_deref(),
        Some("minecraft:oak_stairs[facing=north,half=bottom,shape=straight,waterlogged=false]")
    );
    assert_eq!(
        block_state_entry_for_network_id(stairs_default).map(|entry| entry.registry_id),
        Some("minecraft:oak_stairs")
    );
    assert!(block_state_name_for_network_id(-1).is_none());
    assert!(block_state_name_for_network_id(VANILLA_BLOCK_STATE_COUNT_26_1_2 as i32).is_none());
    assert!(
        block_state_name_for_network_id((VANILLA_BLOCK_STATE_COUNT_26_1_2 - 1) as i32).is_some()
    );
}

#[test]
fn default_state_properties_match_java_default_block_states() {
    // Java: Blocks.OAK_STAIRS default = north/bottom/straight/dry.
    assert_eq!(
        default_state_properties("minecraft:oak_stairs"),
        Some(vec![
            ("facing", "north"),
            ("half", "bottom"),
            ("shape", "straight"),
            ("waterlogged", "false"),
        ])
    );
    // Singleton blocks expose an empty property list.
    assert_eq!(default_state_properties("minecraft:stone"), Some(vec![]));
    // Java: SnowLayerBlock default LAYERS = 1.
    assert_eq!(
        default_state_properties("minecraft:snow"),
        Some(vec![("layers", "1")])
    );
    assert_eq!(default_state_properties("minecraft:not_a_block"), None);
}
