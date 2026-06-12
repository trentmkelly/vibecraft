use super::*;

const CLIENTBOUND_TICKING_STATE_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTickingStatePacket.java"
);
const CLIENTBOUND_TICKING_STEP_JAVA: &str = include_str!(
    "../../../../../decompiled-server-26.1.2/net/minecraft/network/protocol/game/ClientboundTickingStepPacket.java"
);

#[test]
fn clientbound_ticking_packets_match_java_codecs() {
    assert_java_contains(
        CLIENTBOUND_TICKING_STATE_JAVA,
        &[
            "this(input.readFloat(), input.readBoolean());",
            "output.writeFloat(this.tickRate);",
            "output.writeBoolean(this.isFrozen);",
            "return GamePacketTypes.CLIENTBOUND_TICKING_STATE;",
            "listener.handleTickingState(this);",
        ],
    );
    assert_java_contains(
        CLIENTBOUND_TICKING_STEP_JAVA,
        &[
            "this(input.readVarInt());",
            "output.writeVarInt(this.tickSteps);",
            "return GamePacketTypes.CLIENTBOUND_TICKING_STEP;",
            "listener.handleTickingStep(this);",
        ],
    );

    let registry = PlayProtocolRegistry::new();
    assert_eq!(registry.clientbound_name(127), Some("ticking_state"));
    assert_eq!(registry.clientbound_name(128), Some("ticking_step"));

    let ticking_state = ClientboundTickingStatePacket {
        tick_rate: 0.5,
        is_frozen: true,
    };
    let mut state_payload = Vec::new();
    ticking_state.write(&mut state_payload).unwrap();
    assert_eq!(
        state_payload,
        [0.5_f32.to_be_bytes().as_slice(), &[1]].concat()
    );
    assert_eq!(
        ClientboundTickingStatePacket::read(&mut cursor(state_payload)).unwrap(),
        ticking_state
    );

    let ticking_step = ClientboundTickingStepPacket { tick_steps: 300 };
    let mut step_payload = Vec::new();
    ticking_step.write(&mut step_payload).unwrap();
    assert_eq!(step_payload, vec![0xac, 0x02]);
    assert_eq!(
        ClientboundTickingStepPacket::read(&mut cursor(step_payload)).unwrap(),
        ticking_step
    );
}

fn assert_java_contains(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "Java source missing sentinel: {sentinel}"
        );
    }
}
