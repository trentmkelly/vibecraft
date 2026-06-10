use std::path::PathBuf;

use crate::log::LogLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliOptions {
    pub nogui: bool,
    pub init_settings: bool,
    pub demo: bool,
    pub bonus_chest: bool,
    pub force_upgrade: bool,
    pub erase_cache: bool,
    pub recreate_region_files: bool,
    pub report: bool,
    pub safe_mode: bool,
    pub help: bool,
    pub universe: PathBuf,
    pub world: Option<String>,
    pub port: i32,
    pub server_id: Option<String>,
    pub jfr_profile: bool,
    pub pid_file: Option<PathBuf>,
    /// Overrides the default log level.  `None` means fall back to the
    /// `VIBECRAFT_LOG` environment variable, then `LogLevel::Info`.
    pub log_level: Option<LogLevel>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            nogui: false,
            init_settings: false,
            demo: false,
            bonus_chest: false,
            force_upgrade: false,
            erase_cache: false,
            recreate_region_files: false,
            report: false,
            safe_mode: false,
            help: false,
            universe: PathBuf::from("."),
            world: None,
            port: -1,
            server_id: None,
            jfr_profile: false,
            pid_file: None,
            log_level: None,
        }
    }
}

impl CliOptions {
    pub fn parse<I>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = String>,
    {
        let mut options = Self::default();
        let mut iter = args.into_iter();

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--nogui" | "nogui" => options.nogui = true,
                "--initSettings" => options.init_settings = true,
                "--demo" => options.demo = true,
                "--bonusChest" => options.bonus_chest = true,
                "--forceUpgrade" => options.force_upgrade = true,
                "--eraseCache" => options.erase_cache = true,
                "--recreateRegionFiles" => options.recreate_region_files = true,
                "--report" | "--reports" => options.report = true,
                "--safeMode" => options.safe_mode = true,
                "--help" | "-h" => options.help = true,
                "--jfrProfile" => options.jfr_profile = true,
                "--universe" => {
                    options.universe = PathBuf::from(required_value("--universe", &mut iter)?)
                }
                "--world" => options.world = Some(required_value("--world", &mut iter)?),
                "--port" => {
                    let raw = required_value("--port", &mut iter)?;
                    options.port = raw
                        .parse::<i32>()
                        .map_err(|_| format!("Invalid integer for --port: {raw}"))?;
                }
                "--serverId" => options.server_id = Some(required_value("--serverId", &mut iter)?),
                "--pidFile" => {
                    options.pid_file = Some(PathBuf::from(required_value("--pidFile", &mut iter)?))
                }
                "--log-level" => {
                    let raw = required_value("--log-level", &mut iter)?;
                    options.log_level = Some(
                        LogLevel::from_str(&raw)
                            .map_err(|_| format!("Invalid value for --log-level: {raw:?}. Expected info, debug, or trace."))?,
                    );
                }
                _ if arg.starts_with("--") => return Err(format!("Unknown option: {arg}")),
                _ => return Err(format!("Unexpected positional argument: {arg}")),
            }
        }

        Ok(options)
    }

    pub fn help() -> &'static str {
        "Usage: vibecraft [options]\n\
         \n\
         Options:\n\
           --nogui\n\
           --initSettings\n\
           --demo\n\
           --bonusChest\n\
           --forceUpgrade\n\
           --eraseCache\n\
           --recreateRegionFiles\n\
           --report\n\
           --safeMode\n\
           --help\n\
           --universe <path>\n\
           --world <name>\n\
           --port <port>\n\
           --serverId <id>\n\
           --jfrProfile (accepted; profiling is not implemented by VibeCraft)\n\
           --pidFile <path>\n\
           --log-level <info|debug|trace>  (overrides VIBECRAFT_LOG env var)"
    }
}

fn required_value<I>(name: &str, iter: &mut I) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    iter.next()
        .filter(|value| !value.starts_with("--"))
        .ok_or_else(|| format!("Missing required value for {name}"))
}

#[cfg(test)]
mod tests {
    use super::CliOptions;
    use std::path::PathBuf;

