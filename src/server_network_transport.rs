#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLoopGroupKind {
    Nio,
    Epoll,
    Kqueue,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLoopGroupHolderModel {
    kind: EventLoopGroupKind,
    channel_class: &'static str,
    server_channel_class: &'static str,
    group: Option<EventLoopGroupModel>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventLoopGroupModel {
    pub kind: EventLoopGroupKind,
    pub thread_name_format: String,
    pub io_handler_factory: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIntentModel {
    Login,
    Status,
    Transfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryHandshakeAction {
    SetupInboundLogin { transferred: bool },
    SetupOutboundLogin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryServerHandshakeModel {
    connected: bool,
    actions: Vec<MemoryHandshakeAction>,
}

impl EventLoopGroupHolderModel {
    pub fn remote(
        allow_native_transport: bool,
        kqueue_available: bool,
        epoll_available: bool,
    ) -> Self {
        if allow_native_transport {
            if kqueue_available {
                return Self::new(EventLoopGroupKind::Kqueue);
            }

            if epoll_available {
                return Self::new(EventLoopGroupKind::Epoll);
            }
        }

        Self::new(EventLoopGroupKind::Nio)
    }

    pub fn local() -> Self {
        Self::new(EventLoopGroupKind::Local)
    }

    pub fn event_loop_group(&mut self) -> &EventLoopGroupModel {
        if self.group.is_none() {
            self.group = Some(self.create_event_loop_group());
        }

        match &self.group {
            Some(group) => group,
            None => unreachable!("event loop group is initialized above"),
        }
    }

    pub fn channel_class(&self) -> &'static str {
        self.channel_class
    }

    pub fn server_channel_class(&self) -> &'static str {
        self.server_channel_class
    }

    pub fn kind(&self) -> EventLoopGroupKind {
        self.kind
    }

    fn new(kind: EventLoopGroupKind) -> Self {
        let (channel_class, server_channel_class) = match kind {
            EventLoopGroupKind::Nio => ("NioSocketChannel", "NioServerSocketChannel"),
            EventLoopGroupKind::Epoll => ("EpollSocketChannel", "EpollServerSocketChannel"),
            EventLoopGroupKind::Kqueue => ("KQueueSocketChannel", "KQueueServerSocketChannel"),
            EventLoopGroupKind::Local => ("LocalChannel", "LocalServerChannel"),
        };
        Self {
            kind,
            channel_class,
            server_channel_class,
            group: None,
        }
    }

    fn create_event_loop_group(&self) -> EventLoopGroupModel {
        EventLoopGroupModel {
            kind: self.kind,
            thread_name_format: format!("Netty {} IO #%d", self.type_name()),
            io_handler_factory: self.io_handler_factory(),
        }
    }

    fn type_name(&self) -> &'static str {
        match self.kind {
            EventLoopGroupKind::Nio => "NIO",
            EventLoopGroupKind::Epoll => "Epoll",
            EventLoopGroupKind::Kqueue => "Kqueue",
            EventLoopGroupKind::Local => "Local",
        }
    }

    fn io_handler_factory(&self) -> &'static str {
        match self.kind {
            EventLoopGroupKind::Nio => "NioIoHandler.newFactory",
            EventLoopGroupKind::Epoll => "EpollIoHandler.newFactory",
            EventLoopGroupKind::Kqueue => "KQueueIoHandler.newFactory",
            EventLoopGroupKind::Local => "LocalIoHandler.newFactory",
        }
    }
}

impl MemoryServerHandshakeModel {
    pub fn new(connected: bool) -> Self {
        Self {
            connected,
            actions: Vec::new(),
        }
    }

    pub fn handle_intention(&mut self, intention: ClientIntentModel) -> Result<(), String> {
        if intention != ClientIntentModel::Login {
            return Err(format!("Invalid intention {intention:?}"));
        }

        self.actions.push(MemoryHandshakeAction::SetupInboundLogin {
            transferred: false,
        });
        self.actions.push(MemoryHandshakeAction::SetupOutboundLogin);
        Ok(())
    }

    pub fn on_disconnect(&mut self, _details: &str) {}

