#![cfg(test)]
#![allow(dead_code)]

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::Duration;

    use crate::statistics::StatisticsCounter;
    use crate::storage::datafix::TARGET_DATA_VERSION;
    use crate::storage::nbt::{read_named_tag, Tag};
    use crate::storage::world::WorldLayout;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CrashRecoveryCase {
        name: &'static str,
        trigger: &'static str,
        expected_recovery: &'static str,
    }

    fn crash_recovery_matrix() -> Vec<CrashRecoveryCase> {
        vec![
            CrashRecoveryCase {
                name: "killed process",
                trigger: "server process receives SIGKILL before clean shutdown",
                expected_recovery:
                    "restart reuses durable files and ignores incomplete temp writes",
            },
            CrashRecoveryCase {
                name: "corrupt level.dat",
                trigger: "primary level.dat cannot be decoded",
                expected_recovery: "load level.dat_old backup",
            },
            CrashRecoveryCase {
                name: "corrupt player data",
                trigger: "player .dat payload cannot be decoded",
                expected_recovery: "surface an error without panicking or deleting the file",
            },
            CrashRecoveryCase {
                name: "corrupt saved data",
                trigger: "saved data NBT payload cannot be decoded",
                expected_recovery: "surface an error without panicking or accepting partial state",
            },
            CrashRecoveryCase {
                name: "corrupt stats json",
                trigger: "stats sidecar is malformed json",
                expected_recovery: "surface an error without fabricating counters",
            },
        ]
    }

    #[test]
    fn crash_recovery_matrix_covers_killed_process_and_corrupted_inputs() {
        let cases = crash_recovery_matrix();
        assert!(cases.iter().any(|case| case.name == "killed process"));
        assert!(cases.iter().any(|case| case.name == "corrupt level.dat"));
        assert!(cases.iter().any(|case| case.name == "corrupt player data"));
        assert!(cases.iter().any(|case| case.name == "corrupt saved data"));
        assert!(cases.iter().any(|case| case.name == "corrupt stats json"));
    }

    #[test]
    fn killed_process_leaves_last_durable_world_files_recoverable() {
        let path = temp_root("vibecraft-crash-kill");
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let layout = WorldLayout::new(&path);
        let first = level_tag(4790);
        let second = level_tag(TARGET_DATA_VERSION);
        layout.save_level_dat(&first).unwrap();
        layout.save_level_dat(&second).unwrap();

        let pending_tmp = layout.level_dat().with_extension("tmp");
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "printf corrupt-partial > {}; sleep 30",
                shell_escape(&pending_tmp.to_string_lossy())
            ))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        child.kill().unwrap();
        let _ = child.wait();

        assert!(pending_tmp.is_file());
        assert_eq!(layout.load_level_dat_with_backup().unwrap(), second);
        fs::remove_file(&pending_tmp).unwrap();
        assert_eq!(layout.load_level_dat_with_backup().unwrap(), second);

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn corrupted_world_inputs_error_or_fallback_without_panics() {
        let path = temp_root("vibecraft-crash-corrupt");
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();

        let layout = WorldLayout::new(&path);
        let backup = level_tag(4790);
        layout.save_level_dat(&backup).unwrap();
        layout
            .save_level_dat(&level_tag(TARGET_DATA_VERSION))
            .unwrap();
        fs::write(layout.level_dat(), b"not nbt").unwrap();
        assert_eq!(layout.load_level_dat_with_backup().unwrap(), backup);

        let uuid = "00000000-0000-0000-0000-000000000001";
        fs::create_dir_all(layout.playerdata_dir()).unwrap();
        fs::write(layout.player_data_file(uuid), b"corrupt player").unwrap();
        assert!(layout.load_player_data(uuid).is_err());
        assert!(layout.player_data_file(uuid).is_file());

        fs::create_dir_all(layout.data_dir()).unwrap();
        fs::write(layout.saved_data_file("scoreboard"), b"corrupt scoreboard").unwrap();
        assert!(layout.load_scoreboard().is_err());

        let _ = fs::remove_dir_all(&path);
    }

    #[test]
    fn corrupted_parsers_reject_inputs_without_partial_state() {
        let mut corrupt_nbt = &b"\x0a\x00\x04root\x08\x00\x04Name\x00"[..];
        assert!(read_named_tag(&mut corrupt_nbt).is_err());
        assert!(StatisticsCounter::from_vanilla_json("{\"stats\":").is_err());
    }

    fn level_tag(version: i32) -> Tag {
        Tag::Compound(vec![("DataVersion".to_string(), Tag::Int(version))])
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("{}-{}", name, std::process::id()))
    }

    fn shell_escape(value: &str) -> String {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
