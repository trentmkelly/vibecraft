#![allow(dead_code)]

use super::dispatch::PacketFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketTypeModel {
    pub flow: PacketFlow,
    pub id: String,
}

impl PacketTypeModel {
    pub fn new(flow: PacketFlow, id: impl Into<String>) -> Self {
        Self {
            flow,
            id: id.into(),
        }
    }
}

impl std::fmt::Display for PacketTypeModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.flow.id(), self.id)
    }
}

pub trait PacketModel {
    fn packet_type(&self) -> &PacketTypeModel;
    fn handle(&mut self);

    fn is_skippable(&self) -> bool {
        false
    }

    fn is_terminal(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingPacketModel {
    packet_type: PacketTypeModel,
    skippable: bool,
    terminal: bool,
    handled: bool,
}

impl RecordingPacketModel {
    pub fn new(packet_type: PacketTypeModel) -> Self {
        Self {
            packet_type,
            skippable: false,
            terminal: false,
            handled: false,
        }
    }

    pub fn skippable(mut self) -> Self {
        self.skippable = true;
        self
    }

    pub fn terminal(mut self) -> Self {
        self.terminal = true;
        self
    }

    pub fn handled(&self) -> bool {
        self.handled
    }
}

impl PacketModel for RecordingPacketModel {
    fn packet_type(&self) -> &PacketTypeModel {
        &self.packet_type
    }

    fn handle(&mut self) {
        self.handled = true;
    }

    fn is_skippable(&self) -> bool {
        self.skippable
    }

    fn is_terminal(&self) -> bool {
        self.terminal
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningOnDifferentThread;

pub fn ensure_running_on_same_thread(
    is_same_thread: bool,
    mut schedule_if_possible: impl FnMut(),
) -> Result<(), RunningOnDifferentThread> {
    if is_same_thread {
        Ok(())
    } else {
        schedule_if_possible();
        Err(RunningOnDifferentThread)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketCrashDetails {
    pub packet_type: Option<String>,
    pub is_terminal: Option<String>,
    pub is_skippable: Option<String>,
    pub listener_details_filled: bool,
}

pub fn fill_packet_crash_report(packet: Option<&dyn PacketModel>) -> PacketCrashDetails {
    let (packet_type, is_terminal, is_skippable) = packet
        .map(|packet| {
            (
                Some(packet.packet_type().to_string()),
                Some(packet.is_terminal().to_string()),
                Some(packet.is_skippable().to_string()),
            )
        })
        .unwrap_or((None, None, None));
    PacketCrashDetails {
        packet_type,
        is_terminal,
        is_skippable,
        listener_details_filled: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportedPacketException {
    pub reused_existing_report: bool,
    pub title: String,
    pub details: PacketCrashDetails,
}

pub fn make_reported_packet_exception(
    already_reported: bool,
    packet: &dyn PacketModel,
) -> ReportedPacketException {
    ReportedPacketException {
        reused_existing_report: already_reported,
        title: "Main thread packet handler".to_string(),
        details: fill_packet_crash_report(Some(packet)),
    }
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    const PACKET_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/Packet.java"
    );
    const PACKET_TYPE_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/PacketType.java"
    );
    const PACKET_UTILS_JAVA: &str = include_str!(
        "../../../decompiled-server-26.1.2/net/minecraft/network/protocol/PacketUtils.java"
    );

    #[test]
    fn packet_and_packet_type_match_java_surface() {
        for sentinel in [
            "PacketType<? extends Packet<T>> type();",
            "void handle(T listener);",
            "default boolean isSkippable()",
            "return false;",
            "default boolean isTerminal()",
            "return StreamCodec.ofMember(writer, reader);",
        ] {
            assert!(
                PACKET_JAVA.contains(sentinel),
                "missing Packet Java sentinel {sentinel}"
            );
        }
        for sentinel in [
            "public record PacketType<T extends Packet<?>>(PacketFlow flow, Identifier id)",
            "return this.flow.id() + \"/\" + this.id;",
        ] {
            assert!(
                PACKET_TYPE_JAVA.contains(sentinel),
                "missing PacketType Java sentinel {sentinel}"
            );
        }

        let packet_type = PacketTypeModel::new(PacketFlow::Clientbound, "minecraft:add_entity");
        assert_eq!(packet_type.to_string(), "clientbound/minecraft:add_entity");
        let mut packet = RecordingPacketModel::new(packet_type);
        assert!(!packet.is_skippable());
        assert!(!packet.is_terminal());
        assert!(!packet.handled());
        packet.handle();
        assert!(packet.handled());

        let special = RecordingPacketModel::new(PacketTypeModel::new(
            PacketFlow::Serverbound,
            "minecraft:finish_configuration",
        ))
        .skippable()
        .terminal();
        assert!(special.is_skippable());
        assert!(special.is_terminal());
    }

    #[test]
    fn packet_utils_match_java_thread_and_crash_report_contract() {
        for sentinel in [
            "if (!packetProcessor.isSameThread())",
            "packetProcessor.scheduleIfPossible(listener, packet);",
            "throw RunningOnDifferentThreadException.RUNNING_ON_DIFFERENT_THREAD;",
            "if (cause instanceof ReportedException re)",
            "CrashReport.forThrowable(cause, \"Main thread packet handler\")",
            "CrashReportCategory details = report.addCategory(\"Incoming Packet\");",
            "details.setDetail(\"Type\", () -> packet.type().toString());",
            "details.setDetail(\"Is Terminal\", () -> Boolean.toString(packet.isTerminal()));",
            "details.setDetail(\"Is Skippable\", () -> Boolean.toString(packet.isSkippable()));",
            "listener.fillCrashReport(report);",
        ] {
            assert!(
                PACKET_UTILS_JAVA.contains(sentinel),
                "missing PacketUtils Java sentinel {sentinel}"
            );
        }

        let mut scheduled = 0;
        assert_eq!(
            ensure_running_on_same_thread(true, || scheduled += 1),
            Ok(())
        );
        assert_eq!(scheduled, 0);
        assert_eq!(
            ensure_running_on_same_thread(false, || scheduled += 1),
            Err(RunningOnDifferentThread)
        );
        assert_eq!(scheduled, 1);

        let packet = RecordingPacketModel::new(PacketTypeModel::new(
            PacketFlow::Serverbound,
            "minecraft:chat",
        ))
        .skippable();
        assert_eq!(
            fill_packet_crash_report(Some(&packet)),
            PacketCrashDetails {
                packet_type: Some("serverbound/minecraft:chat".to_string()),
                is_terminal: Some("false".to_string()),
                is_skippable: Some("true".to_string()),
                listener_details_filled: true,
            }
        );
        assert_eq!(
            fill_packet_crash_report(None),
            PacketCrashDetails {
                packet_type: None,
                is_terminal: None,
                is_skippable: None,
                listener_details_filled: true,
            }
        );
        assert_eq!(
            make_reported_packet_exception(true, &packet),
            ReportedPacketException {
                reused_existing_report: true,
                title: "Main thread packet handler".to_string(),
                details: fill_packet_crash_report(Some(&packet)),
            }
        );
    }
}
