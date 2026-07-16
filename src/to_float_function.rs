//! Functional float mapping contract matching Minecraft's `ToFloatFunction`.

#![allow(dead_code)]

pub trait ToFloatFunction<T> {
    fn apply_as_float(&self, value: T) -> f32;
}

impl<T, F> ToFloatFunction<T> for F
where
    F: Fn(T) -> f32,
{
    fn apply_as_float(&self, value: T) -> f32 {
        self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::ToFloatFunction;

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn to_float_function_matches_java_source() {
        const JAVA: &str = vibecraft_java_source!("/net/minecraft/util/ToFloatFunction.java");
        assert_eq!(JAVA.lines().count(), 6);
        assert!(JAVA.contains("public interface ToFloatFunction<T>"));
        assert!(JAVA.contains("float applyAsFloat(T value)"));
    }

    #[test]
    fn to_float_function_applies_mapping() {
        let function = |value: i32| value as f32 * 0.5;
        assert_eq!(function.apply_as_float(6), 3.0);
    }
}
