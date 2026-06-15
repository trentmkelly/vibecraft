const DATA_RECIPES_PACKAGE_JAVA: &str =
    vibecraft_java_source!("/net/minecraft/data/recipes/package-info.java");

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipes_package_is_null_marked() {
        assert!(DATA_RECIPES_PACKAGE_JAVA.contains("@NullMarked"));
        assert!(DATA_RECIPES_PACKAGE_JAVA.contains("package net.minecraft.data.recipes;"));
        assert!(DATA_RECIPES_PACKAGE_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
    }

    #[test]
    fn recipes_package_source_counts_match_authoritative_java() {
        assert_eq!(
            count_occurrences(DATA_RECIPES_PACKAGE_JAVA, "@NullMarked"),
            1
        );
        assert_eq!(
            count_occurrences(
                DATA_RECIPES_PACKAGE_JAVA,
                "package net.minecraft.data.recipes;"
            ),
            1
        );
        assert_eq!(
            count_occurrences(
                DATA_RECIPES_PACKAGE_JAVA,
                "import org.jspecify.annotations.NullMarked;"
            ),
            1
        );
    }
}
