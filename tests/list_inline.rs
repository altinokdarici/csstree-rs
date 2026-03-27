//! List tests — Vec<Node> operations on AST children.
//!
//! Ported from: `external/csstree/lib/__tests/list.js`
//!
//! The JS csstree uses a custom doubly-linked List for children.
//! In Rust, children are `Vec<Node>`. These tests verify that all
//! conceptual List operations work correctly on Vec<Node>.

#![allow(
    clippy::useless_vec,
    clippy::vec_init_then_push,
    clippy::implicit_clone,
    clippy::redundant_closure_for_method_calls,
    clippy::map_clone,
    clippy::drain_collect,
    clippy::unnecessary_first_then_check
)]

use csstree::ast::*;
use csstree::parser::{parse, ParseOptions};

// ── Helper functions ──

fn make_ident(name: &str) -> Node {
    Node::Identifier(Identifier {
        loc: None,
        name: name.into(),
    })
}

fn make_number(val: &str) -> Node {
    Node::Number(Number {
        loc: None,
        value: val.into(),
    })
}

fn make_whitespace(val: &str) -> Node {
    Node::WhiteSpace(WhiteSpace {
        loc: None,
        value: val.into(),
    })
}

fn get_stylesheet_children(css: &str) -> Vec<Node> {
    let ast = parse(css, ParseOptions::default());
    match ast {
        Node::StyleSheet(ss) => ss.children,
        _ => panic!("expected StyleSheet"),
    }
}

fn names_of(nodes: &[Node]) -> Vec<String> {
    nodes
        .iter()
        .map(|n| match n {
            Node::Identifier(id) => id.name.clone(),
            Node::Number(num) => num.value.clone(),
            _ => n.node_type().to_string(),
        })
        .collect()
}

// ══════════════════════════════════════════════════════════════
// Size / isEmpty / first / last
// ══════════════════════════════════════════════════════════════

#[test]
fn size_empty_vec() {
    let children: Vec<Node> = vec![];
    assert_eq!(children.len(), 0);
}

#[test]
fn size_one_element() {
    let children = vec![make_ident("foo")];
    assert_eq!(children.len(), 1);
}

#[test]
fn size_two_elements() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    assert_eq!(children.len(), 2);
}

#[test]
fn is_empty_true() {
    let children: Vec<Node> = vec![];
    assert!(children.is_empty());
}

#[test]
fn is_empty_false_one() {
    let children = vec![make_ident("foo")];
    assert!(!children.is_empty());
}

#[test]
fn is_empty_false_two() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    assert!(!children.is_empty());
}

#[test]
fn first_empty() {
    let children: Vec<Node> = vec![];
    assert!(children.first().is_none());
}

#[test]
fn first_one_element() {
    let children = vec![make_ident("foo")];
    assert_eq!(children.first().unwrap().node_type(), "Identifier");
}

#[test]
fn first_two_elements() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    match children.first().unwrap() {
        Node::Identifier(id) => assert_eq!(id.name, "foo"),
        _ => panic!("expected Identifier"),
    }
}

#[test]
fn last_empty() {
    let children: Vec<Node> = vec![];
    assert!(children.last().is_none());
}

#[test]
fn last_one_element() {
    let children = vec![make_ident("foo")];
    match children.last().unwrap() {
        Node::Identifier(id) => assert_eq!(id.name, "foo"),
        _ => panic!("expected Identifier"),
    }
}

#[test]
fn last_two_elements() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    match children.last().unwrap() {
        Node::Identifier(id) => assert_eq!(id.name, "bar"),
        _ => panic!("expected Identifier"),
    }
}

// ══════════════════════════════════════════════════════════════
// from_array / to_array (Vec::from / to_vec / clone)
// ══════════════════════════════════════════════════════════════

#[test]
fn from_array_empty() {
    let arr: Vec<Node> = Vec::from([]);
    assert!(arr.is_empty());
}

#[test]
fn from_array_two_elements() {
    let arr = Vec::from([make_ident("foo"), make_ident("bar")]);
    assert_eq!(arr.len(), 2);
    match &arr[0] {
        Node::Identifier(id) => assert_eq!(id.name, "foo"),
        _ => panic!("expected Identifier"),
    }
    match &arr[1] {
        Node::Identifier(id) => assert_eq!(id.name, "bar"),
        _ => panic!("expected Identifier"),
    }
}

#[test]
fn to_vec_empty() {
    let children: Vec<Node> = vec![];
    let copy = children.to_vec();
    assert!(copy.is_empty());
}

