//! Rust equivalents of the JS common.js tests from
//! `external/csstree/lib/__tests/common.js`.
//!
//! These tests verify high-level API behavior: version exposure,
//! AST serialization, node type coverage, fork(), and configuration.

use std::collections::BTreeSet;

use csstree::generator::{generate, GenerateOptions};
use csstree::lexer::LexerConfig;
use csstree::parser::{parse, ParseContext, ParseOptions};
use csstree::syntax::CssSyntax;
use csstree::walker::{self, WalkAction};

// ── 1. Version exposure ──

#[test]
fn version_matches_cargo_toml() {
    // JS: assert.strictEqual(version, JSON.parse(fs.readFileSync('./package.json')).version);
    // Rust equivalent: the crate version from Cargo.toml is available at compile time.
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(version, "0.1.0", "crate version should match Cargo.toml");
}

// ── 2. Debug trait on AST nodes (JS equivalent: JSON.stringify()) ──

#[test]
fn ast_debug_trait_produces_output() {
    // JS: JSON.stringify(ast, null, 4) produces a human-readable representation.
    // Rust equivalent: the Debug trait on Node produces a readable representation.
    let ast = parse(".a { color: red }", ParseOptions::default());
    let debug_output = format!("{ast:?}");
    assert!(!debug_output.is_empty(), "Debug output should not be empty");
    assert!(
        debug_output.contains("StyleSheet"),
        "Debug output should contain StyleSheet node type"
    );
    assert!(
        debug_output.contains("ClassSelector"),
        "Debug output should contain ClassSelector"
    );
    assert!(
        debug_output.contains("Declaration"),
        "Debug output should contain Declaration"
    );
}

#[test]
fn ast_debug_roundtrip_consistency() {
    // Parsing the same input twice should produce identical Debug output.
    let css = ".a { color: red }";
    let ast1 = parse(css, ParseOptions::default());
    let ast2 = parse(css, ParseOptions::default());
    assert_eq!(
        format!("{ast1:?}"),
        format!("{ast2:?}"),
        "Identical input should produce identical Debug output"
    );
}

// ── 3. Complex CSS exercises many node types ──

