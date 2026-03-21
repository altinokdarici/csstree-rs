//! Parse options controlling parser behavior.

use super::error::CssSyntaxError;
use crate::ast::Node;

/// Which parse context to use as the entry point.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ParseContext {
    /// Parse a full stylesheet (default).
    #[default]
    StyleSheet,
    /// Parse a single selector.
    Selector,
    /// Parse a comma-separated selector list.
    SelectorList,
    /// Parse a declaration value.
    Value,
    /// Parse a single declaration.
    Declaration,
    /// Parse a semicolon-separated declaration list.
    DeclarationList,
    /// Parse a block.
    Block,
    /// Parse a single at-rule.
    Atrule,
    /// Parse an at-rule prelude.
    AtrulePrelude,
    /// Parse a media query list.
    MediaQueryList,
    /// Parse a single media query.
    MediaQuery,
}

/// Callback for parse errors during error recovery.
pub type OnParseError = Box<dyn FnMut(&CssSyntaxError, &Node)>;

/// Callback for comments encountered during parsing.
pub type OnComment = Box<dyn FnMut(&str, &Node)>;

/// Flags controlling which parts of CSS are fully parsed vs left as Raw.
#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "mirrors JS parse options — each bool is an independent toggle")]
pub struct ParseFlags {
    /// Include source locations in AST nodes.
    pub positions: bool,
    /// Parse at-rule preludes into structured AST (vs Raw).
    pub parse_atrule_prelude: bool,
    /// Parse rule preludes/selectors into structured AST (vs Raw).
    pub parse_rule_prelude: bool,
    /// Parse declaration values into structured AST (vs Raw).
    pub parse_value: bool,
    /// Parse custom property values (default false — kept as Raw).
    pub parse_custom_property: bool,
}

impl Default for ParseFlags {
    fn default() -> Self {
        Self {
            positions: false,
            parse_atrule_prelude: true,
            parse_rule_prelude: true,
            parse_value: true,
            parse_custom_property: false,
        }
    }
}

/// Options for the CSS parser.
pub struct ParseOptions {
    /// Which context to parse (default: `StyleSheet`).
    pub context: ParseContext,
    /// Parse behavior flags.
    pub flags: ParseFlags,
    /// Source filename for error messages and locations.
    pub filename: Option<String>,
    /// Starting byte offset (for partial parsing).
    pub offset: usize,
    /// Starting line number (for partial parsing).
    pub line: u32,
    /// Starting column number (for partial parsing).
    pub column: u32,
    /// Callback for recovered parse errors.
    pub on_parse_error: Option<OnParseError>,
    /// Callback for comments.
    pub on_comment: Option<OnComment>,
}

impl std::fmt::Debug for ParseOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseOptions")
            .field("context", &self.context)
            .field("flags", &self.flags)
            .field("filename", &self.filename)
            .field("offset", &self.offset)
            .field("line", &self.line)
            .field("column", &self.column)
            .field("on_parse_error", &self.on_parse_error.as_ref().map(|_| ".."))
            .field("on_comment", &self.on_comment.as_ref().map(|_| ".."))
            .finish()
    }
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            context: ParseContext::default(),
            flags: ParseFlags::default(),
            filename: None,
            offset: 0,
            line: 1,
            column: 1,
            on_parse_error: None,
            on_comment: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_options() {
        let opts = ParseOptions::default();
        assert_eq!(opts.context, ParseContext::StyleSheet);
        assert!(!opts.flags.positions);
        assert!(opts.flags.parse_value);
        assert!(!opts.flags.parse_custom_property);
        assert_eq!(opts.line, 1);
        assert_eq!(opts.column, 1);
    }
}
