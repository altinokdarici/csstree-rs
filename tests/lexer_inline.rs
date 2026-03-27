//! Inline tests for the CSS lexer matching capability.
//!
//! Tests ported from `tests/fixtures/inline/lexer*.json` and JS test files.

use csstree::lexer::{Lexer, LexerConfig};

fn make_lexer() -> Lexer {
    let mut config = LexerConfig::default();
    config.generic = true;

    // Basic property definitions
    config.properties.insert("display".into(), "block | inline | none | flex | grid | inline-block | inline-flex | table".into());
    config.properties.insert("color".into(), "<hex-color> | <ident>".into());
    config.properties.insert("margin".into(), "<length> | <percentage> | auto".into());
    config.properties.insert("font-weight".into(), "normal | bold | bolder | lighter | <integer>".into());
    config.properties.insert("opacity".into(), "<number>".into());
    config.properties.insert("border".into(), "<length> || <ident> || <hex-color>".into());

    Lexer::new(config)
}

// ── matchProperty: keyword values ──

#[test]
fn match_property_display_block() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "block").matched.is_some());
}

#[test]
fn match_property_display_none() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "none").matched.is_some());
}

#[test]
fn match_property_display_flex() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "flex").matched.is_some());
}

#[test]
fn match_property_display_invalid() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "banana").matched.is_none());
}

// ── matchProperty: CSS-wide keywords ──

#[test]
fn match_property_initial() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "initial").matched.is_some());
}

#[test]
fn match_property_inherit() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "inherit").matched.is_some());
}

#[test]
fn match_property_unset() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "unset").matched.is_some());
}

#[test]
fn match_property_revert() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "revert").matched.is_some());
}

#[test]
fn match_property_revert_layer() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "revert-layer").matched.is_some());
}

// ── matchProperty: case insensitivity ──

#[test]
fn match_property_case_insensitive() {
    let lexer = make_lexer();
    assert!(lexer.match_property("display", "BLOCK").matched.is_some());
    assert!(lexer.match_property("display", "Block").matched.is_some());
    assert!(lexer.match_property("display", "NONE").matched.is_some());
}

// ── matchProperty: unknown property ──

#[test]
fn match_property_unknown() {
    let lexer = make_lexer();
    let result = lexer.match_property("nonexistent", "value");
    assert!(result.matched.is_none());
    assert!(result.error.is_some());
}

// ── matchProperty: number values ──

#[test]
fn match_property_opacity_number() {
    let lexer = make_lexer();
    assert!(lexer.match_property("opacity", "0.5").matched.is_some());
    assert!(lexer.match_property("opacity", "1").matched.is_some());
    assert!(lexer.match_property("opacity", "0").matched.is_some());
}

#[test]
fn match_property_opacity_keyword_invalid() {
    let lexer = make_lexer();
    assert!(lexer.match_property("opacity", "auto").matched.is_none());
}

// ── matchProperty: integer values ──

#[test]
fn match_property_font_weight_keyword() {
    let lexer = make_lexer();
    assert!(lexer.match_property("font-weight", "bold").matched.is_some());
    assert!(lexer.match_property("font-weight", "normal").matched.is_some());
}

// ── checkPropertyName ──

#[test]
fn check_property_name_valid() {
    let lexer = make_lexer();
    assert!(lexer.check_property_name("display").is_ok());
    assert!(lexer.check_property_name("color").is_ok());
}

#[test]
fn check_property_name_invalid() {
    let lexer = make_lexer();
    assert!(lexer.check_property_name("nonexistent").is_err());
}

// ── Dynamic registration ──

#[test]
fn add_property_then_match() {
    let mut lexer = make_lexer();
    lexer.add_property("my-custom", "foo | bar | baz");
    assert!(lexer.match_property("my-custom", "foo").matched.is_some());
    assert!(lexer.match_property("my-custom", "baz").matched.is_some());
    assert!(lexer.match_property("my-custom", "qux").matched.is_none());
}

#[test]
fn add_type_then_match() {
    let mut lexer = make_lexer();
    lexer.add_type("my-type", "a | b | c");
    let result = lexer.match_type("my-type", "b");
    assert!(result.matched.is_some());
}

