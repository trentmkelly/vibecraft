const WORLDGEN_PACKAGE_INFO_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/package-info.java"
);

const WORLDGEN_FEATURES_PACKAGE_INFO_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/data/worldgen/features/package-info.java"
);

fn assert_null_marked_package_info(source: &str, package_name: &str) {
    assert_eq!(source.lines().count(), 4);
    assert_eq!(source.match_indices("@NullMarked").count(), 1);
    assert!(source.contains(&format!("package {package_name};")));
    assert!(source.contains("import org.jspecify.annotations.NullMarked;"));
}

#[test]
fn worldgen_package_info_is_null_marked_metadata_only() {
    assert_null_marked_package_info(WORLDGEN_PACKAGE_INFO_JAVA, "net.minecraft.data.worldgen");
}

#[test]
fn worldgen_features_package_info_is_null_marked_metadata_only() {
    assert_null_marked_package_info(
        WORLDGEN_FEATURES_PACKAGE_INFO_JAVA,
        "net.minecraft.data.worldgen.features",
    );
}
