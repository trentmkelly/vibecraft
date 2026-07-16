//! Eager two-dimensional cache matching Minecraft's `StaticCache2D`.

#![allow(dead_code)]

pub struct StaticCache2D<T> {
    min_x: i32,
    min_z: i32,
    size_x: i32,
    size_z: i32,
    cache: Vec<T>,
}

impl<T> StaticCache2D<T> {
    pub fn create<F>(center_x: i32, center_z: i32, range: i32, initializer: F) -> Self
    where
        F: Fn(i32, i32) -> T,
    {
        assert!(range >= 0, "range must not be negative");
        let min_x = center_x - range;
        let min_z = center_z - range;
        let size = 2 * range + 1;
        let mut cache = Vec::with_capacity((size * size) as usize);
        for x in min_x..min_x + size {
            for z in min_z..min_z + size {
                cache.push(initializer(x, z));
            }
        }
        Self {
            min_x,
            min_z,
            size_x: size,
            size_z: size,
            cache,
        }
    }

    pub fn for_each(&self, mut consumer: impl FnMut(&T)) {
        for value in &self.cache {
            consumer(value);
        }
    }

    pub fn get(&self, x: i32, z: i32) -> Result<&T, String> {
        if !self.contains(x, z) {
            Err(format!(
                "Requested out of range value ({x},{z}) from {self}"
            ))
        } else {
            Ok(&self.cache[self.get_index(x, z)])
        }
    }

    pub fn contains(&self, x: i32, z: i32) -> bool {
        let delta_x = x - self.min_x;
        let delta_z = z - self.min_z;
        delta_x >= 0 && delta_x < self.size_x && delta_z >= 0 && delta_z < self.size_z
    }

    fn get_index(&self, x: i32, z: i32) -> usize {
        ((x - self.min_x) * self.size_z + (z - self.min_z)) as usize
    }
}

impl<T> std::fmt::Display for StaticCache2D<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "StaticCache2D[{}, {}, {}, {}]",
            self.min_x,
            self.min_z,
            self.min_x + self.size_x,
            self.min_z + self.size_z
        )
    }
}

#[cfg(test)]
mod tests {
    use super::StaticCache2D;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn static_cache_2d_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/StaticCache2D.java");
        assert_eq!(JAVA.lines().count(), 69);
        for fragment in [
            "int minX = centerX - range",
            "int minZ = centerZ - range",
            "int size = 2 * range + 1",
            "this.cache[this.getIndex(x, z)] = initializer.get(x, z)",
            "Requested out of range value (",
            "StaticCache2D[%d, %d, %d, %d]",
            "T get(int x, int z)",
        ] {
            assert!(JAVA.contains(fragment), "missing StaticCache2D source fragment: {fragment}");
        }
    }

    #[test]
    fn static_cache_2d_initializes_and_bounds_values_like_vanilla() {
        let cache = StaticCache2D::create(0, 0, 1, |x, z| x * 100 + z);
        assert_eq!(cache.to_string(), "StaticCache2D[-1, -1, 2, 2]");
        assert!(cache.contains(-1, -1));
        assert!(cache.contains(1, 1));
        assert!(!cache.contains(2, 0));
        assert_eq!(cache.get(-1, -1), Ok(&-101));
        assert_eq!(cache.get(1, 1), Ok(&101));
        assert_eq!(
            cache.get(2, 0),
            Err("Requested out of range value (2,0) from StaticCache2D[-1, -1, 2, 2]".to_string())
        );

        let mut values = Vec::new();
        cache.for_each(|value| values.push(*value));
        assert_eq!(values.len(), 9);
        assert_eq!(values[0], -101);
        assert_eq!(values[8], 101);
    }
}
