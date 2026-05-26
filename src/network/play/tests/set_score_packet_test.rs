use super::*;

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

#[test]
fn clientbound_set_score_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_SCORE_PACKET_ID, 110);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_SCORE_PACKET_ID),
        Some("set_score")
    );

    let mut empty_optionals = Vec::new();
    ClientboundSetScorePacket {
        owner: "Steve".to_string(),
        objective_name: "deaths".to_string(),
        score: 0,
        display: None,
        number_format: None,
    }
    .write(&mut empty_optionals)
    .unwrap();
    assert_eq!(
        empty_optionals,
        [
            vec![5],
            b"Steve".to_vec(),
            vec![6],
            b"deaths".to_vec(),
            vec![0, 0, 0]
        ]
        .concat()
    );

    let mut populated = Vec::new();
    ClientboundSetScorePacket {
        owner: "Alex".to_string(),
        objective_name: "kills".to_string(),
        score: 300,
        display: Some(text_component("Alex")),
        number_format: Some(NumberFormat::Fixed {
            value: text_component("300"),
        }),
    }
    .write(&mut populated)
    .unwrap();
    assert_eq!(
        populated,
        [
            vec![4],
            b"Alex".to_vec(),
            vec![5],
            b"kills".to_vec(),
            vec![0xac, 0x02, 1, 10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 4],
            b"Alex".to_vec(),
            vec![0, 1, 2, 10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 3],
            b"300".to_vec(),
            vec![0],
        ]
        .concat()
    );
}
