//! Data fixes (Java `com.mojang.datafixers.DataFix`).
//!
//! A Java `DataFix` builds a `TypeRewriteRule` in `makeRule()`. Here a [`Fix`]
//! carries a closure that plays the same role: given the fix's input schema it
//! reports whether the rule *matches* the root type being updated (the rule is a
//! no-op otherwise) and, when it does, rewrites the tag in place.

use crate::storage::nbt::Tag;

use super::references::TypeReference;
use super::schema::Schema;
use super::walk::{everywhere, reaches, Target};

/// The schemas a fix operates with (`DataFix.getInputSchema` / `getOutputSchema`).
pub struct FixContext<'a> {
    /// `changesType ? outputSchema.getParent() : outputSchema`.
    pub input: &'a Schema,
    /// The schema the fix was registered with.
    pub output: &'a Schema,
}

type ApplyFn = dyn Fn(&FixContext<'_>, TypeReference, &mut Tag) -> bool + Send + Sync;

/// A registered rewrite rule.
pub struct Fix {
    /// Human readable name (Java `DataFix.getName()` / the name given to `fixTypeEverywhere`).
    pub name: String,
    apply: Box<ApplyFn>,
}

impl Fix {
    /// A fix with a fully custom rule. `apply` returns whether the rule matched
    /// `root` (Java: the rewrite produced a non-nop view).
    pub fn custom(
        name: impl Into<String>,
        apply: impl Fn(&FixContext<'_>, TypeReference, &mut Tag) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            apply: Box::new(apply),
        }
    }

    /// `fixTypeEverywhere(name, type, f)`: `f` runs on every occurrence of
    /// `target`, then the walk continues into the rewritten node.
    pub fn everywhere(
        name: impl Into<String>,
        target: Target,
        f: impl Fn(&mut Tag) + Send + Sync + 'static,
    ) -> Self {
        Self::everywhere_with(name, target, move |_, tag| f(tag))
    }

    /// The general form of [`Fix::everywhere`]: the callback also receives the
    /// fix context (for fixes that consult their schemas).
    pub fn everywhere_with(
        name: impl Into<String>,
        target: Target,
        f: impl Fn(&FixContext<'_>, &mut Tag) + Send + Sync + 'static,
    ) -> Self {
        Self::custom(name, move |ctx, root, tag| {
            if !reaches(ctx.input, root, &target) {
                return false;
            }
            everywhere(ctx.input, root, &target, tag, &mut |node| f(ctx, node));
            true
        })
    }

    /// Runs the rule; returns whether it matched.
    pub fn apply(&self, ctx: &FixContext<'_>, root: TypeReference, tag: &mut Tag) -> bool {
        (self.apply)(ctx, root, tag)
    }
}

impl Fix {
    /// `TypeRewriteRule.seq(rules...)`: runs the rules in order and reports whether
    /// any of them matched.
    pub fn sequence(name: impl Into<String>, fixes: Vec<Fix>) -> Self {
        Self::custom(name, move |ctx, root, tag| {
            let mut matched = false;
            for fix in &fixes {
                matched |= fix.apply(ctx, root, tag);
            }
            matched
        })
    }
}
