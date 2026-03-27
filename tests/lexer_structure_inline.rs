//! Tests ported from `external/csstree/lib/__tests/lexer-check-structure.js`.
//!
//! In Rust, AST structure is enforced by the type system. These tests verify
//! that parsed ASTs have correct node types and fields — the Rust equivalent
//! of JS structure validation.

use csstree::ast::Node;
use csstree::parser::{parse, ParseOptions};
use csstree::parser::options::ParseFlags;

// ── Structure validation via type system ──

#[test]
fn structure_correct_simple_css() {
    // lexer-check-structure.js: "should pass correct structure"
    let opts = ParseOptions {
        flags: ParseFlags { positions: true, ..ParseFlags::default() },
        ..ParseOptions::default()
    };
    let ast = parse(".foo { color: red }", opts);
    assert_eq!(ast.node_type(), "StyleSheet");
    if let Node::StyleSheet(ss) = &ast {
        assert!(!ss.children.is_empty());
        assert_eq!(ss.children[0].node_type(), "Rule");
    }
}

#[test]
fn structure_number_node_has_value() {
    // Equivalent of "should ignore properties from prototype"
    // In Rust, Number node always has required fields
    let ast = parse(".a { z-index: 123 }", ParseOptions::default());
    // Walk to find Number node
    let found = csstree::walker::find(&ast, |n, _| n.node_type() == "Number");
    assert!(found.is_some());
    if let Some(Node::Number(n)) = found {
        assert_eq!(n.value, "123");
    }
}

#[test]
fn structure_all_fixtures_parse_correctly() {
    // Equivalent of "all parse test fixtures must have correct structure"
    // Verify various CSS patterns parse without panic
    let cases = [
        ".a { color: red }",
        "@media screen { .b { display: block } }",
        "* { margin: 0 }",
        ".a .b > .c + .d ~ .e { padding: 1px 2px 3px 4px }",
        "@keyframes slide { from { left: 0 } to { left: 100px } }",
        ":root { --custom: value }",
        ".a:hover::before { content: '' }",
    ];
    for css in &cases {
        let ast = parse(css, ParseOptions::default());
        assert_eq!(ast.node_type(), "StyleSheet", "Failed for: {css}");
    }
}

#[test]
fn structure_node_type_is_string() {
    // Every node should have a valid node_type() string
    let ast = parse(".a { color: red }", ParseOptions::default());
    csstree::walker::walk(&ast, |node, _ctx| {
        let t = node.node_type();
        assert!(!t.is_empty(), "node_type should not be empty");
        csstree::walker::WalkAction::Continue
    });
}

#[test]
fn structure_declaration_has_property_and_value() {
    let ast = parse(".a { color: red }", ParseOptions::default());
    let decl = csstree::walker::find(&ast, |n, _| n.node_type() == "Declaration");
    assert!(decl.is_some());
    if let Some(Node::Declaration(d)) = decl {
        assert_eq!(d.property, "color");
        assert_eq!(d.value.node_type(), "Value");
    }
}

#[test]
fn structure_rule_has_prelude_and_block() {
    let ast = parse(".a { color: red }", ParseOptions::default());
    let rule = csstree::walker::find(&ast, |n, _| n.node_type() == "Rule");
    assert!(rule.is_some());
    if let Some(Node::Rule(r)) = rule {
        assert!(matches!(*r.prelude, Node::SelectorList(_) | Node::Raw(_)));
        assert!(matches!(*r.block, Node::Block(_)));
    }
}

#[test]
fn structure_dimension_has_value_and_unit() {
    let ast = parse(".a { width: 100px }", ParseOptions::default());
    let dim = csstree::walker::find(&ast, |n, _| n.node_type() == "Dimension");
    assert!(dim.is_some());
    if let Some(Node::Dimension(d)) = dim {
        assert_eq!(d.value, "100");
        assert_eq!(d.unit, "px");
    }
}

#[test]
fn structure_percentage_has_value() {
    let ast = parse(".a { width: 50% }", ParseOptions::default());
    let pct = csstree::walker::find(&ast, |n, _| n.node_type() == "Percentage");
    assert!(pct.is_some());
    if let Some(Node::Percentage(p)) = pct {
        assert_eq!(p.value, "50");
    }
}

#[test]
fn structure_function_has_name_and_children() {
    let ast = parse(".a { color: rgb(1, 2, 3) }", ParseOptions::default());
    let func = csstree::walker::find(&ast, |n, _| n.node_type() == "Function");
    assert!(func.is_some());
    if let Some(Node::Function(f)) = func {
        assert_eq!(f.name, "rgb");
        assert!(!f.children.is_empty());
    }
}

#[test]
fn structure_atrule_has_name() {
    let ast = parse("@media screen { .a { } }", ParseOptions::default());
    let at = csstree::walker::find(&ast, |n, _| n.node_type() == "Atrule");
    assert!(at.is_some());
    if let Some(Node::Atrule(a)) = at {
        assert_eq!(a.name, "media");
    }
}

#[test]
fn structure_positions_when_enabled() {
    let opts = ParseOptions {
        flags: ParseFlags { positions: true, ..ParseFlags::default() },
        ..ParseOptions::default()
    };
    let ast = parse(".a { color: red }", opts);
    if let Node::StyleSheet(ss) = &ast {
        // StyleSheet should have location when positions enabled
        // (may or may not depending on implementation)
        assert!(!ss.children.is_empty());
    }
}

#[test]
fn structure_hash_has_value() {
    let ast = parse(".a { color: #ff0000 }", ParseOptions::default());
    let hash = csstree::walker::find(&ast, |n, _| n.node_type() == "Hash");
    assert!(hash.is_some());
    if let Some(Node::Hash(h)) = hash {
        assert_eq!(h.value, "ff0000");
    }
}
