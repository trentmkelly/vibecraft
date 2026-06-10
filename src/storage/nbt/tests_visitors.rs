const NBT_VISITORS_PACKAGE_INFO_JAVA: &str = include_str!(
    "../../../../decompiled-server-26.1.2/net/minecraft/nbt/visitors/package-info.java"
);

#[test]
fn nbt_visitors_package_info_matches_java_metadata() {
    assert!(NBT_VISITORS_PACKAGE_INFO_JAVA.contains("@NullMarked"));
    assert!(NBT_VISITORS_PACKAGE_INFO_JAVA.contains("package net.minecraft.nbt.visitors;"));
    assert!(NBT_VISITORS_PACKAGE_INFO_JAVA.contains("import org.jspecify.annotations.NullMarked;"));
}
