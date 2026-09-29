//! The fixer itself: schema chain, fix list and the `update` pipeline
//! (Java `DataFixerBuilder` + `DataFixerUpper`).

use crate::storage::nbt::Tag;

use super::decode::{decode, restore_artifacts};
use super::encode::apply_post_write_hooks;
use super::fix::{Fix, FixContext};
use super::references::TypeReference;
use super::schema::{Schema, VersionKey};
use super::template::Tmpl;

/// Handle returned by [`DataFixerBuilder::add_schema`] and consumed by
/// [`DataFixerBuilder::add_fixer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaId(usize);

struct RegisteredFix {
    key: VersionKey,
    input_schema: usize,
    output_schema: usize,
    fix: Fix,
}

/// Why an update could not be performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataFixError {
    /// The requested target version is newer than the last version whose Java
    /// fixes have all been ported, so the chain would be incomplete.
    IncompleteChain {
        /// Highest data version for which the chain is complete.
        complete_through: i32,
        /// The version that was requested.
        requested: i32,
    },
}

impl std::fmt::Display for DataFixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataFixError::IncompleteChain {
                complete_through,
                requested,
            } => write!(
                f,
                "DataFixer chain is only ported through DataVersion {complete_through}; cannot upgrade to {requested}"
            ),
        }
    }
}

/// Mirrors `DataFixerBuilder`: schemas and fixes are added in ascending order.
#[derive(Default)]
pub struct DataFixerBuilder {
    schemas: Vec<Schema>,
    fixes: Vec<RegisteredFix>,
}

impl DataFixerBuilder {
    /// An empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// `addSchema(version, subVersion, factory)`: the new schema derives from the
    /// previously added one; `register` applies the version's overrides.
    pub fn add_schema(
        &mut self,
        version: i32,
        sub_version: i32,
        register: impl FnOnce(&mut Schema),
    ) -> SchemaId {
        let key = VersionKey::with_sub(version, sub_version);
        let mut schema = match self.schemas.last() {
            Some(parent) => Schema::derive(parent, key),
            None => Schema::root(key),
        };
        register(&mut schema);
        self.schemas.push(schema);
        SchemaId(self.schemas.len() - 1)
    }

    /// `addSchema(version, SAME)`: a schema identical to its parent.
    pub fn add_same_schema(&mut self, version: i32, sub_version: i32) -> SchemaId {
        self.add_schema(version, sub_version, |_| {})
    }

    /// `addFixer(fix)`. `changes_type` is the `DataFix` constructor flag: when
    /// set, the fix reads the previous schema's types instead of its own.
    pub fn add_fixer(&mut self, schema: SchemaId, changes_type: bool, fix: Fix) {
        // A type-changing fix always has a parent schema in the chain.
        let input_schema = if changes_type {
            schema.0.saturating_sub(1)
        } else {
            schema.0
        };
        self.fixes.push(RegisteredFix {
            key: self.schemas[schema.0].key,
            input_schema,
            output_schema: schema.0,
            fix,
        });
    }

    /// Finishes the chain. `complete_through` is the highest data version for
    /// which every Java fix has been registered.
    pub fn build(self, complete_through: i32) -> DataFixer {
        DataFixer {
            schemas: self.schemas,
            fixes: self.fixes,
            complete_through,
        }
    }
}

/// The assembled fixer (Java `DataFixerUpper`).
pub struct DataFixer {
    schemas: Vec<Schema>,
    fixes: Vec<RegisteredFix>,
    complete_through: i32,
}

impl DataFixer {
    /// Highest data version for which the chain is complete.
    pub fn complete_through(&self) -> i32 {
        self.complete_through
    }

    /// `DataFixerUpper.getSchema(key)`: the last schema whose key is not above
    /// `version`, or the first schema when `version` predates the chain.
    pub fn schema_at(&self, version: i32) -> &Schema {
        let key = VersionKey::new(version);
        self.schemas
            .iter()
            .rev()
            .find(|schema| schema.key <= key)
            .unwrap_or(&self.schemas[0])
    }

    /// The registered fix names in order, with their version keys (diagnostics/tests).
    pub fn fix_names(&self) -> Vec<(VersionKey, &str)> {
        self.fixes
            .iter()
            .map(|entry| (entry.key, entry.fix.name.as_str()))
            .collect()
    }

    /// `DataFixerUpper.update(type, input, version, newVersion)`.
    ///
    /// Reads `input` with the schema of `from`, runs every fix registered in
    /// `(from, to]`, and returns the result. As in Java, a read failure (or no
    /// fix matching the type) yields the input unchanged.
    pub fn update(
        &self,
        root: TypeReference,
        input: &Tag,
        from: i32,
        to: i32,
    ) -> Result<Tag, DataFixError> {
        if from >= to {
            return Ok(input.clone());
        }
        if to > self.complete_through {
            return Err(DataFixError::IncompleteChain {
                complete_through: self.complete_through,
                requested: to,
            });
        }
        let first_key = VersionKey::new(from);
        let mut work = input.clone();
        let read_schema = self.schema_at(from);
        if read_schema.type_of(root).is_none()
            || decode(read_schema, &Tmpl::Ref(root), &mut work).is_err()
        {
            return Ok(input.clone());
        }
        let mut matched = false;
        for entry in &self.fixes {
            if entry.key <= first_key || entry.key.version > to {
                continue;
            }
            let ctx = FixContext {
                input: &self.schemas[entry.input_schema],
                output: &self.schemas[entry.output_schema],
            };
            matched |= entry.fix.apply(&ctx, root, &mut work);
        }
        if !matched {
            return Ok(input.clone());
        }
        // The value is written with the target version's type.
        if apply_post_write_hooks(self.schema_at(to), &Tmpl::Ref(root), &mut work).is_err() {
            return Ok(input.clone());
        }
        restore_artifacts(&mut work);
        Ok(work)
    }
}
