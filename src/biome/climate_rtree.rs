use super::*;

impl ClimateRTree {
    /// Builds an R-tree from a parameter list.
    /// Mirrors Java's `Climate.RTree.create()`.
    pub fn build(entries: &[ClimateBiomeEntry]) -> Self {
        assert!(
            !entries.is_empty(),
            "Need at least one value to build the search tree."
        );
        let leaves: Vec<Self> = entries
            .iter()
            .map(|e| Self::Leaf {
                parameter_space: e.parameters.parameter_space(),
                biome: e.biome,
            })
            .collect();
        Self::build_internal(leaves)
    }

    fn parameter_space(&self) -> &[ClimateParameter; 7] {
        match self {
            Self::Leaf {
                parameter_space, ..
            }
            | Self::SubTree {
                parameter_space, ..
            } => parameter_space,
        }
    }

    /// Sum of squared per-dimension distances. For SubTree nodes this is a lower-bound
    /// used for pruning; for Leaf nodes it is the exact distance.
    /// Mirrors Java's `Node.distance(long[] target)`.
    fn node_distance(&self, target: &[i64; 7]) -> i64 {
        self.parameter_space()
            .iter()
            .zip(target.iter())
            .map(|(p, &t)| square(p.distance(t)))
            .sum()
    }

    fn bounding_box_of(
        mut iter: impl Iterator<Item = [ClimateParameter; 7]>,
    ) -> [ClimateParameter; 7] {
        let Some(first) = iter.next() else {
            unreachable!("R-tree bounding boxes are only built from non-empty node lists");
        };
        iter.fold(first, |mut acc, ps| {
            for i in 0..7 {
                acc[i] = acc[i].span_parameter(Some(ps[i]));
            }
            acc
        })
    }

    fn bounding_box_cost(ps: &[ClimateParameter; 7]) -> i64 {
        ps.iter().map(|p| (p.max - p.min).abs()).sum()
    }

    fn center(nodes: &[Self], index: usize, dimension: usize) -> i64 {
        let parameter = &nodes[index].parameter_space()[dimension];
        (parameter.min + parameter.max) / 2
    }

    fn sort_indices_by_dimension(indices: &mut [usize], nodes: &[Self], start_dimension: usize) {
        const DIMS: usize = 7;
        indices.sort_by(|&left, &right| {
            for offset in 0..DIMS {
                let dimension = (start_dimension + offset) % DIMS;
                match Self::center(nodes, left, dimension)
                    .cmp(&Self::center(nodes, right, dimension))
                {
                    std::cmp::Ordering::Equal => {}
                    ordering => return ordering,
                }
            }
            std::cmp::Ordering::Equal
        });
    }

    fn bucket_cost(indices: &[usize], nodes: &[Self], bucket_size: usize) -> i64 {
        indices
            .chunks(bucket_size)
            .map(|bucket| {
                let parameter_space =
                    Self::bounding_box_of(bucket.iter().map(|&i| *nodes[i].parameter_space()));
                Self::bounding_box_cost(&parameter_space)
            })
            .sum()
    }

    fn bucket_groups(indices: &[usize], bucket_size: usize) -> Vec<Vec<usize>> {
        indices
            .chunks(bucket_size)
            .map(|bucket| bucket.to_vec())
            .collect()
    }

    fn sort_bucket_groups_by_dimension(
        bucket_groups: &mut [Vec<usize>],
        nodes: &[Self],
        dimension: usize,
    ) {
        bucket_groups.sort_by_key(|bucket| {
            let (lo, hi) = bucket
                .iter()
                .fold((i64::MAX, i64::MIN), |(lo, hi), &index| {
                    let parameter = &nodes[index].parameter_space()[dimension];
                    (lo.min(parameter.min), hi.max(parameter.max))
                });
            ((lo + hi) / 2).abs()
        });
    }

    fn build_children_from_buckets(nodes: Vec<Self>, bucket_groups: Vec<Vec<usize>>) -> Vec<Self> {
        let mut slots: Vec<Option<Self>> = nodes.into_iter().map(Some).collect();
        bucket_groups
            .into_iter()
            .map(|bucket| {
                let bucket_nodes: Vec<Self> = bucket
                    .into_iter()
                    .map(|index| {
                        let Some(node) = slots[index].take() else {
                            unreachable!("bucket groups contain each node index exactly once");
                        };
                        node
                    })
                    .collect();
                Self::build_internal(bucket_nodes)
            })
            .collect()
    }

