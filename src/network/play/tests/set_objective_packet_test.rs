use super::*;

fn text_component(text: &str) -> Tag {
    Tag::Compound(vec![("text".to_string(), Tag::String(text.to_string()))])
}

#[test]
fn clientbound_set_objective_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID, 106);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_SET_OBJECTIVE_PACKET_ID),
        Some("set_objective")
    );

    let mut add_blank = Vec::new();
    ClientboundSetObjectivePacket {
        objective_name: "kills".to_string(),
        method: ObjectiveMethod::Add {
            display_name: text_component("Kills"),
            render_type: ObjectiveRenderType::Hearts,
            number_format: Some(NumberFormat::Blank),
        },
    }
    .write(&mut add_blank)
    .unwrap();
    assert_eq!(
        add_blank,
        [
            vec![5],
            b"kills".to_vec(),
            vec![0, 10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 5],
            b"Kills".to_vec(),
            vec![0, 1, 1, 0],
        ]
        .concat()
    );

    let mut change_no_format = Vec::new();
    ClientboundSetObjectivePacket {
        objective_name: "kills".to_string(),
        method: ObjectiveMethod::Change {
            display_name: text_component("Kills"),
            render_type: ObjectiveRenderType::Integer,
            number_format: None,
        },
    }
    .write(&mut change_no_format)
    .unwrap();
    assert_eq!(
        change_no_format,
        [
            vec![5],
            b"kills".to_vec(),
            vec![2, 10, 8, 0, 4],
            b"text".to_vec(),
            vec![0, 5],
            b"Kills".to_vec(),
            vec![0, 0, 0],
        ]
        .concat()
    );

    let mut remove = Vec::new();
    ClientboundSetObjectivePacket {
        objective_name: "kills".to_string(),
        method: ObjectiveMethod::Remove,
    }
    .write(&mut remove)
    .unwrap();
    assert_eq!(remove, [vec![5], b"kills".to_vec(), vec![1]].concat());
}
