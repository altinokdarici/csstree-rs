//! Extra inline tests ported from `external/csstree/lib/__tests/walk.js`.
//!
//! These cover walk.break, walk.skip, bad-options validation equivalents,
//! and `DeclarationList` iteration via the `declarationList` parse context.

use csstree::ast::Node;
use csstree::parser::{parse, ParseOptions, ParseContext};
use csstree::walker::{walk, walk_full, WalkAction, WalkOptions, VisitFilter};
use std::cell::RefCell;

fn node_name(node: &Node) -> String {
    let base = node.node_type();
    match node {
        Node::ClassSelector(n) => format!("{base}:{}", n.name),
        Node::Declaration(n) => format!("{base}:{}", n.property),
        Node::Identifier(n) => format!("{base}:{}", n.name),
        Node::Atrule(n) => format!("{base}:{}", n.name),
        Node::TypeSelector(n) => format!("{base}:{}", n.name),
        _ => base.to_string(),
    }
}

// ── walk.break in natural order (line 262) ──
// This is the walk.break variant (arrow function returning WalkAction::Break).
// The existing test uses enter(node) { return this.break }, this test uses the
// same WalkAction::Break but with an enter-only walk pattern.

#[test]
fn walk_break_natural_order_enter_only() {
    let css = ".a.b { foo: bar; } .c {} .a.b { foo: bar; }";
    let ast = parse(css, ParseOptions::default());
    let mut actual = Vec::new();

    walk(&ast, |node, _ctx| {
        actual.push(format!("enter {}", node_name(node)));
        if let Node::ClassSelector(n) = node {
            if n.name == "c" {
                return WalkAction::Break;
            }
        }
        WalkAction::Continue
    });

    // Walk should stop at "enter ClassSelector:c"
    assert_eq!(actual.last().unwrap(), "enter ClassSelector:c");
    // Should have visited the first rule completely
    assert!(actual.contains(&"enter Declaration:foo".to_string()));
}

// ── walk.break in reverse order (line 285) ──

#[test]
fn walk_break_reverse_order_enter_only() {
    let css = ".a.b { foo: bar; } .c {} .a.b { foo: bar; }";
    let ast = parse(css, ParseOptions::default());
    let mut actual = Vec::new();

    walk_full(
        &ast,
        &WalkOptions { reverse: true, visit: VisitFilter::All },
        |node, _ctx| {
            actual.push(format!("enter {}", node_name(node)));
            if let Node::ClassSelector(n) = node {
                if n.name == "c" {
                    return WalkAction::Break;
                }
            }
            WalkAction::Continue
        },
        |_, _| WalkAction::Continue,
    );

    // Walk should stop at "enter ClassSelector:c"
    assert_eq!(actual.last().unwrap(), "enter ClassSelector:c");
    // In reverse, the last rule should be visited first
    assert!(actual.contains(&"enter Declaration:foo".to_string()));
}

// ── walk.skip in natural order (line 370/433) ──
// Tests WalkAction::Skip with enter+leave pattern (walk.skip variant)

#[test]
fn walk_skip_natural_order_with_leave() {
    let css = ".a.b { foo: bar } @media all { selector { foo: bar } } .c.d { foo: bar }";
    let ast = parse(css, ParseOptions::default());
    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions::default(),
        |node, _ctx| {
            if let Node::Atrule(n) = node {
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
    // Should have "skip Atrule:media"
    assert!(log.contains(&"skip Atrule:media".to_string()));
    // After skip, should see .c.d content
    let skip_idx = log.iter().position(|s| s == "skip Atrule:media").unwrap();
    let after_skip: Vec<&String> = log[skip_idx + 1..].iter().collect();
    assert!(after_skip.iter().any(|s| s.contains("ClassSelector:c")));
    // Leave fires for the skipped node
    assert!(after_skip.iter().any(|s| s.as_str() == "leave Atrule:media"));
    // But no children of @media should appear
    assert!(!log.iter().any(|s| s.contains("TypeSelector:selector")));
}

// ── walk.skip in reverse order (line 395/524) ──

#[test]
fn walk_skip_reverse_order_with_leave() {
    let css = ".a.b { foo: bar } @media all { selector { foo: bar } } .c.d { foo: bar }";
    let ast = parse(css, ParseOptions::default());
    let log = RefCell::new(Vec::new());

    walk_full(
        &ast,
        &WalkOptions { reverse: true, visit: VisitFilter::All },
        |node, _ctx| {
            if let Node::Atrule(n) = node {
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
    // Should have "skip Atrule:media"
    assert!(log.contains(&"skip Atrule:media".to_string()));
    // In reverse, .c.d is visited before @media
    let skip_idx = log.iter().position(|s| s == "skip Atrule:media").unwrap();
    // Before skip should have .c.d content (reverse visits last rule first)
    assert!(log[..skip_idx].iter().any(|s| s.contains("ClassSelector")));
    // After skip should have .a.b content
    let after_skip: Vec<&String> = log[skip_idx + 1..].iter().collect();
    assert!(after_skip.iter().any(|s| s.contains("ClassSelector:a") || s.contains("ClassSelector:b")));
}

// ── bad options: valid handler verification (line 432/439) ──
// In Rust, the type system prevents bad handlers at compile time.
// Instead, test that valid handlers work correctly.

#[test]
fn valid_enter_handler_works() {
    let ast = parse(".a { color: red }", ParseOptions::default());
    let mut count = 0;
    walk(&ast, |_node, _ctx| {
        count += 1;
        WalkAction::Continue
    });
    assert!(count > 0, "Walk with a valid enter handler should visit nodes");
}

#[test]
fn valid_visit_filter_works() {
    let ast = parse(".a { color: red }", ParseOptions::default());
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
    // Only Declaration should be visited via the callback
    assert_eq!(visited, vec!["Declaration"]);
}

// ── iterate DeclarationList (line 653) ──
// Parse with declarationList context, walk with Declaration visit filter

#[test]
fn iterate_declaration_list_context() {
    let ast = parse("foo: a; bar: b", ParseOptions {
        context: ParseContext::DeclarationList,
        ..ParseOptions::default()
    });

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

    assert_eq!(visited, 2, "Should find 2 declarations in declarationList");
}
