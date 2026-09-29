//! Port of `net.minecraft.util.datafix.schemas.V1510`.

use crate::datafix::schema::Schema;

fn register_entities(schema: &mut Schema) {
    schema.rename_entity(
        "minecraft:commandblock_minecart",
        "minecraft:command_block_minecart",
    );
    schema.rename_entity("minecraft:ender_crystal", "minecraft:end_crystal");
    schema.rename_entity("minecraft:snowman", "minecraft:snow_golem");
    schema.rename_entity("minecraft:evocation_illager", "minecraft:evoker");
    schema.rename_entity("minecraft:evocation_fangs", "minecraft:evoker_fangs");
    schema.rename_entity("minecraft:illusion_illager", "minecraft:illusioner");
    schema.rename_entity("minecraft:vindication_illager", "minecraft:vindicator");
    schema.rename_entity("minecraft:villager_golem", "minecraft:iron_golem");
    schema.rename_entity("minecraft:xp_orb", "minecraft:experience_orb");
    schema.rename_entity("minecraft:xp_bottle", "minecraft:experience_bottle");
    schema.rename_entity("minecraft:eye_of_ender_signal", "minecraft:eye_of_ender");
    schema.rename_entity("minecraft:fireworks_rocket", "minecraft:firework_rocket");
}

/// Applies `V1510` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