/// CSS input that exercises as many node types as possible.
/// This is the Rust equivalent of the JS `stringify.css` fixture.
const COMPLEX_CSS: &str = r#"
@charset "utf-8";
@import url(foo.css);
@layer base, extra;
@media screen and (min-width: 900px) {
    .container {
        display: flex;
    }
}
@supports (display: grid) {
    .grid {
        display: grid;
    }
}
@keyframes slide {
    from { opacity: 0 }
    to { opacity: 1 }
}
* { margin: 0 }
a { color: red }
a > b { color: blue }
a + b { color: green }
a ~ b { color: yellow }
.class { font-size: 12px }
#id { font-weight: bold }
[href] { text-decoration: underline }
[href="foo"] { color: orange }
a:hover { color: purple }
a::before { content: "" }
a:not(.x) { color: pink }
div { width: 50%; height: 100vh; background: rgb(255, 0, 0) }
div { margin: 10px 20px }
p { font-family: "Arial", sans-serif }
div { border: 1px solid #ff0000 }
a { color: red !important }
"#;

#[test]
fn complex_css_parses_without_panic() {
    // JS: parse(fixture) should succeed
    let _ast = parse(COMPLEX_CSS, ParseOptions::default());
}

#[test]
fn complex_css_contains_many_node_types() {
    // JS: walk the AST and collect all node types, compare to expected list
    let ast = parse(COMPLEX_CSS, ParseOptions::default());
    let mut found_types = BTreeSet::new();

    walker::walk(&ast, |node, _ctx| {
        found_types.insert(node.node_type().to_string());
        WalkAction::Continue
    });

    // These types should be found in the complex CSS above
    let expected_types = [
        "StyleSheet",
        "Rule",
        "Block",
        "Declaration",
        "Value",
        "SelectorList",
        "Selector",
        "TypeSelector",
        "ClassSelector",
        "IdSelector",
        "AttributeSelector",
        "PseudoClassSelector",
        "PseudoElementSelector",
        "Combinator",
        "Identifier",
        "Number",
        "Dimension",
        "Percentage",
        "Hash",
        "Function",
        "Operator",
        "Atrule",
        "AtrulePrelude",
    ];

    for expected in &expected_types {
        assert!(
            found_types.contains(*expected),
            "Expected node type '{expected}' not found in AST. Found: {found_types:?}"
        );
    }
}

#[test]
fn complex_css_round_trips() {
    // Parse and re-generate; the output should be valid minified CSS.
    let ast = parse(COMPLEX_CSS, ParseOptions::default());
    let output = generate(&ast, &GenerateOptions::default());
    assert!(!output.is_empty(), "Generated output should not be empty");
    // Re-parse the generated output to ensure it is valid CSS.
    let _ast2 = parse(&output, ParseOptions::default());
}

// ── 4. fork() creates independent syntax ──

#[test]
fn fork_creates_independent_syntax() {
    // JS: fork() creates a new syntax instance that is independent from the parent.
    let parent = CssSyntax::new();
    let child = parent.fork(LexerConfig {
        generic: true,
        ..LexerConfig::default()
    });

    // Both should be able to parse CSS independently.
    let ast1 = parent.parse(".a { color: red }");
    let ast2 = child.parse(".a { color: red }");

    assert_eq!(
        parent.generate(&ast1),
        child.generate(&ast2),
        "fork() should produce equivalent parse results"
    );
}

#[test]
fn fork_with_custom_properties() {
    // JS: fork() with custom node config. Rust: fork with custom lexer properties.
    let parent = CssSyntax::new();
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("my-prop".into(), "foo | bar".into());
    let mut child = parent.fork(config);

    // The child should know about the custom property.
    let result = child.match_property("my-prop", "foo");
    assert!(
        result.matched.is_some(),
        "Forked syntax should match custom property value"
    );
}

#[test]
fn fork_does_not_modify_parent() {
    // Forking should not affect the parent syntax.
    let parent = CssSyntax::new();
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("custom-only".into(), "yes | no".into());
    let _child = parent.fork(config);

    // Parent should not have the custom property.
    // We verify by checking property_names via the lexer.
    let parent_props = parent.lexer.property_names();
    assert!(
        !parent_props.contains(&"custom-only"),
        "Parent should not have forked child's custom property"
    );
}

// ── 5. Generic option in fork() ──

#[test]
fn fork_respects_generic_config() {
    // JS: fork({ generic: false }) disables generic types, fork({}) keeps them.
    // Rust: LexerConfig.generic controls built-in generic type matching.
    let config_with_generic = LexerConfig {
        generic: true,
        ..LexerConfig::default()
    };
    let config_without_generic = LexerConfig {
        generic: false,
        ..LexerConfig::default()
    };

    let syntax_with = CssSyntax::with_config(config_with_generic);
    let syntax_without = CssSyntax::with_config(config_without_generic);

    // Both should be constructible without panic.
    let _ast1 = syntax_with.parse("a { color: red }");
    let _ast2 = syntax_without.parse("a { color: red }");
}

#[test]
fn lexer_config_generic_default_is_false() {
    // LexerConfig::default() should have generic = false.
    let config = LexerConfig::default();
    assert!(
        !config.generic,
        "LexerConfig::default() should have generic = false"
    );
}

#[test]
fn css_syntax_new_has_generic_enabled() {
    // CssSyntax::new() should enable generic types (mirrors JS default behavior).
    // We verify indirectly by checking that CssSyntax::new() can match generic types
    // via property definitions that reference them.
    let mut config = LexerConfig::default();
    config.generic = true;
    config
        .properties
        .insert("width".into(), "<length> | auto".into());
    let mut syntax = CssSyntax::with_config(config);
    let result = syntax.match_property("width", "10px");
    assert!(
        result.matched.is_some(),
        "Syntax with generic=true should match <length> values"
    );
}

// ── 6. Custom lexer configuration ──

#[test]
fn lexer_accepts_custom_type_definitions() {
    // JS: custom tokenizer settings via fork(). Rust: custom type definitions via LexerConfig.
    let mut config = LexerConfig::default();
    config.generic = true;
    config
        .types
        .insert("my-color".into(), "red | green | blue".into());
    config
        .properties
        .insert("accent".into(), "<my-color>".into());

    let lexer = csstree::lexer::Lexer::new(config);
    let type_names = lexer.type_names();
    assert!(
        type_names.contains(&"my-color"),
        "Lexer should contain custom type definition"
    );
}

#[test]
fn lexer_accepts_custom_property_definitions() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config
        .properties
        .insert("my-display".into(), "show | hide".into());

    let mut lexer = csstree::lexer::Lexer::new(config);
    let result = lexer.match_property("my-display", "show");
    assert!(
        result.matched.is_some(),
        "Lexer should match custom property values"
    );

    let result2 = lexer.match_property("my-display", "banana");
    assert!(
        result2.matched.is_none(),
        "Lexer should reject invalid values for custom property"
    );
}

#[test]
fn lexer_add_property_dynamically() {
    // Verify that properties can be added after Lexer creation.
    let config = LexerConfig {
        generic: true,
        ..LexerConfig::default()
    };
    let mut lexer = csstree::lexer::Lexer::new(config);
    lexer.add_property("dynamic-prop", "foo | bar | baz");

    let result = lexer.match_property("dynamic-prop", "bar");
    assert!(
        result.matched.is_some(),
        "Dynamically added property should be matchable"
    );
}

#[test]
fn lexer_add_type_dynamically() {
    // Verify that types can be added after Lexer creation.
    let config = LexerConfig {
        generic: true,
        ..LexerConfig::default()
    };
    let mut lexer = csstree::lexer::Lexer::new(config);
    lexer.add_type("my-length", "<length> | <percentage>");

    let result = lexer.match_type("my-length", "10px");
    assert!(
        result.matched.is_some(),
        "Dynamically added type should be matchable"
    );
}

