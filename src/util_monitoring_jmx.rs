#![allow(dead_code)]

use crate::management_server::{jmx_monitoring_plan, JmxMonitoringPlan, ManagementStartupPlan};
use crate::server_properties::ServerProperties;

pub const OBJECT_NAME: &str = "net.minecraft.server:type=Server";
pub const MBEAN_CLASS_NAME: &str = "MinecraftServerStatistics";
pub const MBEAN_DESCRIPTION: &str = "metrics for dedicated server";
pub const REGISTER_FAILURE_LOG: &str = "Failed to initialise server as JMX bean";

#[derive(Debug, Clone, PartialEq)]
pub struct MinecraftServerStatistics {
    tick_times_nanos: Vec<i64>,
    average_tick_time_ms: f32,
}

impl MinecraftServerStatistics {
    pub fn new(tick_times_nanos: Vec<i64>, average_tick_time_ms: f32) -> Self {
        Self {
            tick_times_nanos,
            average_tick_time_ms,
        }
    }

    pub fn register_jmx_monitoring(
        properties: &ServerProperties,
        management_plan: &ManagementStartupPlan,
    ) -> JmxMonitoringPlan {
        jmx_monitoring_plan(properties, management_plan)
    }

    pub fn get_attribute(&self, attribute: &str) -> Option<AttributeValue> {
        attribute_description(attribute).map(|description| (description.getter)(self))
    }

    pub fn set_attribute(&self, _attribute: Attribute) {}

    pub fn get_attributes(&self, attributes: &[&str]) -> Vec<Attribute> {
        attributes
            .iter()
            .filter_map(|name| {
                attribute_description(name).map(|description| Attribute {
                    name: description.name.to_string(),
                    value: (description.getter)(self),
                })
            })
            .collect()
    }

    pub fn set_attributes(&self, _attributes: &[Attribute]) -> Vec<Attribute> {
        Vec::new()
    }

    pub fn invoke(
        &self,
        _action_name: &str,
        _params: &[AttributeValue],
        _signature: &[&str],
    ) -> Option<AttributeValue> {
        None
    }

