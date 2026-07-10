#![allow(dead_code)]

use crate::jsonrpc_api::JsonRpcPlayerDto;

pub trait ServerPlayerConnection<P> {
    fn get_player(&self) -> &JsonRpcPlayerDto;
    fn send(&mut self, packet: P);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPlayerConnectionModel<P> {
    player: JsonRpcPlayerDto,
    pub sent_packets: Vec<P>,
}

impl<P> ServerPlayerConnectionModel<P> {
    pub const fn new(player: JsonRpcPlayerDto) -> Self {
        Self {
            player,
            sent_packets: Vec::new(),
        }
    }
}

impl<P> ServerPlayerConnection<P> for ServerPlayerConnectionModel<P> {
    fn get_player(&self) -> &JsonRpcPlayerDto {
        &self.player
    }

    fn send(&mut self, packet: P) {
        self.sent_packets.push(packet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Packet(&'static str);

    #[test]
    fn connection_owns_player_and_sends_packets_in_order() {
        let player = JsonRpcPlayerDto::new(
            Some("11111111-1111-1111-1111-111111111111".to_string()),
            Some("Steve".to_string()),
        );
        let mut connection = ServerPlayerConnectionModel::new(player.clone());
        assert_eq!(connection.get_player(), &player);
        connection.send(Packet("first"));
        connection.send(Packet("second"));
        assert_eq!(
            connection.sent_packets,
            vec![Packet("first"), Packet("second")]
        );
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn interface_matches_java_26_1_2() {
        const SOURCE: &str = vibecraft_java_source!(
            "/net/minecraft/server/network/ServerPlayerConnection.java"
        );
        for sentinel in [
            "import net.minecraft.network.protocol.Packet;",
            "import net.minecraft.server.level.ServerPlayer;",
            "public interface ServerPlayerConnection",
            "ServerPlayer getPlayer();",
            "void send(final Packet<?> packet);",
        ] {
            assert!(
                SOURCE.contains(sentinel),
                "ServerPlayerConnection.java missing: {sentinel}"
            );
        }
    }
}
