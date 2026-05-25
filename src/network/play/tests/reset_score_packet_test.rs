use super::*;

#[test]
fn clientbound_reset_score_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_RESET_SCORE_PACKET_ID, 79);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RESET_SCORE_PACKET_ID),
        Some("reset_score")
    );

    let mut with_objective = Vec::new();
    ClientboundResetScorePacket {
        owner: "Alex".to_string(),
        objective_name: Some("kills".to_string()),
    }
    .write(&mut with_objective)
    .unwrap();
    assert_eq!(
        with_objective,
        [vec![4], b"Alex".to_vec(), vec![1, 5], b"kills".to_vec()].concat()
    );

    let mut without_objective = Vec::new();
    ClientboundResetScorePacket {
        owner: "Steve".to_string(),
        objective_name: None,
    }
    .write(&mut without_objective)
    .unwrap();
    assert_eq!(without_objective, [vec![5], b"Steve".to_vec(), vec![0]].concat());
}
