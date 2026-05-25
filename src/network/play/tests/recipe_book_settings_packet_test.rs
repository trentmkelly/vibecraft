use super::*;

#[test]
fn clientbound_recipe_book_settings_packet_matches_java_codec() {
    assert_eq!(CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID, 76);
    let registry = PlayProtocolRegistry::new();
    assert_eq!(
        registry.clientbound_name(CLIENTBOUND_RECIPE_BOOK_SETTINGS_PACKET_ID),
        Some("recipe_book_settings")
    );

    let mut payload = Vec::new();
    ClientboundRecipeBookSettingsPacket {
        crafting: RecipeBookTypeSettings {
            open: true,
            filtering: false,
        },
        furnace: RecipeBookTypeSettings {
            open: false,
            filtering: true,
        },
        blast_furnace: RecipeBookTypeSettings {
            open: true,
            filtering: true,
        },
        smoker: RecipeBookTypeSettings {
            open: false,
            filtering: false,
        },
    }
    .write(&mut payload)
    .unwrap();

    assert_eq!(payload, vec![1, 0, 0, 1, 1, 1, 0, 0]);
}
