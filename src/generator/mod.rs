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
//!   `this.tokenize()` to produce output.

pub mod token_before;

#[allow(clippy::wildcard_imports)] // 49 node types — explicit imports would be unwieldy
use crate::ast::*;
use crate::tokenizer::types::TokenType;
use std::collections::HashSet;

/// Whitespace insertion mode for serialization.
///
/// Controls which token pairs get automatic whitespace between them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GenerateMode {
    /// Safe mode — inserts whitespace for browser compatibility (default).
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
    /// Whether to generate a source map.
    pub source_map: bool,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            mode: GenerateMode::Safe,
            source_map: false,
        }
    }
}

/// A source map mapping entry.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceMapping {
    /// Source filename.
    pub source: String,
    /// Original line (1-based).
    pub original_line: u32,
    /// Original column (0-based).
    pub original_column: u32,
    /// Generated line (1-based).
    pub generated_line: u32,
    /// Generated column (0-based).
    pub generated_column: u32,
}

/// Result of generate with source map.
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// The generated CSS string.
    pub css: String,
    /// Source map mappings (if `source_map` was enabled).
    pub mappings: Vec<SourceMapping>,
}

/// VLQ base64 encoding for source maps.
fn vlq_encode(value: i32) -> String {
    const BASE64: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut vlq = if value < 0 { ((-value) << 1) | 1 } else { value << 1 };
    let mut result = String::new();
    loop {
        let mut digit = usize::try_from(vlq & 0x1F).unwrap_or(0);
        vlq >>= 5;
        if vlq > 0 { digit |= 0x20; }
        result.push(BASE64[digit] as char);
        if vlq == 0 { break; }
    }
    result
}

/// Encode source map mappings to the V3 "mappings" string.
fn encode_mappings(mappings: &[SourceMapping]) -> String {
    let mut result = String::new();
    let mut prev_gen_line: u32 = 1;
    let mut prev_gen_col: u32 = 0;
    let mut prev_src_idx: i32 = 0;
    let mut prev_orig_line: u32 = 0;
    let mut prev_orig_col: u32 = 0;

    for m in mappings {
        // Add semicolons for line gaps
        while prev_gen_line < m.generated_line {
            result.push(';');
            prev_gen_line += 1;
            prev_gen_col = 0;
        }

        if !result.is_empty() && !result.ends_with(';') {
            result.push(',');
        }

        // Field 1: generated column (relative)
        result.push_str(&vlq_encode(i32::try_from(m.generated_column).unwrap_or(0) - i32::try_from(prev_gen_col).unwrap_or(0)));
        // Field 2: source index (always 0, relative)
        result.push_str(&vlq_encode(0 - prev_src_idx));
        prev_src_idx = 0;
        // Field 3: original line (relative)
        result.push_str(&vlq_encode(i32::try_from(m.original_line.saturating_sub(1)).unwrap_or(0) - i32::try_from(prev_orig_line).unwrap_or(0)));
        prev_orig_line = m.original_line.saturating_sub(1);
        // Field 4: original column (relative)
        result.push_str(&vlq_encode(i32::try_from(m.original_column).unwrap_or(0) - i32::try_from(prev_orig_col).unwrap_or(0)));
        prev_orig_col = m.original_column;

        prev_gen_col = m.generated_column;
    }

    result
}

/// Serialize source map to JSON string.
pub fn source_map_to_json(source_file: &str, mappings: &[SourceMapping]) -> String {
    let encoded = encode_mappings(mappings);
    format!(
        r#"{{"version":3,"sources":["{source_file}"],"names":[],"mappings":"{encoded}"}}"#,
    )
}

/// Internal generator state that accumulates CSS output.
#[derive(Debug)]
struct Generator {
    /// Output buffer.
    buffer: String,
    /// Previous token code for whitespace insertion.
    prev_code: u32,
    /// Whitespace-required pair lookup table.
    ws_pairs: HashSet<u32>,
    /// Whether source map tracking is enabled.
    source_map: bool,
    /// Current generated line (1-based).
    gen_line: u32,
    /// Current generated column (0-based).
    gen_column: u32,
    /// Collected source mappings.
    mappings: Vec<SourceMapping>,
}

