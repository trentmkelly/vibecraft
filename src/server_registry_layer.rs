#![allow(dead_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryLayer {
    Static,
    Worldgen,
    Dimensions,
    Reloadable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryAccessKind {
    Empty,
    StaticRegistryOfRegistries,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryLayerAccess {
    layers: Vec<(RegistryLayer, RegistryAccessKind)>,
}

impl RegistryLayer {
    pub const VALUES: [Self; 4] = [
        Self::Static,
        Self::Worldgen,
        Self::Dimensions,
        Self::Reloadable,
    ];

    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Static => "STATIC",
            Self::Worldgen => "WORLDGEN",
            Self::Dimensions => "DIMENSIONS",
            Self::Reloadable => "RELOADABLE",
        }
    }

    pub fn create_registry_access() -> RegistryLayerAccess {
        RegistryLayerAccess {
            layers: Self::VALUES
                .into_iter()
                .map(|layer| {
                    let access = if layer == Self::Static {
                        RegistryAccessKind::StaticRegistryOfRegistries
                    } else {
                        RegistryAccessKind::Empty
                    };
                    (layer, access)
                })
                .collect(),
        }
    }
}

impl RegistryLayerAccess {
    pub fn layers(&self) -> &[(RegistryLayer, RegistryAccessKind)] {
        &self.layers
    }

    pub fn access_for(&self, layer: RegistryLayer) -> Option<RegistryAccessKind> {
        self.layers
            .iter()
            .find_map(|(candidate, access)| (*candidate == layer).then_some(*access))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str = vibecraft_java_source!("/net/minecraft/server/RegistryLayer.java");

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_utility_registry_layer_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains("public enum RegistryLayer"));
        assert!(JAVA_SOURCE.contains("STATIC,\n   WORLDGEN,\n   DIMENSIONS,\n   RELOADABLE;"));
        assert!(JAVA_SOURCE.contains("private static final List<RegistryLayer> VALUES = List.of(values());"));
        assert!(JAVA_SOURCE.contains(
            "private static final RegistryAccess.Frozen STATIC_ACCESS = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);"
        ));
        assert!(JAVA_SOURCE.contains("public static LayeredRegistryAccess<RegistryLayer> createRegistryAccess()"));
        assert!(JAVA_SOURCE.contains("return new LayeredRegistryAccess<>(VALUES).replaceFrom(STATIC, STATIC_ACCESS);"));
    }

    #[test]
    fn server_utility_registry_layer_values_match_java_order() {
        assert_eq!(
            RegistryLayer::VALUES.map(RegistryLayer::serialized_name),
            ["STATIC", "WORLDGEN", "DIMENSIONS", "RELOADABLE"]
        );
    }

    #[test]
    fn server_utility_registry_layer_create_registry_access_replaces_static_only() {
        let access = RegistryLayer::create_registry_access();

        assert_eq!(
            access.layers(),
            &[
                (
                    RegistryLayer::Static,
                    RegistryAccessKind::StaticRegistryOfRegistries
                ),
                (RegistryLayer::Worldgen, RegistryAccessKind::Empty),
                (RegistryLayer::Dimensions, RegistryAccessKind::Empty),
                (RegistryLayer::Reloadable, RegistryAccessKind::Empty),
            ]
        );
        assert_eq!(
            access.access_for(RegistryLayer::Static),
            Some(RegistryAccessKind::StaticRegistryOfRegistries)
        );
        assert_eq!(
            access.access_for(RegistryLayer::Reloadable),
            Some(RegistryAccessKind::Empty)
        );
    }
}