    pub fn is_accepting_messages(&self) -> bool {
        self.connected
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    pub fn actions(&self) -> &[MemoryHandshakeAction] {
        &self.actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_holder_prefers_kqueue_then_epoll_then_nio_like_java() {
        assert_eq!(
            EventLoopGroupHolderModel::remote(true, true, true).kind(),
            EventLoopGroupKind::Kqueue
        );
        assert_eq!(
            EventLoopGroupHolderModel::remote(true, false, true).kind(),
            EventLoopGroupKind::Epoll
        );
        assert_eq!(
            EventLoopGroupHolderModel::remote(true, false, false).kind(),
            EventLoopGroupKind::Nio
        );
        assert_eq!(
            EventLoopGroupHolderModel::remote(false, true, true).kind(),
            EventLoopGroupKind::Nio
        );
    }

    #[test]
    fn holder_channel_classes_thread_names_and_lazy_group_match_java() {
        let mut nio = EventLoopGroupHolderModel::remote(false, false, false);
        assert_eq!(nio.channel_class(), "NioSocketChannel");
        assert_eq!(nio.server_channel_class(), "NioServerSocketChannel");
        let first = nio.event_loop_group().clone();
        let second = nio.event_loop_group().clone();
        assert_eq!(first, second);
        assert_eq!(first.thread_name_format, "Netty NIO IO #%d");
        assert_eq!(first.io_handler_factory, "NioIoHandler.newFactory");

        let mut local = EventLoopGroupHolderModel::local();
        assert_eq!(local.channel_class(), "LocalChannel");
        assert_eq!(local.server_channel_class(), "LocalServerChannel");
        assert_eq!(
            local.event_loop_group(),
            &EventLoopGroupModel {
                kind: EventLoopGroupKind::Local,
                thread_name_format: "Netty Local IO #%d".to_string(),
                io_handler_factory: "LocalIoHandler.newFactory",
            }
        );
    }

    #[test]
    fn memory_handshake_accepts_only_login_and_sets_login_protocols() {
        let mut listener = MemoryServerHandshakeModel::new(true);

        assert!(listener.is_accepting_messages());
        assert_eq!(listener.handle_intention(ClientIntentModel::Login), Ok(()));
        assert_eq!(
            listener.actions(),
            &[
                MemoryHandshakeAction::SetupInboundLogin { transferred: false },
                MemoryHandshakeAction::SetupOutboundLogin,
            ]
        );

        assert_eq!(
            MemoryServerHandshakeModel::new(true).handle_intention(ClientIntentModel::Status),
            Err("Invalid intention Status".to_string())
        );
        assert_eq!(
            MemoryServerHandshakeModel::new(true).handle_intention(ClientIntentModel::Transfer),
            Err("Invalid intention Transfer".to_string())
        );

        listener.on_disconnect("ignored");
        assert!(listener.is_accepting_messages());
        listener.set_connected(false);
        assert!(!listener.is_accepting_messages());
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn transport_sources_match_java_26_1_2() {
        const EVENT_LOOP_GROUP_HOLDER: &str =
            vibecraft_java_source!("/net/minecraft/server/network/EventLoopGroupHolder.java");
        const MEMORY_HANDSHAKE: &str = vibecraft_java_source!(
            "/net/minecraft/server/network/MemoryServerHandshakePacketListenerImpl.java"
        );

        for sentinel in [
            "public abstract class EventLoopGroupHolder",
            "new EventLoopGroupHolder(\"NIO\", NioSocketChannel.class, NioServerSocketChannel.class)",
            "return NioIoHandler.newFactory();",
            "new EventLoopGroupHolder(\"Epoll\", EpollSocketChannel.class, EpollServerSocketChannel.class)",
            "return EpollIoHandler.newFactory();",
            "new EventLoopGroupHolder(\"Kqueue\", KQueueSocketChannel.class, KQueueServerSocketChannel.class)",
            "return KQueueIoHandler.newFactory();",
            "new EventLoopGroupHolder(\"Local\", LocalChannel.class, LocalServerChannel.class)",
            "return LocalIoHandler.newFactory();",
            "public static EventLoopGroupHolder remote(final boolean allowNativeTransport)",
            "if (KQueue.isAvailable())",
            "return KQUEUE;",
            "if (Epoll.isAvailable())",
            "return EPOLL;",
            "return NIO;",
            "public static EventLoopGroupHolder local()",
            "return LOCAL;",
            "setNameFormat(\"Netty \" + this.type + \" IO #%d\").setDaemon(true).build();",
            "return new MultiThreadIoEventLoopGroup(this.createThreadFactory(), this.ioHandlerFactory());",
            "public EventLoopGroup eventLoopGroup()",
            "synchronized (this)",
            "this.group = result;",
            "public Class<? extends Channel> channelCls()",
            "public Class<? extends ServerChannel> serverChannelCls()",
        ] {
            assert!(
                EVENT_LOOP_GROUP_HOLDER.contains(sentinel),
                "EventLoopGroupHolder.java is missing sentinel: {sentinel}"
            );
        }

        for sentinel in [
            "public class MemoryServerHandshakePacketListenerImpl implements ServerHandshakePacketListener",
            "private final MinecraftServer server;",
            "private final Connection connection;",
            "public void handleIntention(final ClientIntentionPacket packet)",
            "if (packet.intention() != ClientIntent.LOGIN)",
            "throw new UnsupportedOperationException(\"Invalid intention \" + packet.intention());",
            "this.connection.setupInboundProtocol(LoginProtocols.SERVERBOUND, new ServerLoginPacketListenerImpl(this.server, this.connection, false));",
            "this.connection.setupOutboundProtocol(LoginProtocols.CLIENTBOUND);",
            "public void onDisconnect(final DisconnectionDetails details)",
            "public boolean isAcceptingMessages()",
            "return this.connection.isConnected();",
        ] {
            assert!(
                MEMORY_HANDSHAKE.contains(sentinel),
                "MemoryServerHandshakePacketListenerImpl.java is missing sentinel: {sentinel}"
            );
        }
    }
}