    /// Recursive tree builder. Mirrors Java's `RTree.build(dimensions, children)`.
    ///
    /// - Size 1: return the single node directly.
    /// - Size ≤ 6: sort by |center| magnitude, wrap in a SubTree.
    /// - Size > 6: try each of 7 dimensions; pick the one minimising total
    ///   bounding-box cost across buckets of size 6^floor(log(n−0.01)/log(6)).
    ///   Sort winning buckets by |center| of that dimension, recurse into each.
    fn build_internal(nodes: Vec<Self>) -> Self {
        const MAX_CHILDREN: usize = 6;
        const DIMS: usize = 7;

        if nodes.len() == 1 {
            let Some(node) = nodes.into_iter().next() else {
                unreachable!("node length was checked before consuming the vector");
            };
            return node;
        }

        if nodes.len() <= MAX_CHILDREN {
            let mut nodes = nodes;
            nodes.sort_by_key(|n| {
                n.parameter_space()
                    .iter()
                    .map(|p| ((p.min + p.max) / 2).abs())
                    .sum::<i64>()
            });
            let ps = Self::bounding_box_of(nodes.iter().map(|n| *n.parameter_space()));
            return Self::SubTree {
                parameter_space: ps,
                children: nodes,
            };
        }

        let n = nodes.len();
        // Java: (int)Math.pow(6.0, Math.floor(Math.log(n - 0.01) / Math.log(6.0)))
        let expected: usize = {
            let log6 = 6.0_f64.ln();
            let exp = ((n as f64 - 0.01).ln() / log6).floor() as u32;
            6usize.pow(exp).max(1)
        };

        let mut indices: Vec<usize> = (0..n).collect();
        let mut min_cost = i64::MAX;
        let mut best_dim = 0usize;

        for d in 0..DIMS {
            // Lexicographic sort: primary = dim d, tiebreak by (d+1)%7 … (d+6)%7.
            // Matches Java's `sort(children, dimensions, d, false)`.
            Self::sort_indices_by_dimension(&mut indices, &nodes, d);
            let cost = Self::bucket_cost(&indices, &nodes, expected);
            if cost < min_cost {
                min_cost = cost;
                best_dim = d;
            }
        }

        // Re-sort by the winning dimension (matches Java's re-sort before grouping).
        Self::sort_indices_by_dimension(&mut indices, &nodes, best_dim);
        let mut bucket_groups = Self::bucket_groups(&indices, expected);

        // Sort bucket groups by |center| of best_dim (matches Java's `sort(minBuckets, …, true)`).
        Self::sort_bucket_groups_by_dimension(&mut bucket_groups, &nodes, best_dim);
        let children = Self::build_children_from_buckets(nodes, bucket_groups);

        let ps = Self::bounding_box_of(children.iter().map(|c| *c.parameter_space()));
        Self::SubTree {
            parameter_space: ps,
            children,
        }
    }

    /// Entry point for nearest-neighbour search. Mirrors Java's `RTree.search()`.
    pub fn find_nearest(&self, target: [i64; 7]) -> &'static str {
        match self {
            Self::Leaf { biome, .. } => biome,
            Self::SubTree { .. } => {
                let mut best_dist = i64::MAX;
                let mut best_biome = "";
                self.search_subtree(&target, &mut best_dist, &mut best_biome);
                best_biome
            }
        }
    }

    /// Recursive subtree search with bounding-box pruning.
    /// Mirrors Java's `SubTree.search()`.
    fn search_subtree(
        &self,
        target: &[i64; 7],
        best_dist: &mut i64,
        best_biome: &mut &'static str,
    ) {
        if let Self::SubTree { children, .. } = self {
            for child in children {
                let bound = child.node_distance(target);
                if bound < *best_dist {
                    match child {
                        Self::Leaf { biome, .. } => {
                            *best_dist = bound;
                            *best_biome = biome;
                        }
                        Self::SubTree { .. } => {
                            child.search_subtree(target, best_dist, best_biome);
                        }
                    }
                }
            }
        }
    }
}