const REVERSE_SOLIDUS: u8 = 0x5C;

/// Strip CSS comments (`/* ... */`) from a string.
fn strip_css_comments(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            // Skip to end of comment
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i + 1 < bytes.len() {
                i += 2; // skip */
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
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
        source_map: options.source_map,
        gen_line: 1,
        gen_column: 0,
        mappings: Vec::new(),
    };
    ctx.node(node);
    ctx.buffer
}

/// Generate CSS with source map.
pub fn generate_with_source_map(node: &Node, options: &GenerateOptions) -> GenerateResult {
    let ws_pairs = token_before::build_pairs(options.mode);
    let mut ctx = Generator {
        buffer: String::new(),
        prev_code: 0,
        ws_pairs,
        source_map: true,
        gen_line: 1,
        gen_column: 0,
        mappings: Vec::new(),
    };
    ctx.node(node);
    GenerateResult {
        css: ctx.buffer,
        mappings: ctx.mappings,
    }
}

// ── Core generator methods ──

impl Generator {
    /// Emit a token with auto-whitespace insertion.
    fn token(&mut self, token_type: TokenType, value: &str) {
        self.prev_code =
            token_before::token_before(self.prev_code, token_type, value, &self.ws_pairs);

        // If bit 0 is set, insert a whitespace
        if self.prev_code & 1 != 0 {
            self.buffer.push(' ');
            self.gen_column += 1;
        }

        self.emit_raw(value);

        // After a backslash delimiter, emit a newline (prevents broken escapes)
        if token_type == TokenType::Delim
            && value.as_bytes().first().copied() == Some(REVERSE_SOLIDUS)
        {
            self.buffer.push('\n');
            self.gen_line += 1;
            self.gen_column = 0;
        }
    }

    /// Emit raw text and track position for source maps.
    fn emit_raw(&mut self, value: &str) {
        self.buffer.push_str(value);
        if self.source_map {
            for ch in value.bytes() {
                if ch == b'\n' {
                    self.gen_line += 1;
                    self.gen_column = 0;
                } else {
                    self.gen_column += 1;
                }
            }
        }
    }

    /// Record a source mapping for a node with location info.
    fn record_mapping(&mut self, loc: Option<&Loc>) {
        if !self.source_map {
            return;
        }
        if let Some(loc) = loc {
            self.mappings.push(SourceMapping {
                source: loc.source.clone().unwrap_or_default(),
                original_line: loc.start.line,
                original_column: loc.start.column.saturating_sub(1), // 1-based → 0-based
                generated_line: self.gen_line,
                generated_column: self.gen_column,
            });
        }
    }

    /// Re-tokenize a chunk and emit each token.
    fn tokenize_chunk(&mut self, chunk: &str) {
        let mut first = true;
        crate::tokenizer::tokenize(chunk, |token_type, start, end| {
            let value = &chunk[start..end];
            // Suppress auto-whitespace for internal tokens (not the first)
            if first {
                self.token(token_type, value);
                first = false;
            } else {
                // Emit without whitespace check for subsequent tokens
                self.prev_code =
                    token_before::encode_token(token_type, value);
                self.buffer.push_str(value);
            }
        });
    }

    /// Emit children of a node.
    fn children(&mut self, children: &[Node]) {
        for child in children {
            self.node(child);
        }
    }

    /// Emit children with space after colon operators (for feature expressions).
    fn children_with_feature_colon(&mut self, children: &[Node]) {
        for child in children {
            if let Node::Operator(op) = child {
                if op.value == ":" {
                    // Emit ": " with space for feature expressions
                    self.buffer.push_str(": ");
                    self.prev_code = token_before::encode_token(TokenType::Colon, ":");
                    continue;
                }
            }
            self.node(child);
        }
    }

    /// Emit children with a delimiter between them.
    fn children_delimited(&mut self, children: &[Node], delim_type: TokenType, delim: &str) {
        for (i, child) in children.iter().enumerate() {
            if i > 0 {
                self.token(delim_type, delim);
            }
            self.node(child);
        }
    }

