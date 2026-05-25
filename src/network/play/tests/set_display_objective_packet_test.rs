use super::*;

#[test]
fn clientbound_set_display_objective_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID, 98);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_DISPLAY_OBJECTIVE_PACKET_ID),
        Some("set_display_objective")
    );

    let mut sidebar = Vec::new();
    ClientboundSetDisplayObjectivePacket {
        slot: 1,
        objective_name: "sidebar".to_string(),
    }
    .write(&mut sidebar)
    .unwrap();
    assert_eq!(sidebar, [vec![1, 7], b"sidebar".to_vec()].concat());

    let mut team_white_clear = Vec::new();
    ClientboundSetDisplayObjectivePacket {
        slot: 18,
        objective_name: String::new(),
    }
    .write(&mut team_white_clear)
    .unwrap();
    assert_eq!(team_white_clear, vec![18, 0]);
}
