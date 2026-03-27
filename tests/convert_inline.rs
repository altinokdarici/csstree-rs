//! Inline tests ported from `external/csstree/lib/__tests/convert.js`.
//!
//! In JS, `toPlainObject` serializes AST to JSON and `fromPlainObject` restores it.
//! In Rust, the equivalent is `Clone` (deep copy) + round-trip through `generate`.

use csstree::generator::{generate, GenerateOptions};
use csstree::parser::{parse, ParseOptions};

/// Equivalent to JS `fromPlainObject` test:
/// Parse CSS, clone the AST (simulating serialization), generate CSS from clone,
/// and verify it matches the original.
#[test]
fn from_plain_object_round_trip() {
    let css = ".test{a:123}";
    let ast = parse(css, ParseOptions::default());
    let cloned = ast.clone();
    let actual = generate(&cloned, &GenerateOptions::default());
    assert_eq!(actual, css);
}

/// Equivalent to JS `toPlainObject` test:
/// Parse CSS, clone the AST, verify the clone equals the original (structural equality).
#[test]
fn to_plain_object_structural_equality() {
    let css = ".test{a:123}";
    let ast = parse(css, ParseOptions::default());
    let cloned = ast.clone();
    assert_eq!(ast, cloned, "Cloned AST should be structurally equal to original");
}
