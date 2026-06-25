use super::*;

const CLIENTBOUND_MERCHANT_OFFERS_JAVA: &str = vibecraft_java_source!("/net/minecraft/network/protocol/game/ClientboundMerchantOffersPacket.java");
const MERCHANT_OFFERS_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/item/trading/MerchantOffers.java");
const MERCHANT_OFFER_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/item/trading/MerchantOffer.java");
const ITEM_COST_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/world/item/trading/ItemCost.java");

#[test]
fn clientbound_merchant_offers_packet_matches_java_codec() {
    assert_java_sources_match_merchant_offer_wire_order();
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
            vec![0x80, 0x01, 1, 5, 3, 0, 1, 6, 0, 0, 1, 7, 2, 0, 1],
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

#[test]
fn clientbound_merchant_offers_packet_matches_java_absent_second_cost() {
    let mut payload = Vec::new();
    ClientboundMerchantOffersPacket {
        container_id: 2,
        offers: vec![MerchantOfferData {
            base_cost_a: ItemCostData {
                item_id: 5,
                count: 1,
                components: RawDataComponentExactPredicate::empty(),
            },
            result: RawItemStack {
                count: 4,
                item_id: Some(6),
                components: RawDataComponentPatch::empty(),
            },
            cost_b: None,
            out_of_stock: false,
            uses: 0,
            max_uses: 16,
            xp: 2,
            special_price_diff: 0,
            price_multiplier: 0.2,
            demand: 0,
        }],
        villager_level: 1,
        villager_xp: 5,
        show_progress: false,
        can_restock: true,
    }
    .write(&mut payload)
    .unwrap();
    assert_eq!(
        payload,
        [
            vec![2, 1, 5, 1, 0, 4, 6, 0, 0, 0, 0],
            0_i32.to_be_bytes().to_vec(),
            16_i32.to_be_bytes().to_vec(),
            2_i32.to_be_bytes().to_vec(),
            0_i32.to_be_bytes().to_vec(),
            0.2_f32.to_be_bytes().to_vec(),
            0_i32.to_be_bytes().to_vec(),
            vec![1, 5, 0, 1],
        ]
        .concat()
    );
}

fn assert_java_sources_match_merchant_offer_wire_order() {
    assert_java_contains(
        CLIENTBOUND_MERCHANT_OFFERS_JAVA,
        &[
            "this.containerId = input.readContainerId();",
            "this.offers = MerchantOffers.STREAM_CODEC.decode(input);",
            "this.villagerLevel = input.readVarInt();",
            "this.villagerXp = input.readVarInt();",
            "this.showProgress = input.readBoolean();",
            "this.canRestock = input.readBoolean();",
            "output.writeContainerId(this.containerId);",
            "MerchantOffers.STREAM_CODEC.encode(output, this.offers);",
            "output.writeVarInt(this.villagerLevel);",
            "output.writeVarInt(this.villagerXp);",
            "output.writeBoolean(this.showProgress);",
            "output.writeBoolean(this.canRestock);",
            "return GamePacketTypes.CLIENTBOUND_MERCHANT_OFFERS;",
            "listener.handleMerchantOffers(this);",
        ],
        "ClientboundMerchantOffersPacket",
    );
    assert_java_contains(
        MERCHANT_OFFERS_JAVA,
        &["MerchantOffer.STREAM_CODEC", "ByteBufCodecs.collection(MerchantOffers::new)"],
        "MerchantOffers",
    );
    assert_java_contains(
        MERCHANT_OFFER_JAVA,
        &[
            "ItemCost.STREAM_CODEC.encode(output, offer.getItemCostA());",
            "ItemStack.STREAM_CODEC.encode(output, offer.getResult());",
            "ItemCost.OPTIONAL_STREAM_CODEC.encode(output, offer.getItemCostB());",
            "output.writeBoolean(offer.isOutOfStock());",
            "output.writeInt(offer.getUses());",
            "output.writeInt(offer.getMaxUses());",
            "output.writeInt(offer.getXp());",
            "output.writeInt(offer.getSpecialPriceDiff());",
            "output.writeFloat(offer.getPriceMultiplier());",
            "output.writeInt(offer.getDemand());",
        ],
        "MerchantOffer",
    );
    assert_java_contains(
        ITEM_COST_JAVA,
        &[
            "Item.STREAM_CODEC, ItemCost::item",
            "ByteBufCodecs.VAR_INT, ItemCost::count",
            "DataComponentExactPredicate.STREAM_CODEC, ItemCost::components",
            "OPTIONAL_STREAM_CODEC = STREAM_CODEC.apply(ByteBufCodecs::optional)",
        ],
        "ItemCost",
    );
}

fn assert_java_contains(source: &str, sentinels: &[&str], class_name: &str) {
    for sentinel in sentinels {
        assert!(source.contains(sentinel), "missing {class_name} sentinel {sentinel}");
    }
}
