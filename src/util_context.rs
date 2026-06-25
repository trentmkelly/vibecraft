#![allow(dead_code)]

use crate::registry::Identifier;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

pub type ContextKeyRef<T> = Rc<ContextKey<T>>;

#[derive(Debug)]
pub struct ContextKey<T> {
    name: Identifier,
    _marker: PhantomData<fn() -> T>,
}

impl<T> ContextKey<T> {
    pub fn new(name: Identifier) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }

    pub fn vanilla(name: &str) -> Result<Self, String> {
        Ok(Self::new(Identifier::with_default_namespace(name)?))
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}

impl<T> fmt::Display for ContextKey<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "<parameter {}>", self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextKeySet {
    required: Vec<KeyIdentity>,
    allowed: Vec<KeyIdentity>,
}

impl ContextKeySet {
    pub fn builder() -> ContextKeySetBuilder {
        ContextKeySetBuilder::default()
    }

    pub fn required(&self) -> &[KeyIdentity] {
        &self.required
    }

    pub fn allowed(&self) -> &[KeyIdentity] {
        &self.allowed
    }

    fn requires(&self, key_id: usize) -> bool {
        self.required.iter().any(|key| key.id == key_id)
    }

    fn allows(&self, key_id: usize) -> bool {
        self.allowed.iter().any(|key| key.id == key_id)
    }
}

impl fmt::Display for ContextKeySet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rendered = self
            .allowed
            .iter()
            .map(|key| {
                if self.requires(key.id) {
                    format!("!{}", key.name)
                } else {
                    key.name.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        write!(formatter, "[{rendered}]")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyIdentity {
    id: usize,
    name: Identifier,
}

impl fmt::Display for KeyIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "<parameter {}>", self.name)
    }
}

#[derive(Debug, Default)]
pub struct ContextKeySetBuilder {
    required: Vec<KeyIdentity>,
    optional: Vec<KeyIdentity>,
}

impl ContextKeySetBuilder {
    pub fn required<T>(mut self, param: &ContextKeyRef<T>) -> Result<Self, String> {
        let key = KeyIdentity::from_ref(param);
        if contains_key_id(&self.optional, key.id) {
            return Err(format!("Parameter {} is already optional", key.name));
        }
        push_unique(&mut self.required, key);
        Ok(self)
    }

    pub fn optional<T>(mut self, param: &ContextKeyRef<T>) -> Result<Self, String> {
        let key = KeyIdentity::from_ref(param);
        if contains_key_id(&self.required, key.id) {
            return Err(format!("Parameter {} is already required", key.name));
        }
        push_unique(&mut self.optional, key);
        Ok(self)
    }

    pub fn build(self) -> ContextKeySet {
        let mut allowed = self.required.clone();
        for key in &self.optional {
            push_unique(&mut allowed, key.clone());
        }
        ContextKeySet {
            required: self.required,
            allowed,
        }
    }
}

impl KeyIdentity {
    fn from_ref<T>(key: &ContextKeyRef<T>) -> Self {
        Self {
            id: key_id(key),
            name: key.name().clone(),
        }
    }
}

fn key_id<T>(key: &ContextKeyRef<T>) -> usize {
    Rc::as_ptr(key) as *const () as usize
}

fn contains_key_id(keys: &[KeyIdentity], id: usize) -> bool {
    keys.iter().any(|key| key.id == id)
}

fn push_unique(keys: &mut Vec<KeyIdentity>, key: KeyIdentity) {
    if !contains_key_id(keys, key.id) {
        keys.push(key);
    }
}

#[derive(Clone)]
pub struct ContextMap {
    params: Rc<RefCell<HashMap<usize, ContextValue>>>,
}

impl ContextMap {
    pub fn builder() -> ContextMapBuilder {
        ContextMapBuilder::default()
    }

    pub fn has<T>(&self, key: &ContextKeyRef<T>) -> bool {
        self.params.borrow().contains_key(&key_id(key))
    }

