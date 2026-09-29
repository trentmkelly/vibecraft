//! Port of `net.minecraft.util.datafix.schemas.V705`.

use super::v704_item_to_block_entity::ITEM_TO_BLOCK_ENTITY;
use super::v705_item_to_entity::ITEM_TO_ENTITY;
use super::v99;
use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::{dsl, ChoiceSet};
use crate::storage::nbt::Tag;

/// `V705.ADD_NAMES`: `V99.addNames` with the block entity table of `V704` and
/// the namespaced entity table of `V705`.
pub fn add_names(input: &mut Tag) {
    v99::add_names_with(input, ITEM_TO_BLOCK_ENTITY, ITEM_TO_ENTITY);
}

#[allow(clippy::too_many_lines)] // declarative registry mirroring one Java method
fn register_entities(schema: &mut Schema) {
    schema.clear_entities();
    schema.register_entity(
        "minecraft:area_effect_cloud",
        dsl::optional_fields(vec![("Particle", dsl::reference(r::PARTICLE))]),
    );
    schema.register_simple_entity("minecraft:armor_stand");
    schema.register_entity(
        "minecraft:arrow",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:bat");
    schema.register_simple_entity("minecraft:blaze");
    schema.register_simple_entity("minecraft:boat");
    schema.register_simple_entity("minecraft:cave_spider");
    schema.register_entity(
        "minecraft:chest_minecart",
        dsl::optional_fields(vec![
            ("DisplayTile", dsl::reference(r::BLOCK_NAME)),
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_simple_entity("minecraft:chicken");
    schema.register_entity(
        "minecraft:commandblock_minecart",
        dsl::optional_fields(vec![
            ("DisplayTile", dsl::reference(r::BLOCK_NAME)),
            ("LastOutput", dsl::reference(r::TEXT_COMPONENT)),
        ]),
    );
    schema.register_simple_entity("minecraft:cow");
    schema.register_simple_entity("minecraft:creeper");
    schema.register_entity(
        "minecraft:donkey",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:dragon_fireball");
    schema.register_entity(
        "minecraft:egg",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:elder_guardian");
    schema.register_simple_entity("minecraft:ender_crystal");
    schema.register_simple_entity("minecraft:ender_dragon");
    schema.register_entity(
        "minecraft:enderman",
        dsl::optional_fields(vec![("carried", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:endermite");
    schema.register_entity(
        "minecraft:ender_pearl",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:eye_of_ender_signal");
    schema.register_entity(
        "minecraft:falling_block",
        dsl::optional_fields(vec![
            ("Block", dsl::reference(r::BLOCK_NAME)),
            ("TileEntityData", dsl::reference(r::BLOCK_ENTITY)),
        ]),
    );
    schema.register_entity(
        "minecraft:fireball",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_entity(
        "minecraft:fireworks_rocket",
        dsl::optional_fields(vec![("FireworksItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_entity(
        "minecraft:furnace_minecart",
        dsl::optional_fields(vec![("DisplayTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:ghast");
    schema.register_simple_entity("minecraft:giant");
    schema.register_simple_entity("minecraft:guardian");
    schema.register_entity(
        "minecraft:hopper_minecart",
        dsl::optional_fields(vec![
            ("DisplayTile", dsl::reference(r::BLOCK_NAME)),
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
        ]),
    );
    schema.register_entity(
        "minecraft:horse",
        dsl::optional_fields(vec![
            ("ArmorItem", dsl::reference(r::ITEM_STACK)),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:husk");
    schema.register_entity(
        "minecraft:item",
        dsl::optional_fields(vec![("Item", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_entity(
        "minecraft:item_frame",
        dsl::optional_fields(vec![("Item", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:leash_knot");
    schema.register_simple_entity("minecraft:magma_cube");
    schema.register_entity(
        "minecraft:minecart",
        dsl::optional_fields(vec![("DisplayTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:mooshroom");
    schema.register_entity(
        "minecraft:mule",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:ocelot");
    schema.register_simple_entity("minecraft:painting");
    schema.register_simple_entity("minecraft:parrot");
    schema.register_simple_entity("minecraft:pig");
    schema.register_simple_entity("minecraft:polar_bear");
    schema.register_entity(
        "minecraft:potion",
        dsl::optional_fields(vec![
            ("Potion", dsl::reference(r::ITEM_STACK)),
            ("inTile", dsl::reference(r::BLOCK_NAME)),
        ]),
    );
    schema.register_simple_entity("minecraft:rabbit");
    schema.register_simple_entity("minecraft:sheep");
    schema.register_simple_entity("minecraft:shulker");
    schema.register_simple_entity("minecraft:shulker_bullet");
    schema.register_simple_entity("minecraft:silverfish");
    schema.register_simple_entity("minecraft:skeleton");
    schema.register_entity(
        "minecraft:skeleton_horse",
        dsl::optional_fields(vec![("SaddleItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:slime");
    schema.register_entity(
        "minecraft:small_fireball",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_entity(
        "minecraft:snowball",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:snowman");
    schema.register_entity(
        "minecraft:spawner_minecart",
        dsl::optional_fields_with_rest(
            vec![("DisplayTile", dsl::reference(r::BLOCK_NAME))],
            dsl::reference(r::UNTAGGED_SPAWNER),
        ),
    );
    schema.register_entity(
        "minecraft:spectral_arrow",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:spider");
    schema.register_simple_entity("minecraft:squid");
    schema.register_simple_entity("minecraft:stray");
    schema.register_simple_entity("minecraft:tnt");
    schema.register_entity(
        "minecraft:tnt_minecart",
        dsl::optional_fields(vec![("DisplayTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_entity(
        "minecraft:villager",
        dsl::optional_fields(vec![
            ("Inventory", dsl::list(dsl::reference(r::ITEM_STACK))),
            (
                "Offers",
                dsl::optional_fields(vec![(
                    "Recipes",
                    dsl::list(dsl::reference(r::VILLAGER_TRADE)),
                )]),
            ),
        ]),
    );
    schema.register_simple_entity("minecraft:villager_golem");
    schema.register_simple_entity("minecraft:witch");
    schema.register_simple_entity("minecraft:wither");
    schema.register_simple_entity("minecraft:wither_skeleton");
    schema.register_entity(
        "minecraft:wither_skull",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:wolf");
    schema.register_entity(
        "minecraft:xp_bottle",
        dsl::optional_fields(vec![("inTile", dsl::reference(r::BLOCK_NAME))]),
    );
    schema.register_simple_entity("minecraft:xp_orb");
    schema.register_simple_entity("minecraft:zombie");
    schema.register_entity(
        "minecraft:zombie_horse",
        dsl::optional_fields(vec![("SaddleItem", dsl::reference(r::ITEM_STACK))]),
    );
    schema.register_simple_entity("minecraft:zombie_pigman");
    schema.register_entity(
        "minecraft:zombie_villager",
        dsl::optional_fields(vec![(
            "Offers",
            dsl::optional_fields(vec![(
                "Recipes",
                dsl::list(dsl::reference(r::VILLAGER_TRADE)),
            )]),
        )]),
    );
    schema.register_simple_entity("minecraft:evocation_fangs");
    schema.register_simple_entity("minecraft:evocation_illager");
    schema.register_simple_entity("minecraft:illusion_illager");
    schema.register_entity(
        "minecraft:llama",
        dsl::optional_fields(vec![
            ("Items", dsl::list(dsl::reference(r::ITEM_STACK))),
            ("SaddleItem", dsl::reference(r::ITEM_STACK)),
            ("DecorItem", dsl::reference(r::ITEM_STACK)),
        ]),
    );
    schema.register_simple_entity("minecraft:llama_spit");
    schema.register_simple_entity("minecraft:vex");
    schema.register_simple_entity("minecraft:vindication_illager");
}

fn register_types(schema: &mut Schema) {
    schema.register_type(
        r::ENTITY,
        dsl::and(vec![
            dsl::reference(r::ENTITY_EQUIPMENT),
            dsl::optional_fields_with_rest(
                vec![("CustomName", dsl::string())],
                dsl::namespaced_choice(ChoiceSet::Entities),
            ),
        ]),
    );
    schema.register_type(
        r::ITEM_STACK,
        dsl::hook(
            dsl::optional_fields(vec![
                ("id", dsl::reference(r::ITEM_NAME)),
                ("tag", v99::item_stack_tag()),
            ]),
            add_names,
        ),
    );
}

/// Applies `V705` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
    register_types(schema);
}
