pub const GAMETEST_MAIN_UTIL_SERVER_RUNTIME_TODO: &str = "gametest-main-util-server-runtime";
pub const DEFAULT_GAMETEST_UNIVERSE_DIR: &str = "gametestserver";
pub const GAMETEST_LEVEL_NAME: &str = "gametestworld";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestMainOptionsModel {
    pub universe: String,
    pub report: Option<String>,
    pub tests: Option<String>,
    pub verify: bool,
    pub repeat_count: i32,
    pub packs: Option<String>,
    pub help: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameTestMainDecision {
    PrintHelp,
    VerifyRequiresTests {
        exit_code: i32,
        message: &'static str,
    },
    Launch(GameTestServerLaunchModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTestServerLaunchModel {
    pub universe_path: String,
    pub level_name: &'static str,
    pub report: Option<String>,
    pub tests: Option<String>,
    pub verify: bool,
    pub repeat_count: i32,
    pub packs: Option<String>,
    pub verify_ignores_repeat_count: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackSourceEntryModel {
    pub name: String,
    pub is_directory: bool,
    pub has_pack_mcmeta: bool,
    pub is_zip: bool,
}

impl GameTestMainOptionsModel {
    pub fn parse(args: &[&str]) -> Result<Self, String> {
        let mut options = Self {
            universe: DEFAULT_GAMETEST_UNIVERSE_DIR.to_string(),
            report: None,
            tests: None,
            verify: false,
            repeat_count: 1,
            packs: None,
            help: false,
        };

        let mut index = 0;
        while index < args.len() {
            match args[index] {
                "--help" => options.help = true,
                "--universe" => {
                    index += 1;
                    options.universe = required_arg(args, index, "--universe")?.to_string();
                }
                "--report" => {
                    index += 1;
                    options.report = Some(required_arg(args, index, "--report")?.to_string());
                }
                "--tests" => {
                    index += 1;
                    options.tests = Some(required_arg(args, index, "--tests")?.to_string());
                }
                "--verify" => {
                    index += 1;
                    options.verify = parse_bool(required_arg(args, index, "--verify")?)?;
                }
                "--repeatCount" => {
                    index += 1;
                    options.repeat_count = required_arg(args, index, "--repeatCount")?
                        .parse::<i32>()
                        .map_err(|err| format!("invalid --repeatCount: {err}"))?;
                }
                "--packs" => {
                    index += 1;
                    options.packs = Some(required_arg(args, index, "--packs")?.to_string());
                }
                _ => {}
            }
            index += 1;
        }

        Ok(options)
    }

    pub fn decision(&self, repeat_count_was_present: bool) -> GameTestMainDecision {
        if self.help {
            return GameTestMainDecision::PrintHelp;
        }
        if self.verify && self.tests.is_none() {
            return GameTestMainDecision::VerifyRequiresTests {
                exit_code: -1,
                message: "Please specify a test selection to run the verify option. For example: --verify --tests example:test_something_*",
            };
        }

        GameTestMainDecision::Launch(GameTestServerLaunchModel {
            universe_path: self.universe.clone(),
            level_name: GAMETEST_LEVEL_NAME,
            report: self.report.clone(),
            tests: self.tests.clone(),
            verify: self.verify,
            repeat_count: self.repeat_count,
            packs: self.packs.clone(),
            verify_ignores_repeat_count: self.verify && repeat_count_was_present,
        })
    }
}

pub fn gametest_main_pack_copy_targets(entries: &[PackSourceEntryModel]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| (entry.is_directory && entry.has_pack_mcmeta) || entry.is_zip)
        .map(|entry| entry.name.clone())
        .collect()
}

fn required_arg<'a>(args: &'a [&str], index: usize, option: &str) -> Result<&'a str, String> {
    args.get(index)
        .copied()
        .ok_or_else(|| format!("{option} requires an argument"))
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("invalid boolean {other}")),
    }
}