#[test]
fn to_vec_preserves_elements() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let copy = children.to_vec();
    assert_eq!(names_of(&copy), vec!["foo", "bar"]);
}

#[test]
fn to_json_equivalent() {
    // In JS, toJSON() returns an array. In Rust, Vec is already serializable.
    let children = vec![make_ident("foo"), make_ident("bar")];
    assert_eq!(children.len(), 2);
    assert_eq!(names_of(&children), vec!["foo", "bar"]);
}

// ══════════════════════════════════════════════════════════════
// Iteration (forEach equivalent)
// ══════════════════════════════════════════════════════════════

#[test]
fn iterate_forward() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let mut collected: Vec<String> = Vec::new();
    for node in &children {
        if let Node::Identifier(id) = node {
            collected.push(id.name.clone());
        }
    }
    assert_eq!(collected, vec!["foo", "bar"]);
}

#[test]
fn iterate_reverse() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let mut collected: Vec<String> = Vec::new();
    for node in children.iter().rev() {
        if let Node::Identifier(id) = node {
            collected.push(id.name.clone());
        }
    }
    assert_eq!(collected, vec!["bar", "foo"]);
}

#[test]
fn nested_iterate() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let mut outer_count = 0;
    for _outer in &children {
        for _inner in &children {
            outer_count += 1;
        }
    }
    // 2 * 2 = 4
    assert_eq!(outer_count, 4);
}

// ══════════════════════════════════════════════════════════════
// reduce / reduceRight
// ══════════════════════════════════════════════════════════════

#[test]
fn reduce_concatenate() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let result = children.iter().fold(String::new(), |acc, node| {
        if let Node::Identifier(id) = node {
            acc + &id.name
        } else {
            acc
        }
    });
    assert_eq!(result, "foobar");
}

#[test]
fn reduce_right_concatenate() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let result = children.iter().rev().fold(String::new(), |acc, node| {
        if let Node::Identifier(id) = node {
            acc + &id.name
        } else {
            acc
        }
    });
    assert_eq!(result, "barfoo");
}

// ══════════════════════════════════════════════════════════════
// some / any
// ══════════════════════════════════════════════════════════════

#[test]
fn some_basic_true() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    assert!(children.iter().any(|n| n.node_type() == "Identifier"));
}

#[test]
fn some_basic_false() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    assert!(!children.iter().any(|n| n.node_type() == "Number"));
}

#[test]
fn some_specific_match() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let found = children.iter().any(|n| {
        matches!(n, Node::Identifier(id) if id.name == "bar")
    });
    assert!(found);
}

// ══════════════════════════════════════════════════════════════
// map
// ══════════════════════════════════════════════════════════════

#[test]
fn map_empty() {
    let children: Vec<Node> = vec![];
    let mapped: Vec<Node> = children.iter().map(|n| n.clone()).collect();
    assert!(mapped.is_empty());
}

#[test]
fn map_transform() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let mapped: Vec<Node> = children
        .iter()
        .map(|n| match n {
            Node::Identifier(id) => make_ident(&id.name.to_uppercase()),
            other => other.clone(),
        })
        .collect();
    assert_eq!(names_of(&mapped), vec!["FOO", "BAR"]);
    // Original unchanged
    assert_eq!(names_of(&children), vec!["foo", "bar"]);
}

// ══════════════════════════════════════════════════════════════
// filter
// ══════════════════════════════════════════════════════════════

#[test]
fn filter_empty() {
    let children: Vec<Node> = vec![];
    let filtered: Vec<&Node> = children.iter().filter(|_| true).collect();
    assert!(filtered.is_empty());
}

#[test]
fn filter_all() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let filtered: Vec<&Node> = children
        .iter()
        .filter(|n| n.node_type() == "Identifier")
        .collect();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn filter_specific() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let filtered: Vec<&Node> = children
        .iter()
        .filter(|n| matches!(n, Node::Identifier(id) if id.name == "bar"))
        .collect();
    assert_eq!(filtered.len(), 1);
    // Original unchanged
    assert_eq!(children.len(), 2);
}

// ══════════════════════════════════════════════════════════════
// clear
// ══════════════════════════════════════════════════════════════

#[test]
fn clear_empty_list() {
    let mut children: Vec<Node> = vec![];
    assert!(children.is_empty());
    children.clear();
    assert!(children.is_empty());
}

