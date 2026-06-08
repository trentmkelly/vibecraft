#![allow(dead_code)]

use crate::world_version::WorldVersionModel;

pub const BYTES_PER_MEBIBYTE: i64 = 1_048_576;
const ONE_GIGA: f64 = 1_000_000_000.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaMemoryUsageModel {
    pub init: i64,
    pub used: i64,
    pub committed: i64,
    pub max: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaRuntimeMemoryModel {
    pub max: i64,
    pub total: i64,
    pub free: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaSystemPropertiesModel {
    pub os_name: String,
    pub os_arch: String,
    pub os_version: String,
    pub java_version: String,
    pub java_vendor: String,
    pub java_vm_name: String,
    pub java_vm_info: String,
    pub java_vm_vendor: String,
}

impl JavaSystemPropertiesModel {
    pub fn operating_system(&self) -> String {
        format!(
            "{} ({}) version {}",
            self.os_name, self.os_arch, self.os_version
        )
    }

    pub fn java_version(&self) -> String {
        format!("{}, {}", self.java_version, self.java_vendor)
    }

    pub fn java_vm_version(&self) -> String {
        format!(
            "{} ({}), {}",
            self.java_vm_name, self.java_vm_info, self.java_vm_vendor
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JavaRuntimeSnapshotModel {
    pub properties: JavaSystemPropertiesModel,
    pub memory: JavaRuntimeMemoryModel,
    pub heap_memory: JavaMemoryUsageModel,
    pub non_heap_memory: JavaMemoryUsageModel,
    pub available_processors: i32,
    pub input_arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMemoryModel {
    pub capacity: i64,
    pub clock_speed: i64,
    pub memory_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualMemoryModel {
    pub virtual_max: i64,
    pub virtual_in_use: i64,
    pub swap_total: i64,
    pub swap_used: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalMemoryModel {
    pub physical_memory: Vec<PhysicalMemoryModel>,
    pub virtual_memory: VirtualMemoryModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphicsCardModel {
    pub name: String,
    pub vendor: String,
    pub vram: i64,
    pub device_id: String,
    pub version_info: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorIdentifierModel {
    pub vendor: String,
    pub name: String,
    pub identifier: String,
    pub microarchitecture: String,
    pub vendor_freq: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorModel {
    pub identifier: ProcessorIdentifierModel,
    pub physical_package_count: i32,
    pub physical_processor_count: i32,
    pub logical_processor_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageSpaceModel {
    NotSet,
    InvalidPath,
    Error,
    AvailableAndTotal { available: i64, total: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoragePathModel {
    pub id: String,
    pub space: StorageSpaceModel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareModel {
    pub processor: Result<ProcessorModel, String>,
    pub graphics_cards: Result<Vec<GraphicsCardModel>, String>,
    pub memory: Result<GlobalMemoryModel, String>,
    pub storage: Result<Vec<StoragePathModel>, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemReportModel {
    entries: Vec<(String, String)>,
}

impl SystemReportModel {
    pub fn empty() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn new(
        version: &WorldVersionModel,
        runtime: &JavaRuntimeSnapshotModel,
        hardware: Result<&HardwareModel, String>,
    ) -> Self {
        let mut report = Self {
            entries: Vec::new(),
        };
        report.set_detail("Minecraft Version", version.name.clone());
        report.set_detail("Minecraft Version ID", version.id.clone());
        report.set_detail("Operating System", runtime.properties.operating_system());
        report.set_detail("Java Version", runtime.properties.java_version());
        report.set_detail("Java VM Version", runtime.properties.java_vm_version());
        report.set_detail("Memory", print_runtime_memory(&runtime.memory));
        report.set_detail("Memory (heap)", print_memory_usage(&runtime.heap_memory));
        report.set_detail(
            "Memory (non-head)",
            print_memory_usage(&runtime.non_heap_memory),
        );
        report.set_detail("CPUs", runtime.available_processors.to_string());

        if let Ok(hardware) = hardware {
            report.put_hardware(hardware);
        }

        report.set_detail("JVM Flags", print_jvm_flags(&runtime.input_arguments, "-X"));
        report.set_detail(
            "Debug Flags",
            print_jvm_flags(&runtime.input_arguments, "-DMC_DEBUG_"),
        );
        report
    }

    pub fn set_detail(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        if let Some((_, existing_value)) = self
            .entries
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            *existing_value = value;
        } else {
            self.entries.push((key, value));
        }
    }

    pub fn set_detail_result(&mut self, key: impl Into<String>, value: Result<String, String>) {
        self.set_detail(key, value.unwrap_or_else(|_| "ERR".to_string()));
    }

    pub fn entries(&self) -> &[(String, String)] {
        &self.entries
    }

    fn put_hardware(&mut self, hardware: &HardwareModel) {
        if let Ok(processor) = &hardware.processor {
            self.put_processor(processor);
        }
        if let Ok(graphics_cards) = &hardware.graphics_cards {
            self.put_graphics(graphics_cards);
        }
        if let Ok(memory) = &hardware.memory {
            self.put_memory(memory);
        }
        if let Ok(storage) = &hardware.storage {
            self.put_storage(storage);
        }
    }

    fn put_processor(&mut self, processor: &ProcessorModel) {
        let identifier = &processor.identifier;
        self.set_detail("Processor Vendor", identifier.vendor.clone());
        self.set_detail("Processor Name", identifier.name.clone());
        self.set_detail("Identifier", identifier.identifier.clone());
        self.set_detail("Microarchitecture", identifier.microarchitecture.clone());
        self.set_detail(
            "Frequency (GHz)",
            format_two_decimals(identifier.vendor_freq as f64 / ONE_GIGA),
        );
        self.set_detail(
            "Number of physical packages",
            processor.physical_package_count.to_string(),
        );
        self.set_detail(
            "Number of physical CPUs",
            processor.physical_processor_count.to_string(),
        );
        self.set_detail(
            "Number of logical CPUs",
            processor.logical_processor_count.to_string(),
        );
    }

    fn put_graphics(&mut self, graphics_cards: &[GraphicsCardModel]) {
        for (gpu_index, graphics_card) in graphics_cards.iter().enumerate() {
            let prefix = format!("Graphics card #{gpu_index} ");
            self.set_detail(format!("{prefix}name"), graphics_card.name.clone());
            self.set_detail(format!("{prefix}vendor"), graphics_card.vendor.clone());
            self.set_detail(
                format!("{prefix}VRAM (MiB)"),
                format_two_decimals(size_in_mib_f64(graphics_card.vram)),
            );
            self.set_detail(format!("{prefix}deviceId"), graphics_card.device_id.clone());
            self.set_detail(
                format!("{prefix}versionInfo"),
                graphics_card.version_info.clone(),
            );
        }
    }

    fn put_memory(&mut self, memory: &GlobalMemoryModel) {
        self.put_physical_memory(&memory.physical_memory);
        self.put_virtual_memory(&memory.virtual_memory);
    }

    fn put_physical_memory(&mut self, memory_packages: &[PhysicalMemoryModel]) {
        for (memory_slot, physical_memory) in memory_packages.iter().enumerate() {
            let prefix = format!("Memory slot #{memory_slot} ");
            self.set_detail(
                format!("{prefix}capacity (MiB)"),
                format_two_decimals(size_in_mib_f64(physical_memory.capacity)),
            );
            self.set_detail(
                format!("{prefix}clockSpeed (GHz)"),
                format_two_decimals(physical_memory.clock_speed as f64 / ONE_GIGA),
            );
            self.set_detail(format!("{prefix}type"), physical_memory.memory_type.clone());
        }
    }

    fn put_virtual_memory(&mut self, virtual_memory: &VirtualMemoryModel) {
        self.set_detail(
            "Virtual memory max (MiB)",
            format_two_decimals(size_in_mib_f64(virtual_memory.virtual_max)),
        );
        self.set_detail(
            "Virtual memory used (MiB)",
            format_two_decimals(size_in_mib_f64(virtual_memory.virtual_in_use)),
        );
        self.set_detail(
            "Swap memory total (MiB)",
            format_two_decimals(size_in_mib_f64(virtual_memory.swap_total)),
        );
        self.set_detail(
            "Swap memory used (MiB)",
            format_two_decimals(size_in_mib_f64(virtual_memory.swap_used)),
        );
    }

    fn put_storage(&mut self, storage: &[StoragePathModel]) {
        for storage_path in storage {
            let value = match storage_path.space {
                StorageSpaceModel::NotSet => "<path not set>".to_string(),
                StorageSpaceModel::InvalidPath => "<invalid path>".to_string(),
                StorageSpaceModel::Error => "ERR".to_string(),
                StorageSpaceModel::AvailableAndTotal { available, total } => format!(
                    "available: {}, total: {}",
                    format_two_decimals(size_in_mib_f64(available)),
                    format_two_decimals(size_in_mib_f64(total))
                ),
            };
            self.set_detail(
                format!("Space in storage for {} (MiB)", storage_path.id),
                value,
            );
        }
    }

    pub fn append_to_crash_report_string(&self, builder: &mut String) {
        builder.push_str("-- System Details --\n");
        builder.push_str("Details:");
        for (key, value) in &self.entries {
            builder.push_str("\n\t");
            builder.push_str(key);
            builder.push_str(": ");
            builder.push_str(value);
        }
    }

    pub fn to_line_separated_string(&self) -> String {
        self.entries
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>()
            .join(java_line_separator())
    }
}

pub fn print_memory_usage(memory_usage: &JavaMemoryUsageModel) -> String {
    format!(
        "init: {:03}MiB, used: {:03}MiB, committed: {:03}MiB, max: {:03}MiB",
        memory_usage.init / BYTES_PER_MEBIBYTE,
        memory_usage.used / BYTES_PER_MEBIBYTE,
        memory_usage.committed / BYTES_PER_MEBIBYTE,
        memory_usage.max / BYTES_PER_MEBIBYTE,
    )
}

pub fn print_runtime_memory(memory: &JavaRuntimeMemoryModel) -> String {
    format!(
        "{} bytes ({} MiB) / {} bytes ({} MiB) up to {} bytes ({} MiB)",
        memory.free,
        memory.free / BYTES_PER_MEBIBYTE,
        memory.total,
        memory.total / BYTES_PER_MEBIBYTE,
        memory.max,
        memory.max / BYTES_PER_MEBIBYTE,
    )
}

pub fn print_jvm_flags(arguments: &[String], prefix: &str) -> String {
    let selected = arguments
        .iter()
        .filter(|argument| argument.starts_with(prefix))
        .cloned()
        .collect::<Vec<_>>();
    format!("{} total; {}", selected.len(), selected.join(" "))
}

pub fn size_in_mib(bytes: i64) -> f32 {
    bytes as f32 / BYTES_PER_MEBIBYTE as f32
}

fn size_in_mib_f64(bytes: i64) -> f64 {
    bytes as f64 / BYTES_PER_MEBIBYTE as f64
}

fn format_two_decimals(value: f64) -> String {
    format!("{value:.2}")
}

fn java_line_separator() -> &'static str {
    if cfg!(windows) {
        "\r\n"
    } else {
        "\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_version::WorldVersionModel;

    fn runtime_snapshot() -> JavaRuntimeSnapshotModel {
        JavaRuntimeSnapshotModel {
            properties: JavaSystemPropertiesModel {
                os_name: "Linux".to_string(),
                os_arch: "amd64".to_string(),
                os_version: "6.1".to_string(),
                java_version: "25".to_string(),
                java_vendor: "Mojang".to_string(),
                java_vm_name: "OpenJDK 64-Bit Server VM".to_string(),
                java_vm_info: "mixed mode".to_string(),
                java_vm_vendor: "Mojang".to_string(),
            },
            memory: JavaRuntimeMemoryModel {
                max: 512 * BYTES_PER_MEBIBYTE,
                total: 256 * BYTES_PER_MEBIBYTE,
                free: 128 * BYTES_PER_MEBIBYTE,
            },
            heap_memory: JavaMemoryUsageModel {
                init: BYTES_PER_MEBIBYTE,
                used: 2 * BYTES_PER_MEBIBYTE,
                committed: 3 * BYTES_PER_MEBIBYTE,
                max: 4 * BYTES_PER_MEBIBYTE,
            },
            non_heap_memory: JavaMemoryUsageModel {
                init: 5 * BYTES_PER_MEBIBYTE,
                used: 6 * BYTES_PER_MEBIBYTE,
                committed: 7 * BYTES_PER_MEBIBYTE,
                max: 8 * BYTES_PER_MEBIBYTE,
            },
            available_processors: 12,
            input_arguments: vec![
                "-Xmx512M".to_string(),
                "-DMC_DEBUG_ENABLED=true".to_string(),
                "-Dfoo=bar".to_string(),
            ],
        }
    }

    fn hardware_model() -> HardwareModel {
        HardwareModel {
            processor: Ok(ProcessorModel {
                identifier: ProcessorIdentifierModel {
                    vendor: "GenuineIntel".to_string(),
                    name: "Test CPU".to_string(),
                    identifier: "family 6".to_string(),
                    microarchitecture: "x86".to_string(),
                    vendor_freq: 3_500_000_000,
                },
                physical_package_count: 1,
                physical_processor_count: 6,
                logical_processor_count: 12,
            }),
            graphics_cards: Ok(vec![GraphicsCardModel {
                name: "GPU".to_string(),
                vendor: "Vendor".to_string(),
                vram: 2 * BYTES_PER_MEBIBYTE,
                device_id: "dev".to_string(),
                version_info: "v1".to_string(),
            }]),
            memory: Ok(GlobalMemoryModel {
                physical_memory: vec![PhysicalMemoryModel {
                    capacity: 8 * BYTES_PER_MEBIBYTE,
                    clock_speed: 3_200_000_000,
                    memory_type: "DDR5".to_string(),
                }],
                virtual_memory: VirtualMemoryModel {
                    virtual_max: 16 * BYTES_PER_MEBIBYTE,
                    virtual_in_use: 4 * BYTES_PER_MEBIBYTE,
                    swap_total: 2 * BYTES_PER_MEBIBYTE,
                    swap_used: BYTES_PER_MEBIBYTE,
                },
            }),
            storage: Ok(vec![
                StoragePathModel {
                    id: "workdir".to_string(),
                    space: StorageSpaceModel::AvailableAndTotal {
                        available: 10 * BYTES_PER_MEBIBYTE,
                        total: 20 * BYTES_PER_MEBIBYTE,
                    },
                },
                StoragePathModel {
                    id: "jna.tmpdir".to_string(),
                    space: StorageSpaceModel::NotSet,
                },
                StoragePathModel {
                    id: "java.io.tmpdir".to_string(),
                    space: StorageSpaceModel::InvalidPath,
                },
                StoragePathModel {
                    id: "io.netty.native.workdir".to_string(),
                    space: StorageSpaceModel::Error,
                },
            ]),
        }
    }

    #[test]
    fn system_report_formats_memory_and_jvm_flags_like_java() {
        assert_eq!(
            print_memory_usage(&JavaMemoryUsageModel {
                init: BYTES_PER_MEBIBYTE,
                used: 12 * BYTES_PER_MEBIBYTE,
                committed: 123 * BYTES_PER_MEBIBYTE,
                max: 1234 * BYTES_PER_MEBIBYTE,
            }),
            "init: 001MiB, used: 012MiB, committed: 123MiB, max: 1234MiB"
        );
        assert_eq!(
            print_jvm_flags(
                &[
                    "-Xmx2G".to_string(),
                    "-DMC_DEBUG_ENABLED=true".to_string(),
                    "-Xms1G".to_string(),
                ],
                "-X",
            ),
            "2 total; -Xmx2G -Xms1G"
        );
        assert_eq!(size_in_mib(1_572_864), 1.5);
    }

    #[test]
    fn system_report_constructor_preserves_java_entry_order() {
        let version = WorldVersionModel::current_26_1_2();
        let hardware = hardware_model();
        let report = SystemReportModel::new(&version, &runtime_snapshot(), Ok(&hardware));
        let keys = report
            .entries()
            .iter()
            .map(|(key, _)| key.as_str())
            .take(12)
            .collect::<Vec<_>>();

        assert_eq!(
            keys,
            vec![
                "Minecraft Version",
                "Minecraft Version ID",
                "Operating System",
                "Java Version",
                "Java VM Version",
                "Memory",
                "Memory (heap)",
                "Memory (non-head)",
                "CPUs",
                "Processor Vendor",
                "Processor Name",
                "Identifier",
            ]
        );
    }

    #[test]
    fn system_report_collects_hardware_and_storage_fields_like_java() {
        let version = WorldVersionModel::current_26_1_2();
        let hardware = hardware_model();
        let report = SystemReportModel::new(&version, &runtime_snapshot(), Ok(&hardware));
        let entries = report.entries();

        assert!(entries.contains(&("Processor Vendor".to_string(), "GenuineIntel".to_string())));
        assert!(entries.contains(&("Frequency (GHz)".to_string(), "3.50".to_string())));
        assert!(entries.contains(&(
            "Graphics card #0 VRAM (MiB)".to_string(),
            "2.00".to_string()
        )));
        assert!(entries.contains(&(
            "Memory slot #0 capacity (MiB)".to_string(),
            "8.00".to_string()
        )));
        assert!(entries.contains(&("Virtual memory max (MiB)".to_string(), "16.00".to_string())));
        assert!(entries.contains(&(
            "Space in storage for workdir (MiB)".to_string(),
            "available: 10.00, total: 20.00".to_string()
        )));
        assert!(entries.contains(&(
            "Space in storage for jna.tmpdir (MiB)".to_string(),
            "<path not set>".to_string()
        )));
        assert!(entries.contains(&(
            "Space in storage for java.io.tmpdir (MiB)".to_string(),
            "<invalid path>".to_string()
        )));
        assert!(entries.contains(&(
            "Space in storage for io.netty.native.workdir (MiB)".to_string(),
            "ERR".to_string()
        )));
    }

    #[test]
    fn system_report_ignores_failed_hardware_groups_like_java() {
        let version = WorldVersionModel::current_26_1_2();
        let hardware = HardwareModel {
            processor: Err("processor failed".to_string()),
            graphics_cards: Err("graphics failed".to_string()),
            memory: Err("memory failed".to_string()),
            storage: Err("storage failed".to_string()),
        };
        let report = SystemReportModel::new(&version, &runtime_snapshot(), Ok(&hardware));

        assert!(report
            .entries()
            .iter()
            .any(|(key, value)| key == "JVM Flags" && value == "1 total; -Xmx512M"));
        assert!(!report
            .entries()
            .iter()
            .any(|(key, _)| key == "Processor Vendor"));
    }

    #[test]
    fn system_report_set_detail_result_matches_supplier_error_fallback() {
        let mut report = SystemReportModel {
            entries: Vec::new(),
        };
        report.set_detail_result("Ok", Ok("value".to_string()));
        report.set_detail_result("Fail", Err("boom".to_string()));
        report.set_detail("Ok", "replacement");

        assert_eq!(
            report.entries(),
            &[
                ("Ok".to_string(), "replacement".to_string()),
                ("Fail".to_string(), "ERR".to_string()),
            ]
        );
    }

    #[test]
    fn system_report_output_methods_match_java_formatting() {
        let mut report = SystemReportModel {
            entries: Vec::new(),
        };
        report.set_detail("Minecraft Version", "26.1.2");
        report.set_detail("CPUs", "12");

        let mut crash = String::new();
        report.append_to_crash_report_string(&mut crash);
        assert_eq!(
            crash,
            "-- System Details --\nDetails:\n\tMinecraft Version: 26.1.2\n\tCPUs: 12"
        );
        assert_eq!(
            report.to_line_separated_string(),
            format!("Minecraft Version: 26.1.2{}CPUs: 12", java_line_separator())
        );
    }
}