    /// Emit children with semicolons after declarations.
    fn children_with_decl_semicolons(&mut self, children: &[Node]) {
        let mut prev_was_decl = false;
        for child in children {
            if prev_was_decl {
                self.token(TokenType::Semicolon, ";");
            }
            self.node(child);
            prev_was_decl = matches!(child, Node::Declaration(_));
        }
    }

    // ── Node dispatch ──

    /// Generate CSS for a single AST node.
    #[allow(clippy::too_many_lines)]
    fn node(&mut self, node: &Node) {
        match node {
            Node::StyleSheet(n) => self.children(&n.children),
            Node::Rule(n) => {
                self.record_mapping(n.loc.as_ref());
                self.node(&n.prelude);
                self.node(&n.block);
            }
            Node::Atrule(n) => {
                self.record_mapping(n.loc.as_ref());
                self.token(TokenType::AtKeyword, &format!("@{}", n.name));
                if let Some(prelude) = &n.prelude {
                    self.node(prelude);
                }
                if let Some(block) = &n.block {
                    self.node(block);
                } else {
                    self.token(TokenType::Semicolon, ";");
                }
            }
            Node::AtrulePrelude(n) => self.children(&n.children),
            Node::Block(n) => {
                self.token(TokenType::LeftCurlyBracket, "{");
                self.children_with_decl_semicolons(&n.children);
                self.token(TokenType::RightCurlyBracket, "}");
            }
            Node::Declaration(n) => {
                self.record_mapping(n.loc.as_ref());
                self.token(TokenType::Ident, &n.property);
                self.token(TokenType::Colon, ":");
                self.node(&n.value);
                if n.important {
                    self.token(TokenType::Delim, "!");
                    self.token(TokenType::Ident, "important");
                }
            }
            Node::DeclarationList(n) => {
                self.children_with_decl_semicolons(&n.children);
            }
            Node::SelectorList(n) => {
                self.children_delimited(&n.children, TokenType::Comma, ",");
            }
            Node::Selector(n) => self.children(&n.children),
            Node::Value(n) => self.children(&n.children),
            Node::Function(n) => {
                self.token(TokenType::Function, &format!("{}(", n.name));
                // For feature-like functions (style, supports), add space after colon
                let lower = n.name.to_ascii_lowercase();
                if matches!(lower.as_str(), "style" | "func") {
                    self.children_with_feature_colon(&n.children);
                } else {
                    self.children(&n.children);
                }
                self.token(TokenType::RightParenthesis, ")");
            }
            Node::Parentheses(n) => {
                self.token(TokenType::LeftParenthesis, "(");
                self.children(&n.children);
                self.token(TokenType::RightParenthesis, ")");
            }
            Node::Brackets(n) => {
                self.token(TokenType::Delim, "[");
                self.children(&n.children);
                self.token(TokenType::Delim, "]");
            }

            // ── Simple token nodes ──

            Node::Hash(n) => self.token(TokenType::Hash, &format!("#{}", n.value)),
            Node::Identifier(n) => self.token(TokenType::Ident, &n.name),
            Node::Number(n) => self.token(TokenType::Number, &n.value),
            Node::Dimension(n) => {
                self.token(TokenType::Dimension, &format!("{}{}", n.value, n.unit));
            }
            Node::Percentage(n) => {
                self.token(TokenType::Percentage, &format!("{}%", n.value));
            }
            Node::StringNode(n) => {
                // Normalize single quotes to double quotes (matching JS csstree)
                let val = if n.value.starts_with('\'') && n.value.ends_with('\'') {
                    let inner = &n.value[1..n.value.len() - 1];
                    format!("\"{}\"", inner.replace('"', "\\\"").replace("\\'", "'"))
                } else {
                    n.value.clone()
                };
                self.token(TokenType::String, &val);
            }
            Node::Operator(n) => self.tokenize_chunk(&n.value),
            Node::Raw(n) => {
                if n.verbatim {
                    // Emit verbatim (var() fallback, :unknown() args, etc.)
                    self.tokenize_chunk(&n.value);
                } else {
                    let stripped = strip_css_comments(&n.value);
                    let value = if stripped.trim().is_empty() && n.value.contains("/*") {
                        if stripped.is_empty() { String::new() } else { " ".to_string() }
                    } else {
                        stripped
                    };
                    self.tokenize_chunk(&value);
                }
            }
            Node::UnicodeRange(n) => self.token(TokenType::Ident, &n.value),
            Node::Url(n) => self.token(TokenType::Url, &format!("url({})", n.value)),

            // ── Selector nodes ──

            Node::TypeSelector(n) => self.tokenize_chunk(&n.name),
            Node::ClassSelector(n) => {
                self.token(TokenType::Delim, ".");
                self.token(TokenType::Ident, &n.name);
            }
            Node::IdSelector(n) => {
                // Use Delim instead of Hash to avoid whitespace issues (matches JS)
                self.token(TokenType::Delim, &format!("#{}", n.name));
            }
            Node::AttributeSelector(n) => {
                self.token(TokenType::Delim, "[");
                self.node(&n.name);
                if let Some(matcher) = &n.matcher {
                    self.tokenize_chunk(matcher);
                    if let Some(val) = &n.value {
                        self.node(val);
                    }
                }
                if let Some(flags) = &n.flags {
                    self.token(TokenType::Ident, flags);
                }
                self.token(TokenType::Delim, "]");
            }
            Node::Combinator(n) => self.tokenize_chunk(&n.name),
            Node::NestingSelector(_) => self.token(TokenType::Delim, "&"),
            Node::PseudoClassSelector(n) => {
                self.token(TokenType::Colon, ":");
                if let Some(children) = &n.children {
                    self.token(TokenType::Function, &format!("{}(", n.name));
                    self.children(children);
                    self.token(TokenType::RightParenthesis, ")");
                } else {
                    self.token(TokenType::Ident, &n.name);
                }
            }
            Node::PseudoElementSelector(n) => {
                self.token(TokenType::Colon, ":");
                self.token(TokenType::Colon, ":");
                if let Some(children) = &n.children {
                    self.token(TokenType::Function, &format!("{}(", n.name));
                    self.children(children);
                    self.token(TokenType::RightParenthesis, ")");
                } else {
                    self.token(TokenType::Ident, &n.name);
                }
            }

            // ── At-rule / media nodes ──

            Node::MediaQueryList(n) => {
                self.children_delimited(&n.children, TokenType::Comma, ",");
            }
            Node::MediaQuery(n) => {
                if let Some(modifier) = &n.modifier {
                    self.token(TokenType::Ident, modifier);
                }
                if let Some(media_type) = &n.media_type {
                    self.token(TokenType::Ident, media_type);
                    if n.condition.is_some() {
                        self.token(TokenType::Ident, "and");
                    }
                }
                if let Some(condition) = &n.condition {
                    self.node(condition);
                }
            }
            Node::Condition(n) => {
                for (i, child) in n.children.iter().enumerate() {
                    if i > 0 {
                        self.token(TokenType::Ident, &n.kind);
                    }
                    self.node(child);
                }
            }
            Node::Feature(n) => {
                self.token(TokenType::LeftParenthesis, "(");
                self.token(TokenType::Ident, &n.name);
                if let Some(value) = &n.value {
                    self.token(TokenType::Colon, ":");
                    self.node(value);
                }
                self.token(TokenType::RightParenthesis, ")");
            }
            Node::FeatureFunction(n) => {
                self.token(TokenType::Function, &format!("{}(", n.feature));
                self.node(&n.value);
                self.token(TokenType::RightParenthesis, ")");
            }
            Node::FeatureRange(n) => {
                self.token(TokenType::LeftParenthesis, "(");
                self.node(&n.left);
                self.tokenize_chunk(&n.left_comparison);
                self.node(&n.middle);
                if let Some(right_cmp) = &n.right_comparison {
                    self.tokenize_chunk(right_cmp);
                    if let Some(right) = &n.right {
                        self.node(right);
                    }
                }
                self.token(TokenType::RightParenthesis, ")");
            }
            Node::GeneralEnclosed(n) => {
                if let Some(func_name) = &n.function {
                    self.token(TokenType::Function, &format!("{func_name}("));
                    self.children(&n.children);
                    self.token(TokenType::RightParenthesis, ")");
                } else {
                    self.token(TokenType::LeftParenthesis, "(");
                    self.children(&n.children);
                    self.token(TokenType::RightParenthesis, ")");
                }
            }
            Node::LayerList(n) => {
                self.children_delimited(&n.children, TokenType::Comma, ",");
            }
            Node::Layer(n) => self.token(TokenType::Ident, &n.name),
            Node::Scope(n) => {
                if let Some(root) = &n.root {
                    self.token(TokenType::LeftParenthesis, "(");
                    self.node(root);
                    self.token(TokenType::RightParenthesis, ")");
                }
                if let Some(limit) = &n.limit {
                    self.token(TokenType::Ident, "to");
                    self.token(TokenType::LeftParenthesis, "(");
                    self.node(limit);
                    self.token(TokenType::RightParenthesis, ")");
                }
            }
            Node::SupportsDeclaration(n) => {
                self.token(TokenType::LeftParenthesis, "(");
                self.node(&n.declaration);
                self.token(TokenType::RightParenthesis, ")");
            }

            // ── Misc nodes ──

            Node::AnPlusB(n) => {
                if let Some(a) = &n.a {
                    self.token(TokenType::Ident, &format!("{a}n"));
                    if let Some(b) = &n.b {
                        let b_val: i64 = b.parse().unwrap_or(0);
                        if b_val >= 0 {
                            self.token(TokenType::Delim, "+");
                        }
                        self.token(TokenType::Number, b);
                    }
                } else if let Some(b) = &n.b {
                    self.token(TokenType::Number, b);
                }
            }
            Node::Nth(n) => {
                self.node(&n.nth);
                if let Some(selector) = &n.selector {
                    self.token(TokenType::Ident, "of");
                    self.node(selector);
                }
            }
            Node::Ratio(n) => {
                self.node(&n.left);
                self.token(TokenType::Delim, "/");
                if let Some(right) = &n.right {
                    self.node(right);
                }
            }
            Node::WhiteSpace(n) => self.token(TokenType::WhiteSpace, &n.value),
            Node::Comment(n) => {
                // Only emit important comments (/*! ... */) — strip regular comments
                if n.value.starts_with('!') {
                    self.token(TokenType::Comment, &format!("/*{}*/", n.value));
                }
            }
            Node::Cdo(_) => self.token(TokenType::Cdo, "<!--"),
            Node::Cdc(_) => self.token(TokenType::Cdc, "-->"),
        }
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
    fn generate_empty_stylesheet() {
        let node = Node::StyleSheet(StyleSheet { loc: None, children: vec![] });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "");
    }

