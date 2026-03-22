//! CSS code generator — converts AST nodes back to CSS text.
//!
//! ## Architecture (from JS `external/csstree/lib/generator/`)
//!
//! The generator walks an AST tree and produces CSS text. Key components:
//!
//! - **`create.js`** — Factory that creates a generator from config. The generator
//!   maintains a string buffer, a `prevCode` for whitespace insertion tracking,
//!   and four key methods:
//!   - `node(node)` — dispatch to the per-type generate function
//!   - `children(node, delimiter?)` — iterate children, optionally inserting delimiters
//!   - `token(type, value)` — emit a token, with auto-whitespace insertion via `tokenBefore`
//!   - `tokenize(chunk)` — re-tokenize a raw string and emit each token
//!   - `emit(value)` — append raw text to the buffer
//!
//! - **`token-before.js`** — Whitespace insertion logic per CSS serialization spec §9.
//!   Uses a lookup table of (prevTokenCode, nextTokenCode) pairs that require a
//!   separating space. Two modes: `spec` (strict W3C pairs) and `safe` (adds extra
//!   pairs for browser compatibility). Each token is encoded as a u32 combining
//!   the token type and the first character code.
//!
//! - **Per-node generate functions** — Each of the 49 AST node types has a `generate()`
//!   function that calls `this.token()`, `this.node()`, `this.children()`, or
//!   `this.tokenize()` to produce output. Patterns:
//!   - Simple tokens: `Hash → token(Hash, '#' + value)`
//!   - Children: `StyleSheet → children(node)`
//!   - Delimited: `SelectorList → children(node, () => token(Comma, ','))`
//!   - Compound: `Declaration → token(Ident, property) + token(Colon, ':') + node(value)`
//!   - Re-tokenized: `TypeSelector → tokenize(name)`, `Raw → tokenize(value)`
//!
//! - **Options**: `mode` ('spec' or 'safe'), `sourceMap`, `decorator`
//!
//! ## Rust design
//!
//! - Single `generate(node) -> String` function (no factory pattern needed)
//! - `GenerateMode` enum for spec vs safe whitespace rules
//! - Token-before logic as a function with a `HashSet<u32>` lookup table
//! - Each Node variant matched in a single `match` statement

pub mod token_before;

use crate::ast::Node;
use std::collections::HashSet;

/// Whitespace insertion mode for serialization.
///
/// Controls which token pairs get automatic whitespace between them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GenerateMode {
    /// Safe mode — inserts whitespace for browser compatibility (default).
    ///
    /// Includes all W3C spec pairs plus additional pairs needed
    /// for IE11 and other browsers.
    #[default]
    Safe,
    /// Spec mode — strict W3C CSS Syntax §9 serialization pairs only.
    Spec,
}

/// Options for CSS code generation.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerateOptions {
    /// Whitespace insertion mode.
    pub mode: GenerateMode,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            mode: GenerateMode::Safe,
        }
    }
}

/// Internal generator state that accumulates CSS output.
#[derive(Debug)]
#[allow(dead_code)] // fields used in step 3
struct Generator {
    /// Output buffer.
    buffer: String,
    /// Previous token code for whitespace insertion.
    prev_code: u32,
    /// Whitespace-required pair lookup table.
    ws_pairs: HashSet<u32>,
}

/// Generate CSS text from an AST node.
///
/// This is the main public API. Walks the AST and produces a CSS string.
pub fn generate(node: &Node, options: &GenerateOptions) -> String {
    let ws_pairs = token_before::build_pairs(options.mode);
    let mut ctx = Generator {
        buffer: String::new(),
        prev_code: 0,
        ws_pairs,
    };
    ctx.node(node);
    ctx.buffer
}

impl Generator {
    #[allow(clippy::unused_self)] // placeholder — implemented in step 3
    fn node(&mut self, _node: &Node) {
        // Placeholder — implemented in step 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        let opts = GenerateOptions::default();
        assert_eq!(opts.mode, GenerateMode::Safe);
    }

    #[test]
    fn spec_mode() {
        let opts = GenerateOptions {
            mode: GenerateMode::Spec,
        };
        assert_eq!(opts.mode, GenerateMode::Spec);
    }

    #[test]
    fn generate_empty_stylesheet() {
        use crate::ast::StyleSheet;
        let node = Node::StyleSheet(StyleSheet {
            loc: None,
            children: vec![],
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "");
    }
}
