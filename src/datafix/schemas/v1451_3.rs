//! Port of `net.minecraft.util.datafix.schemas.V1451_3`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_entities(schema: &mut Schema) {
    schema.register_simple_entity("minecraft:egg");
    schema.register_simple_entity("minecraft:ender_pearl");
    schema.register_simple_entity("minecraft:fireball");
    schema.register_entity(
        "minecraft:potion",
        dsl::optional_fields(vec![("Potion", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:small_fireball");
    schema.register_simple_entity("minecraft:snowball");
    schema.register_simple_entity("minecraft:wither_skull");
    schema.register_simple_entity("minecraft:xp_bottle");
    schema.register_entity(
        "minecraft:arrow",
        dsl::optional_fields(vec![("inBlockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_entity(
        "minecraft:enderman",
        dsl::optional_fields(vec![("carriedBlockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_entity(
        "minecraft:falling_block",
        dsl::optional_fields(vec![
            ("BlockState", dsl::reference(r::BLOCK_STATE)),
            ("TileEntityData", dsl::reference(r::BLOCK_ENTITY)),
        ]),
    );
    schema.register_entity(
        "minecraft:spectral_arrow",
        dsl::optional_fields(vec![("inBlockState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_entity(
        "minecraft:chest_minecart",
        dsl::optional_fields(vec![
            ("DisplayState", dsl::reference(r::BLOCK_STATE)),
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_entity(
        "minecraft:commandblock_minecart",
        dsl::optional_fields(vec![
            ("DisplayState", dsl::reference(r::BLOCK_STATE)),
            ("LastOutput", dsl::reference(r::TEXT_COMPONENT)),
        ]),
    );
    schema.register_entity(
        "minecraft:furnace_minecart",
        dsl::optional_fields(vec![("DisplayState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_entity(
        "minecraft:hopper_minecart",
        dsl::optional_fields(vec![
            ("DisplayState", dsl::reference(r::BLOCK_STATE)),
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_entity(
        "minecraft:minecart",
        dsl::optional_fields(vec![("DisplayState", dsl::reference(r::BLOCK_STATE))]),
    );
    schema.register_entity(
        "minecraft:spawner_minecart",
        dsl::optional_fields_with_rest(
            vec![("DisplayState", dsl::reference(r::BLOCK_STATE))],
            dsl::reference(r::UNTAGGED_SPAWNER),
        ),
    );
    schema.register_entity(
        "minecraft:tnt_minecart",
        dsl::optional_fields(vec![("DisplayState", dsl::reference(r::BLOCK_STATE))]),
    );
}

/// Applies `V1451_3` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
