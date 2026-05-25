use super::*;

#[test]
fn clientbound_update_advancements_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID, 130);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_UPDATE_ADVANCEMENTS_PACKET_ID),
        Some("update_advancements")
    );

    let advancement_id = Identifier::parse("minecraft:story/mine_stone").unwrap();
    let parent_id = Identifier::parse("minecraft:story/root").unwrap();
    let removed_id = Identifier::parse("minecraft:story/hidden").unwrap();
    // Java DisplayInfo.STREAM_CODEC payload for:
    // title="Title", description="Desc", icon=(item id 1, count 1, empty patch),
    // frame=CHALLENGE, flags=showToast|hidden, x=1.5, y=-2.0.
    let display_payload = [
        vec![8, 0, 5],
        b"Title".to_vec(),
        vec![8, 0, 4],
        b"Desc".to_vec(),
        vec![1, 1, 0, 0, 1, 0, 0, 0, 6, 0x3f, 0xc0, 0, 0, 0xc0, 0, 0, 0],
    ]
    .concat();
    let mut payload = Vec::new();
    ClientboundAdvancementsPacket {
        reset: false,
        added: vec![AdvancementHolderData {
            id: advancement_id.clone(),
            value: AdvancementData {
                parent: Some(parent_id),
                display_payload: Some(display_payload.clone()),
                requirements: vec![
                    vec!["stone".to_string()],
                    vec!["iron".to_string(), "coal".to_string()],
                ],
                sends_telemetry_event: false,
            },
        }],
        removed: vec![removed_id],
        progress: vec![(
            advancement_id,
            AdvancementProgressData {
                criteria: vec![
                    (
                        "stone".to_string(),
                        CriterionProgressData {
                            obtained_epoch_millis: Some(1000),
                        },
                    ),
                    (
                        "iron".to_string(),
                        CriterionProgressData {
                            obtained_epoch_millis: None,
                        },
                    ),
                ],
            },
        )],
        show_advancements: false,
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(
        payload,
        [
            vec![0, 1, 26],
            b"minecraft:story/mine_stone".to_vec(),
            vec![1, 20],
            b"minecraft:story/root".to_vec(),
            vec![1],
            display_payload,
            vec![2, 1, 5],
            b"stone".to_vec(),
            vec![2, 4],
            b"iron".to_vec(),
            vec![4],
            b"coal".to_vec(),
            vec![0, 1, 22],
            b"minecraft:story/hidden".to_vec(),
            vec![1, 26],
            b"minecraft:story/mine_stone".to_vec(),
            vec![2, 5],
            b"stone".to_vec(),
            vec![1, 0, 0, 0, 0, 0, 0, 3, 0xe8, 4],
            b"iron".to_vec(),
            vec![0, 0],
        ]
        .concat()
    );
}
