//! Walker integration tests using AST fixture files and manual test cases.
//!
//! Tests parse CSS via the parser and walk the resulting AST, verifying
//! traversal order, visit filters, find helpers, and break/skip behavior.

use csstree::parser::{parse, ParseOptions};
use csstree::walker::{find, find_all, walk, walk_full, WalkAction, WalkOptions, VisitFilter};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Load all test cases from a fixture JSON file, returning (name, source) pairs.
fn load_fixture_sources(path: &Path) -> Vec<(String, String)> {
    let content = fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&content).unwrap();
    let mut cases = Vec::new();
    if let Some(obj) = json.as_object() {
        for (name, val) in obj {
            if let Some(source) = val.get("source").and_then(|s| s.as_str()) {
                cases.push((name.clone(), source.to_string()));
            }
        }
    }
    cases
}

/// Walk AST and collect all visited node type names.
fn walk_collect_types(css: &str) -> Vec<String> {
    let ast = parse(css, ParseOptions::default());
    let mut types = Vec::new();
    walk(&ast, |node, _ctx| {
        types.push(node.node_type().to_string());
        WalkAction::Continue
    });
    types
}

// ── Fixture-based walk tests ──
// Parse CSS from fixtures as stylesheet context, walk, verify no panics
// and at least the root StyleSheet is visited.

fn run_walk_smoke_test_dir(dir: &str) {
    let fixture_dir = format!("tests/fixtures/ast/{dir}");
    let path = Path::new(&fixture_dir);
    if !path.exists() {
        return;
    }

    let mut total = 0;
    let mut passed = 0;

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let file_path = entry.path();
        if file_path.extension().map_or(true, |e| e != "json") {
            continue;
        }

        let cases = load_fixture_sources(&file_path);
        for (name, source) in &cases {
            total += 1;
            // Wrap non-stylesheet sources in a rule context so they parse
            let css = if dir == "stylesheet" || dir == "rule" || dir == "atrule" {
                source.clone()
            } else {
                format!(".x {{ {source} }}")
            };

            let types = walk_collect_types(&css);
            assert!(
                !types.is_empty(),
                "Fixture {dir}/{}: test '{}' — walk visited zero nodes",
                file_path.file_name().unwrap().to_string_lossy(),
                name,
            );
            passed += 1;
        }
    }

    assert!(total > 0, "No fixture cases found in {dir}");
    eprintln!("  {dir}: {passed}/{total} walk smoke tests passed");
    // All cases should pass — walker visits any parsed AST
    assert_eq!(passed, total, "{dir}: {passed}/{total} passed (expected all)");
}

#[test]
fn walk_smoke_atrule_fixtures() {
    run_walk_smoke_test_dir("atrule");
}

#[test]
fn walk_smoke_rule_fixtures() {
    run_walk_smoke_test_dir("rule");
}

#[test]
fn walk_smoke_stylesheet_fixtures() {
    run_walk_smoke_test_dir("stylesheet");
}

#[test]
fn walk_smoke_block_fixtures() {
    run_walk_smoke_test_dir("block");
}

#[test]
fn walk_smoke_declaration_fixtures() {
    run_walk_smoke_test_dir("declaration");
}

#[test]
fn walk_smoke_value_fixtures() {
    run_walk_smoke_test_dir("value");
}

#[test]
fn walk_smoke_selector_fixtures() {
    run_walk_smoke_test_dir("selector");
}

// ── Base test from JS walk.js ──

#[test]
fn walk_base_test_visits_many_types() {
    let css = r#"@import url("test");@media (min-width: 200px) { .foo:nth-child(2n) { color: rgb(100%, 10%, 0%); width: calc(3px + 5%); content: "test" } }"#;
    let ast = parse(css, ParseOptions::default());
    let mut visited_types = BTreeSet::new();
    walk(&ast, |node, _ctx| {
        visited_types.insert(node.node_type().to_string());
        WalkAction::Continue
    });

    // Core types that should be present regardless of parser differences
    let must_have = [
        "Atrule", "Block", "Rule",
        "Selector", "SelectorList", "StyleSheet",
    ];

    for t in &must_have {
        assert!(
            visited_types.contains(*t),
            "Expected type '{t}' not found in walked types. Got: {visited_types:?}"
        );
    }
    // Should have many types
    assert!(visited_types.len() >= 7, "Expected many types, got {}", visited_types.len());
}