#[test]
fn clear_non_empty_list() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    assert!(!children.is_empty());
    children.clear();
    assert!(children.is_empty());
}

// ══════════════════════════════════════════════════════════════
// copy / clone
// ══════════════════════════════════════════════════════════════

#[test]
fn clone_creates_independent_copy() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let copy = children.clone();

    // Same data
    assert_eq!(names_of(&copy), names_of(&children));
    // Independent (different allocations)
    assert_eq!(copy.len(), children.len());
}

#[test]
fn clone_modify_does_not_affect_original() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let mut copy = children.clone();
    copy.push(make_ident("baz"));

    assert_eq!(children.len(), 2);
    assert_eq!(copy.len(), 3);
}

// ══════════════════════════════════════════════════════════════
// prepend (insert at 0)
// ══════════════════════════════════════════════════════════════

#[test]
fn prepend_to_empty() {
    let mut children: Vec<Node> = vec![];
    children.insert(0, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux"]);
}

#[test]
fn prepend_to_one() {
    let mut children = vec![make_ident("foo")];
    children.insert(0, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux", "foo"]);
}

#[test]
fn prepend_to_two() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.insert(0, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux", "foo", "bar"]);
}

// ══════════════════════════════════════════════════════════════
// append (push)
// ══════════════════════════════════════════════════════════════

#[test]
fn append_to_empty() {
    let mut children: Vec<Node> = vec![];
    children.push(make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux"]);
}

#[test]
fn append_to_one() {
    let mut children = vec![make_ident("foo")];
    children.push(make_ident("qux"));
    assert_eq!(names_of(&children), vec!["foo", "qux"]);
}

#[test]
fn append_to_two() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.push(make_ident("qux"));
    assert_eq!(names_of(&children), vec!["foo", "bar", "qux"]);
}

// ══════════════════════════════════════════════════════════════
// push / pop
// ══════════════════════════════════════════════════════════════

#[test]
fn push_pop_basic() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    let tail = children.pop().unwrap();
    match &tail {
        Node::Identifier(id) => assert_eq!(id.name, "bar"),
        _ => panic!("expected Identifier"),
    }
    let head = children.pop().unwrap();
    match &head {
        Node::Identifier(id) => assert_eq!(id.name, "foo"),
        _ => panic!("expected Identifier"),
    }
    assert!(children.is_empty());
}

#[test]
fn pop_empty_returns_none() {
    let mut children: Vec<Node> = vec![];
    assert!(children.pop().is_none());
}

// ══════════════════════════════════════════════════════════════
// shift (remove first) / unshift (insert at 0)
// ══════════════════════════════════════════════════════════════

#[test]
fn shift_removes_first() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    let head = children.remove(0);
    match &head {
        Node::Identifier(id) => assert_eq!(id.name, "foo"),
        _ => panic!("expected Identifier"),
    }
    let tail = children.remove(0);
    match &tail {
        Node::Identifier(id) => assert_eq!(id.name, "bar"),
        _ => panic!("expected Identifier"),
    }
    assert!(children.is_empty());
}

#[test]
fn unshift_inserts_at_front() {
    let mut children: Vec<Node> = vec![];
    children.insert(0, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux"]);

    let mut children = vec![make_ident("foo")];
    children.insert(0, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux", "foo"]);

    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.insert(0, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["qux", "foo", "bar"]);
}

// ══════════════════════════════════════════════════════════════
// insert (before ref item)
// ══════════════════════════════════════════════════════════════

#[test]
fn insert_append_when_no_ref() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.push(make_ident("after_tail"));
    assert_eq!(names_of(&children).last().unwrap(), "after_tail");
}

#[test]
fn insert_before_head() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.insert(0, make_ident("before_head"));
    assert_eq!(names_of(&children).first().unwrap(), "before_head");
}

#[test]
fn insert_in_middle() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.insert(1, make_ident("qux"));
    assert_eq!(names_of(&children), vec!["foo", "qux", "bar"]);
}

// ══════════════════════════════════════════════════════════════
// remove
// ══════════════════════════════════════════════════════════════

#[test]
fn remove_clears_list() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.remove(1); // remove bar
    children.remove(0); // remove foo
    assert!(children.is_empty());
}

#[test]
fn remove_in_reverse_order() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.remove(0); // remove foo
    children.remove(0); // remove bar (now at index 0)
    assert!(children.is_empty());
}