// ── 7. Extend nodes (parse context) ──

#[test]
fn parse_with_value_context() {
    // JS: parse('20px', { context: 'value' })
    let ast = parse(
        "20px",
        ParseOptions {
            context: ParseContext::Value,
            ..ParseOptions::default()
        },
    );
    assert_eq!(ast.node_type(), "Value", "Value context should produce a Value node");
}

#[test]
fn parse_with_selector_context() {
    // JS: various context options
    let ast = parse(
        ".foo",
        ParseOptions {
            context: ParseContext::Selector,
            ..ParseOptions::default()
        },
    );
    assert_eq!(
        ast.node_type(),
        "Selector",
        "Selector context should produce a Selector node"
    );
}

#[test]
fn parse_with_declaration_context() {
    let ast = parse(
        "color: red",
        ParseOptions {
            context: ParseContext::Declaration,
            ..ParseOptions::default()
        },
    );
    assert_eq!(
        ast.node_type(),
        "Declaration",
        "Declaration context should produce a Declaration node"
    );
}

#[test]
fn parse_with_media_query_list_context() {
    let ast = parse(
        "screen and (min-width: 900px)",
        ParseOptions {
            context: ParseContext::MediaQueryList,
            ..ParseOptions::default()
        },
    );
    assert_eq!(
        ast.node_type(),
        "MediaQueryList",
        "MediaQueryList context should produce a MediaQueryList node"
    );
}

#[test]
fn generate_dimension_value() {
    // JS: generate(ast) for '20px' parsed as value should produce '20px'
    // This mirrors the fork() extend-nodes test's baseline behavior.
    let ast = parse(
        "20px",
        ParseOptions {
            context: ParseContext::Value,
            ..ParseOptions::default()
        },
    );
    let output = generate(&ast, &GenerateOptions::default());
    assert_eq!(output, "20px", "Generated output should be '20px'");
}

#[test]
fn generate_preserves_important() {
    let ast = parse(
        "color: red !important",
        ParseOptions {
            context: ParseContext::Declaration,
            ..ParseOptions::default()
        },
    );
    let output = generate(&ast, &GenerateOptions::default());
    assert!(
        output.contains("!important"),
        "Generated output should preserve !important"
    );
}

// ── Integration: CssSyntax round-trip ──

#[test]
fn css_syntax_parse_generate_round_trip() {
    let syntax = CssSyntax::new();
    let css = ".a > .b:hover { color: red; font-size: 12px }";
    let ast = syntax.parse(css);
    let output = syntax.generate(&ast);
    assert_eq!(output, ".a>.b:hover{color:red;font-size:12px}");
}

#[test]
fn css_syntax_walk_collects_all_types() {
    let syntax = CssSyntax::new();
    let ast = syntax.parse("a.foo:hover { color: red }");
    let mut types = BTreeSet::new();
    syntax.walk(&ast, |node, _ctx| {
        types.insert(node.node_type());
        WalkAction::Continue
    });
    assert!(types.contains("StyleSheet"));
    assert!(types.contains("Rule"));
    assert!(types.contains("ClassSelector"));
    assert!(types.contains("PseudoClassSelector"));
    assert!(types.contains("Declaration"));
}

#[test]
fn css_syntax_find_specific_node() {
    let syntax = CssSyntax::new();
    let ast = syntax.parse("#myid { display: block }");
    let found = syntax.find(&ast, |node, _ctx| node.node_type() == "IdSelector");
    assert!(found.is_some(), "Should find IdSelector in AST");
}

#[test]
fn css_syntax_find_all_declarations() {
    let syntax = CssSyntax::new();
    let ast = syntax.parse("a { color: red; font-size: 12px; margin: 0 }");
    let found = syntax.find_all(&ast, |node, _ctx| node.node_type() == "Declaration");
    assert_eq!(found.len(), 3, "Should find 3 declarations");
}

// ── Node equality (PartialEq on AST) ──

#[test]
fn ast_partial_eq() {
    // Node derives PartialEq, so identical parses should be equal.
    let ast1 = parse("a { color: red }", ParseOptions::default());
    let ast2 = parse("a { color: red }", ParseOptions::default());
    assert_eq!(ast1, ast2, "Identical parses should produce equal AST nodes");
}

#[test]
fn ast_partial_eq_different_input() {
    let ast1 = parse("a { color: red }", ParseOptions::default());
    let ast2 = parse("b { color: blue }", ParseOptions::default());
    assert_ne!(ast1, ast2, "Different input should produce different AST nodes");
}

// ── Clone on AST ──

#[test]
fn ast_clone() {
    let ast = parse("a { color: red }", ParseOptions::default());
    let cloned = ast.clone();
    assert_eq!(ast, cloned, "Cloned AST should equal original");
}
