use std::collections::{BTreeMap, BTreeSet};

use crate::registry::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementTreeHolderModel {
    id: Identifier,
    parent: Option<Identifier>,
}

impl AdvancementTreeHolderModel {
    pub fn new(id: Identifier, parent: Option<Identifier>) -> Self {
        Self { id, parent }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn parent(&self) -> Option<&Identifier> {
        self.parent.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdvancementNodeModel {
    holder: AdvancementTreeHolderModel,
    parent: Option<Identifier>,
    children: BTreeSet<Identifier>,
}

impl AdvancementNodeModel {
    pub fn new(holder: AdvancementTreeHolderModel, parent: Option<Identifier>) -> Self {
        Self {
            holder,
            parent,
            children: BTreeSet::new(),
        }
    }

    pub fn holder(&self) -> &AdvancementTreeHolderModel {
        &self.holder
    }

    pub fn id(&self) -> &Identifier {
        self.holder.id()
    }

    pub fn parent(&self) -> Option<&Identifier> {
        self.parent.as_ref()
    }

    pub fn children(&self) -> impl Iterator<Item = &Identifier> {
        self.children.iter()
    }

    pub fn add_child(&mut self, child: Identifier) {
        self.children.insert(child);
    }

    pub fn root<'a>(&'a self, tree: &'a AdvancementTreeModel) -> &'a AdvancementNodeModel {
        Self::get_root(tree, self.id())
    }

    pub fn get_root<'a>(
        tree: &'a AdvancementTreeModel,
        id: &Identifier,
    ) -> &'a AdvancementNodeModel {
        let mut root = tree.get(id).expect("advancement node must exist");
        while let Some(parent) = root.parent() {
            root = tree
                .get(parent)
                .expect("parent advancement node must exist");
        }
        root
    }

    pub fn java_equals(&self, other: &Self) -> bool {
        self.holder.id() == other.holder.id()
    }

    pub fn java_hash_key(&self) -> &Identifier {
        self.holder.id()
    }

    pub fn java_to_string(&self) -> String {
        self.holder.id().to_string()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementTreeModel {
    nodes: BTreeMap<Identifier, AdvancementNodeModel>,
    roots: BTreeSet<Identifier>,
    tasks: BTreeSet<Identifier>,
    listener: Option<AdvancementTreeListenerModel>,
    logs: Vec<AdvancementTreeLog>,
}

impl AdvancementTreeModel {
    pub fn add_all(&mut self, advancements: impl IntoIterator<Item = AdvancementTreeHolderModel>) {
        let mut advancements_to_add = advancements.into_iter().collect::<Vec<_>>();
        while !advancements_to_add.is_empty() {
            let before = advancements_to_add.len();
            let mut deferred = Vec::new();
            for holder in advancements_to_add {
                if !self.try_insert(holder.clone()) {
                    deferred.push(holder);
                }
            }
            if deferred.len() == before {
                self.logs.push(AdvancementTreeLog::Error(format!(
                    "Couldn't load advancements: {}",
                    holder_list_display(&deferred)
                )));
                break;
            }
            advancements_to_add = deferred;
        }

        self.logs.push(AdvancementTreeLog::Info(format!(
            "Loaded {} advancements",
            self.nodes.len()
        )));
    }

    fn try_insert(&mut self, holder: AdvancementTreeHolderModel) -> bool {
        let parent = holder.parent().cloned();
        if parent
            .as_ref()
            .is_some_and(|id| !self.nodes.contains_key(id))
        {
            return false;
        }

        let id = holder.id().clone();
        let node = AdvancementNodeModel::new(holder, parent.clone());
        if let Some(parent) = &parent {
            self.nodes
                .get_mut(parent)
                .expect("parent presence checked above")
                .add_child(id.clone());
        }

        self.nodes.insert(id.clone(), node);
        if parent.is_none() {
            self.roots.insert(id.clone());
            if let Some(listener) = &mut self.listener {
                listener.on_add_advancement_root(id);
            }
        } else {
            self.tasks.insert(id.clone());
            if let Some(listener) = &mut self.listener {
                listener.on_add_advancement_task(id);
            }
        }

        true
    }

    pub fn remove(&mut self, ids: &BTreeSet<Identifier>) {
        for id in ids {
            if self.nodes.contains_key(id) {
                self.remove_node(id);
            } else {
                self.logs.push(AdvancementTreeLog::Warn(format!(
                    "Told to remove advancement {id} but I don't know what that is"
                )));
            }
        }
    }

    fn remove_node(&mut self, id: &Identifier) {
        let Some(node) = self.nodes.get(id).cloned() else {
            return;
        };
        for child in node.children() {
            self.remove_node(child);
        }

        self.logs.push(AdvancementTreeLog::Info(format!(
            "Forgot about advancement {}",
            node.holder().id()
        )));
        self.nodes.remove(id);
        if node.parent().is_none() {
            self.roots.remove(id);
            if let Some(listener) = &mut self.listener {
                listener.on_remove_advancement_root(id.clone());
            }
        } else {
            self.tasks.remove(id);
            if let Some(listener) = &mut self.listener {
                listener.on_remove_advancement_task(id.clone());
            }
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.roots.clear();
        self.tasks.clear();
        if let Some(listener) = &mut self.listener {
            listener.on_advancements_cleared();
        }
    }

    pub fn roots(&self) -> impl Iterator<Item = &AdvancementNodeModel> {
        self.roots.iter().filter_map(|id| self.nodes.get(id))
    }

    pub fn nodes(&self) -> impl Iterator<Item = &AdvancementNodeModel> {
        self.nodes.values()
    }

    pub fn get(&self, id: &Identifier) -> Option<&AdvancementNodeModel> {
        self.nodes.get(id)
    }

    pub fn set_listener(&mut self, listener: Option<AdvancementTreeListenerModel>) {
        self.listener = listener;
        if let Some(listener) = &mut self.listener {
            for root in &self.roots {
                listener.on_add_advancement_root(root.clone());
            }
            for task in &self.tasks {
                listener.on_add_advancement_task(task.clone());
            }
        }
    }

    pub fn listener(&self) -> Option<&AdvancementTreeListenerModel> {
        self.listener.as_ref()
    }

    pub fn logs(&self) -> &[AdvancementTreeLog] {
        &self.logs
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdvancementTreeListenerModel {
    events: Vec<AdvancementTreeListenerEvent>,
}

impl AdvancementTreeListenerModel {
    fn on_add_advancement_root(&mut self, id: Identifier) {
        self.events.push(AdvancementTreeListenerEvent::AddRoot(id));
    }

    fn on_remove_advancement_root(&mut self, id: Identifier) {
        self.events
            .push(AdvancementTreeListenerEvent::RemoveRoot(id));
    }

    fn on_add_advancement_task(&mut self, id: Identifier) {
        self.events.push(AdvancementTreeListenerEvent::AddTask(id));
    }

    fn on_remove_advancement_task(&mut self, id: Identifier) {
        self.events
            .push(AdvancementTreeListenerEvent::RemoveTask(id));
    }

    fn on_advancements_cleared(&mut self) {
        self.events
            .push(AdvancementTreeListenerEvent::AdvancementsCleared);
    }

    pub fn events(&self) -> &[AdvancementTreeListenerEvent] {
        &self.events
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancementTreeListenerEvent {
    AddRoot(Identifier),
    RemoveRoot(Identifier),
    AddTask(Identifier),
    RemoveTask(Identifier),
    AdvancementsCleared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvancementTreeLog {
    Info(String),
    Warn(String),
    Error(String),
}

fn holder_list_display(holders: &[AdvancementTreeHolderModel]) -> String {
    let entries = holders
        .iter()
        .map(|holder| holder.id().to_string())
        .collect::<Vec<_>>();
    format!("[{}]", entries.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::parse(value).unwrap()
    }

    fn holder(value: &str, parent: Option<&str>) -> AdvancementTreeHolderModel {
        AdvancementTreeHolderModel::new(id(value), parent.map(id))
    }

    #[test]
    fn advancement_node_matches_java_identity_root_children_and_string_contracts() {
        let mut tree = AdvancementTreeModel::default();
        tree.add_all([
            holder("minecraft:story/root", None),
            holder("minecraft:story/mine_stone", Some("minecraft:story/root")),
            holder(
                "minecraft:story/smelt_iron",
                Some("minecraft:story/mine_stone"),
            ),
        ]);

        let root = tree.get(&id("minecraft:story/root")).unwrap();
        let mine_stone = tree.get(&id("minecraft:story/mine_stone")).unwrap();
        let duplicate_mine_stone = AdvancementNodeModel::new(
            holder("minecraft:story/mine_stone", None),
            Some(id("minecraft:other")),
        );

        assert_eq!(mine_stone.parent(), Some(&id("minecraft:story/root")));
        assert_eq!(
            root.children().cloned().collect::<Vec<_>>(),
            vec![id("minecraft:story/mine_stone")]
        );
        assert_eq!(mine_stone.root(&tree).id(), &id("minecraft:story/root"));
        assert!(mine_stone.java_equals(&duplicate_mine_stone));
        assert_eq!(
            mine_stone.java_hash_key(),
            &id("minecraft:story/mine_stone")
        );
        assert_eq!(mine_stone.java_to_string(), "minecraft:story/mine_stone");
    }

    #[test]
    fn advancement_tree_add_all_defers_until_parents_exist_and_replays_listener() {
        let mut tree = AdvancementTreeModel::default();
        tree.add_all([
            holder("minecraft:story/mine_stone", Some("minecraft:story/root")),
            holder("minecraft:story/root", None),
        ]);

        assert_eq!(tree.nodes().count(), 2);
        assert_eq!(
            tree.roots()
                .map(|node| node.id().clone())
                .collect::<Vec<_>>(),
            vec![id("minecraft:story/root")]
        );
        assert_eq!(
            tree.logs().last(),
            Some(&AdvancementTreeLog::Info(
                "Loaded 2 advancements".to_string()
            ))
        );

        tree.set_listener(Some(AdvancementTreeListenerModel::default()));
        assert_eq!(
            tree.listener().unwrap().events(),
            &[
                AdvancementTreeListenerEvent::AddRoot(id("minecraft:story/root")),
                AdvancementTreeListenerEvent::AddTask(id("minecraft:story/mine_stone")),
            ]
        );
    }

    #[test]
    fn advancement_tree_remove_clear_and_unknown_logs_match_java_listener_paths() {
        let mut tree = AdvancementTreeModel::default();
        tree.add_all([
            holder("minecraft:story/root", None),
            holder("minecraft:story/mine_stone", Some("minecraft:story/root")),
            holder(
                "minecraft:story/smelt_iron",
                Some("minecraft:story/mine_stone"),
            ),
        ]);
        tree.set_listener(Some(AdvancementTreeListenerModel::default()));

        tree.remove(&BTreeSet::from([id("minecraft:story/mine_stone")]));
        assert!(tree.get(&id("minecraft:story/mine_stone")).is_none());
        assert!(tree.get(&id("minecraft:story/smelt_iron")).is_none());
        assert!(tree.get(&id("minecraft:story/root")).is_some());
        assert_eq!(
            tree.listener().unwrap().events(),
            &[
                AdvancementTreeListenerEvent::AddRoot(id("minecraft:story/root")),
                AdvancementTreeListenerEvent::AddTask(id("minecraft:story/mine_stone")),
                AdvancementTreeListenerEvent::AddTask(id("minecraft:story/smelt_iron")),
                AdvancementTreeListenerEvent::RemoveTask(id("minecraft:story/smelt_iron")),
                AdvancementTreeListenerEvent::RemoveTask(id("minecraft:story/mine_stone")),
            ]
        );

        tree.remove(&BTreeSet::from([id("minecraft:missing")]));
        assert!(tree.logs().contains(&AdvancementTreeLog::Warn(
            "Told to remove advancement minecraft:missing but I don't know what that is"
                .to_string()
        )));

        tree.clear();
        assert_eq!(tree.nodes().count(), 0);
        assert!(tree.roots().next().is_none());
        assert_eq!(
            tree.listener().unwrap().events().last(),
            Some(&AdvancementTreeListenerEvent::AdvancementsCleared)
        );
    }

    #[test]
    fn advancement_tree_unloadable_children_log_error_and_stop_like_java() {
        let mut tree = AdvancementTreeModel::default();
        tree.add_all([holder(
            "minecraft:story/mine_stone",
            Some("minecraft:story/root"),
        )]);

        assert!(tree.get(&id("minecraft:story/mine_stone")).is_none());
        assert_eq!(
            tree.logs(),
            &[
                AdvancementTreeLog::Error(
                    "Couldn't load advancements: [minecraft:story/mine_stone]".to_string()
                ),
                AdvancementTreeLog::Info("Loaded 0 advancements".to_string()),
            ]
        );
    }
}