#[test]
fn remove_by_retain() {
    let mut children = vec![make_ident("foo"), make_ident("bar"), make_ident("baz")];
    children.retain(|n| !matches!(n, Node::Identifier(id) if id.name == "bar"));
    assert_eq!(names_of(&children), vec!["foo", "baz"]);
}

// ══════════════════════════════════════════════════════════════
// replace
// ══════════════════════════════════════════════════════════════

#[test]
fn replace_item() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children[1] = make_ident("qux");
    assert_eq!(names_of(&children), vec!["foo", "qux"]);
}

#[test]
fn replace_with_splice() {
    // Replace one item with multiple (like JS List.replace with a list)
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    let replacement = vec![make_ident("baz"), make_ident("qux")];
    children.splice(1..2, replacement);
    assert_eq!(names_of(&children), vec!["foo", "baz", "qux"]);
}

// ══════════════════════════════════════════════════════════════
// prependList / appendList / insertList
// ══════════════════════════════════════════════════════════════

#[test]
fn prepend_list_to_non_empty() {
    let mut list2 = vec![make_ident("foo"), make_ident("bar")];
    let mut list1 = vec![make_ident("baz")];
    let mut combined = Vec::new();
    combined.append(&mut list1);
    combined.append(&mut list2);
    list2 = combined;
    assert_eq!(names_of(&list2), vec!["baz", "foo", "bar"]);
    assert!(list1.is_empty());
}

#[test]
fn prepend_list_to_empty() {
    let mut target: Vec<Node> = vec![];
    let mut source = vec![make_ident("foo"), make_ident("bar")];
    target.append(&mut source);
    assert_eq!(names_of(&target), vec!["foo", "bar"]);
    assert!(source.is_empty());
}

#[test]
fn prepend_empty_list_to_non_empty() {
    let mut list2 = vec![make_ident("foo"), make_ident("bar")];
    let mut empty: Vec<Node> = vec![];
    let mut combined = Vec::new();
    combined.append(&mut empty);
    combined.append(&mut list2);
    list2 = combined;
    assert_eq!(names_of(&list2), vec!["foo", "bar"]);
}

#[test]
fn append_list_to_non_empty() {
    let mut list2 = vec![make_ident("foo"), make_ident("bar")];
    let mut list1 = vec![make_ident("baz")];
    list2.append(&mut list1);
    assert_eq!(names_of(&list2), vec!["foo", "bar", "baz"]);
    assert!(list1.is_empty());
}

#[test]
fn append_list_to_empty() {
    let mut target: Vec<Node> = vec![];
    let mut source = vec![make_ident("foo"), make_ident("bar")];
    target.append(&mut source);
    assert_eq!(names_of(&target), vec!["foo", "bar"]);
    assert!(source.is_empty());
}

#[test]
fn append_empty_list_to_non_empty() {
    let mut list2 = vec![make_ident("foo"), make_ident("bar")];
    let mut empty: Vec<Node> = vec![];
    list2.append(&mut empty);
    assert_eq!(names_of(&list2), vec!["foo", "bar"]);
}

#[test]
fn insert_list_in_middle() {
    let mut list2 = vec![make_ident("foo"), make_ident("bar")];
    let insertion = vec![make_ident("baz")];
    // Insert before index 1 (before "bar")
    list2.splice(1..1, insertion);
    assert_eq!(names_of(&list2), vec!["foo", "baz", "bar"]);
}

#[test]
fn insert_list_to_empty() {
    let mut target: Vec<Node> = vec![];
    let source = vec![make_ident("foo"), make_ident("bar")];
    target.splice(0..0, source);
    assert_eq!(names_of(&target), vec!["foo", "bar"]);
}

#[test]
fn insert_empty_list_to_non_empty() {
    let mut list2 = vec![make_ident("foo"), make_ident("bar")];
    let empty: Vec<Node> = vec![];
    list2.splice(1..1, empty);
    assert_eq!(names_of(&list2), vec!["foo", "bar"]);
}

// ══════════════════════════════════════════════════════════════
// find / position on Vec<Node>
// ══════════════════════════════════════════════════════════════

#[test]
fn find_on_children() {
    let children = vec![make_ident("foo"), make_number("42"), make_ident("bar")];
    let found = children.iter().find(|n| n.node_type() == "Number");
    assert!(found.is_some());
    match found.unwrap() {
        Node::Number(num) => assert_eq!(num.value, "42"),
        _ => panic!("expected Number"),
    }
}

