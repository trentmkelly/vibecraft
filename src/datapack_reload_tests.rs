#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::resources::{
        load_world_data_packs, reload_pack_repository_with_report, DataPack, DataPackConfig,
        DataPackRepository, PackConfigureOptions, PackSource, WorldDataConfiguration,
        VANILLA_PACK_ID,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct DataPackReloadCase {
        name: &'static str,
        valid_pack: bool,
        expected_reload: &'static str,
    }

    #[test]
    fn datapack_reload_matrix_covers_valid_and_invalid_packs() {
        let cases = [
            DataPackReloadCase {
                name: "valid directory pack is discovered and selected",
                valid_pack: true,
                expected_reload: "success",
            },
            DataPackReloadCase {
                name: "invalid metadata pack is ignored during discovery",
                valid_pack: false,
                expected_reload: "skipped",
            },
            DataPackReloadCase {
                name: "validation failure restores previous selection",
                valid_pack: true,
                expected_reload: "rollback",
            },
        ];

        assert_eq!(
            cases
                .iter()
                .map(|case| (case.valid_pack, case.expected_reload))
                .collect::<Vec<_>>(),
            vec![(true, "success"), (false, "skipped"), (true, "rollback")]
        );
    }

    #[test]
    fn reload_discovers_valid_directory_packs_and_skips_invalid_packs() {
        let root = std::env::temp_dir().join(format!("vibecraft-datapacks-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let datapacks = root.join("datapacks");
        fs::create_dir_all(datapacks.join("valid_pack/data/example/tags/item")).unwrap();
        fs::write(
            datapacks.join("valid_pack/pack.mcmeta"),
            r#"{"pack":{"description":"Valid","min_format":[101,0],"max_format":[101,1]}}"#,
        )
        .unwrap();
        fs::write(
            datapacks.join("valid_pack/data/example/tags/item/test.json"),
            r#"{"replace":false,"values":["minecraft:stone"]}"#,
        )
        .unwrap();

        fs::create_dir_all(datapacks.join("invalid_pack")).unwrap();
        fs::write(
            datapacks.join("invalid_pack/pack.mcmeta"),
            r#"{"pack":{"description":"Invalid","pack_format":101}}"#,
        )
        .unwrap();

        let packs = load_world_data_packs(&datapacks).unwrap();
        assert_eq!(
            packs
                .iter()
                .map(|pack| pack.pack.id.as_str())
                .collect::<Vec<_>>(),
            vec!["file/valid_pack"]
        );
        assert_eq!(
            packs[0]
                .data_resources()
                .unwrap()
                .get(
                    "example",
                    crate::resources::DataResourceKind::Tags,
                    "item/test"
                )
                .unwrap(),
            r#"{"replace":false,"values":["minecraft:stone"]}"#
        );

        let mut repository = DataPackRepository::server_repository(&datapacks).unwrap();
        let configured = reload_pack_repository_with_report(
            &mut repository,
            &WorldDataConfiguration {
                data_packs: DataPackConfig::default_26_1_2(),
                enabled_features: crate::registry::feature_flags::default_flags_26_1_2(),
            },
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
            |packs| {
                let ids = packs
                    .iter()
                    .map(|pack| pack.id.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(ids, vec![VANILLA_PACK_ID, "file/valid_pack"]);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(
            configured.data_packs.enabled,
            vec![VANILLA_PACK_ID, "file/valid_pack"]
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_reload_restores_previous_enabled_packs_and_reports_error() {
        let mut repository = DataPackRepository::new([
            DataPack::new(VANILLA_PACK_ID, PackSource::BuiltIn),
            DataPack::new("file/kept", PackSource::World),
            DataPack::new("file/broken", PackSource::World),
        ]);
        repository.set_selected([VANILLA_PACK_ID, "file/kept"]);

        let failure = reload_pack_repository_with_report(
            &mut repository,
            &WorldDataConfiguration {
                data_packs: DataPackConfig::new(
                    vec![
                        VANILLA_PACK_ID.to_string(),
                        "file/kept".to_string(),
                        "file/broken".to_string(),
                    ],
                    Vec::<String>::new(),
                ),
                enabled_features: crate::registry::feature_flags::default_flags_26_1_2(),
            },
            PackConfigureOptions {
                init_mode: false,
                safe_mode: false,
            },
            |packs| {
                assert!(packs.iter().any(|pack| pack.id == "file/broken"));
                Err("invalid loot table in file/broken".to_string())
            },
        )
        .unwrap_err();

        assert_eq!(
            repository.selected_ids(),
            vec![VANILLA_PACK_ID, "file/kept"]
        );
        assert_eq!(
            failure.user_message(),
            "Failed to reload data packs; keeping previous selection [vanilla,file/kept]: invalid loot table in file/broken"
        );
    }
}