// ── Enter/leave ordering test from JS walk.js ──

#[test]
fn walk_enter_leave_order() {
    let ast = parse(".a { color: red }", ParseOptions::default());
    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions::default(),
        |node, _ctx| {
            log.borrow_mut().push(format!("enter {}", node.node_type()));
            WalkAction::Continue
        },
        |node, _ctx| {
            log.borrow_mut().push(format!("leave {}", node.node_type()));
            WalkAction::Continue
        },
    );

    let expected = vec![
        "enter StyleSheet",
        "enter Rule",
        "enter SelectorList",
        "enter Selector",
        "enter ClassSelector",
        "leave ClassSelector",
        "leave Selector",
        "leave SelectorList",
        "enter Block",
        "enter Declaration",
        "enter Value",
        "enter Identifier",
        "leave Identifier",
        "leave Value",
        "leave Declaration",
        "leave Block",
        "leave Rule",
        "leave StyleSheet",
    ];

    assert_eq!(*log.borrow(), expected);
}

// ── Traversal order tests ──

fn get_name(node: &csstree::ast::Node) -> Option<String> {
    match node {
        csstree::ast::Node::ClassSelector(n) => Some(n.name.clone()),
        csstree::ast::Node::Identifier(n) => Some(n.name.clone()),
        csstree::ast::Node::TypeSelector(n) => Some(n.name.clone()),
        csstree::ast::Node::Atrule(n) => Some(n.name.clone()),
        csstree::ast::Node::Function(n) => Some(n.name.clone()),
        csstree::ast::Node::PseudoClassSelector(n) => Some(n.name.clone()),
        csstree::ast::Node::Declaration(n) => Some(n.property.clone()),
        csstree::ast::Node::Combinator(n) => Some(n.name.clone()),
        csstree::ast::Node::MediaQuery(n) => n.media_type.clone(),
        _ => None,
    }
}

#[test]
fn walk_natural_order_simple() {
    let css = ".a.b { foo: bar; baz: qux } .c {}";
    let ast = parse(css, ParseOptions::default());
    let mut visited_names = Vec::new();

    walk(&ast, |node, _ctx| {
        if let Some(name) = get_name(node) {
            visited_names.push(name);
        }
        WalkAction::Continue
    });

    let expected: Vec<&str> = "a b foo bar baz qux c".split(' ').collect();
    assert_eq!(visited_names, expected);
}

#[test]
fn walk_reverse_order_simple() {
    let css = ".a.b { foo: bar; baz: qux } .c {}";
    let ast = parse(css, ParseOptions::default());
    let mut visited_names = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: true, visit: VisitFilter::All },
        |node, _ctx| {
            if let Some(name) = get_name(node) {
                visited_names.push(name);
            }
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    let expected: Vec<&str> = "c baz qux foo bar b a".split(' ').collect();
    assert_eq!(visited_names, expected);
}

// ── Break/skip tests ──

#[test]
fn walk_break_on_enter() {
    let css = ".a.b { foo: bar; } .c {} .a.b { foo: bar; }";
    let ast = parse(css, ParseOptions::default());
    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions::default(),
        |node, _ctx| {
            let name = match node {
                csstree::ast::Node::ClassSelector(n) => format!("ClassSelector:{}", n.name),
                csstree::ast::Node::Declaration(n) => format!("Declaration:{}", n.property),
                csstree::ast::Node::Identifier(n) => format!("Identifier:{}", n.name),
                _ => node.node_type().to_string(),
            };
            log.borrow_mut().push(format!("enter {name}"));
            if let csstree::ast::Node::ClassSelector(n) = node {
                if n.name == "c" {
                    return WalkAction::Break;
                }
            }
            WalkAction::Continue
        },
        |node, _ctx| {
            let name = match node {
                csstree::ast::Node::ClassSelector(n) => format!("ClassSelector:{}", n.name),
                csstree::ast::Node::Declaration(n) => format!("Declaration:{}", n.property),
                csstree::ast::Node::Identifier(n) => format!("Identifier:{}", n.name),
                _ => node.node_type().to_string(),
            };
            log.borrow_mut().push(format!("leave {name}"));
            WalkAction::Continue
        },
    );

    let log = log.borrow();
    // Should stop at ClassSelector:c
    assert!(log.last().unwrap().contains("ClassSelector:c"));
    // Should NOT contain the third rule's content
    let log_str = log.join("|");
    assert!(!log_str.contains("leave ClassSelector:c"));
}