#[test]
fn find_position() {
    let children = vec![make_ident("foo"), make_number("42"), make_ident("bar")];
    let pos = children
        .iter()
        .position(|n| n.node_type() == "Number");
    assert_eq!(pos, Some(1));
}

#[test]
fn find_not_found() {
    let children = vec![make_ident("foo"), make_ident("bar")];
    let found = children.iter().find(|n| n.node_type() == "Number");
    assert!(found.is_none());
}

// ══════════════════════════════════════════════════════════════
// Parsed AST children operations
// ══════════════════════════════════════════════════════════════

#[test]
fn parsed_stylesheet_has_children() {
    let children = get_stylesheet_children("a {} b {}");
    assert_eq!(children.len(), 2);
    assert!(children.iter().all(|n| n.node_type() == "Rule"));
}

#[test]
fn parsed_children_iterate() {
    let children = get_stylesheet_children(".a { color: red } .b { display: block }");
    let types: Vec<&str> = children.iter().map(|n| n.node_type()).collect();
    assert_eq!(types, vec!["Rule", "Rule"]);
}

#[test]
fn parsed_children_clone_and_modify() {
    let children = get_stylesheet_children("a {} b {}");
    let mut copy = children.clone();
    copy.push(Node::Rule(Rule {
        loc: None,
        prelude: Box::new(Node::SelectorList(SelectorList {
            loc: None,
            children: vec![Node::Selector(Selector {
                loc: None,
                children: vec![Node::TypeSelector(TypeSelector {
                    loc: None,
                    name: "c".into(),
                })],
            })],
        })),
        block: Box::new(Node::Block(Block {
            loc: None,
            children: vec![],
        })),
    }));
    assert_eq!(children.len(), 2);
    assert_eq!(copy.len(), 3);
}

#[test]
fn parsed_children_filter() {
    let ast = parse(
        ".a { color: red; display: block }",
        ParseOptions::default(),
    );
    // Walk to get the Block's children (declarations)
    let block_children = match &ast {
        Node::StyleSheet(ss) => match &ss.children[0] {
            Node::Rule(rule) => match rule.block.as_ref() {
                Node::Block(block) => &block.children,
                _ => panic!("expected Block"),
            },
            _ => panic!("expected Rule"),
        },
        _ => panic!("expected StyleSheet"),
    };
    let declarations: Vec<&Node> = block_children
        .iter()
        .filter(|n| n.node_type() == "Declaration")
        .collect();
    assert_eq!(declarations.len(), 2);
}

#[test]
fn parsed_children_remove_by_retain() {
    let ast = parse(
        ".a { color: red; display: block; margin: 0 }",
        ParseOptions::default(),
    );
    let mut block_children = match ast {
        Node::StyleSheet(ss) => match ss.children.into_iter().next().unwrap() {
            Node::Rule(rule) => match *rule.block {
                Node::Block(block) => block.children,
                _ => panic!("expected Block"),
            },
            _ => panic!("expected Rule"),
        },
        _ => panic!("expected StyleSheet"),
    };
    assert_eq!(block_children.len(), 3);
    // Remove the "display" declaration
    block_children.retain(|n| {
        !matches!(n, Node::Declaration(d) if d.property == "display")
    });
    assert_eq!(block_children.len(), 2);
}

// ══════════════════════════════════════════════════════════════
// Mixed type children operations
// ══════════════════════════════════════════════════════════════

#[test]
fn mixed_type_children() {
    let children: Vec<Node> = vec![
        make_ident("red"),
        make_whitespace(" "),
        make_number("42"),
    ];
    let types: Vec<&str> = children.iter().map(|n| n.node_type()).collect();
    assert_eq!(types, vec!["Identifier", "WhiteSpace", "Number"]);
}

#[test]
fn remove_by_type() {
    let mut children: Vec<Node> = vec![
        make_ident("red"),
        make_whitespace(" "),
        make_number("42"),
        make_whitespace(" "),
        make_ident("blue"),
    ];
    children.retain(|n| n.node_type() != "WhiteSpace");
    assert_eq!(children.len(), 3);
    let types: Vec<&str> = children.iter().map(|n| n.node_type()).collect();
    assert_eq!(types, vec!["Identifier", "Number", "Identifier"]);
}

// ══════════════════════════════════════════════════════════════
// Iterator chaining and functional operations
// ══════════════════════════════════════════════════════════════

