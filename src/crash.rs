#![allow(dead_code)]

use std::backtrace::Backtrace;
use std::fmt;
use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::block_update::BlockPos;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaUncaughtExceptionModel {
    pub thread_name: String,
    pub throwable: JavaThrowableModel,
}

impl JavaUncaughtExceptionModel {
    pub fn new(thread_name: impl Into<String>, throwable: JavaThrowableModel) -> Self {
        Self {
            thread_name: thread_name.into(),
            throwable,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JavaUncaughtExceptionLogAction {
    WithThrowable {
        message: &'static str,
        throwable: JavaThrowableModel,
    },
    Message {
        message: &'static str,
    },
    ThreadThrowable {
        thread_name: String,
        throwable: JavaThrowableModel,
    },
}

pub fn default_uncaught_exception_handler_actions(
    exception: JavaUncaughtExceptionModel,
) -> Vec<JavaUncaughtExceptionLogAction> {
    vec![JavaUncaughtExceptionLogAction::WithThrowable {
        message: "Caught previously unhandled exception :",
        throwable: exception.throwable,
    }]
}

pub fn default_uncaught_exception_handler_with_name_actions(
    exception: JavaUncaughtExceptionModel,
) -> Vec<JavaUncaughtExceptionLogAction> {
    vec![
        JavaUncaughtExceptionLogAction::Message {
            message: "Caught previously unhandled exception :",
        },
        JavaUncaughtExceptionLogAction::ThreadThrowable {
            thread_name: exception.thread_name,
            throwable: exception.throwable,
        },
    ]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaLevelHeightAccessorModel {
    pub min_y: i32,
    pub max_y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaStackTraceElementModel {
    pub class_name: String,
    pub method_name: String,
    pub file_name: String,
    pub line_number: i32,
    pub native_method: bool,
}

impl JavaStackTraceElementModel {
    pub fn new(
        class_name: impl Into<String>,
        method_name: impl Into<String>,
        file_name: impl Into<String>,
        line_number: i32,
    ) -> Self {
        Self {
            class_name: class_name.into(),
            method_name: method_name.into(),
            file_name: file_name.into(),
            line_number,
            native_method: false,
        }
    }
}

impl fmt::Display for JavaStackTraceElementModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.native_method {
            write!(
                formatter,
                "{}.{}(Native Method)",
                self.class_name, self.method_name
            )
        } else {
            write!(
                formatter,
                "{}.{}({}:{})",
                self.class_name, self.method_name, self.file_name, self.line_number
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaThrowableModel {
    pub class_simple_name: String,
    pub message: Option<String>,
}

impl JavaThrowableModel {
    pub fn new(class_simple_name: impl Into<String>, message: Option<impl Into<String>>) -> Self {
        Self {
            class_simple_name: class_simple_name.into(),
            message: message.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashReportDetailValue {
    Null,
    Text(String),
    Throwable(JavaThrowableModel),
}

impl CrashReportDetailValue {
    fn entry_string(&self) -> String {
        match self {
            Self::Null => "~~NULL~~".to_string(),
            Self::Text(value) => value.clone(),
            Self::Throwable(throwable) => format!(
                "~~ERROR~~ {}: {}",
                throwable.class_simple_name,
                throwable.message.as_deref().unwrap_or("null")
            ),
        }
    }
}

impl From<&str> for CrashReportDetailValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<String> for CrashReportDetailValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<i32> for CrashReportDetailValue {
    fn from(value: i32) -> Self {
        Self::Text(value.to_string())
    }
}

impl From<JavaThrowableModel> for CrashReportDetailValue {
    fn from(value: JavaThrowableModel) -> Self {
        Self::Throwable(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportCategoryEntryModel {
    key: String,
    value: String,
}

impl CrashReportCategoryEntryModel {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportCategoryModel {
    title: String,
    entries: Vec<CrashReportCategoryEntryModel>,
    stack_trace: Vec<JavaStackTraceElementModel>,
}

impl CrashReportCategoryModel {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            entries: Vec::new(),
            stack_trace: Vec::new(),
        }
    }

    pub fn set_detail(&mut self, key: impl Into<String>, value: impl Into<CrashReportDetailValue>) {
        self.entries.push(CrashReportCategoryEntryModel {
            key: key.into(),
            value: value.into().entry_string(),
        });
    }

    pub fn set_detail_null(&mut self, key: impl Into<String>) {
        self.set_detail(key, CrashReportDetailValue::Null);
    }

    pub fn set_detail_callback(
        &mut self,
        key: impl Into<String>,
        callback: impl FnOnce() -> Result<CrashReportDetailValue, JavaThrowableModel>,
    ) {
        let key = key.into();
        match callback() {
            Ok(value) => self.set_detail(key, value),
            Err(error) => self.set_detail_error(key, error),
        }
    }

    pub fn set_detail_error(&mut self, key: impl Into<String>, error: JavaThrowableModel) {
        self.set_detail(key, error);
    }

    pub fn fill_in_stack_trace_from_thread_trace(
        &mut self,
        full_trace: &[JavaStackTraceElementModel],
        nested_offset: usize,
    ) -> usize {
        if full_trace.is_empty() {
            return 0;
        }

        let start = 3 + nested_offset;
        self.stack_trace = full_trace[start..].to_vec();
        self.stack_trace.len()
    }

    pub fn validate_stack_trace(
        &mut self,
        source: Option<JavaStackTraceElementModel>,
        next: Option<JavaStackTraceElementModel>,
    ) -> bool {
        let Some(source) = source else {
            return false;
        };
        let Some(current) = self.stack_trace.first() else {
            return false;
        };

        if current.native_method == source.native_method
            && current.class_name == source.class_name
            && current.file_name == source.file_name
            && current.method_name == source.method_name
        {
            if next.is_some() != (self.stack_trace.len() > 1) {
                return false;
            }

            if next.is_some_and(|next| self.stack_trace[1] != next) {
                return false;
            }

            self.stack_trace[0] = source;
            true
        } else {
            false
        }
    }

    pub fn trim_stacktrace(&mut self, length: usize) {
        self.stack_trace
            .truncate(self.stack_trace.len().saturating_sub(length));
    }

    pub fn details(&self) -> String {
        let mut builder = String::new();
        self.get_details(&mut builder);
        builder
    }

    pub fn get_details(&self, builder: &mut String) {
        builder.push_str("-- ");
        builder.push_str(&self.title);
        builder.push_str(" --\n");
        builder.push_str("Details:");

        for entry in &self.entries {
            builder.push_str("\n\t");
            builder.push_str(entry.key());
            builder.push_str(": ");
            builder.push_str(entry.value());
        }

        if !self.stack_trace.is_empty() {
            builder.push_str("\nStacktrace:");

            for element in &self.stack_trace {
                builder.push_str("\n\tat ");
                builder.push_str(&element.to_string());
            }
        }
    }

    pub fn stacktrace(&self) -> &[JavaStackTraceElementModel] {
        &self.stack_trace
    }
}

pub fn crash_report_format_location(x: f64, y: f64, z: f64) -> String {
    format!("{x:.2},{y:.2},{z:.2}")
}

pub fn crash_report_format_block_location(
    level: JavaLevelHeightAccessorModel,
    pos: BlockPos,
) -> String {
    crash_report_format_block_location_xyz(level, pos.x, pos.y, pos.z)
}

pub fn crash_report_format_block_location_xyz(
    level: JavaLevelHeightAccessorModel,
    x: i32,
    y: i32,
    z: i32,
) -> String {
    let section_x = block_to_section_coord(x);
    let section_y = block_to_section_coord(y);
    let section_z = block_to_section_coord(z);
    let relative_x = x & 15;
    let relative_y = y & 15;
    let relative_z = z & 15;
    let min_block_x = section_to_block_coord(section_x);
    let min_block_y = level.min_y;
    let min_block_z = section_to_block_coord(section_z);
    let max_block_x = section_to_block_coord(section_x + 1) - 1;
    let max_block_y = level.max_y;
    let max_block_z = section_to_block_coord(section_z + 1) - 1;
    let region_x = x >> 9;
    let region_z = z >> 9;
    let min_chunk_x = region_x << 5;
    let min_chunk_z = region_z << 5;
    let max_chunk_x = ((region_x + 1) << 5) - 1;
    let max_chunk_z = ((region_z + 1) << 5) - 1;
    let min_region_block_x = region_x << 9;
    let min_region_block_y = level.min_y;
    let min_region_block_z = region_z << 9;
    let max_region_block_x = ((region_x + 1) << 9) - 1;
    let max_region_block_y = level.max_y;
    let max_region_block_z = ((region_z + 1) << 9) - 1;

    format!(
        "World: ({x},{y},{z}), Section: (at {relative_x},{relative_y},{relative_z} in {section_x},{section_y},{section_z}; chunk contains blocks {min_block_x},{min_block_y},{min_block_z} to {max_block_x},{max_block_y},{max_block_z}), Region: ({region_x},{region_z}; contains chunks {min_chunk_x},{min_chunk_z} to {max_chunk_x},{max_chunk_z}, blocks {min_region_block_x},{min_region_block_y},{min_region_block_z} to {max_region_block_x},{max_region_block_y},{max_region_block_z})"
    )
}

pub fn crash_report_format_location_in_level(
    level: JavaLevelHeightAccessorModel,
    x: f64,
    y: f64,
    z: f64,
) -> String {
    let pos = BlockPos {
        x: x.floor() as i32,
        y: y.floor() as i32,
        z: z.floor() as i32,
    };
    format!(
        "{:.2},{:.2},{:.2} - {}",
        x,
        y,
        z,
        crash_report_format_block_location(level, pos)
    )
}

fn block_to_section_coord(block_coord: i32) -> i32 {
    block_coord >> 4
}

fn section_to_block_coord(section_coord: i32) -> i32 {
    section_coord << 4
}

#[cfg(test)]
mod tests {
    use super::{
        crash_report_format_block_location_xyz, crash_report_format_location,
        crash_report_format_location_in_level, CrashReport, CrashReportCategoryModel,
        CrashReportDetailValue, JavaLevelHeightAccessorModel, JavaStackTraceElementModel,
        JavaThrowableModel,
    };
    use crate::block_update::BlockPos;
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

    #[test]
    fn crash_report_category_entry_values_match_java_entry_constructor() {
        let mut category = CrashReportCategoryModel::new("Level");
        category.set_detail("String", "stone");
        category.set_detail("Number", 7);
        category.set_detail_null("Null");
        category.set_detail_error(
            "Failure",
            JavaThrowableModel::new("IllegalStateException", Some("bad state")),
        );
        category.set_detail_error(
            "Missing message",
            JavaThrowableModel::new("NullPointerException", Option::<String>::None),
        );

        assert_eq!(
            category.details(),
            "-- Level --\nDetails:\n\tString: stone\n\tNumber: 7\n\tNull: ~~NULL~~\n\tFailure: ~~ERROR~~ IllegalStateException: bad state\n\tMissing message: ~~ERROR~~ NullPointerException: null"
        );
    }

    #[test]
    fn crash_report_category_detail_callback_matches_java_callable_handling() {
        let mut category = CrashReportCategoryModel::new("Callbacks");
        category.set_detail_callback("Ok", || Ok(CrashReportDetailValue::from("value")));
        category.set_detail_callback("Err", || {
            Err(JavaThrowableModel::new("RuntimeException", Some("boom")))
        });

        assert_eq!(
            category.details(),
            "-- Callbacks --\nDetails:\n\tOk: value\n\tErr: ~~ERROR~~ RuntimeException: boom"
        );
    }

    #[test]
    fn crash_report_category_location_formatting_matches_java_math() {
        let level = JavaLevelHeightAccessorModel {
            min_y: -64,
            max_y: 319,
        };

        assert_eq!(
            crash_report_format_location(1.234, -5.0, 9.876),
            "1.23,-5.00,9.88"
        );
        assert_eq!(
            crash_report_format_location_in_level(level, -1.2, 70.9, 512.0),
            "-1.20,70.90,512.00 - World: (-2,70,512), Section: (at 14,6,0 in -1,4,32; chunk contains blocks -16,-64,512 to -1,319,527), Region: (-1,1; contains chunks -32,32 to -1,63, blocks -512,-64,512 to -1,319,1023)"
        );
        assert_eq!(
            crash_report_format_block_location_xyz(level, 32, -1, -33),
            "World: (32,-1,-33), Section: (at 0,15,15 in 2,-1,-3; chunk contains blocks 32,-64,-48 to 47,319,-33), Region: (0,-1; contains chunks 0,-32 to 31,-1, blocks 0,-64,-512 to 511,319,-1)"
        );
    }

    #[test]
    fn crash_report_category_stacktrace_render_trim_and_validate_match_java() {
        let full = vec![
            JavaStackTraceElementModel::new("java.lang.Thread", "getStackTrace", "Thread.java", 1),
            JavaStackTraceElementModel::new(
                "net.minecraft.CrashReportCategory",
                "fillInStackTrace",
                "CrashReportCategory.java",
                126,
            ),
            JavaStackTraceElementModel::new(
                "net.minecraft.CrashReport",
                "addCategory",
                "CrashReport.java",
                145,
            ),
            JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 10),
            JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 20),
        ];
        let mut category = CrashReportCategoryModel::new("Stack");

        assert_eq!(category.fill_in_stack_trace_from_thread_trace(&full, 0), 2);
        assert_eq!(
            category.stacktrace(),
            &[
                JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 10),
                JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 20),
            ]
        );

        let source = JavaStackTraceElementModel::new("game.Source", "tick", "Source.java", 99);
        let next = JavaStackTraceElementModel::new("game.Next", "run", "Next.java", 20);
        assert!(category.validate_stack_trace(Some(source.clone()), Some(next)));
        assert_eq!(category.stacktrace()[0], source);
        assert!(category.details().contains("\nStacktrace:\n\tat game.Source.tick(Source.java:99)\n\tat game.Next.run(Next.java:20)"));

        assert!(!category.validate_stack_trace(
            Some(JavaStackTraceElementModel::new(
                "other.Source",
                "tick",
                "Source.java",
                99
            )),
            Some(JavaStackTraceElementModel::new(
                "game.Next",
                "run",
                "Next.java",
                20
            )),
        ));

        category.trim_stacktrace(1);
        assert_eq!(
            category.stacktrace(),
            &[JavaStackTraceElementModel::new(
                "game.Source",
                "tick",
                "Source.java",
                99
            )]
        );
    }

    #[test]
    fn crash_report_category_block_location_uses_block_pos() {
        let level = JavaLevelHeightAccessorModel {
            min_y: 0,
            max_y: 255,
        };
        assert_eq!(
            super::crash_report_format_block_location(
                level,
                BlockPos {
                    x: 15,
                    y: 64,
                    z: 16,
                },
            ),
            "World: (15,64,16), Section: (at 15,0,0 in 0,4,1; chunk contains blocks 0,0,16 to 15,255,31), Region: (0,0; contains chunks 0,0 to 31,31, blocks 0,0,0 to 511,255,511)"
        );
    }

    #[test]
    fn default_uncaught_exception_handler_matches_java_single_logger_call() {
        let throwable = JavaThrowableModel::new("IllegalStateException", Some("boom"));
        let exception = super::JavaUncaughtExceptionModel::new("Server thread", throwable.clone());

        assert_eq!(
            super::default_uncaught_exception_handler_actions(exception),
            vec![super::JavaUncaughtExceptionLogAction::WithThrowable {
                message: "Caught previously unhandled exception :",
                throwable,
            }]
        );
    }

    #[test]
    fn default_uncaught_exception_handler_with_name_matches_java_two_logger_calls() {
        let throwable = JavaThrowableModel::new("RuntimeException", Some("bad tick"));
        let exception = super::JavaUncaughtExceptionModel::new("Worker-1", throwable.clone());

        assert_eq!(
            super::default_uncaught_exception_handler_with_name_actions(exception),
            vec![
                super::JavaUncaughtExceptionLogAction::Message {
                    message: "Caught previously unhandled exception :",
                },
                super::JavaUncaughtExceptionLogAction::ThreadThrowable {
                    thread_name: "Worker-1".to_string(),
                    throwable,
                },
            ]
        );
    }
}