    #[test]
    fn generate_simple_rule() {
        let node = Node::Rule(Rule {
            loc: None,
            prelude: Box::new(Node::SelectorList(SelectorList {
                loc: None,
                children: vec![Node::Selector(Selector {
                    loc: None,
                    children: vec![Node::TypeSelector(TypeSelector {
                        loc: None,
                        name: "a".to_string(),
                    })],
                })],
            })),
            block: Box::new(Node::Block(Block {
                loc: None,
                children: vec![Node::Declaration(Declaration {
                    loc: None,
                    important: false,
                    property: "color".to_string(),
                    value: Box::new(Node::Value(Value {
                        loc: None,
                        children: vec![Node::Identifier(Identifier {
                            loc: None,
                            name: "red".to_string(),
                        })],
                    })),
                })],
            })),
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "a{color:red}");
    }

    #[test]
    fn generate_declaration_with_important() {
        let node = Node::Declaration(Declaration {
            loc: None,
            important: true,
            property: "color".to_string(),
            value: Box::new(Node::Value(Value {
                loc: None,
                children: vec![Node::Identifier(Identifier {
                    loc: None,
                    name: "red".to_string(),
                })],
            })),
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "color:red!important");
    }

    #[test]
    fn generate_hash_value() {
        let node = Node::Hash(Hash { loc: None, value: "ff0000".to_string() });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "#ff0000");
    }

    #[test]
    fn generate_class_selector() {
        let node = Node::ClassSelector(ClassSelector { loc: None, name: "foo".to_string() });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, ".foo");
    }

