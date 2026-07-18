//! Value functions with declared output bounds, matching `BoundedFloatFunction`.

#![allow(dead_code)]

/// A function whose output exposes its known lower and upper limits.
pub trait BoundedFloatFunction<C>: Sized {
    fn apply(&self, value: C) -> f32;

    fn min_value(&self) -> f32;

    fn max_value(&self) -> f32;

    /// Pre-composes this function with `mapper`, preserving its output bounds.
    fn comap<C2, Mapper>(self, mapper: Mapper) -> ComappedBoundedFloatFunction<Self, Mapper, C>
    where
        Mapper: Fn(C2) -> C,
    {
        ComappedBoundedFloatFunction {
            outer: self,
            mapper,
            input: std::marker::PhantomData,
        }
    }
}

/// Java `createUnlimited`: a function with no finite output bounds.
#[derive(Debug, Clone, Copy)]
pub struct UnlimitedFloatFunction<F> {
    function: F,
}

pub fn create_unlimited<C, F>(function: F) -> UnlimitedFloatFunction<F>
where
    F: Fn(C) -> f32,
{
    UnlimitedFloatFunction { function }
}

/// Java `IDENTITY`, constructed on demand because Rust closures are not constants.
pub fn identity() -> UnlimitedFloatFunction<impl Fn(f32) -> f32> {
    create_unlimited(|value| value)
}

impl<C, F> BoundedFloatFunction<C> for UnlimitedFloatFunction<F>
where
    F: Fn(C) -> f32,
{
    fn apply(&self, value: C) -> f32 {
        (self.function)(value)
    }

    fn min_value(&self) -> f32 {
        f32::NEG_INFINITY
    }

    fn max_value(&self) -> f32 {
        f32::INFINITY
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComappedBoundedFloatFunction<Outer, Mapper, C> {
    outer: Outer,
    mapper: Mapper,
    input: std::marker::PhantomData<fn(C)>,
}

impl<C, C2, Outer, Mapper> BoundedFloatFunction<C2>
    for ComappedBoundedFloatFunction<Outer, Mapper, C>
where
    Outer: BoundedFloatFunction<C>,
    Mapper: Fn(C2) -> C,
{
    fn apply(&self, value: C2) -> f32 {
        self.outer.apply((self.mapper)(value))
    }

    fn min_value(&self) -> f32 {
        self.outer.min_value()
    }

    fn max_value(&self) -> f32 {
        self.outer.max_value()
    }
}

#[cfg(test)]
mod tests {
    use super::{create_unlimited, identity, BoundedFloatFunction};

    #[cfg(vibecraft_has_decompiled_sources)]
    #[test]
    fn bounded_float_function_matches_java_source() {
        const JAVA: &str =
            vibecraft_java_source!("/net/minecraft/util/BoundedFloatFunction.java");
        assert_eq!(JAVA.lines().count(), 52);
        for fragment in [
            "BoundedFloatFunction<Float> IDENTITY = createUnlimited(input -> input)",
            "float apply(final C c)",
            "float minValue()",
            "float maxValue()",
            "static BoundedFloatFunction<Float> createUnlimited",
            "return Float.NEGATIVE_INFINITY",
            "return Float.POSITIVE_INFINITY",
            "default <C2> BoundedFloatFunction<C2> comap",
            "return outer.apply(function.apply(c2))",
        ] {
            assert!(
                JAVA.contains(fragment),
                "missing BoundedFloatFunction source fragment: {fragment}"
            );
        }
    }

    #[test]
    fn unlimited_and_identity_functions_have_infinite_bounds() {
        let doubled = create_unlimited(|value: f32| value * 2.0);
        assert_eq!(doubled.apply(1.25), 2.5);
        assert_eq!(doubled.min_value(), f32::NEG_INFINITY);
        assert_eq!(doubled.max_value(), f32::INFINITY);
        assert_eq!(identity().apply(-3.5), -3.5);
    }

    #[test]
    fn comap_transforms_inputs_and_forwards_outer_bounds() {
        let lengths = create_unlimited(|value: String| value.len() as f32);
        let mapped = lengths.comap(|value: &str| value.to_owned());
        assert_eq!(mapped.apply("minecraft"), 9.0);
        assert_eq!(mapped.min_value(), f32::NEG_INFINITY);
        assert_eq!(mapped.max_value(), f32::INFINITY);
    }
}