#[test]
fn enumerate_children() {
    let children = vec![make_ident("a"), make_ident("b"), make_ident("c")];
    let indexed: Vec<(usize, String)> = children
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let name = match n {
                Node::Identifier(id) => id.name.clone(),
                _ => String::new(),
            };
            (i, name)
        })
        .collect();
    assert_eq!(
        indexed,
        vec![
            (0, "a".to_string()),
            (1, "b".to_string()),
            (2, "c".to_string()),
        ]
    );
}

#[test]
fn chain_two_vecs() {
    let a = vec![make_ident("foo")];
    let b = vec![make_ident("bar"), make_ident("baz")];
    let chained: Vec<&Node> = a.iter().chain(b.iter()).collect();
    assert_eq!(chained.len(), 3);
}

#[test]
fn take_and_skip() {
    let children = vec![
        make_ident("a"),
        make_ident("b"),
        make_ident("c"),
        make_ident("d"),
    ];
    let first_two: Vec<&Node> = children.iter().take(2).collect();
    assert_eq!(first_two.len(), 2);
    let skip_two: Vec<&Node> = children.iter().skip(2).collect();
    assert_eq!(skip_two.len(), 2);
}

#[test]
fn windows_pairs() {
    let children = vec![make_ident("a"), make_ident("b"), make_ident("c")];
    let pairs: Vec<(&Node, &Node)> = children.windows(2).map(|w| (&w[0], &w[1])).collect();
    assert_eq!(pairs.len(), 2);
}

// ══════════════════════════════════════════════════════════════
// Drain / splice advanced operations
// ══════════════════════════════════════════════════════════════

#[test]
fn drain_all() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    let drained: Vec<Node> = children.drain(..).collect();
    assert!(children.is_empty());
    assert_eq!(drained.len(), 2);
}

#[test]
fn drain_range() {
    let mut children = vec![
        make_ident("a"),
        make_ident("b"),
        make_ident("c"),
        make_ident("d"),
    ];
    let drained: Vec<Node> = children.drain(1..3).collect();
    assert_eq!(names_of(&children), vec!["a", "d"]);
    assert_eq!(names_of(&drained), vec!["b", "c"]);
}

#[test]
fn splice_replace_range() {
    let mut children = vec![make_ident("a"), make_ident("b"), make_ident("c")];
    let removed: Vec<Node> = children
        .splice(1..2, vec![make_ident("x"), make_ident("y")])
        .collect();
    assert_eq!(names_of(&children), vec!["a", "x", "y", "c"]);
    assert_eq!(names_of(&removed), vec!["b"]);
}

// ══════════════════════════════════════════════════════════════
// truncate / extend
// ══════════════════════════════════════════════════════════════

#[test]
fn truncate_children() {
    let mut children = vec![make_ident("a"), make_ident("b"), make_ident("c")];
    children.truncate(1);
    assert_eq!(names_of(&children), vec!["a"]);
}

#[test]
fn extend_children() {
    let mut children = vec![make_ident("a")];
    let more = vec![make_ident("b"), make_ident("c")];
    children.extend(more);
    assert_eq!(names_of(&children), vec!["a", "b", "c"]);
}

// ══════════════════════════════════════════════════════════════
// swap / reverse
// ══════════════════════════════════════════════════════════════

#[test]
fn swap_elements() {
    let mut children = vec![make_ident("foo"), make_ident("bar")];
    children.swap(0, 1);
    assert_eq!(names_of(&children), vec!["bar", "foo"]);
}

#[test]
fn reverse_children() {
    let mut children = vec![make_ident("a"), make_ident("b"), make_ident("c")];
    children.reverse();
    assert_eq!(names_of(&children), vec!["c", "b", "a"]);
}

// ══════════════════════════════════════════════════════════════
// dedup / sort-like operations
// ══════════════════════════════════════════════════════════════

#[test]
fn dedup_by_type() {
    let mut children = vec![
        make_whitespace(" "),
        make_whitespace("  "),
        make_ident("foo"),
        make_number("42"),
    ];
    children.dedup_by(|a, b| a.node_type() == b.node_type());
    // The two consecutive WhiteSpace nodes are deduped into one
    assert_eq!(children.len(), 3);
}

#[test]
fn partition_by_type() {
    let children = vec![
        make_ident("foo"),
        make_whitespace(" "),
        make_number("42"),
        make_whitespace("  "),
        make_ident("bar"),
    ];
    let (idents, rest): (Vec<&Node>, Vec<&Node>) = children
        .iter()
        .partition(|n| n.node_type() == "Identifier");
    assert_eq!(idents.len(), 2);
    assert_eq!(rest.len(), 3);
}
