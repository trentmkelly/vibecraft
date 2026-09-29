//! Port of `net.minecraft.util.datafix.schemas.V702`.

use crate::datafix::references as r;
use crate::datafix::schema::Schema;
use crate::datafix::template::dsl;
fn register_entities(schema: &mut Schema) {
    schema.register_entity(
        "ZombieVillager",
        dsl::optional_fields(vec![(
            "Offers",
            dsl::optional_fields(vec![(
                "Recipes",
                dsl::list(dsl::reference(r::VILLAGER_TRADE)),
            )]),
        )]),
    );
    schema.register_simple_entity("Husk");
}

/// Applies `V702` on top of its parent schema.
pub fn build(schema: &mut Schema) {
    register_entities(schema);
}
