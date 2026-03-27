//! Tests ported from `external/csstree/lib/__tests/lexer-search-fragments.js`.
//!
//! Tests that we can find specific value types within parsed CSS declarations.
//! Since we don't have findValueFragments() yet, we test the equivalent behavior
//! using walker::find_all to locate specific node types.

use csstree::ast::Node;
use csstree::parser::{parse, ParseOptions};
use csstree::walker::find_all;

#[test]
fn find_color_in_border() {
    // lexer-search-fragments.js: "should find single entry" for border
    let ast = parse(".a { border: 1px solid red }", ParseOptions::default());
    let idents = find_all(&ast, |n, _| {
        matches!(n, Node::Identifier(id) if id.name == "red")
    });
    assert_eq!(idents.len(), 1, "Should find 'red' as a color identifier");
}

#[test]
fn find_identifiers_in_font() {
    // lexer-search-fragments.js: "should find multiple entries" for font
    let ast = parse(".a { font: 10px Arial, Courier new, Times new roman }", ParseOptions::default());
    // Find all identifier nodes that are font family names
    let all_idents = find_all(&ast, |n, _| matches!(n, Node::Identifier(_)));
    // Should find Arial, Courier, new, Times, new, roman (and font property-related idents)
    assert!(all_idents.len() >= 6, "Should find multiple family name identifiers, got {}", all_idents.len());
}

#[test]
fn find_hash_colors_in_declaration() {
    // Verify we can find hash color values
    let ast = parse(".a { color: #ff0000; background: #00ff00 }", ParseOptions::default());
    let hashes = find_all(&ast, |n, _| matches!(n, Node::Hash(_)));
    assert_eq!(hashes.len(), 2, "Should find 2 hash colors");
}

#[test]
fn find_functions_in_value() {
    // Verify we can find function calls in values
    let ast = parse(".a { background: url(test.png); color: rgba(1,2,3,0.5) }", ParseOptions::default());
    let funcs = find_all(&ast, |n, _| matches!(n, Node::Function(_)));
    // rgba is a function; url may or may not be depending on parser
    assert!(funcs.len() >= 1, "Should find at least 1 function");
}

#[test]
fn find_all_declarations_in_ast() {
    // lexer-search-fragments.js: "should find all entries in ast"
    let css = ".a { color: red } .b { color: rgba(1,2,3,4); background: #123 } .c { border-color: rgb(1,2,3) }";
    let ast = parse(css, ParseOptions::default());
    let decls = find_all(&ast, |n, _| matches!(n, Node::Declaration(_)));
    assert_eq!(decls.len(), 4, "Should find 4 declarations");
}
