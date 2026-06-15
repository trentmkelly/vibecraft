#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancementVisibilityDisplay {
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementVisibilityNode {
    id: Identifier,
    parent: Option<Identifier>,
    children: Vec<Identifier>,
    display: Option<AdvancementVisibilityDisplay>,
}

impl AdvancementVisibilityNode {
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn parent(&self) -> Option<&Identifier> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> &[Identifier] {
        &self.children
    }

    pub fn display(&self) -> Option<AdvancementVisibilityDisplay> {
        self.display
    }

    fn root<'a>(&'a self, tree: &'a AdvancementVisibilityTree) -> &'a Self {
        let mut root = self;
        while let Some(parent) = root.parent() {
            root = tree
                .node(parent)
                .unwrap_or_else(|| panic!("missing advancement parent {parent}"));
        }
        root
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementVisibilityTree {
    nodes: BTreeMap<Identifier, AdvancementVisibilityNode>,
}

impl AdvancementVisibilityTree {
    pub fn insert(
        &mut self,
        id: Identifier,
        parent: Option<Identifier>,
        display: Option<AdvancementVisibilityDisplay>,
    ) {
        if let Some(parent_id) = &parent {
            let parent_node = self
                .nodes
                .get_mut(parent_id)
                .unwrap_or_else(|| panic!("parent {parent_id} must be inserted before child {id}"));
            parent_node.children.push(id.clone());
        }
        self.nodes.insert(
            id.clone(),
            AdvancementVisibilityNode {
                id,
                parent,
                children: Vec::new(),
                display,
            },
        );
    }

    pub fn node(&self, id: &Identifier) -> Option<&AdvancementVisibilityNode> {
        self.nodes.get(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisibilityRule {
    Show,
    Hide,
    NoChange,
}

pub fn evaluate_advancement_visibility(
    tree: &AdvancementVisibilityTree,
    start: &Identifier,
    done: &BTreeSet<Identifier>,
) -> Vec<(Identifier, bool)> {
    let node = tree
        .node(start)
        .unwrap_or_else(|| panic!("missing advancement node {start}"));
    let root = node.root(tree);
    let mut ascendants = vec![
        VisibilityRule::NoChange,
        VisibilityRule::NoChange,
        VisibilityRule::NoChange,
    ];
    let mut output = Vec::new();
    evaluate_visibility(root, tree, &mut ascendants, done, &mut output);
    output
}

fn evaluate_visibility(
    node: &AdvancementVisibilityNode,
    tree: &AdvancementVisibilityTree,
    ascendants: &mut Vec<VisibilityRule>,
    done: &BTreeSet<Identifier>,
    output: &mut Vec<(Identifier, bool)>,
) -> bool {
    let is_self_done = done.contains(node.id());
    let descendant_visibility = evaluate_visibility_rule(node.display(), is_self_done);
    let mut is_self_or_descendant_done = is_self_done;
    ascendants.push(descendant_visibility);

    for child_id in node.children() {
        let child = tree
            .node(child_id)
            .unwrap_or_else(|| panic!("missing advancement child {child_id}"));
        is_self_or_descendant_done |= evaluate_visibility(child, tree, ascendants, done, output);
    }

    let visible = is_self_or_descendant_done || evaluate_visibility_for_unfinished_node(ascendants);
    ascendants.pop();
    output.push((node.id().clone(), visible));
    is_self_or_descendant_done
}

fn evaluate_visibility_rule(
    display: Option<AdvancementVisibilityDisplay>,
    is_done: bool,
) -> VisibilityRule {
    match display {
        None => VisibilityRule::Hide,
        Some(_) if is_done => VisibilityRule::Show,
        Some(display) if display.hidden => VisibilityRule::Hide,
        Some(_) => VisibilityRule::NoChange,
    }
}

fn evaluate_visibility_for_unfinished_node(ascendants: &[VisibilityRule]) -> bool {
    for depth in 0..=2 {
        let Some(visibility) = ascendants.get(ascendants.len().saturating_sub(1 + depth)) else {
            return false;
        };
        match visibility {
            VisibilityRule::Show => return true,
            VisibilityRule::Hide => return false,
            VisibilityRule::NoChange => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/advancements/AdvancementVisibilityEvaluator.java");

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn visible() -> Option<AdvancementVisibilityDisplay> {
        Some(AdvancementVisibilityDisplay { hidden: false })
    }

    fn hidden() -> Option<AdvancementVisibilityDisplay> {
        Some(AdvancementVisibilityDisplay { hidden: true })
    }

    fn tree_with_chain(displays: &[(&str, Option<AdvancementVisibilityDisplay>)]) -> AdvancementVisibilityTree {
        let mut tree = AdvancementVisibilityTree::default();
        let mut parent = None;
        for (node_id, display) in displays {
            let node_id = id(node_id);
            tree.insert(node_id.clone(), parent.clone(), *display);
            parent = Some(node_id);
        }
        tree
    }

    fn output_ids(output: Vec<(Identifier, bool)>) -> Vec<(String, bool)> {
        output
            .into_iter()
            .map(|(id, visible)| (id.to_string(), visible))
            .collect()
    }

    #[test]
    fn visibility_rule_matches_java_display_and_done_cases() {
        assert_eq!(evaluate_visibility_rule(None, false), VisibilityRule::Hide);
        assert_eq!(evaluate_visibility_rule(None, true), VisibilityRule::Hide);
        assert_eq!(evaluate_visibility_rule(hidden(), false), VisibilityRule::Hide);
        assert_eq!(evaluate_visibility_rule(hidden(), true), VisibilityRule::Show);
        assert_eq!(evaluate_visibility_rule(visible(), false), VisibilityRule::NoChange);
        assert_eq!(evaluate_visibility_rule(visible(), true), VisibilityRule::Show);
    }

    #[test]
    fn unfinished_node_scans_current_parent_and_grandparent_rules() {
        assert!(evaluate_visibility_for_unfinished_node(&[
            VisibilityRule::NoChange,
            VisibilityRule::Show,
            VisibilityRule::NoChange,
            VisibilityRule::NoChange,
        ]));
        assert!(!evaluate_visibility_for_unfinished_node(&[
            VisibilityRule::Show,
            VisibilityRule::NoChange,
            VisibilityRule::NoChange,
            VisibilityRule::NoChange,
        ]));
        assert!(!evaluate_visibility_for_unfinished_node(&[
            VisibilityRule::Show,
            VisibilityRule::Hide,
            VisibilityRule::NoChange,
        ]));
    }

    #[test]
    fn visibility_is_evaluated_from_root_and_output_is_post_order() {
        let tree = tree_with_chain(&[
            ("minecraft:root", visible()),
            ("minecraft:child", visible()),
            ("minecraft:grandchild", visible()),
        ]);
        let output = evaluate_advancement_visibility(&tree, &id("minecraft:grandchild"), &BTreeSet::new());

        assert_eq!(
            output_ids(output),
            vec![
                ("minecraft:grandchild".to_string(), false),
                ("minecraft:child".to_string(), false),
                ("minecraft:root".to_string(), false),
            ]
        );
    }

    #[test]
    fn done_descendant_makes_ancestors_visible_and_done_node_shows_descendants() {
        let tree = tree_with_chain(&[
            ("minecraft:root", visible()),
            ("minecraft:child", visible()),
            ("minecraft:grandchild", visible()),
            ("minecraft:great_grandchild", visible()),
        ]);
        let done = BTreeSet::from([id("minecraft:grandchild")]);
        let output = evaluate_advancement_visibility(&tree, &id("minecraft:child"), &done);

        assert_eq!(
            output_ids(output),
            vec![
                ("minecraft:great_grandchild".to_string(), true),
                ("minecraft:grandchild".to_string(), true),
                ("minecraft:child".to_string(), true),
                ("minecraft:root".to_string(), true),
            ]
        );
    }

    #[test]
    fn hidden_or_displayless_ancestor_stops_unfinished_visibility() {
        let hidden_tree = tree_with_chain(&[
            ("minecraft:root", visible()),
            ("minecraft:hidden", hidden()),
            ("minecraft:child", visible()),
        ]);
        let displayless_tree = tree_with_chain(&[
            ("minecraft:root", visible()),
            ("minecraft:displayless", None),
            ("minecraft:child", visible()),
        ]);

        assert_eq!(
            output_ids(evaluate_advancement_visibility(
                &hidden_tree,
                &id("minecraft:root"),
                &BTreeSet::from([id("minecraft:root")]),
            )),
            vec![
                ("minecraft:child".to_string(), false),
                ("minecraft:hidden".to_string(), false),
                ("minecraft:root".to_string(), true),
            ]
        );
        assert_eq!(
            output_ids(evaluate_advancement_visibility(
                &displayless_tree,
                &id("minecraft:root"),
                &BTreeSet::from([id("minecraft:root")]),
            )),
            vec![
                ("minecraft:child".to_string(), false),
                ("minecraft:displayless".to_string(), false),
                ("minecraft:root".to_string(), true),
            ]
        );
    }

    #[test]
    fn completed_hidden_advancement_is_visible_despite_hidden_display() {
        let tree = tree_with_chain(&[("minecraft:root", hidden())]);
        let output = evaluate_advancement_visibility(
            &tree,
            &id("minecraft:root"),
            &BTreeSet::from([id("minecraft:root")]),
        );

        assert_eq!(output_ids(output), vec![("minecraft:root".to_string(), true)]);
    }

    #[test]
    #[cfg(vibecraft_has_decompiled_sources)]
    fn advancement_visibility_evaluator_source_matches_java_26_1_2() {
        for sentinel in [
            "private static final int VISIBILITY_DEPTH = 2;",
            "private static AdvancementVisibilityEvaluator.VisibilityRule evaluateVisibilityRule",
            "display.isEmpty()",
            "return AdvancementVisibilityEvaluator.VisibilityRule.HIDE;",
            "display.get().isHidden() ? AdvancementVisibilityEvaluator.VisibilityRule.HIDE",
            "private static boolean evaluateVisiblityForUnfinishedNode",
            "for (int i = 0; i <= 2; i++)",
            "ascendants.peek(i)",
            "private static boolean evaluateVisibility(",
            "boolean isSelfDone = isDoneTest.test(node);",
            "ascendants.push(descendantVisibility);",
            "for (AdvancementNode child : node.children())",
            "boolean visiblity = isSelfOrDescendantDone || evaluateVisiblityForUnfinishedNode(ascendants);",
            "output.accept(node, visiblity);",
            "AdvancementNode root = node.root();",
            "Stack<AdvancementVisibilityEvaluator.VisibilityRule> visibilityStack = new ObjectArrayList();",
            "void accept(AdvancementNode advancement, boolean visible);",
            "SHOW,\n      HIDE,\n      NO_CHANGE;",
        ] {
            assert!(
                JAVA_SOURCE.contains(sentinel),
                "AdvancementVisibilityEvaluator.java is missing sentinel: {sentinel}"
            );
        }
    }
}
