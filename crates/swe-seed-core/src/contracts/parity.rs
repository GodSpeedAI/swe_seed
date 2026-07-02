//! Parity trait (spec 0019). Hand-written Rust types implement [`BamlParity`];
//! a test asserts each matches its `.baml` counterpart on field/variant names.

/// The `.baml`-level shape a Rust type claims to mirror.
#[derive(Debug, Clone)]
pub enum BamlShape {
    /// Struct mirroring a `.baml` `class`. `fields` are field names (any order);
    /// `field_types` are the parallel `.baml` type strings (e.g. `string`,
    /// `string[]`, `LayerName`) so type drift also fails parity (spec 0019).
    Class {
        fields: Vec<&'static str>,
        field_types: Vec<&'static str>,
    },
    /// Enum mirroring a `.baml` `enum`. `variants` are variant names (any order).
    Enum { variants: Vec<&'static str> },
}

/// A Rust type that mirrors a `.baml` `class`/`enum` 1:1.
pub trait BamlParity {
    /// Exact `.baml` type name (e.g. `LayerCapability`).
    fn baml_name() -> &'static str;
    /// Declared `.baml` shape for drift detection.
    fn baml_shape() -> BamlShape;
}