    pub fn get_or_throw<T: Clone + 'static>(&self, key: &ContextKeyRef<T>) -> Result<T, String> {
        self.get_optional(key)
            .ok_or_else(|| key.name().to_string())
    }

    pub fn get_optional<T: Clone + 'static>(&self, key: &ContextKeyRef<T>) -> Option<T> {
        self.params
            .borrow()
            .get(&key_id(key))
            .and_then(|entry| entry.value.downcast_ref::<T>().cloned())
    }

    pub fn get_or_default<T: Clone + 'static>(
        &self,
        key: &ContextKeyRef<T>,
        default: Option<T>,
    ) -> Option<T> {
        self.get_optional(key).or(default)
    }
}

#[derive(Default, Clone)]
pub struct ContextMapBuilder {
    params: Rc<RefCell<HashMap<usize, ContextValue>>>,
}

impl ContextMapBuilder {
    pub fn with_parameter<T: 'static>(&self, param: &ContextKeyRef<T>, value: T) -> &Self {
        self.params.borrow_mut().insert(
            key_id(param),
            ContextValue {
                name: param.name().clone(),
                value: Box::new(value),
            },
        );
        self
    }

    pub fn with_optional_parameter<T: 'static>(
        &self,
        param: &ContextKeyRef<T>,
        value: Option<T>,
    ) -> &Self {
        if let Some(value) = value {
            self.with_parameter(param, value);
        } else {
            self.params.borrow_mut().remove(&key_id(param));
        }
        self
    }

    pub fn get_parameter<T: Clone + 'static>(&self, param: &ContextKeyRef<T>) -> Result<T, String> {
        self.get_optional_parameter(param)
            .ok_or_else(|| param.name().to_string())
    }

    pub fn get_optional_parameter<T: Clone + 'static>(
        &self,
        param: &ContextKeyRef<T>,
    ) -> Option<T> {
        self.params
            .borrow()
            .get(&key_id(param))
            .and_then(|entry| entry.value.downcast_ref::<T>().cloned())
    }

    pub fn create(&self, param_set: &ContextKeySet) -> Result<ContextMap, String> {
        let params = self.params.borrow();
        let not_allowed = params
            .iter()
            .filter(|(id, _)| !param_set.allows(**id))
            .map(|(_, value)| value.name.clone())
            .collect::<Vec<_>>();
        if !not_allowed.is_empty() {
            return Err(format!(
                "Parameters not allowed in this parameter set: {}",
                format_parameter_names(&not_allowed)
            ));
        }

        let missing_required = param_set
            .required()
            .iter()
            .filter(|key| !params.contains_key(&key.id))
            .map(|key| key.name.clone())
            .collect::<Vec<_>>();
        if !missing_required.is_empty() {
            return Err(format!(
                "Missing required parameters: {}",
                format_parameter_names(&missing_required)
            ));
        }

        Ok(ContextMap {
            params: Rc::clone(&self.params),
        })
    }
}

struct ContextValue {
    name: Identifier,
    value: Box<dyn Any>,
}

impl Clone for ContextValue {
    fn clone(&self) -> Self {
        panic!("ContextValue mirrors Java Object storage and is not cloneable")
    }
}

