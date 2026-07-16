//! Mutable two-value tuple matching Minecraft's `Tuple`.

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tuple<A, B> {
    a: A,
    b: B,
}

impl<A, B> Tuple<A, B> {
    pub const fn new(a: A, b: B) -> Self {
        Self { a, b }
    }

    pub const fn get_a(&self) -> &A {
        &self.a
    }

    pub fn set_a(&mut self, a: A) {
        self.a = a;
    }

    pub const fn get_b(&self) -> &B {
        &self.b
    }

    pub fn set_b(&mut self, b: B) {
        self.b = b;
    }
}

#[cfg(test)]
mod tests {
    use super::Tuple;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn tuple_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/Tuple.java");
        assert_eq!(JAVA.lines().count(), 27);
        for fragment in [
            "public class Tuple<A, B>",
            "private A a",
            "private B b",
            "public Tuple(final A a, final B b)",
            "public A getA()",
            "public void setA(final A a)",
            "public B getB()",
            "public void setB(final B b)",
        ] {
            assert!(JAVA.contains(fragment), "missing Tuple source fragment: {fragment}");
        }
    }

    #[test]
    fn tuple_accessors_and_mutators_preserve_values() {
        let mut tuple = Tuple::new("left", 1);
        assert_eq!(tuple.get_a(), &"left");
        assert_eq!(tuple.get_b(), &1);
        tuple.set_a("updated");
        tuple.set_b(2);
        assert_eq!(tuple, Tuple::new("updated", 2));
    }
}
