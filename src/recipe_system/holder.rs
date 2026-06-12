use std::{
    fmt,
    hash::{Hash, Hasher},
};

use super::RecipeKind;

/// `RecipeHolder` — Java's `(ResourceKey<Recipe<?>> id, Recipe value)` record.
///
/// Java interns `ResourceKey`s and defines equality/hash/toString in terms of the
/// key, so the Rust holder intentionally compares and hashes by `id` only.
#[derive(Debug, Clone, Eq)]
pub struct RecipeHolder {
    pub id: &'static str,
    pub recipe: RecipeKind,
}

impl PartialEq for RecipeHolder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for RecipeHolder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for RecipeHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ResourceKey[minecraft:recipe / {}]", self.id)
    }
}

impl RecipeHolder {
    #[cfg(test)]
    pub fn get_id(&self) -> &'static str {
        self.id
    }

    #[cfg(test)]
    pub fn get_serializer(&self) -> &'static str {
        self.recipe.serializer()
    }

    #[cfg(test)]
    pub fn get_type(&self) -> &'static str {
        self.recipe.recipe_type()
    }

    #[cfg(test)]
    pub fn get_result_item(&self) -> Option<super::ItemAmount> {
        self.recipe.assemble()
    }
}
