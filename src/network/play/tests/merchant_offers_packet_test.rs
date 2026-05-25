use super::*;

#[test]
fn clientbound_merchant_offers_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID, 52);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_MERCHANT_OFFERS_PACKET_ID),
        Some("merchant_offers")
    );

    let mut payload = Vec::new();
    ClientboundMerchantOffersPacket {
        container_id: 128,
        offers: vec![MerchantOfferData {
            base_cost_a: ItemCostData {
                item_id: 5,
                count: 3,
                components: RawDataComponentExactPredicate::empty(),
            },
            result: RawItemStack {
                count: 1,
                item_id: Some(6),
                components: RawDataComponentPatch::empty(),
            },
            cost_b: Some(ItemCostData {
                item_id: 7,
                count: 2,
                components: RawDataComponentExactPredicate::empty(),
            }),
            out_of_stock: true,
            uses: 1,
            max_uses: 12,
            xp: 4,
            special_price_diff: -2,
            price_multiplier: 0.05,
            demand: 9,
        }],
        villager_level: 3,
        villager_xp: 120,
        show_progress: true,
        can_restock: false,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        [
            vec![
                0x80, 0x01, 1, 5, 3, 0, 1, 6, 0, 0, 1, 7, 2, 0, 1
            ],
            1_i32.to_be_bytes().to_vec(),
            12_i32.to_be_bytes().to_vec(),
            4_i32.to_be_bytes().to_vec(),
            (-2_i32).to_be_bytes().to_vec(),
            0.05_f32.to_be_bytes().to_vec(),
            9_i32.to_be_bytes().to_vec(),
            vec![3, 120, 1, 0],
        ]
        .concat()
    );
}