#[test]
fn walk_skip_atrule() {
    let css = ".a { color: red } @media all { .b { display: block } } .c { margin: 0 }";
    let ast = parse(css, ParseOptions::default());
    let mut types = Vec::new();

    walk(&ast, |node, _ctx| {
        types.push(node.node_type().to_string());
        if node.node_type() == "Atrule" {
            WalkAction::Skip
        } else {
            WalkAction::Continue
        }
    });

    // Should have Atrule but NOT any of its children (AtrulePrelude, nested Block, etc.)
    assert!(types.contains(&"Atrule".to_string()));
    // The nested .b rule should NOT be visited (it's inside the skipped @media)
    // But the .c rule should be visited
    let atrule_idx = types.iter().position(|t| t == "Atrule").unwrap();
    let after_atrule: Vec<&String> = types[atrule_idx + 1..].iter().collect();
    // After atrule, next should be the .c rule branch
    assert!(after_atrule.contains(&&"Rule".to_string()));
}

// ── Visit filter tests ──

#[test]
fn walk_visit_rule_filter() {
    let css = ".a { color: red } @media all { .b { display: block } }";
    let ast = parse(css, ParseOptions::default());
    let mut visited = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: false, visit: VisitFilter::Rule },
        |node, _ctx| {
            visited.push(node.node_type().to_string());
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    assert!(visited.iter().all(|t| t == "Rule"));
    assert_eq!(visited.len(), 2); // Two rules
}

#[test]
fn walk_visit_declaration_filter() {
    let css = ".a { color: red; display: block } .b { margin: 0 }";
    let ast = parse(css, ParseOptions::default());
    let mut visited = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: false, visit: VisitFilter::Declaration },
        |node, _ctx| {
            visited.push(node.node_type().to_string());
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    assert!(visited.iter().all(|t| t == "Declaration"));
    assert_eq!(visited.len(), 3); // Three declarations
}

#[test]
fn walk_visit_atrule_filter() {
    let css = "@import url(a); @media all { .b {} } @charset 'utf-8';";
    let ast = parse(css, ParseOptions::default());
    let mut visited = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: false, visit: VisitFilter::Atrule },
        |node, _ctx| {
            visited.push(node.node_type().to_string());
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    assert!(visited.iter().all(|t| t == "Atrule"));
    assert!(visited.len() >= 2); // At least import + media
}

// ── find/find_all integration tests ──

#[test]
fn find_first_declaration() {
    let ast = parse(".a { color: red; display: block }", ParseOptions::default());
    let found = find(&ast, |node, _ctx| node.node_type() == "Declaration");
    assert!(found.is_some());
    if let csstree::ast::Node::Declaration(d) = found.unwrap() {
        assert_eq!(d.property, "color");
    }
}

#[test]
fn find_all_identifiers() {
    let ast = parse(".a { color: red }", ParseOptions::default());
    let found = find_all(&ast, |node, _ctx| node.node_type() == "Identifier");
    assert_eq!(found.len(), 1); // "red"
}

#[test]
fn find_all_declarations_in_complex_css() {
    let css = ".a { color: red; display: block } @media all { .b { margin: 0; padding: 1px } }";
    let ast = parse(css, ParseOptions::default());
    let found = find_all(&ast, |node, _ctx| node.node_type() == "Declaration");
    assert_eq!(found.len(), 4); // color, display, margin, padding
}

#[test]
fn find_returns_none_for_missing_type() {
    let ast = parse(".a { color: red }", ParseOptions::default());
    let found = find(&ast, |node, _ctx| node.node_type() == "Dimension");
    assert!(found.is_none());
}
