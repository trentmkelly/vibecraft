#![allow(dead_code)]

use std::backtrace::Backtrace;
use std::fmt;
use std::fs;
use std::panic;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::block_update::BlockPos;
use crate::report_type::ReportTypeModel;
use crate::system_report::SystemReportModel;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportReferenceModel {
    title: String,
    exception: JavaThrowableModel,
}

impl CrashReportReferenceModel {
    pub fn new(title: impl Into<String>, exception: JavaThrowableModel) -> Self {
        Self {
            title: title.into(),
            exception,
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_exception(&self) -> &JavaThrowableModel {
        &self.exception
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedExceptionModel {
    report: CrashReportReferenceModel,
}

impl ReportedExceptionModel {
    pub fn new(report: CrashReportReferenceModel) -> Self {
        Self { report }
    }

    pub fn get_report(&self) -> &CrashReportReferenceModel {
        &self.report
    }

    pub fn get_cause(&self) -> &JavaThrowableModel {
        self.report.get_exception()
    }

    pub fn get_message(&self) -> &str {
        self.report.get_title()
    }
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
                    "VibeCraft Version".to_string(),
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

    pub fn from_watchdog_tick(
        tick_count: u64,
        tick_elapsed: Duration,
        max_tick_time: Duration,
        world_root: &Path,
    ) -> Self {
        Self {
            title: "Watching Server".to_string(),
            details: vec![
                (
                    "VibeCraft Version".to_string(),
                    env!("CARGO_PKG_VERSION").to_string(),
                ),
                (
                    "Minecraft Target".to_string(),
                    "Java Edition 26.1.2".to_string(),
                ),
                ("Process ID".to_string(), std::process::id().to_string()),
                ("Thread".to_string(), "Server Watchdog".to_string()),
                (
                    "Performance stats".to_string(),
                    format!(
                        "tick={tick_count}, elapsed={}ms, maxTickTime={}ms",
                        tick_elapsed.as_millis(),
                        max_tick_time.as_millis()
                    ),
                ),
                ("World Root".to_string(), world_root.display().to_string()),
                ("OS".to_string(), std::env::consts::OS.to_string()),
                (
                    "Architecture".to_string(),
                    std::env::consts::ARCH.to_string(),
                ),
            ],
            backtrace: Backtrace::force_capture().to_string(),
        }
    }

    pub fn summary(&self) -> String {
        format!("VibeCraft crashed: {}", self.title)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str("---- VibeCraft Crash Report ----\n");
        out.push_str("// This report is generated when VibeCraft panics.\n\n");
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
    pub stack_trace: Vec<JavaStackTraceElementModel>,
    pub cause: Option<Box<JavaThrowableModel>>,
    pub reported_exception_report: Option<Box<CrashReportReferenceModel>>,
}

impl JavaThrowableModel {
    pub fn new(class_simple_name: impl Into<String>, message: Option<impl Into<String>>) -> Self {
        Self {
            class_simple_name: class_simple_name.into(),
            message: message.map(Into::into),
            stack_trace: Vec::new(),
            cause: None,
            reported_exception_report: None,
        }
    }

    pub fn with_stack_trace(mut self, stack_trace: Vec<JavaStackTraceElementModel>) -> Self {
        self.stack_trace = stack_trace;
        self
    }

    pub fn completion_exception(cause: JavaThrowableModel) -> Self {
        Self {
            class_simple_name: "CompletionException".to_string(),
            message: None,
            stack_trace: Vec::new(),
            cause: Some(Box::new(cause)),
            reported_exception_report: None,
        }
    }

    pub fn reported_exception(report: CrashReportReferenceModel) -> Self {
        Self {
            class_simple_name: "ReportedException".to_string(),
            message: Some(report.get_title().to_string()),
            stack_trace: Vec::new(),
            cause: Some(Box::new(report.get_exception().clone())),
            reported_exception_report: Some(Box::new(report)),
        }
    }

    fn class_name_for_printing(&self) -> String {
        if self.class_simple_name.contains('.') {
            self.class_simple_name.clone()
        } else {
            format!("java.lang.{}", self.class_simple_name)
        }
    }

    fn with_title_message_for_null_message(&self, title: &str) -> Self {
        if self.message.is_some() {
            return self.clone();
        }

        match self.class_simple_name.as_str() {
            "NullPointerException" | "StackOverflowError" | "OutOfMemoryError" => {
                let mut replacement = self.clone();
                replacement.message = Some(title.to_string());
                replacement
            }
            _ => self.clone(),
        }
    }

    pub fn print_stack_trace_string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.class_name_for_printing());
        if let Some(message) = &self.message {
            out.push_str(": ");
            out.push_str(message);
        }
        out.push('\n');
        for element in &self.stack_trace {
            out.push_str("\tat ");
            out.push_str(&element.to_string());
            out.push('\n');
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashReportSaveError {
    Io(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashReportPreloadAction {
    AllocateMemoryReserve,
    BuildFriendlyReport { title: &'static str },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReportModel {
    title: String,
    exception: JavaThrowableModel,
    details: Vec<CrashReportCategoryModel>,
    save_file: Option<PathBuf>,
    tracking_stack_trace: bool,
    uncategorized_stack_trace: Option<Vec<JavaStackTraceElementModel>>,
    system_report: SystemReportModel,
}

impl CrashReportModel {
    pub fn new(
        title: impl Into<String>,
        exception: JavaThrowableModel,
        system_report: SystemReportModel,
    ) -> Self {
        Self {
            title: title.into(),
            exception,
            details: Vec::new(),
            save_file: None,
            tracking_stack_trace: true,
            uncategorized_stack_trace: Some(Vec::new()),
            system_report,
        }
    }

    pub fn get_title(&self) -> &str {
        &self.title
    }

    pub fn get_exception(&self) -> &JavaThrowableModel {
        &self.exception
    }

    pub fn get_save_file(&self) -> Option<&Path> {
        self.save_file.as_deref()
    }

    pub fn get_system_report(&self) -> &SystemReportModel {
        &self.system_report
    }

    pub fn get_exception_message(&self) -> String {
        self.exception
            .with_title_message_for_null_message(&self.title)
            .print_stack_trace_string()
    }

    pub fn add_category_from_thread_trace(
        &mut self,
        name: impl Into<String>,
        nested_offset: usize,
        current_thread_trace: &[JavaStackTraceElementModel],
    ) -> &CrashReportCategoryModel {
        let mut category = CrashReportCategoryModel::new(name);
        if self.tracking_stack_trace {
            let size =
                category.fill_in_stack_trace_from_thread_trace(current_thread_trace, nested_offset);
            let full_trace = &self.exception.stack_trace;
            let trace_index = full_trace.len() as isize - size as isize;
            let source = usize::try_from(trace_index)
                .ok()
                .and_then(|index| full_trace.get(index))
                .cloned();
            let next = if full_trace.len() + 1 >= size {
                full_trace.get(full_trace.len() + 1 - size).cloned()
            } else {
                None
            };

            self.tracking_stack_trace = category.validate_stack_trace(source, next);
            if full_trace.len() >= size
                && trace_index >= 0
                && (trace_index as usize) < full_trace.len()
            {
                self.uncategorized_stack_trace = Some(full_trace[..trace_index as usize].to_vec());
            } else {
                self.tracking_stack_trace = false;
            }
        }

        self.details.push(category);
        let index = self.details.len() - 1;
        &self.details[index]
    }

    pub fn get_details(&mut self, current_thread_name: &str) -> String {
        let mut builder = String::new();
        self.append_details(&mut builder, current_thread_name);
        builder
    }

    pub fn append_details(&mut self, builder: &mut String, current_thread_name: &str) {
        if self
            .uncategorized_stack_trace
            .as_ref()
            .is_none_or(Vec::is_empty)
            && !self.details.is_empty()
        {
            self.uncategorized_stack_trace = Some(
                self.details[0]
                    .stacktrace()
                    .iter()
                    .take(1)
                    .cloned()
                    .collect(),
            );
        }

        if let Some(stack_trace) = &self.uncategorized_stack_trace {
            if !stack_trace.is_empty() {
                builder.push_str("-- Head --\n");
                builder.push_str("Thread: ");
                builder.push_str(current_thread_name);
                builder.push_str("\nStacktrace:\n");
                for element in stack_trace {
                    builder.push_str("\tat ");
                    builder.push_str(&element.to_string());
                    builder.push('\n');
                }
                builder.push('\n');
            }
        }

        for entry in &self.details {
            entry.get_details(builder);
            builder.push_str("\n\n");
        }

        self.system_report.append_to_crash_report_string(builder);
    }

    pub fn get_friendly_report_at(
        &mut self,
        report_type: ReportTypeModel,
        extra_comments: &[&str],
        current_thread_name: &str,
        formatted_time: &str,
        nanos: u128,
    ) -> String {
        let mut builder = String::new();
        report_type.append_header_at_nanos(&mut builder, extra_comments, nanos);
        builder.push_str("Time: ");
        builder.push_str(formatted_time);
        builder.push('\n');
        builder.push_str("Description: ");
        builder.push_str(&self.title);
        builder.push_str("\n\n");
        builder.push_str(&self.get_exception_message());
        builder.push_str(
            "\n\nA detailed walkthrough of the error, its code path and all known details is as follows:\n",
        );
        builder.push_str(&"-".repeat(87));
        builder.push_str("\n\n");
        self.append_details(&mut builder, current_thread_name);
        builder
    }

    pub fn save_to_file_at(
        &mut self,
        save_file: &Path,
        report_type: ReportTypeModel,
        extra_comments: &[&str],
        current_thread_name: &str,
        formatted_time: &str,
        nanos: u128,
    ) -> bool {
        if self.save_file.is_some() {
            return false;
        }

        let result = (|| -> std::io::Result<()> {
            if let Some(parent) = save_file.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(
                save_file,
                self.get_friendly_report_at(
                    report_type,
                    extra_comments,
                    current_thread_name,
                    formatted_time,
                    nanos,
                ),
            )
        })();

        if result.is_ok() {
            self.save_file = Some(save_file.to_path_buf());
            true
        } else {
            false
        }
    }

    pub fn for_throwable(
        throwable: JavaThrowableModel,
        title: &str,
        system_report: SystemReportModel,
    ) -> Self {
        let mut current = throwable;
        while current.class_simple_name == "CompletionException" {
            let Some(cause) = current.cause.take() else {
                break;
            };
            current = *cause;
        }

        if let Some(report) = current.reported_exception_report {
            Self::new(
                report.get_title().to_string(),
                report.get_exception().clone(),
                system_report,
            )
        } else {
            Self::new(title.to_string(), current, system_report)
        }
    }

    pub fn preload_actions() -> Vec<CrashReportPreloadAction> {
        vec![
            CrashReportPreloadAction::AllocateMemoryReserve,
            CrashReportPreloadAction::BuildFriendlyReport {
                title: "Don't panic!",
            },
        ]
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
mod tests;
