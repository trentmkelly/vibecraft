#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityHarnessConfig {
    pub case_name: String,
    pub seed: i64,
    pub port_base: u16,
    pub minecraft_version: String,
    pub official_jar: PathBuf,
    pub vibecraft_bin: PathBuf,
    pub root: PathBuf,
    pub extra_properties: Vec<(String, String)>,
}

impl ParityHarnessConfig {
    pub fn default_in_workspace(
        workspace: impl AsRef<Path>,
        vibecraft_bin: impl Into<PathBuf>,
    ) -> Self {
        let workspace = workspace.as_ref();
        Self {
            case_name: "parity".to_string(),
            seed: 8675309,
            port_base: 25_565,
            minecraft_version: "26.1.2".to_string(),
            official_jar: workspace.join("server.jar"),
            vibecraft_bin: vibecraft_bin.into(),
            root: workspace.join("parity-runs"),
            extra_properties: Vec::new(),
        }
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_properties.push((key.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityCase {
    pub official: ServerLaunch,
    pub rebuilt: ServerLaunch,
    pub shared_seed: i64,
    pub shared_properties: Vec<(String, String)>,
}

impl ParityCase {
    pub fn prepare(config: &ParityHarnessConfig) -> io::Result<Self> {
        let official_dir = config.root.join(&config.case_name).join("official");
        let rebuilt_dir = config.root.join(&config.case_name).join("vibecraft");
        fs::create_dir_all(&official_dir)?;
        fs::create_dir_all(&rebuilt_dir)?;
        let properties = shared_properties(config);
        write_server_files(&official_dir, &properties, config.port_base)?;
        write_server_files(&rebuilt_dir, &properties, config.port_base + 1)?;
        Ok(Self {
            official: ServerLaunch {
                kind: ServerKind::OfficialJar,
                work_dir: official_dir,
                executable: PathBuf::from("java"),
                args: vec![
                    "-Xms512M".to_string(),
                    "-Xmx512M".to_string(),
                    "-jar".to_string(),
                    config.official_jar.display().to_string(),
                    "--nogui".to_string(),
                ],
                port: config.port_base,
                version: config.minecraft_version.clone(),
            },
            rebuilt: ServerLaunch {
                kind: ServerKind::VibeCraft,
                work_dir: rebuilt_dir,
                executable: config.vibecraft_bin.clone(),
                args: vec!["--nogui".to_string()],
                port: config.port_base + 1,
                version: config.minecraft_version.clone(),
            },
            shared_seed: config.seed,
            shared_properties: properties,
        })
    }

    pub fn spawn_pair(&self) -> io::Result<RunningParityPair> {
        Ok(RunningParityPair {
            official: self.official.spawn()?,
            rebuilt: self.rebuilt.spawn()?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerKind {
    OfficialJar,
    VibeCraft,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerLaunch {
    pub kind: ServerKind,
    pub work_dir: PathBuf,
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub port: u16,
    pub version: String,
}

impl ServerLaunch {
    pub fn command_line(&self) -> Vec<String> {
        let mut command = vec![self.executable.display().to_string()];
        command.extend(self.args.clone());
        command
    }

    pub fn spawn(&self) -> io::Result<Child> {
        Command::new(&self.executable)
            .args(&self.args)
            .current_dir(&self.work_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

#[derive(Debug)]
pub struct RunningParityPair {
    pub official: Child,
    pub rebuilt: Child,
}

impl RunningParityPair {
    pub fn terminate(mut self) {
        terminate_child(&mut self.official);
        terminate_child(&mut self.rebuilt);
    }
}

pub fn create_unique_run_root(base: impl AsRef<Path>, prefix: &str) -> PathBuf {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    base.as_ref()
        .join(format!("{prefix}-{millis}-{}", std::process::id()))
}

pub fn normalize_log_line(line: &str, root: &Path, port: u16) -> String {
    line.replace(&root.display().to_string(), "<run-dir>")
        .replace(&port.to_string(), "<port>")
}

fn shared_properties(config: &ParityHarnessConfig) -> Vec<(String, String)> {
    let mut properties = vec![
        ("eula".to_string(), "true".to_string()),
        ("online-mode".to_string(), "false".to_string()),
        ("enforce-secure-profile".to_string(), "false".to_string()),
        ("level-seed".to_string(), config.seed.to_string()),
        ("level-name".to_string(), "world".to_string()),
        (
            "motd".to_string(),
            format!("VibeCraft parity {}", config.minecraft_version),
        ),
    ];
    for (key, value) in &config.extra_properties {
        upsert_property(&mut properties, key, value);
    }
    properties
}

fn upsert_property(properties: &mut Vec<(String, String)>, key: &str, value: &str) {
    if let Some((_, existing)) = properties.iter_mut().find(|(entry, _)| entry == key) {
        *existing = value.to_string();
    } else {
        properties.push((key.to_string(), value.to_string()));
    }
}

fn write_server_files(dir: &Path, properties: &[(String, String)], port: u16) -> io::Result<()> {
    fs::write(dir.join("eula.txt"), "eula=true\n")?;
    let mut effective = properties.to_vec();
    upsert_property(&mut effective, "server-port", &port.to_string());
    let body = effective
        .iter()
        .filter(|(key, _)| key != "eula")
        .map(|(key, value)| format!("{key}={value}\n"))
        .collect::<String>();
    fs::write(dir.join("server.properties"), body)
}

fn terminate_child(child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_root(name: &str) -> PathBuf {
        create_unique_run_root(std::env::temp_dir(), name)
    }

    #[test]
    fn prepares_official_and_rebuilt_dirs_with_same_seed_and_properties_except_ports() {
        let root = temp_root("vibecraft-parity-prepare");
        let config = ParityHarnessConfig {
            case_name: "login".to_string(),
            seed: 12345,
            port_base: 31_000,
            minecraft_version: "26.1.2".to_string(),
            official_jar: PathBuf::from("/workspace/server.jar"),
            vibecraft_bin: PathBuf::from("/workspace/VibeCraft/target/debug/vibecraft"),
            root: root.clone(),
            extra_properties: vec![("difficulty".to_string(), "hard".to_string())],
        };
        let case = ParityCase::prepare(&config).unwrap();
        assert_eq!(case.shared_seed, 12345);
        assert_eq!(case.official.port, 31_000);
        assert_eq!(case.rebuilt.port, 31_001);
        let official_props =
            fs::read_to_string(case.official.work_dir.join("server.properties")).unwrap();
        let rebuilt_props =
            fs::read_to_string(case.rebuilt.work_dir.join("server.properties")).unwrap();
        assert_ne!(official_props, rebuilt_props);
        assert_eq!(
            official_props.replace("server-port=31000", "server-port=<port>"),
            rebuilt_props.replace("server-port=31001", "server-port=<port>")
        );
        assert!(official_props.contains("level-seed=12345\n"));
        assert!(official_props.contains("online-mode=false\n"));
        assert!(official_props.contains("difficulty=hard\n"));
        assert_eq!(
            fs::read_to_string(case.official.work_dir.join("eula.txt")).unwrap(),
            "eula=true\n"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn command_lines_launch_java_jar_and_vibecraft_binary_from_separate_workdirs() {
        let root = temp_root("vibecraft-parity-command");
        let config = ParityHarnessConfig::default_in_workspace(
            "/workspace",
            "/workspace/VibeCraft/target/debug/vibecraft",
        );
        let config = ParityHarnessConfig {
            root: root.clone(),
            ..config
        };
        let case = ParityCase::prepare(&config).unwrap();
        assert_eq!(
            case.official.command_line(),
            vec![
                "java",
                "-Xms512M",
                "-Xmx512M",
                "-jar",
                "/workspace/server.jar",
                "--nogui"
            ]
        );
        assert_eq!(
            case.rebuilt.command_line(),
            vec!["/workspace/VibeCraft/target/debug/vibecraft", "--nogui"]
        );
        assert_ne!(case.official.work_dir, case.rebuilt.work_dir);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn properties_are_upserted_so_scenarios_can_override_defaults() {
        let root = temp_root("vibecraft-parity-upsert");
        let config = ParityHarnessConfig {
            root: root.clone(),
            port_base: 32_000,
            ..ParityHarnessConfig::default_in_workspace(
                "/workspace",
                "/workspace/VibeCraft/target/debug/vibecraft",
            )
            .with_property("level-name", "same-world")
        };
        let case = ParityCase::prepare(&config).unwrap();
        let props = fs::read_to_string(case.official.work_dir.join("server.properties")).unwrap();
        assert_eq!(props.matches("server-port=").count(), 1);
        assert!(props.contains("server-port=32000\n"));
        assert!(props.contains("level-name=same-world\n"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn log_normalizer_redacts_run_dir_and_ports_for_diffable_artifacts() {
        let root = PathBuf::from("/tmp/vibecraft-parity-123");
        assert_eq!(
            normalize_log_line(
                "[Server thread/INFO]: Starting /tmp/vibecraft-parity-123 on port 25565",
                &root,
                25565,
            ),
            "[Server thread/INFO]: Starting <run-dir> on port <port>"
        );
    }
}
