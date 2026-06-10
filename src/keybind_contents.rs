use std::sync::{Mutex, OnceLock};

use crate::chat_component::{Component, Style};

pub const KEYBIND_CONTENTS_CODEC_FIELD: &str = "keybind";

#[derive(Debug, Clone)]
pub struct KeybindSupplierModel {
    name: String,
    resolve: fn(&str) -> Component,
}

#[derive(Debug, Clone)]
pub struct KeybindContentsModel {
    name: String,
    name_resolver: Option<KeybindSupplierModel>,
}

type KeyResolver = fn(&str) -> KeybindSupplierModel;

impl KeybindSupplierModel {
    pub fn new(name: impl Into<String>, resolve: fn(&str) -> Component) -> Self {
        Self {
            name: name.into(),
            resolve,
        }
    }

    pub fn get(&self) -> Component {
        (self.resolve)(&self.name)
    }
}

impl KeybindContentsModel {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            name_resolver: None,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn codec_field(&self) -> &'static str {
        KEYBIND_CONTENTS_CODEC_FIELD
    }

    pub fn visit<T>(&mut self, output: &mut impl FnMut(&str) -> Option<T>) -> Option<T> {
        output(&self.get_nested_component().get_string())
    }

    pub fn visit_styled<T>(
        &mut self,
        output: &mut impl FnMut(&Style, &str) -> Option<T>,
        current_style: &Style,
    ) -> Option<T> {
        output(current_style, &self.get_nested_component().get_string())
    }

    pub fn java_hash_code(&self) -> i32 {
        let mut hash = 0i32;
        for unit in self.name.encode_utf16() {
            hash = hash.wrapping_mul(31).wrapping_add(unit as i32);
        }
        hash
    }

    fn get_nested_component(&mut self) -> Component {
        if self.name_resolver.is_none() {
            self.name_resolver = Some(key_resolver()(&self.name));
        }
        self.name_resolver
            .as_ref()
            .unwrap_or_else(|| panic!("keybind resolver is cached before use"))
            .get()
    }
}

impl std::fmt::Display for KeybindContentsModel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "keybind{{{}}}", self.name)
    }
}

impl PartialEq for KeybindContentsModel {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for KeybindContentsModel {}

pub fn set_key_resolver(resolver: KeyResolver) {
    *resolver_slot()
        .lock()
        .unwrap_or_else(|_| panic!("keybind resolver lock should not be poisoned")) = resolver;
}

pub fn reset_key_resolver() {
    set_key_resolver(default_key_resolver);
}

fn key_resolver() -> KeyResolver {
    *resolver_slot()
        .lock()
        .unwrap_or_else(|_| panic!("keybind resolver lock should not be poisoned"))
}

fn resolver_slot() -> &'static Mutex<KeyResolver> {
    static KEY_RESOLVER: OnceLock<Mutex<KeyResolver>> = OnceLock::new();
    KEY_RESOLVER.get_or_init(|| Mutex::new(default_key_resolver))
}

fn default_key_resolver(name: &str) -> KeybindSupplierModel {
    KeybindSupplierModel::new(name, literal_component)
}

fn literal_component(name: &str) -> Component {
    Component::literal(name)
}

#[cfg(all(test, vibecraft_has_decompiled_sources))]
mod tests {
    use super::*;
    use crate::chat_component::TextColor;
    use std::sync::{MutexGuard, OnceLock};