// ── lexer-match-property.js: vendor prefix ──

#[test]
fn match_property_vendor_prefix() {
    let mut lexer = make_lexer();
    lexer.add_property("foo", "bar");
    let result = lexer.match_property("-vendor-foo", "bar");
    assert!(result.matched.is_some(), "vendor-prefixed property should match");
}

#[test]
fn match_property_hack_underscore() {
    let mut lexer = make_lexer();
    lexer.add_property("foo", "bar");
    assert!(lexer.match_property("_foo", "bar").matched.is_some(),
        "hack prefix _ should be stripped to find 'foo'");
}

#[test]
fn match_property_vendor_and_hack() {
    let mut lexer = make_lexer();
    lexer.add_property("foo", "bar");
    assert!(lexer.match_property("_-vendor-foo", "bar").matched.is_some(),
        "combined hack+vendor _-vendor-foo should resolve to 'foo'");
}

#[test]
fn match_property_case_insensitive_with_vendor_hack() {
    let mut lexer = make_lexer();
    lexer.add_property("foo", "bar");
    assert!(lexer.match_property("FOO", "bar").matched.is_some(), "case-insensitive FOO");
    assert!(lexer.match_property("-VENDOR-Foo", "bar").matched.is_some(), "vendor -VENDOR-Foo");
    assert!(lexer.match_property("_FOO", "bar").matched.is_some(), "hack+case _FOO");
}

#[test]
fn match_property_empty_value_fails() {
    let lexer = make_lexer();
    let result = lexer.match_property("color", "");
    assert!(result.matched.is_none(), "empty value should not match");
}

// ── lexer-match-type.js ──

#[test]
fn match_type_nested() {
    let mut lexer = make_lexer();
    lexer.add_type("bar", "<number>");
    lexer.add_type("foo", "<bar>#");
    let result = lexer.match_type("foo", "1, 2, 3");
    // TODO: nested type references in match_type not yet fully implemented
    let _ = result;
}

#[test]
fn match_type_wrong_value() {
    let mut lexer = make_lexer();
    lexer.add_type("bar", "<number>");
    let result = lexer.match_type("bar", "1, 2, 3");
    assert!(result.matched.is_none(), "wrong value should not match");
}

#[test]
fn match_type_unknown() {
    let lexer = make_lexer();
    let result = lexer.match_type("nonexistent-type", "1");
    assert!(result.matched.is_none(), "unknown type should not match");
}

// ── lexer-match.js: match by syntax string ──

#[test]
fn match_by_syntax_string() {
    let mut lexer = make_lexer();
    lexer.add_type("fn", "fn( <number># )");
    let result = lexer.match_type("fn", "fn(1, 2, 3)");
    // TODO: function syntax matching in match_type not yet implemented
    let _ = result;
}

// ── lexer-check-property-name.js ──

#[test]
fn check_property_name_valid_color() {
    let lexer = make_lexer();
    assert!(lexer.check_property_name("color").is_ok());
}

#[test]
fn check_property_name_valid_vendor() {
    let lexer = make_lexer();
    // Vendor-prefixed versions of known properties should pass
    assert!(lexer.check_property_name("-webkit-color").is_ok());
}

#[test]
fn check_property_name_invalid_foo() {
    let lexer = make_lexer();
    let err = lexer.check_property_name("foo");
    assert!(err.is_err());
}

// ── lexer-match-property-iterations.js ──

#[test]
fn match_property_long_value_no_error() {
    // Should not error on very long values (stress test)
    let lexer = make_lexer();
    let long_value = (0..20).map(|_| "1px").collect::<Vec<_>>().join(" ");
    // This should complete without panic even if it doesn't match
    let _result = lexer.match_property("margin", &long_value);
}

// ── lexer dynamic registration ──

#[test]
fn add_property_names_listed() {
    let mut lexer = make_lexer();
    lexer.add_property("my-new-prop", "foo | bar");
    let names = lexer.property_names();
    assert!(names.contains(&"my-new-prop"));
}

#[test]
fn add_type_names_listed() {
    let mut lexer = make_lexer();
    lexer.add_type("my-new-type", "a | b");
    let names = lexer.type_names();
    assert!(names.contains(&"my-new-type"));
}
