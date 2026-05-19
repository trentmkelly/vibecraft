use std::backtrace::Backtrace;
use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let report = CrashReport::from_panic(info);
        match report.write_to_dir(Path::new("crash-reports")) {
            Ok(path) => eprintln!("Saved crash report to {}", path.display()),
            Err(err) => eprintln!("Failed to save crash report: {err}"),
        }
        eprintln!("{}", report.summary());
    }));
}

#[derive(Debug, Clone)]
pub struct CrashReport {
    title: String,
    details: Vec<(String, String)>,
    backtrace: String,
}

impl CrashReport {
    pub fn from_panic(info: &panic::PanicHookInfo<'_>) -> Self {
        let payload = if let Some(message) = info.payload().downcast_ref::<&str>() {
            (*message).to_string()
        } else if let Some(message) = info.payload().downcast_ref::<String>() {
            message.clone()
        } else {
            "unknown panic payload".to_string()
        };

        let location = info
            .location()
            .map(|location| {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            })
            .unwrap_or_else(|| "unknown".to_string());
        let thread = thread::current()
            .name()
            .map(str::to_string)
            .unwrap_or_else(|| "unnamed".to_string());

        Self {
            title: payload,
            details: vec![
                (
                    "RustCraft Version".to_string(),
                    env!("CARGO_PKG_VERSION").to_string(),
                ),
                (
                    "Minecraft Target".to_string(),
                    "Java Edition 26.1.2".to_string(),
                ),
                ("Process ID".to_string(), std::process::id().to_string()),
                ("Thread".to_string(), thread),
                ("Location".to_string(), location),
                ("OS".to_string(), std::env::consts::OS.to_string()),
                (
                    "Architecture".to_string(),
                    std::env::consts::ARCH.to_string(),
                ),
                (
                    "World State".to_string(),
                    "runtime world loading not implemented".to_string(),
                ),
            ],
            backtrace: Backtrace::force_capture().to_string(),
        }
    }

    pub fn summary(&self) -> String {
        format!("RustCraft crashed: {}", self.title)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("---- RustCraft Crash Report ----\n");
        out.push_str("// This report is generated when RustCraft panics.\n\n");
        out.push_str("Description: ");
        out.push_str(&self.title);
        out.push_str("\n\n");
        out.push_str("-- System Details --\n");
        for (key, value) in &self.details {
            out.push_str(key);
            out.push_str(": ");
            out.push_str(value);
            out.push('\n');
        }
        out.push_str("\n-- Backtrace --\n");
        out.push_str(&self.backtrace);
        out
    }

    pub fn write_to_dir(&self, dir: &Path) -> std::io::Result<PathBuf> {
        fs::create_dir_all(dir)?;
        let path = dir.join(format!("crash-{}-server.txt", timestamp()));
        fs::write(&path, self.render())?;
        Ok(path)
    }
}

fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::CrashReport;
    use std::fs;

    #[test]
    fn rendered_crash_report_contains_required_sections() {
        let report = CrashReport {
            title: "boom".to_string(),
            details: vec![
                ("Thread".to_string(), "main".to_string()),
                ("World State".to_string(), "not loaded".to_string()),
            ],
            backtrace: "trace".to_string(),
        };

        let rendered = report.render();
        assert!(rendered.contains("---- RustCraft Crash Report ----"));
        assert!(rendered.contains("Description: boom"));
        assert!(rendered.contains("-- System Details --"));
        assert!(rendered.contains("World State: not loaded"));
        assert!(rendered.contains("-- Backtrace --"));
    }

    #[test]
    fn writes_vanilla_named_server_crash_report_file() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rustcraft-crash-report-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let report = CrashReport {
            title: "boom".to_string(),
            details: vec![("Thread".to_string(), "main".to_string())],
            backtrace: "trace".to_string(),
        };

        let path = report.write_to_dir(&dir).unwrap();
        let file_name = path.file_name().unwrap().to_string_lossy();
        assert!(file_name.starts_with("crash-"));
        assert!(file_name.ends_with("-server.txt"));
        assert!(fs::read_to_string(path)
            .unwrap()
            .contains("Description: boom"));

        let _ = fs::remove_dir_all(&dir);
    }
}