    pub fn mbean_info(&self) -> MBeanInfo {
        MBeanInfo {
            class_name: MBEAN_CLASS_NAME,
            description: MBEAN_DESCRIPTION,
            attributes: ATTRIBUTE_DESCRIPTIONS
                .iter()
                .map(|description| description.as_mbean_attribute_info())
                .collect(),
            notifications: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    LongArray(Vec<i64>),
    Float(f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MBeanInfo {
    pub class_name: &'static str,
    pub description: &'static str,
    pub attributes: Vec<MBeanAttributeInfo>,
    pub notifications: Vec<MBeanNotificationInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MBeanAttributeInfo {
    pub name: &'static str,
    pub type_simple_name: &'static str,
    pub description: &'static str,
    pub readable: bool,
    pub writable: bool,
    pub is_is: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MBeanNotificationInfo;

#[derive(Clone, Copy)]
struct AttributeDescription {
    name: &'static str,
    getter: fn(&MinecraftServerStatistics) -> AttributeValue,
    description: &'static str,
    type_simple_name: &'static str,
}

impl AttributeDescription {
    fn as_mbean_attribute_info(&self) -> MBeanAttributeInfo {
        MBeanAttributeInfo {
            name: self.name,
            type_simple_name: self.type_simple_name,
            description: self.description,
            readable: true,
            writable: false,
            is_is: false,
        }
    }
}

const ATTRIBUTE_DESCRIPTIONS: &[AttributeDescription] = &[
    AttributeDescription {
        name: "tickTimes",
        getter: tick_times,
        description: "Historical tick times (ms)",
        type_simple_name: "long[]",
    },
    AttributeDescription {
        name: "averageTickTime",
        getter: average_tick_time,
        description: "Current average tick time (ms)",
        type_simple_name: "long",
    },
];

fn attribute_description(name: &str) -> Option<AttributeDescription> {
    ATTRIBUTE_DESCRIPTIONS
        .iter()
        .copied()
        .find(|description| description.name == name)
}

fn tick_times(statistics: &MinecraftServerStatistics) -> AttributeValue {
    AttributeValue::LongArray(statistics.tick_times_nanos.clone())
}

fn average_tick_time(statistics: &MinecraftServerStatistics) -> AttributeValue {
    AttributeValue::Float(statistics.average_tick_time_ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::management_server::{AllowedOrigins, TlsEndpoint};

    #[test]
    fn java_source_pins_jmx_registration_identity_and_failure_policy() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/monitoring/jmx/MinecraftServerStatistics.java");
        assert!(source.contains("implements DynamicMBean"));
        assert!(source.contains("new ObjectName(\"net.minecraft.server:type=Server\")"));
        assert!(source.contains("Failed to initialise server as JMX bean"));

        assert_eq!(OBJECT_NAME, "net.minecraft.server:type=Server");
        assert_eq!(REGISTER_FAILURE_LOG, "Failed to initialise server as JMX bean");
    }

    #[test]
    fn attributes_match_java_dynamic_mbean_surface() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/monitoring/jmx/MinecraftServerStatistics.java");
        assert!(source.contains("new MinecraftServerStatistics.AttributeDescription(\"tickTimes\""));
        assert!(source.contains("new MinecraftServerStatistics.AttributeDescription(\"averageTickTime\""));
        assert!(source.contains("long[].class"));
        assert!(source.contains("long.class"));
        assert!(source.contains("return this.server.getCurrentSmoothedTickTime()"));
        assert!(source.contains("return this.server.getTickTimesNanos()"));

        let statistics = MinecraftServerStatistics::new(vec![50, 45, 40], 47.5);
        assert_eq!(
            statistics.get_attribute("tickTimes"),
            Some(AttributeValue::LongArray(vec![50, 45, 40]))
        );
        assert_eq!(
            statistics.get_attribute("averageTickTime"),
            Some(AttributeValue::Float(47.5))
        );
        assert_eq!(statistics.get_attribute("missing"), None);

        let attributes = statistics.get_attributes(&["missing", "averageTickTime", "tickTimes"]);
        assert_eq!(
            attributes,
            vec![
                Attribute {
                    name: "averageTickTime".to_string(),
                    value: AttributeValue::Float(47.5),
                },
                Attribute {
                    name: "tickTimes".to_string(),
                    value: AttributeValue::LongArray(vec![50, 45, 40]),
                },
            ]
        );
    }

    #[test]
    fn mbean_info_preserves_java_class_description_and_read_only_attributes() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/monitoring/jmx/MinecraftServerStatistics.java");
        assert!(source.contains("MinecraftServerStatistics.class.getSimpleName()"));
        assert!(source.contains("\"metrics for dedicated server\""));
        assert!(source.contains("new MBeanNotificationInfo[0]"));
        assert!(source.contains("new MBeanAttributeInfo(this.name, this.type.getSimpleName(), this.description, true, false, false)"));

        let info = MinecraftServerStatistics::new(Vec::new(), 0.0).mbean_info();
        assert_eq!(info.class_name, "MinecraftServerStatistics");
        assert_eq!(info.description, "metrics for dedicated server");
        assert!(info.notifications.is_empty());
        assert_eq!(
            info.attributes,
            vec![
                MBeanAttributeInfo {
                    name: "tickTimes",
                    type_simple_name: "long[]",
                    description: "Historical tick times (ms)",
                    readable: true,
                    writable: false,
                    is_is: false,
                },
                MBeanAttributeInfo {
                    name: "averageTickTime",
                    type_simple_name: "long",
                    description: "Current average tick time (ms)",
                    readable: true,
                    writable: false,
                    is_is: false,
                },
            ]
        );
    }

    #[test]
    fn mutating_and_invocation_methods_are_noops_like_java() {
        let source =
            vibecraft_java_source!("/net/minecraft/util/monitoring/jmx/MinecraftServerStatistics.java");
        assert!(source.contains("public void setAttribute(final Attribute attribute)"));
        assert!(source.contains("return new AttributeList();"));
        assert!(source.contains("public @Nullable Object invoke"));
        assert!(source.contains("return null;"));

        let statistics = MinecraftServerStatistics::new(vec![1], 2.0);
        statistics.set_attribute(Attribute {
            name: "averageTickTime".to_string(),
            value: AttributeValue::Float(99.0),
        });
        assert_eq!(
            statistics.get_attribute("averageTickTime"),
            Some(AttributeValue::Float(2.0))
        );
        assert!(statistics.set_attributes(&[]).is_empty());
        assert_eq!(statistics.invoke("reset", &[], &[]), None);
    }

    #[test]
    fn registration_uses_management_metrics_export_as_rust_equivalent() {
        let mut properties = match ServerProperties::load_or_default(std::path::Path::new(
            "definitely-missing-jmx-test-server.properties",
        )) {
            Ok(properties) => properties,
            Err(err) => panic!("failed to load default server properties: {err}"),
        };
        let management_plan = ManagementStartupPlan::Listen {
            host: "localhost".to_string(),
            port: 25585,
            tls: Some(TlsEndpoint {
                keystore: "server.p12".into(),
                password: "secret".into(),
            }),
            allowed_origins: AllowedOrigins::Empty,
        };
        assert_eq!(
            MinecraftServerStatistics::register_jmx_monitoring(&properties, &management_plan),
            JmxMonitoringPlan::Disabled
        );

        properties.set("enable-jmx-monitoring", "true");
        assert_eq!(
            MinecraftServerStatistics::register_jmx_monitoring(&properties, &management_plan),
            JmxMonitoringPlan::EquivalentMetricsExport {
                endpoint: "server/metrics",
                transport: "json-rpc-management",
            }
        );
        assert_eq!(
            MinecraftServerStatistics::register_jmx_monitoring(
                &properties,
                &ManagementStartupPlan::Disabled,
            ),
            JmxMonitoringPlan::RefusedManagementDisabled
        );
    }
}
