use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct AdvancementLayoutInput {
    pub id: String,
    pub has_display: bool,
    pub children: Vec<AdvancementLayoutInput>,
}

impl AdvancementLayoutInput {
    pub fn visible(id: &str, children: Vec<Self>) -> Self {
        Self {
            id: id.to_string(),
            has_display: true,
            children,
        }
    }

    pub fn invisible(id: &str, children: Vec<Self>) -> Self {
        Self {
            id: id.to_string(),
            has_display: false,
            children,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdvancementLayoutPosition {
    pub x: i32,
    pub y: f32,
}

#[derive(Debug, Clone)]
struct TreeNodePositionModel {
    id: String,
    parent: Option<usize>,
    previous_sibling: Option<usize>,
    child_index: i32,
    children: Vec<usize>,
    ancestor: usize,
    thread: Option<usize>,
    x: i32,
    y: f32,
    mod_value: f32,
    change: f32,
    shift: f32,
}

#[derive(Debug, Default)]
pub struct TreeNodePositionLayout {
    nodes: Vec<TreeNodePositionModel>,
}

impl TreeNodePositionLayout {
    pub fn run(
        root: &AdvancementLayoutInput,
    ) -> Result<BTreeMap<String, AdvancementLayoutPosition>, String> {
        if !root.has_display {
            return Err("Can't position children of an invisible root!".to_string());
        }

        let mut layout = Self::default();
        let root_index = layout.add_position_node(root, None, None, 1, 0)?;
        layout.first_walk(root_index);
        let min = layout.second_walk(root_index, 0.0, 0, layout.nodes[root_index].y);
        if min < 0.0 {
            layout.third_walk(root_index, -min);
        }

        let mut positions = BTreeMap::new();
        layout.finalize_position(root_index, &mut positions);
        Ok(positions)
    }

    fn add_position_node(
        &mut self,
        input: &AdvancementLayoutInput,
        parent: Option<usize>,
        previous_sibling: Option<usize>,
        child_index: i32,
        depth: i32,
    ) -> Result<usize, String> {
        if !input.has_display {
            return Err("Can't position an invisible advancement!".to_string());
        }

        let index = self.nodes.len();
        self.nodes.push(TreeNodePositionModel {
            id: input.id.clone(),
            parent,
            previous_sibling,
            child_index,
            children: Vec::new(),
            ancestor: index,
            thread: None,
            x: depth,
            y: -1.0,
            mod_value: 0.0,
            change: 0.0,
            shift: 0.0,
        });

        let mut previous = None;
        for child in &input.children {
            previous = self.add_child(index, child, previous)?;
        }
        Ok(index)
    }

    fn add_child(
        &mut self,
        parent: usize,
        input: &AdvancementLayoutInput,
        previous: Option<usize>,
    ) -> Result<Option<usize>, String> {
        if input.has_display {
            let child_index = self.nodes[parent].children.len() as i32 + 1;
            let depth = self.nodes[parent].x + 1;
            let child =
                self.add_position_node(input, Some(parent), previous, child_index, depth)?;
            self.nodes[parent].children.push(child);
            Ok(Some(child))
        } else {
            let mut previous = previous;
            for grandchild in &input.children {
                previous = self.add_child(parent, grandchild, previous)?;
            }
            Ok(previous)
        }
    }

    fn first_walk(&mut self, index: usize) {
        if self.nodes[index].children.is_empty() {
            self.nodes[index].y = self.nodes[index]
                .previous_sibling
                .map_or(0.0, |previous| self.nodes[previous].y + 1.0);
            return;
        }

        let mut default_ancestor = None;
        let children = self.nodes[index].children.clone();
        for child in children {
            self.first_walk(child);
            let fallback = default_ancestor.unwrap_or(child);
            default_ancestor = Some(self.apportion(child, fallback));
        }

        self.execute_shifts(index);
        let children = &self.nodes[index].children;
        let midpoint = (self.nodes[children[0]].y + self.nodes[*children.last().unwrap()].y) / 2.0;
        if let Some(previous) = self.nodes[index].previous_sibling {
            self.nodes[index].y = self.nodes[previous].y + 1.0;
            self.nodes[index].mod_value = self.nodes[index].y - midpoint;
        } else {
            self.nodes[index].y = midpoint;
        }
    }

    fn second_walk(&mut self, index: usize, mod_sum: f32, depth: i32, mut min: f32) -> f32 {
        self.nodes[index].y += mod_sum;
        self.nodes[index].x = depth;
        if self.nodes[index].y < min {
            min = self.nodes[index].y;
        }

        let children = self.nodes[index].children.clone();
        for child in children {
            min = self.second_walk(child, mod_sum + self.nodes[index].mod_value, depth + 1, min);
        }
        min
    }

    fn third_walk(&mut self, index: usize, offset: f32) {
        self.nodes[index].y += offset;
        let children = self.nodes[index].children.clone();
        for child in children {
            self.third_walk(child, offset);
        }
    }

    fn execute_shifts(&mut self, index: usize) {
        let mut shift = 0.0;
        let mut change = 0.0;
        let children = self.nodes[index].children.clone();
        for child in children.into_iter().rev() {
            self.nodes[child].y += shift;
            self.nodes[child].mod_value += shift;
            change += self.nodes[child].change;
            shift += self.nodes[child].shift + change;
        }
    }

    fn previous_or_thread(&self, index: usize) -> Option<usize> {
        self.nodes[index]
            .thread
            .or_else(|| self.nodes[index].children.first().copied())
    }

    fn next_or_thread(&self, index: usize) -> Option<usize> {
        self.nodes[index]
            .thread
            .or_else(|| self.nodes[index].children.last().copied())
    }

    fn apportion(&mut self, index: usize, mut default_ancestor: usize) -> usize {
        let Some(previous_sibling) = self.nodes[index].previous_sibling else {
            return default_ancestor;
        };

        let mut vir = index;
        let mut vor = index;
        let mut vil = previous_sibling;
        let parent = self.nodes[index].parent.expect("non-root node has parent");
        let mut vol = self.nodes[parent].children[0];
        let mut sir = self.nodes[index].mod_value;
        let mut sor = self.nodes[index].mod_value;
        let mut sil = self.nodes[vil].mod_value;
        let mut sol = self.nodes[vol].mod_value;

        while self.next_or_thread(vil).is_some() && self.previous_or_thread(vir).is_some() {
            vil = self.next_or_thread(vil).unwrap();
            vir = self.previous_or_thread(vir).unwrap();
            vol = self.previous_or_thread(vol).unwrap();
            vor = self.next_or_thread(vor).unwrap();
            self.nodes[vor].ancestor = index;
            let shift = self.nodes[vil].y + sil - (self.nodes[vir].y + sir) + 1.0;
            if shift > 0.0 {
                let ancestor = self.get_ancestor(vil, index, default_ancestor);
                self.move_subtree(ancestor, index, shift);
                sir += shift;
                sor += shift;
            }

            sil += self.nodes[vil].mod_value;
            sir += self.nodes[vir].mod_value;
            sol += self.nodes[vol].mod_value;
            sor += self.nodes[vor].mod_value;
        }

        if self.next_or_thread(vil).is_some() && self.next_or_thread(vor).is_none() {
            self.nodes[vor].thread = self.next_or_thread(vil);
            self.nodes[vor].mod_value += sil - sor;
        } else {
            if self.previous_or_thread(vir).is_some() && self.previous_or_thread(vol).is_none() {
                self.nodes[vol].thread = self.previous_or_thread(vir);
                self.nodes[vol].mod_value += sir - sol;
            }
            default_ancestor = index;
        }

        default_ancestor
    }

    fn move_subtree(&mut self, left: usize, right: usize, shift: f32) {
        let subtrees = self.nodes[right].child_index - self.nodes[left].child_index;
        if subtrees != 0 {
            let shift_per_subtree = shift / subtrees as f32;
            self.nodes[right].change -= shift_per_subtree;
            self.nodes[left].change += shift_per_subtree;
        }

        self.nodes[right].shift += shift;
        self.nodes[right].y += shift;
        self.nodes[right].mod_value += shift;
    }

    fn get_ancestor(&self, index: usize, other: usize, default_ancestor: usize) -> usize {
        let ancestor = self.nodes[index].ancestor;
        let other_parent = self.nodes[other].parent.expect("non-root node has parent");
        if self.nodes[other_parent].children.contains(&ancestor) {
            ancestor
        } else {
            default_ancestor
        }
    }

    fn finalize_position(
        &self,
        index: usize,
        positions: &mut BTreeMap<String, AdvancementLayoutPosition>,
    ) {
        positions.insert(
            self.nodes[index].id.clone(),
            AdvancementLayoutPosition {
                x: self.nodes[index].x,
                y: self.nodes[index].y,
            },
        );
        for child in &self.nodes[index].children {
            self.finalize_position(*child, positions);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_node_position_rejects_invisible_roots_like_java() {
        assert_eq!(
            TreeNodePositionLayout::run(&AdvancementLayoutInput::invisible("root", Vec::new()))
                .unwrap_err(),
            "Can't position children of an invisible root!"
        );
    }

    #[test]
    fn tree_node_position_places_visible_siblings_at_java_unit_spacing() {
        let positions = TreeNodePositionLayout::run(&AdvancementLayoutInput::visible(
            "root",
            vec![
                AdvancementLayoutInput::visible("left", Vec::new()),
                AdvancementLayoutInput::visible("right", Vec::new()),
            ],
        ))
        .unwrap();

        assert_eq!(
            positions.get("root"),
            Some(&AdvancementLayoutPosition { x: 0, y: 0.5 })
        );
        assert_eq!(
            positions.get("left"),
            Some(&AdvancementLayoutPosition { x: 1, y: 0.0 })
        );
        assert_eq!(
            positions.get("right"),
            Some(&AdvancementLayoutPosition { x: 1, y: 1.0 })
        );
    }

    #[test]
    fn tree_node_position_flattens_invisible_children_like_java_add_child() {
        let positions = TreeNodePositionLayout::run(&AdvancementLayoutInput::visible(
            "root",
            vec![
                AdvancementLayoutInput::invisible(
                    "hidden",
                    vec![AdvancementLayoutInput::visible("grandchild", Vec::new())],
                ),
                AdvancementLayoutInput::visible("sibling", Vec::new()),
            ],
        ))
        .unwrap();

        assert!(!positions.contains_key("hidden"));
        assert_eq!(
            positions.get("grandchild"),
            Some(&AdvancementLayoutPosition { x: 1, y: 0.0 })
        );
        assert_eq!(
            positions.get("sibling"),
            Some(&AdvancementLayoutPosition { x: 1, y: 1.0 })
        );
    }

    #[test]
    fn tree_node_position_apportions_deeper_subtrees_without_overlap() {
        let positions = TreeNodePositionLayout::run(&AdvancementLayoutInput::visible(
            "root",
            vec![
                AdvancementLayoutInput::visible(
                    "left",
                    vec![
                        AdvancementLayoutInput::visible("left_a", Vec::new()),
                        AdvancementLayoutInput::visible("left_b", Vec::new()),
                    ],
                ),
                AdvancementLayoutInput::visible(
                    "right",
                    vec![AdvancementLayoutInput::visible("right_a", Vec::new())],
                ),
            ],
        ))
        .unwrap();

        assert_eq!(positions.get("left_a").unwrap().x, 2);
        assert_eq!(positions.get("right_a").unwrap().x, 2);
        assert!(positions.get("right_a").unwrap().y > positions.get("left_b").unwrap().y);
        assert_eq!(positions.get("root").unwrap().x, 0);
    }
}
