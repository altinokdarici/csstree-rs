//! AST walker — traverses CSS AST nodes with enter/leave callbacks.
//!
//! ## Architecture (from JS `external/csstree/lib/walker/`)
//!
//! The walker provides depth-first traversal of AST nodes with callbacks.
//!
//! - **`create.js`** — Factory that creates a walker from config. The walker
//!   uses per-node-type iterators to visit child fields. Key features:
//!   - `enter` callback — called before visiting children
//!   - `leave` callback — called after visiting children
//!   - `visit` option — filter to only visit a specific node type
//!   - `reverse` option — visit children in reverse order
//!   - `break` sentinel — early termination from the walk
//!   - `skip` sentinel — skip visiting children of the current node
//!   - `context` object — tracks current position in the tree
//!     (stylesheet, atrule, rule, selector, block, declaration, function)
//!
//! - **Helper methods**:
//!   - `find(ast, fn)` — find first node matching predicate
//!   - `findLast(ast, fn)` — find last node matching predicate (reverse walk)
//!   - `findAll(ast, fn)` — find all nodes matching predicate

use crate::ast::Node;

// ── Walk action (control flow) ──

/// Control flow action returned by walk callbacks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WalkAction {
    /// Continue walking normally.
    #[default]
    Continue,
    /// Skip visiting children of the current node.
    Skip,
    /// Stop the entire walk immediately.
    Break,
}

// ── Walk context (position tracking) ──

/// Tracks the current position in the AST during a walk.
///
/// Each field is set when entering the corresponding node type and restored
/// when leaving. This mirrors the JS walker's `context` object which has
/// fields for: root, stylesheet, atrule, atrulePrelude, rule, selector,
/// block, declaration, function.
///
/// In Rust we use indices into a flat node list or simply track the node type
/// names. For now we store `Option<*const Node>` raw pointers for zero-cost
/// context tracking (the walker borrows the AST immutably for the duration).
#[derive(Debug, Clone, Default)]
pub struct WalkContext<'a> {
    /// The root node passed to `walk()`.
    pub root: Option<&'a Node>,
    /// The nearest enclosing `StyleSheet` node.
    pub stylesheet: Option<&'a Node>,
    /// The nearest enclosing `Atrule` node.
    pub atrule: Option<&'a Node>,
    /// The nearest enclosing `AtrulePrelude` node.
    pub atrule_prelude: Option<&'a Node>,
    /// The nearest enclosing `Rule` node.
    pub rule: Option<&'a Node>,
    /// The nearest enclosing `Selector` node.
    pub selector: Option<&'a Node>,
    /// The nearest enclosing `Block` node.
    pub block: Option<&'a Node>,
    /// The nearest enclosing `Declaration` node.
    pub declaration: Option<&'a Node>,
    /// The nearest enclosing `Function` node.
    pub function: Option<&'a Node>,
}

/// Which node types can be targeted by the `visit` filter.
///
/// Maps to the JS walker's `visit` option which accepts a node type string.
/// Only node types that have child fields can be visited (leaf nodes like
/// `Number`, `Hash`, etc. are not valid visit targets since they have no
/// children to iterate).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VisitFilter {
    /// Visit all node types (no filter).
    #[default]
    All,
    /// Only invoke callbacks for `Atrule` nodes.
    Atrule,
    /// Only invoke callbacks for `Rule` nodes.
    Rule,
    /// Only invoke callbacks for `Declaration` nodes.
    Declaration,
    /// Only invoke callbacks for a specific node type by name.
    NodeType(&'static str),
}

impl VisitFilter {
    /// Check if this filter matches the given node.
    pub fn matches(&self, node: &Node) -> bool {
        match self {
            Self::All => true,
            Self::Atrule => node.node_type() == "Atrule",
            Self::Rule => node.node_type() == "Rule",
            Self::Declaration => node.node_type() == "Declaration",
            Self::NodeType(name) => node.node_type() == *name,
        }
    }
}

/// Options for the `walk` function.
#[derive(Debug, Default)]
pub struct WalkOptions {
    /// Visit children in reverse order.
    pub reverse: bool,
    /// Only invoke callbacks for nodes matching this filter.
    pub visit: VisitFilter,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Declaration, Identifier, Raw, Rule, Block, Value};

    #[test]
    fn walk_action_default_is_continue() {
        assert_eq!(WalkAction::default(), WalkAction::Continue);
    }

    #[test]
    fn walk_action_equality() {
        assert_ne!(WalkAction::Break, WalkAction::Skip);
        assert_ne!(WalkAction::Skip, WalkAction::Continue);
        assert_ne!(WalkAction::Break, WalkAction::Continue);
    }

    #[test]
    fn walk_context_default_is_all_none() {
        let ctx = WalkContext::default();
        assert!(ctx.root.is_none());
        assert!(ctx.stylesheet.is_none());
        assert!(ctx.atrule.is_none());
        assert!(ctx.atrule_prelude.is_none());
        assert!(ctx.rule.is_none());
        assert!(ctx.selector.is_none());
        assert!(ctx.block.is_none());
        assert!(ctx.declaration.is_none());
        assert!(ctx.function.is_none());
    }

    #[test]
    fn walk_context_tracks_references() {
        let node = Node::Rule(Rule {
            loc: None,
            prelude: Box::new(Node::Raw(Raw { loc: None, value: "div".into() })),
            block: Box::new(Node::Block(Block { loc: None, children: vec![] })),
        });
        let mut ctx = WalkContext::default();
        ctx.root = Some(&node);
        ctx.rule = Some(&node);
        assert!(ctx.root.is_some());
        assert_eq!(ctx.root.unwrap().node_type(), "Rule");
    }

    #[test]
    fn visit_filter_matches_all() {
        let filter = VisitFilter::All;
        let node = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert!(filter.matches(&node));
    }

    #[test]
    fn visit_filter_matches_specific_type() {
        let filter = VisitFilter::Declaration;
        let decl = Node::Declaration(Declaration {
            loc: None,
            important: false,
            property: "color".into(),
            value: Box::new(Node::Value(Value { loc: None, children: vec![] })),
        });
        let ident = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert!(filter.matches(&decl));
        assert!(!filter.matches(&ident));
    }

    #[test]
    fn visit_filter_node_type_string() {
        let filter = VisitFilter::NodeType("Function");
        let func = Node::Function(crate::ast::Function { loc: None, name: "rgb".into(), children: vec![] });
        let ident = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert!(filter.matches(&func));
        assert!(!filter.matches(&ident));
    }

    #[test]
    fn walk_options_default() {
        let opts = WalkOptions::default();
        assert!(!opts.reverse);
        assert_eq!(opts.visit, VisitFilter::All);
    }
}
