//! Unweighted entry expansion for `SetContainerContents`, preserving Java's
//! immediate output visitor and short-circuit composite semantics.

use super::runtime_engine::apply_entry_functions;
use super::*;

impl LootEntry {
    /// `SetContainerContents.run`: every successfully expanded entry creates
    /// stacks immediately through `LootTable.createStackSplitter`. Entry weights
    /// do not gate this path, and a failed sequence keeps earlier emitted stacks.
    pub(super) fn create_contents(&self, context: &mut LootContext) -> Vec<LootStack> {
        let mut contents = Vec::new();
        self.expand_contents(context, &mut contents);
        contents
    }

    fn expand_contents(&self, context: &mut LootContext, contents: &mut Vec<LootStack>) -> bool {
        match self {
            Self::Conditional { conditions, entry } => {
                conditions_hold(conditions, context) && entry.expand_contents(context, contents)
            }
            Self::Alternatives(children) => children
                .iter()
                .any(|entry| entry.expand_contents(context, contents)),
            Self::Sequence(children) => children
                .iter()
                .all(|entry| entry.expand_contents(context, contents)),
            Self::Group(children) => {
                for entry in children {
                    entry.expand_contents(context, contents);
                }
                true
            }
            Self::Tag {
                tag,
                expand,
                conditions,
                functions,
                ..
            } => {
                if !conditions_hold(conditions, context) {
                    return false;
                }
                // Both expanded and nonexpanded TagEntry yield every member in
                // this path; an empty tag still reports successful expansion.
                let stacks = context
                    .tags
                    .get(tag)
                    .into_iter()
                    .flatten()
                    .map(|item| LootStack::new(item, 1))
                    .collect();
                // `TagEntry.expandTag` emits anonymous EntryBase values which
                // bypass the singleton's decorated function chain.
                let stacks = if *expand {
                    stacks
                } else {
                    apply_entry_functions(functions, stacks, context)
                };
                extend_contents(contents, stacks);
                true
            }
            Self::Unmodeled { .. } => false,
            _ => {
                let mut expanded = Vec::new();
                self.expand(context, &mut expanded);
                let success = !expanded.is_empty();
                for entry in expanded {
                    extend_contents(contents, entry.create(context));
                }
                success
            }
        }
    }
}

/// The container splitter retains empty outputs as slots. Java's splitter accepts
/// a count below the maximum directly; `ItemContainerContents.fromItems` keeps
/// those gaps until the final nonempty slot.
fn extend_contents(contents: &mut Vec<LootStack>, stacks: Vec<LootStack>) {
    for stack in stacks {
        if stack.count < stack.max_stack_size() {
            contents.push(stack);
        } else {
            contents.extend(split_stacks(vec![stack]));
        }
    }
}

/// True when every condition matches (`LootItemConditions.andConditions`).
pub(super) fn conditions_hold(conditions: &[LootCondition], context: &LootContext) -> bool {
    conditions
        .iter()
        .all(|condition| condition.matches(context))
}
