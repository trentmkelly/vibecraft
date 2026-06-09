use crate::registry::{Identifier, Lifecycle, Registry};

const BOOTSTRAP_CONTEXT_JAVA: &str = include_str!(
    "../../decompiled-server-26.1.2/net/minecraft/data/worldgen/BootstrapContext.java"
);

fn count_occurrences(source: &str, needle: &str) -> usize {
    source.match_indices(needle).count()
}

fn assert_source_contains_all(source: &str, sentinels: &[&str]) {
    for sentinel in sentinels {
        assert!(
            source.contains(sentinel),
            "BootstrapContext.java is missing sentinel: {sentinel}"
        );
    }
}

fn register_with_java_default_lifecycle<T>(
    registry: &mut Registry<T>,
    location: Identifier,
    value: T,
) -> Result<crate::registry::ResourceKey<T>, String> {
    registry.register(location, value, Lifecycle::Stable)
}

#[test]
fn bootstrap_context_java_contract_pins_default_stable_registration() {
    assert_eq!(BOOTSTRAP_CONTEXT_JAVA.lines().count(), 17);
    assert_source_contains_all(
        BOOTSTRAP_CONTEXT_JAVA,
        &[
            "package net.minecraft.data.worldgen;",
            "import com.mojang.serialization.Lifecycle;",
            "import net.minecraft.core.Holder;",
            "import net.minecraft.core.HolderGetter;",
            "import net.minecraft.core.Registry;",
            "import net.minecraft.resources.ResourceKey;",
            "public interface BootstrapContext<T> {",
            "Holder.Reference<T> register(ResourceKey<T> key, T value, Lifecycle lifecycle);",
            "default Holder.Reference<T> register(final ResourceKey<T> key, final T value) {",
            "return this.register(key, value, Lifecycle.stable());",
            "<S> HolderGetter<S> lookup(ResourceKey<? extends Registry<? extends S>> key);",
        ],
    );

    assert_eq!(
        count_occurrences(BOOTSTRAP_CONTEXT_JAVA, "Holder.Reference<T> register"),
        2
    );
    assert_eq!(
        count_occurrences(BOOTSTRAP_CONTEXT_JAVA, "Lifecycle.stable()"),
        1
    );
    assert_eq!(
        count_occurrences(BOOTSTRAP_CONTEXT_JAVA, "HolderGetter<S> lookup"),
        1
    );
}

#[test]
fn rust_registry_registration_matches_bootstrap_context_reference_semantics() {
    let registry_id = Identifier::parse("minecraft:worldgen/configured_feature").unwrap();
    let mut registry = Registry::new(registry_id.clone());
    let patch = Identifier::parse("minecraft:flower_plain").unwrap();
    let ore = Identifier::parse("minecraft:ore_diamond").unwrap();

    let default_key =
        register_with_java_default_lifecycle(&mut registry, patch.clone(), "flower patch")
            .expect("default registration should create a resource key");
    let explicit_key = registry
        .register(ore.clone(), "diamond ore", Lifecycle::Experimental)
        .expect("explicit lifecycle registration should create a resource key");

    assert_eq!(default_key.registry(), &registry_id);
    assert_eq!(default_key.location(), &patch);
    assert_eq!(explicit_key.registry(), &registry_id);
    assert_eq!(explicit_key.location(), &ore);

    let default_entry = registry
        .get(&patch)
        .expect("registered key should be lookupable");
    assert_eq!(default_entry.id(), 0);
    assert_eq!(default_entry.value(), &"flower patch");
    assert_eq!(default_entry.lifecycle(), Lifecycle::Stable);

    let explicit_entry = registry
        .get(&ore)
        .expect("registered key should be lookupable");
    assert_eq!(explicit_entry.id(), 1);
    assert_eq!(explicit_entry.value(), &"diamond ore");
    assert_eq!(explicit_entry.lifecycle(), Lifecycle::Experimental);
}

#[test]
fn rust_registry_lookup_preserves_bootstrap_context_registry_boundaries() {
    let feature_registry_id = Identifier::parse("minecraft:worldgen/configured_feature").unwrap();
    let carver_registry_id = Identifier::parse("minecraft:worldgen/configured_carver").unwrap();
    let mut features = Registry::new(feature_registry_id.clone());
    let mut carvers = Registry::new(carver_registry_id.clone());
    let shared_location = Identifier::parse("minecraft:cave").unwrap();

    features
        .register(shared_location.clone(), "feature cave", Lifecycle::Stable)
        .unwrap();
    carvers
        .register(shared_location.clone(), "carver cave", Lifecycle::Stable)
        .unwrap();

    let feature_entry = features.get(&shared_location).unwrap();
    let carver_entry = carvers.get(&shared_location).unwrap();
    assert_eq!(feature_entry.key().registry(), &feature_registry_id);
    assert_eq!(carver_entry.key().registry(), &carver_registry_id);
    assert_eq!(feature_entry.value(), &"feature cave");
    assert_eq!(carver_entry.value(), &"carver cave");

    let duplicate = features.register(shared_location, "duplicate", Lifecycle::Stable);
    assert!(duplicate
        .unwrap_err()
        .contains("duplicate registry key: minecraft:cave"));
}