    #[test]
    fn parses_vanilla_main_options() {
        let options = CliOptions::parse(
            [
                "--nogui",
                "--initSettings",
                "--demo",
                "--bonusChest",
                "--forceUpgrade",
                "--eraseCache",
                "--recreateRegionFiles",
                "--report",
                "--safeMode",
                "--universe",
                "worlds",
                "--world",
                "test",
                "--port",
                "25566",
                "--serverId",
                "abc",
                "--jfrProfile",
                "--pidFile",
                "server.pid",
            ]
            .into_iter()
            .map(String::from),
        )
        .unwrap();

        assert!(options.nogui);
        assert!(options.init_settings);
        assert!(options.demo);
        assert!(options.bonus_chest);
        assert!(options.force_upgrade);
        assert!(options.erase_cache);
        assert!(options.recreate_region_files);
        assert!(options.report);
        assert!(options.safe_mode);
        assert_eq!(options.universe, PathBuf::from("worlds"));
        assert_eq!(options.world.as_deref(), Some("test"));
        assert_eq!(options.port, 25566);
        assert_eq!(options.server_id.as_deref(), Some("abc"));
        assert!(options.jfr_profile);
        assert_eq!(options.pid_file, Some(PathBuf::from("server.pid")));
    }

    #[test]
    fn parses_legacy_positional_nogui_like_java_main() {
        let options = CliOptions::parse(["nogui"].into_iter().map(String::from)).unwrap();
        assert!(options.nogui);
    }

    #[test]
    fn rejects_unknown_options() {
        let err = CliOptions::parse(["--wat"].into_iter().map(String::from)).unwrap_err();
        assert!(err.contains("Unknown option"));
    }

    #[test]
    fn help_documents_jfr_profile_as_noop() {
        assert!(CliOptions::help()
            .contains("--jfrProfile (accepted; profiling is not implemented by VibeCraft)"));
    }

    #[test]
    fn help_flag_parses_and_documents_vanilla_main_options() {
        let options = CliOptions::parse(["--help"].into_iter().map(String::from)).unwrap();
        assert!(options.help);

        let help = CliOptions::help();
        for flag in [
            "--nogui",
            "--initSettings",
            "--demo",
            "--bonusChest",
            "--forceUpgrade",
            "--eraseCache",
            "--recreateRegionFiles",
            "--safeMode",
            "--help",
            "--universe <path>",
            "--world <name>",
            "--port <port>",
            "--serverId <id>",
            "--jfrProfile",
            "--pidFile <path>",
        ] {
            assert!(
                help.contains(flag),
                "help output should document Java Main flag {flag}"
            );
        }
    }

    #[test]
    fn log_level_flag_parses_all_variants() {
        use crate::log::LogLevel;

        for (flag_val, expected) in &[
            ("info", LogLevel::Info),
            ("debug", LogLevel::Debug),
            ("trace", LogLevel::Trace),
            ("INFO", LogLevel::Info),
            ("Debug", LogLevel::Debug),
            ("TRACE", LogLevel::Trace),
        ] {
            let options =
                CliOptions::parse(["--log-level", flag_val].into_iter().map(String::from))
                    .unwrap_or_else(|e| panic!("parse failed for {flag_val:?}: {e}"));
            assert_eq!(
                options.log_level.as_ref(),
                Some(expected),
                "wrong level for {flag_val:?}"
            );
        }
    }

    #[test]
    fn log_level_defaults_to_none() {
        let options = CliOptions::default();
        assert_eq!(options.log_level, None);
    }

    #[test]
    fn log_level_flag_rejects_unknown_value() {
        let err = CliOptions::parse(["--log-level", "verbose"].into_iter().map(String::from))
            .unwrap_err();
        assert!(
            err.contains("verbose"),
            "error message should mention the bad value; got: {err}"
        );
    }

    #[test]
    fn help_documents_log_level_flag() {
        assert!(
            CliOptions::help().contains("--log-level"),
            "help text must document --log-level"
        );
    }
}
