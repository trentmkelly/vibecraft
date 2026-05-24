use super::*;

#[test]
fn attribute_command_rejects_non_living_targets_separately_from_missing_attributes() {
    let mut state = ServerCommandState {
        entity_states: vec![EntityState {
            entity: EntityRef {
                id: "minecart".to_string(),
                display_name: "Minecart".to_string(),
            },
            kind: EntityKind::NonLiving,
            dimension: "minecraft:overworld".to_string(),
        }],
        entity_attributes: vec![EntityAttributeState {
            target: "minecart".to_string(),
            attribute: "minecraft:max_health".to_string(),
            default_base: 20.0,
            base: 20.0,
            modifiers: Vec::new(),
        }],
        ..ServerCommandState::default()
    };

    assert_eq!(
        execute_builtin_command(
            &mut state,
            LevelBasedPermissionSet::GAMEMASTER,
            "attribute minecart minecraft:max_health get",
        ),
        Err(CommandError::AttributeNotLiving)
    );
}
