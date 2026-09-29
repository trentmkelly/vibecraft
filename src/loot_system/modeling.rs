//! Whether a decoded loot table is entirely representable by the runtime model.
//!
//! The JSON codec decodes every vanilla `LootTable` document, but the runtime model
//! only *evaluates* a subset of the entry, condition, function and number provider
//! types faithfully. The remainder decodes into `Unmodeled` variants that keep the
//! validated document without evaluating it. A table containing any of them must not
//! replace the built-in drop logic, which is what [`LootTable::is_fully_modeled`]
//! tells callers; [`LootTable::unmodeled_parts`] names what is missing.

use super::*;

impl LootTable {
    /// True when no `Unmodeled` entry, condition, function or number provider occurs
    /// anywhere in the table, so evaluating it is faithful to Java.
    pub fn is_fully_modeled(&self) -> bool {
        self.unmodeled_parts().is_empty()
    }

    /// The distinct unmodeled parts of the table as `"<category> <type id>"`
    /// (`"condition minecraft:match_tool"`), sorted.
    pub fn unmodeled_parts(&self) -> Vec<String> {
        let mut parts = Vec::new();
        for pool in &self.pools {
            pool.collect_unmodeled(&mut parts);
        }
        collect_functions(&self.functions, &mut parts);
        parts.sort();
        parts.dedup();
        parts
    }
}

fn collect_conditions(conditions: &[LootCondition], out: &mut Vec<String>) {
    conditions.iter().for_each(|c| c.collect_unmodeled(out));
}

fn collect_functions(functions: &[LootFunction], out: &mut Vec<String>) {
    functions.iter().for_each(|f| f.collect_unmodeled(out));
}

impl LootPool {
    fn collect_unmodeled(&self, out: &mut Vec<String>) {
        self.entries.iter().for_each(|e| e.collect_unmodeled(out));
        collect_conditions(&self.conditions, out);
        collect_functions(&self.functions, out);
        self.rolls.collect_unmodeled(out);
        self.bonus_rolls.collect_unmodeled(out);
    }
}

impl LootEntry {
    fn collect_unmodeled(&self, out: &mut Vec<String>) {
        match self {
            Self::Item {
                conditions,
                functions,
                ..
            }
            | Self::Empty {
                conditions,
                functions,
                ..
            }
            | Self::Tag {
                conditions,
                functions,
                ..
            }
            | Self::WeightedNestedTable {
                conditions,
                functions,
                ..
            }
            | Self::Dynamic {
                conditions,
                functions,
                ..
            } => {
                collect_conditions(conditions, out);
                collect_functions(functions, out);
            }
            Self::Alternatives(children) | Self::Sequence(children) | Self::Group(children) => {
                children.iter().for_each(|c| c.collect_unmodeled(out));
            }
            Self::Conditional { conditions, entry } => {
                collect_conditions(conditions, out);
                entry.collect_unmodeled(out);
            }
            Self::NestedTable(_) => {}
            Self::Unmodeled { kind, .. } => out.push(format!("entry {kind}")),
        }
    }
}

impl LootCondition {
    fn collect_unmodeled(&self, out: &mut Vec<String>) {
        match self {
            Self::Inverted(term) => term.collect_unmodeled(out),
            Self::AllOf(terms) | Self::AnyOf(terms) => collect_conditions(terms, out),
            Self::ValueCheck { provider, .. } => provider.collect_unmodeled(out),
            Self::Unmodeled { kind, .. } => out.push(format!("condition {kind}")),
            _ => {}
        }
    }
}

impl NumberProvider {
    fn collect_unmodeled(&self, out: &mut Vec<String>) {
        match self {
            Self::UniformProvider { min, max } => {
                min.collect_unmodeled(out);
                max.collect_unmodeled(out);
            }
            Self::BinomialProvider { n, p } => {
                n.collect_unmodeled(out);
                p.collect_unmodeled(out);
            }
            Self::Sum(summands) => summands.iter().for_each(|s| s.collect_unmodeled(out)),
            Self::Unmodeled { kind, .. } => out.push(format!("number {kind}")),
            _ => {}
        }
    }
}

impl LootFunction {
    fn collect_unmodeled(&self, out: &mut Vec<String>) {
        match self {
            Self::SetCount(provider)
            | Self::AddCount(provider)
            | Self::EnchantedCountIncrease {
                count: provider, ..
            }
            | Self::SetDamage(provider)
            | Self::SetOminousBottleAmplifier(provider)
            | Self::AddLootingBonus {
                per_level: provider,
                ..
            }
            | Self::ApplyFortuneBonus {
                per_level: provider,
                ..
            }
            | Self::EnchantWithLevels {
                levels: provider, ..
            } => provider.collect_unmodeled(out),
            Self::ModifyContents(functions) | Self::Sequence(functions) => {
                collect_functions(functions, out);
            }
            Self::Filtered {
                condition,
                function,
            } => {
                condition.collect_unmodeled(out);
                function.collect_unmodeled(out);
            }
            Self::Unmodeled { kind, .. } => out.push(format!("function {kind}")),
            _ => {}
        }
    }
}