    #[test]
    fn generate_pseudo_class() {
        let node = Node::PseudoClassSelector(PseudoClassSelector {
            loc: None,
            name: "hover".to_string(),
            children: None,
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, ":hover");
    }

    #[test]
    fn generate_function() {
        let node = Node::Function(Function {
            loc: None,
            name: "rgb".to_string(),
            children: vec![
                Node::Number(Number { loc: None, value: "255".to_string() }),
                Node::Operator(Operator { loc: None, value: ",".to_string() }),
                Node::Number(Number { loc: None, value: "0".to_string() }),
                Node::Operator(Operator { loc: None, value: ",".to_string() }),
                Node::Number(Number { loc: None, value: "0".to_string() }),
            ],
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "rgb(255,0,0)");
    }

    #[test]
    fn generate_dimension() {
        let node = Node::Dimension(Dimension {
            loc: None,
            value: "10".to_string(),
            unit: "px".to_string(),
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "10px");
    }

    #[test]
    fn generate_atrule_no_block() {
        let node = Node::Atrule(Atrule {
            loc: None,
            name: "charset".to_string(),
            prelude: Some(Box::new(Node::AtrulePrelude(AtrulePrelude {
                loc: None,
                children: vec![Node::StringNode(StringNode {
                    loc: None,
                    value: "'utf-8'".to_string(),
                })],
            }))),
            block: None,
        });
        let result = generate(&node, &GenerateOptions::default());
        assert_eq!(result, "@charset \"utf-8\";");
    }

    // ── Round-trip tests (parse → generate) ──

    fn round_trip(css: &str, expected: &str) {
        let ast = crate::parser::parse(css, crate::parser::ParseOptions::default());
        let result = generate(&ast, &GenerateOptions::default());
        assert_eq!(result, expected, "round-trip failed for input: {css:?}");
    }

    #[test]
    fn round_trip_simple_rule() {
        round_trip("a { color: red }", "a{color:red}");
    }

    #[test]
    fn round_trip_multiple_declarations() {
        round_trip(
            "a { color: red; font-size: 12px }",
            "a{color:red;font-size:12px}",
        );
    }

    #[test]
    fn round_trip_class_selector() {
        round_trip(".foo { display: block }", ".foo{display:block}");
    }

    #[test]
    fn round_trip_id_selector() {
        round_trip("#bar { margin: 0 }", "#bar{margin:0}");
    }

    #[test]
    fn round_trip_compound_selector() {
        round_trip("a.foo#bar { color: red }", "a.foo#bar{color:red}");
    }

    #[test]
    fn round_trip_descendant_combinator() {
        round_trip("a b { color: red }", "a b{color:red}");
    }

    #[test]
    fn round_trip_important() {
        round_trip("a { color: red !important }", "a{color:red!important}");
    }

    #[test]
    fn round_trip_at_rule_no_block() {
        round_trip("@charset 'utf-8';", "@charset \"utf-8\";");
    }

    #[test]
    fn round_trip_function_value() {
        round_trip(
            "a { color: rgb(255, 0, 0) }",
            "a{color:rgb(255,0,0)}",
        );
    }

    #[test]
    fn round_trip_percentage() {
        round_trip("a { width: 50% }", "a{width:50%}");
    }

    #[test]
    fn round_trip_pseudo_class() {
        round_trip("a:hover { color: red }", "a:hover{color:red}");
    }

    #[test]
    fn round_trip_empty_stylesheet() {
        round_trip("", "");
    }

    #[test]
    fn round_trip_multiple_rules() {
        round_trip(
            "a { color: red } b { color: blue }",
            "a{color:red}b{color:blue}",
        );
    }
}
