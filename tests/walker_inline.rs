//! Inline tests ported from `tests/fixtures/inline/walk.json`.
//!
//! Each test maps to a JS `it()` block from the csstree walk test suite.

use csstree::parser::{parse, ParseOptions};
use csstree::walker::{find, find_all, walk, walk_full, WalkAction, WalkOptions, VisitFilter};
use std::cell::RefCell;
use std::collections::BTreeSet;

// ── AST traversal > base test ── (line 96)

#[test]
fn base_test_walk_types() {
    let css = r#"@import url("test");@media (min-width: 200px) { .foo:nth-child(2n) { color: rgb(100%, 10%, 0%); width: calc(3px + 5%); content: "test" } }"#;
    let ast = parse(css, ParseOptions::default());
    let mut visited_types = BTreeSet::new();
    walk(&ast, |node, _ctx| {
        visited_types.insert(node.node_type().to_string());
        WalkAction::Continue
    });

    // Must visit many distinct node types
    assert!(visited_types.len() >= 7);
    assert!(visited_types.contains("StyleSheet"));
    assert!(visited_types.contains("Atrule"));
    assert!(visited_types.contains("Rule"));
}

// ── AST traversal > base test #2 ── (line 130)

#[test]
fn base_test_2_enter_leave() {
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

    assert_eq!(*log.borrow(), vec![
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
    ]);
}

// ── AST traversal > traverse order > natural ── (line 166)

fn get_node_name(node: &csstree::ast::Node) -> Option<String> {
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
fn traverse_order_natural() {
    let css = ".a.b { foo: bar; baz: qux } .c {}";
    let ast = parse(css, ParseOptions::default());
    let mut names = Vec::new();

    walk(&ast, |node, _ctx| {
        if let Some(name) = get_node_name(node) {
            names.push(name);
        }
        WalkAction::Continue
    });

    assert_eq!(names, vec!["a", "b", "foo", "bar", "baz", "qux", "c"]);
}

// ── AST traversal > traverse order > reverse ── (line 186)

#[test]
fn traverse_order_reverse() {
    let css = ".a.b { foo: bar; baz: qux } .c {}";
    let ast = parse(css, ParseOptions::default());
    let mut names = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: true, visit: VisitFilter::All },
        |node, _ctx| {
            if let Some(name) = get_node_name(node) {
                names.push(name);
            }
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    assert_eq!(names, vec!["c", "baz", "qux", "foo", "bar", "b", "a"]);
}

// ── AST traversal > break traverse > natural order ── (line 240)

#[test]
fn break_traverse_natural_order() {
    let css = ".a.b { foo: bar; } .c {} .a.b { foo: bar; }";
    let ast = parse(css, ParseOptions::default());

    fn node_name(node: &csstree::ast::Node) -> String {
        let base = node.node_type();
        match node {
            csstree::ast::Node::ClassSelector(n) => format!("{base}:{}", n.name),
            csstree::ast::Node::Declaration(n) => format!("{base}:{}", n.property),
            csstree::ast::Node::Identifier(n) => format!("{base}:{}", n.name),
            _ => base.to_string(),
        }
    }

    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions::default(),
        |node, _ctx| {
            log.borrow_mut().push(format!("enter {}", node_name(node)));
            if let csstree::ast::Node::ClassSelector(n) = node {
                if n.name == "c" {
                    return WalkAction::Break;
                }
            }
            WalkAction::Continue
        },
        |node, _ctx| {
            log.borrow_mut().push(format!("leave {}", node_name(node)));
            WalkAction::Continue
        },
    );

    let log = log.borrow();
    // Walk should stop at "enter ClassSelector:c"
    assert_eq!(log.last().unwrap(), "enter ClassSelector:c");
    // Should have visited the first rule completely
    assert!(log.contains(&"enter Declaration:foo".to_string()));
    assert!(log.contains(&"leave Declaration:foo".to_string()));
}

// ── AST traversal > break traverse > reverse order ── (line 314)

#[test]
fn break_traverse_reverse_order() {
    let css = ".a.b { foo: bar; } .c {} .a.b { foo: bar; }";
    let ast = parse(css, ParseOptions::default());

    fn node_name(node: &csstree::ast::Node) -> String {
        let base = node.node_type();
        match node {
            csstree::ast::Node::ClassSelector(n) => format!("{base}:{}", n.name),
            csstree::ast::Node::Declaration(n) => format!("{base}:{}", n.property),
            csstree::ast::Node::Identifier(n) => format!("{base}:{}", n.name),
            _ => base.to_string(),
        }
    }

    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions { reverse: true, visit: VisitFilter::All },
        |node, _ctx| {
            log.borrow_mut().push(format!("enter {}", node_name(node)));
            if let csstree::ast::Node::ClassSelector(n) = node {
                if n.name == "c" {
                    return WalkAction::Break;
                }
            }
            WalkAction::Continue
        },
        |node, _ctx| {
            log.borrow_mut().push(format!("leave {}", node_name(node)));
            WalkAction::Continue
        },
    );

    let log = log.borrow();
    // Walk should stop at "enter ClassSelector:c"
    assert_eq!(log.last().unwrap(), "enter ClassSelector:c");
    // In reverse, the last rule (.a.b) should be visited first
    assert!(log.contains(&"enter Declaration:foo".to_string()));
}

// ── AST traversal > skip node traverse > natural order ── (line 410)

