use std::collections::BTreeMap;

use crate::advancement_system::AdvancementDefinition;
use crate::advancement_tree::{AdvancementTreeHolderModel, AdvancementTreeModel};
use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementHolderModel {
    id: Identifier,
    value: AdvancementDefinition,
}

impl AdvancementHolderModel {
    pub fn new(id: Identifier, value: AdvancementDefinition) -> Self {
        Self { id, value }
    }

    pub fn value(&self) -> &AdvancementDefinition {
        &self.value
    }

    fn tree_holder(&self) -> AdvancementTreeHolderModel {
        AdvancementTreeHolderModel::new(self.id.clone(), self.value.parent.clone())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ServerAdvancementManagerModel {
    advancements: BTreeMap<Identifier, AdvancementHolderModel>,
    tree: AdvancementTreeModel,
    validation_warnings: Vec<String>,
    positioned_visible_roots: Vec<Identifier>,
}

impl ServerAdvancementManagerModel {
    pub fn apply(&mut self, preparations: BTreeMap<Identifier, AdvancementDefinition>) {
        let mut advancements = BTreeMap::new();
        self.validation_warnings.clear();
        self.positioned_visible_roots.clear();

        for (id, advancement) in preparations {
            self.validate(&id, &advancement);
            advancements.insert(id.clone(), AdvancementHolderModel::new(id, advancement));
        }

        let mut tree = AdvancementTreeModel::default();
        tree.add_all(
            advancements
                .values()
                .map(AdvancementHolderModel::tree_holder),
        );

        for root in tree.roots() {
            let root_id = root.id();
            if advancements
                .get(root_id)
                .and_then(|holder| holder.value().display.as_ref())
                .is_some()
            {
                self.positioned_visible_roots.push(root_id.clone());
            }
        }

        self.advancements = advancements;
        self.tree = tree;
    }

    fn validate(&mut self, id: &Identifier, advancement: &AdvancementDefinition) {
        if advancement.criteria.is_empty() {
            self.validation_warnings
                .push(format!("Found validation problems in advancement {id}: criteria cannot be empty"));
        }
        for requirement in &advancement.requirements {
            for criterion in requirement {
                if !advancement.criteria.contains(criterion) {
                    self.validation_warnings.push(format!(
                        "Found validation problems in advancement {id}: unknown criterion {criterion}"
                    ));
                }
            }
        }
    }

    pub fn get(&self, id: &Identifier) -> Option<&AdvancementHolderModel> {
        self.advancements.get(id)
    }

    pub fn tree(&self) -> &AdvancementTreeModel {
        &self.tree
    }

    pub fn get_all_advancements(&self) -> impl Iterator<Item = &AdvancementHolderModel> {
        self.advancements.values()
    }

    pub fn validation_warnings(&self) -> &[String] {
        &self.validation_warnings
    }

    pub fn positioned_visible_roots(&self) -> &[Identifier] {
        &self.positioned_visible_roots
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advancement_system::{AdvancementDisplay, AdvancementFrame, AdvancementRewards};

    #[cfg(vibecraft_has_decompiled_sources)]
    const JAVA_SOURCE: &str =
        vibecraft_java_source!("/net/minecraft/server/ServerAdvancementManager.java");

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap_or_else(|err| panic!("{err}"))
    }

    fn display() -> AdvancementDisplay {
        AdvancementDisplay {
            icon: id("minecraft:stone"),
            title: "Title".to_string(),
            description: "Description".to_string(),
            background: None,
            frame: AdvancementFrame::Task,
            show_toast: true,
            announce_chat: true,
            hidden: false,
            x: 0.0,
            y: 0.0,
        }
    }

    fn advancement(
        id: &str,
        parent: Option<&str>,
        display: Option<AdvancementDisplay>,
    ) -> AdvancementDefinition {
        AdvancementDefinition::all_of(
            id,
            parent,
            &["tick"],
            AdvancementRewards::default(),
            display,
        )
        .unwrap_or_else(|err| panic!("{err}"))
    }

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn server_advancement_manager_matches_java_source_shape() {
        assert!(JAVA_SOURCE.contains(
            "public class ServerAdvancementManager extends SimpleJsonResourceReloadListener<Advancement>"
        ));
        assert!(JAVA_SOURCE.contains("private Map<Identifier, AdvancementHolder> advancements = Map.of();"));
        assert!(JAVA_SOURCE.contains("private AdvancementTree tree = new AdvancementTree();"));
        assert!(JAVA_SOURCE.contains("super(registries, Advancement.CODEC, Registries.ADVANCEMENT);"));
        assert!(JAVA_SOURCE.contains("protected void apply(final Map<Identifier, Advancement> preparations"));
        assert!(JAVA_SOURCE.contains("this.validate(id, advancement);"));
        assert!(JAVA_SOURCE.contains("builder.put(id, new AdvancementHolder(id, advancement));"));
        assert!(JAVA_SOURCE.contains("this.advancements = builder.buildOrThrow();"));
        assert!(JAVA_SOURCE.contains("tree.addAll(this.advancements.values());"));
        assert!(JAVA_SOURCE.contains("TreeNodePosition.run(root);"));
        assert!(JAVA_SOURCE.contains("public @Nullable AdvancementHolder get(final Identifier id)"));
        assert!(JAVA_SOURCE.contains("public AdvancementTree tree()"));
        assert!(JAVA_SOURCE.contains("public Collection<AdvancementHolder> getAllAdvancements()"));
    }

    #[test]
    fn server_advancement_manager_apply_replaces_holder_map_and_rebuilds_tree() {
        let mut manager = ServerAdvancementManagerModel::default();
        let mut first = BTreeMap::new();
        first.insert(
            id("minecraft:story/root"),
            advancement("minecraft:story/root", None, Some(display())),
        );
        first.insert(
            id("minecraft:story/mine_stone"),
            advancement("minecraft:story/mine_stone", Some("minecraft:story/root"), None),
        );

        manager.apply(first);

        assert!(manager.get(&id("minecraft:story/root")).is_some());
        assert_eq!(manager.get_all_advancements().count(), 2);
        assert!(manager.tree().get(&id("minecraft:story/mine_stone")).is_some());
        assert_eq!(
            manager.positioned_visible_roots(),
            &[id("minecraft:story/root")]
        );

        let mut second = BTreeMap::new();
        second.insert(
            id("minecraft:nether/root"),
            advancement("minecraft:nether/root", None, None),
        );
        manager.apply(second);

        assert!(manager.get(&id("minecraft:story/root")).is_none());
        assert!(manager.get(&id("minecraft:nether/root")).is_some());
        assert_eq!(manager.get_all_advancements().count(), 1);
        assert!(manager.positioned_visible_roots().is_empty());
    }

    #[test]
    fn server_advancement_manager_validation_collects_requirement_problems() {
        let mut invalid = advancement("minecraft:test/root", None, Some(display()));
        invalid.requirements = vec![vec!["missing".to_string()]];
        let mut preparations = BTreeMap::new();
        preparations.insert(id("minecraft:test/root"), invalid);

        let mut manager = ServerAdvancementManagerModel::default();
        manager.apply(preparations);

        assert_eq!(manager.validation_warnings().len(), 1);
        assert!(manager.validation_warnings()[0].contains(
            "Found validation problems in advancement minecraft:test/root: unknown criterion missing"
        ));
    }
}
