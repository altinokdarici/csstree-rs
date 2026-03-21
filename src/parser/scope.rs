//! Scope recognizers for different parsing contexts.
//!
//! Each scope defines how tokens map to AST nodes within a context
//! (selector, value, at-rule prelude).

use crate::ast::Node;

/// Trait for scope recognizers that drive `read_sequence`.
pub trait Scope {
    /// Try to parse the next node from the current token. Returns `None` to stop.
    fn get_node(&self, parser: &mut super::Parser) -> Option<Node>;

    /// Handle whitespace between nodes (e.g. implicit combinators in selectors).
    fn on_whitespace(&self, _parser: &mut super::Parser, _next: Option<&Node>, _children: &mut Vec<Node>) {
        // Default: do nothing (whitespace is skipped)
    }
}