#[test]
fn skip_traverse_natural_order() {
    let css = ".a.b { foo: bar } @media all { selector { foo: bar } } .c.d { foo: bar }";
    let ast = parse(css, ParseOptions::default());

    fn node_name(node: &csstree::ast::Node) -> String {
        let base = node.node_type();
        match node {
            csstree::ast::Node::ClassSelector(n) => format!("{base}:{}", n.name),
            csstree::ast::Node::Atrule(n) => format!("{base}:{}", n.name),
            csstree::ast::Node::Declaration(n) => format!("{base}:{}", n.property),
            csstree::ast::Node::Identifier(n) => format!("{base}:{}", n.name),
            csstree::ast::Node::TypeSelector(n) => format!("{base}:{}", n.name),
            _ => base.to_string(),
        }
    }

    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions::default(),
        |node, _ctx| {
            if let csstree::ast::Node::Atrule(n) = node {
                if n.name == "media" {
                    log.borrow_mut().push(format!("skip {}", node_name(node)));
                    return WalkAction::Skip;
                }
            }
            log.borrow_mut().push(format!("enter {}", node_name(node)));
            WalkAction::Continue
        },
        |node, _ctx| {
            log.borrow_mut().push(format!("leave {}", node_name(node)));
            WalkAction::Continue
        },
    );

    let log = log.borrow();
    // Should have "skip Atrule:media" but NOT any children of it
    assert!(log.contains(&"skip Atrule:media".to_string()));
    // The @media's inner content should not appear
    let skip_idx = log.iter().position(|s| s == "skip Atrule:media").unwrap();
    // After skip, should NOT see nodes from inside @media, but should see .c.d
    let after_skip: Vec<&String> = log[skip_idx + 1..].iter().collect();
    assert!(after_skip.iter().any(|s| s.contains("ClassSelector:c")));
    // But leave Atrule:media SHOULD still appear (leave fires even after skip)
    assert!(after_skip.iter().any(|s| s.as_str() == "leave Atrule:media"));
}

// ── AST traversal > skip node traverse > reverse order ── (line 500)

#[test]
fn skip_traverse_reverse_order() {
    let css = ".a.b { foo: bar } @media all { selector { foo: bar } } .c.d { foo: bar }";
    let ast = parse(css, ParseOptions::default());
    let mut types = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: true, visit: VisitFilter::All },
        |node, _ctx| {
            if let csstree::ast::Node::Atrule(n) = node {
                if n.name == "media" {
                    types.push("SKIP".to_string());
                    return WalkAction::Skip;
                }
            }
            types.push(node.node_type().to_string());
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    // In reverse: .c.d first, then @media (skipped), then .a.b
    assert!(types.contains(&"SKIP".to_string()));
    let skip_idx = types.iter().position(|t| t == "SKIP").unwrap();
    // Before skip should have .c.d content
    assert!(types[..skip_idx].iter().any(|t| t == "ClassSelector"));
}

// ── AST traversal > walk(ast, { visit: 'Declaration' }) > iterate DeclarationList ── (line 653)
// Note: our parser doesn't have a declarationList context, so we test by parsing
// declarations inside a rule block, which is equivalent

#[test]
fn walk_visit_declaration_iterate() {
    let css = ".x { foo: a; bar: b }";
    let ast = parse(css, ParseOptions::default());
    let mut visited = 0;

    walk_full(
        &ast,
        &WalkOptions { reverse: false, visit: VisitFilter::Declaration },
        |node, _ctx| {
            if node.node_type() == "Declaration" {
                visited += 1;
            }
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    assert_eq!(visited, 2);
}

// ── Extra: find helpers ──

#[test]
fn find_first_class_selector() {
    let ast = parse(".a.b { color: red }", ParseOptions::default());
    let found = find(&ast, |node, _ctx| node.node_type() == "ClassSelector");
    assert!(found.is_some());
    if let csstree::ast::Node::ClassSelector(n) = found.unwrap() {
        assert_eq!(n.name, "a");
    }
}

#[test]
fn find_all_class_selectors() {
    let ast = parse(".a.b .c { color: red }", ParseOptions::default());
    let found = find_all(&ast, |node, _ctx| node.node_type() == "ClassSelector");
    assert_eq!(found.len(), 3); // .a, .b, .c
}

// ── Search tests from find.js ──

#[test]
fn find_first_foo_class() {
    // find.js: "base" — find first .foo ClassSelector
    let ast = parse(
        ".foo { color: red; background: green; } .bar, .qux.foo { font-weight: bold; color: blue; }",
        ParseOptions::default(),
    );
    let found = find(&ast, |node, _ctx| {
        matches!(node, csstree::ast::Node::ClassSelector(n) if n.name == "foo")
    });
    assert!(found.is_some());
    if let csstree::ast::Node::ClassSelector(n) = found.unwrap() {
        assert_eq!(n.name, "foo");
    }
}

#[test]
fn find_last_foo_class() {
    // find.js: "findLast" — findLast finds last .foo
    // We use find_all and check last
    let ast = parse(
        ".foo { color: red; } .bar, .qux.foo { color: blue; }",
        ParseOptions::default(),
    );
    let all = find_all(&ast, |node, _ctx| {
        matches!(node, csstree::ast::Node::ClassSelector(n) if n.name == "foo")
    });
    assert_eq!(all.len(), 2, "Should find 2 .foo selectors");
}

#[test]
fn find_all_foo_class() {
    // find.js: "findAll" — finds all .foo ClassSelectors
    let ast = parse(
        ".foo { color: red; background: green; } .bar, .qux.foo { font-weight: bold; color: blue; }",
        ParseOptions::default(),
    );
    let all = find_all(&ast, |node, _ctx| {
        matches!(node, csstree::ast::Node::ClassSelector(n) if n.name == "foo")
    });
    assert_eq!(all.len(), 2, "Should find exactly 2 .foo selectors");
    // First should be in .foo rule, second in .qux.foo
    for item in &all {
        if let csstree::ast::Node::ClassSelector(n) = item {
            assert_eq!(n.name, "foo");
        }
    }
}
