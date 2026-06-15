#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JavaPackageInfo {
    pub package: &'static str,
    pub null_marked: bool,
}

pub const CHAT_PACKAGE_INFOS: [JavaPackageInfo; 5] = [
    JavaPackageInfo {
        package: "net.minecraft.network.chat",
        null_marked: true,
    },
    JavaPackageInfo {
        package: "net.minecraft.network.chat.contents",
        null_marked: true,
    },
    JavaPackageInfo {
        package: "net.minecraft.network.chat.contents.data",
        null_marked: true,
    },
    JavaPackageInfo {
        package: "net.minecraft.network.chat.contents.objects",
        null_marked: true,
    },
    JavaPackageInfo {
        package: "net.minecraft.network.chat.numbers",
        null_marked: true,
    },
];

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;

    fn java_package_info_source(package: &str) -> &'static str {
        match package {
            "net.minecraft.network.chat" => vibecraft_java_source!("/net/minecraft/network/chat/package-info.java"),
            "net.minecraft.network.chat.contents" => vibecraft_java_source!("/net/minecraft/network/chat/contents/package-info.java"),
            "net.minecraft.network.chat.contents.data" => vibecraft_java_source!("/net/minecraft/network/chat/contents/data/package-info.java"),
            "net.minecraft.network.chat.contents.objects" => vibecraft_java_source!("/net/minecraft/network/chat/contents/objects/package-info.java"),
            "net.minecraft.network.chat.numbers" => vibecraft_java_source!("/net/minecraft/network/chat/numbers/package-info.java"),
            other => panic!("unknown package-info package {other}"),
        }
    }

    #[test]
    fn chat_package_info_files_are_null_marked_metadata_only() {
        assert_eq!(CHAT_PACKAGE_INFOS.len(), 5);
        for info in CHAT_PACKAGE_INFOS {
            let source = java_package_info_source(info.package);
            assert_eq!(info.null_marked, source.contains("@NullMarked"));
            assert!(source.contains(&format!("package {};", info.package)));
            assert!(source.contains("import org.jspecify.annotations.NullMarked;"));
            assert_eq!(source.matches("@NullMarked").count(), 1);
        }
    }
}