fn format_parameter_names(names: &[Identifier]) -> String {
    let rendered = names
        .iter()
        .map(|name| format!("<parameter {name}>"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{rendered}]")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn must_ok<T, E: fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(err) => panic!("expected Ok(..), got Err({err:?})"),
        }
    }

    fn must_err<T>(result: Result<T, String>) -> String {
        match result {
            Ok(_) => panic!("expected Err(..), got Ok(..)"),
            Err(err) => err,
        }
    }

    fn key<T>(path: &str) -> ContextKeyRef<T> {
        Rc::new(must_ok(ContextKey::vanilla(path)))
    }

    #[test]
    fn context_key_matches_java_name_and_display_contract() {
        let source = vibecraft_java_source!("/net/minecraft/util/context/ContextKey.java");
        assert!(source.contains("Identifier.withDefaultNamespace(name)"));
        assert!(source.contains("return \"<parameter \" + this.name + \">\""));

        let key = key::<i32>("entity");
        assert_eq!(key.name().to_string(), "minecraft:entity");
        assert_eq!(key.to_string(), "<parameter minecraft:entity>");
    }

    #[test]
    fn context_key_set_uses_identity_and_rejects_required_optional_conflicts() {
        let source = vibecraft_java_source!("/net/minecraft/util/context/ContextKeySet.java");
        assert!(source.contains("Sets.newIdentityHashSet()"));
        assert!(source.contains("Parameter \" + param.name() + \" is already optional"));
        assert!(source.contains("Parameter \" + param.name() + \" is already required"));
        assert!(source.contains("(this.required.contains(k) ? \"!\" : \"\") + k.name()"));

        let required = key::<i32>("entity");
        let optional = key::<String>("origin");
        let same_name_distinct = key::<i32>("entity");

        let set = must_ok(ContextKeySet::builder().required(&required))
            .optional(&optional);
        let set = must_ok(set).build();
        assert_eq!(set.required().len(), 1);
        assert_eq!(set.allowed().len(), 2);
        assert!(set.allows(key_id(&required)));
        assert!(!set.allows(key_id(&same_name_distinct)));
        assert_eq!(set.to_string(), "[!minecraft:entity, minecraft:origin]");

        assert_eq!(
            must_err(ContextKeySet::builder().optional(&required).and_then(|builder| builder.required(&required))),
            "Parameter minecraft:entity is already optional"
        );
        assert_eq!(
            must_err(ContextKeySet::builder().required(&required).and_then(|builder| builder.optional(&required))),
            "Parameter minecraft:entity is already required"
        );
    }

    #[test]
    fn context_map_builder_validates_allowed_and_required_identity_keys() {
        let source = vibecraft_java_source!("/net/minecraft/util/context/ContextMap.java");
        assert!(source.contains("new IdentityHashMap<>()"));
        assert!(source.contains("Parameters not allowed in this parameter set: "));
        assert!(source.contains("Missing required parameters: "));
        assert!(source.contains("throw new NoSuchElementException(param.name().toString())"));

        let required = key::<i32>("entity");
        let optional = key::<String>("origin");
        let forbidden = key::<bool>("damage_source");
        let set = must_ok(must_ok(ContextKeySet::builder().required(&required)).optional(&optional)).build();

        let builder = ContextMap::builder();
        builder.with_parameter(&forbidden, true);
        assert_eq!(
            must_err(builder.create(&set)),
            "Parameters not allowed in this parameter set: [<parameter minecraft:damage_source>]"
        );

        builder.with_optional_parameter(&forbidden, None);
        assert_eq!(
            must_err(builder.create(&set)),
            "Missing required parameters: [<parameter minecraft:entity>]"
        );

        builder.with_parameter(&required, 42);
        builder.with_optional_parameter(&optional, Some("spawn".to_string()));
        let map = must_ok(builder.create(&set));
        assert!(map.has(&required));
        assert_eq!(map.get_or_throw(&required), Ok(42));
        assert_eq!(map.get_optional(&optional), Some("spawn".to_string()));
        assert_eq!(map.get_or_default(&key::<i32>("missing"), Some(7)), Some(7));
    }

    #[test]
    fn context_map_preserves_java_builder_backing_map_aliasing() {
        let source = vibecraft_java_source!("/net/minecraft/util/context/ContextMap.java");
        assert!(source.contains("return new ContextMap(this.params)"));

        let required = key::<i32>("entity");
        let set = must_ok(ContextKeySet::builder().required(&required)).build();
        let builder = ContextMap::builder();
        builder.with_parameter(&required, 1);
        let map = must_ok(builder.create(&set));
        assert_eq!(map.get_or_throw(&required), Ok(1));

        builder.with_parameter(&required, 2);
        assert_eq!(map.get_or_throw(&required), Ok(2));
        builder.with_optional_parameter(&required, None);
        assert_eq!(map.get_or_throw::<i32>(&required), Err("minecraft:entity".to_string()));
    }
}