    const KEYBIND_RESOLVER_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/KeybindResolver.java"
    );
    const KEYBIND_CONTENTS_JAVA: &str = include_str!(
        "../../decompiled-server-26.1.2/net/minecraft/network/chat/contents/KeybindContents.java"
    );

    fn bracket_supplier(name: &str) -> KeybindSupplierModel {
        KeybindSupplierModel::new(name, |name| Component::literal(format!("[{name}]")))
    }

    fn angle_supplier(name: &str) -> KeybindSupplierModel {
        KeybindSupplierModel::new(name, |name| Component::literal(format!("<{name}>")))
    }

    fn keybind_test_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|_| panic!("keybind test lock should not be poisoned"))
    }

    #[test]
    fn keybind_resolver_default_and_setter_match_java() {
        let _guard = keybind_test_lock();
        reset_key_resolver();
        for sentinel in [
            "static Function<String, Supplier<Component>> keyResolver = name -> () -> Component.literal(name);",
            "public static void setKeyResolver(final Function<String, Supplier<Component>> resolver)",
            "keyResolver = resolver;",
        ] {
            assert!(
                KEYBIND_RESOLVER_JAVA.contains(sentinel),
                "missing KeybindResolver Java sentinel: {sentinel}"
            );
        }

        let mut contents = KeybindContentsModel::new("key.jump");
        assert_eq!(
            contents.visit(&mut |text| Some(text.to_string())),
            Some("key.jump".to_string())
        );

        set_key_resolver(bracket_supplier);
        let mut remapped = KeybindContentsModel::new("key.jump");
        assert_eq!(
            remapped.visit(&mut |text| Some(text.to_string())),
            Some("[key.jump]".to_string())
        );
        reset_key_resolver();
    }

    #[test]
    fn keybind_contents_lazily_caches_supplier_and_delegates_visits() {
        let _guard = keybind_test_lock();
        reset_key_resolver();
        for sentinel in [
            "Codec.STRING.fieldOf(\"keybind\").forGetter(o -> o.name)",
            "private final String name;",
            "private @Nullable Supplier<Component> nameResolver;",
            "if (this.nameResolver == null)",
            "this.nameResolver = KeybindResolver.keyResolver.apply(this.name);",
            "return this.nameResolver.get();",
            "return this.getNestedComponent().visit(output);",
            "return this.getNestedComponent().visit(output, currentStyle);",
            "this == o ? true : o instanceof KeybindContents that && this.name.equals(that.name)",
            "return \"keybind{\" + this.name + \"}\";",
            "return this.name;",
            "return MAP_CODEC;",
        ] {
            assert!(
                KEYBIND_CONTENTS_JAVA.contains(sentinel),
                "missing KeybindContents Java sentinel: {sentinel}"
            );
        }

        set_key_resolver(bracket_supplier);
        let mut contents = KeybindContentsModel::new("key.use");
        assert_eq!(
            contents.visit(&mut |text| Some(text.to_string())),
            Some("[key.use]".to_string())
        );

        set_key_resolver(angle_supplier);
        assert_eq!(
            contents.visit(&mut |text| Some(text.to_string())),
            Some("[key.use]".to_string()),
            "existing contents keep the supplier resolved on first visit"
        );

        let mut next = KeybindContentsModel::new("key.use");
        let style = Style::empty().with_color(
            TextColor::parse("green").unwrap_or_else(|| panic!("missing green text color")),
        );
        assert_eq!(
            next.visit_styled(
                &mut |visited_style, text| Some((visited_style.clone(), text.to_string())),
                &style,
            ),
            Some((style, "<key.use>".to_string()))
        );
        reset_key_resolver();
    }

    #[test]
    fn keybind_contents_identity_hash_display_and_codec_are_name_based() {
        let _guard = keybind_test_lock();
        reset_key_resolver();
        let first = KeybindContentsModel::new("key.attack");
        let mut second = KeybindContentsModel::new("key.attack");
        let other = KeybindContentsModel::new("key.use");

        set_key_resolver(bracket_supplier);
        let _ = second.visit::<()>(&mut |_| None);
        assert_eq!(first, second);
        assert_ne!(first, other);
        assert_eq!(first.to_string(), "keybind{key.attack}");
        assert_eq!(first.get_name(), "key.attack");
        assert_eq!(first.codec_field(), "keybind");
        assert_eq!(first.java_hash_code(), java_string_hash("key.attack"));
        reset_key_resolver();
    }

    fn java_string_hash(value: &str) -> i32 {
        value.encode_utf16().fold(0i32, |hash, unit| {
            hash.wrapping_mul(31).wrapping_add(unit as i32)
        })
    }
}
