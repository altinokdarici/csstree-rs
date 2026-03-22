//! CSS Value Definition Syntax parser, generator, and walker.
//!
//! Parses W3C CSS Value Definition Syntax strings (e.g. `<length> | auto`)
//! into an AST, generates them back, and walks the tree.
//!
//! Reference: `external/csstree/lib/definition-syntax/`
//!
//! ## Architecture
//!
//! ### Scanner (`scanner.js`)
//! Low-level character reader: `charCode()`, `nextCharCode()`, `skipWs()`,
//! `scanWord()`, `scanNumber()`, `scanString()`. Tracks position and raises
//! `SyntaxError` on invalid input.
//!
//! ### Parser (`parse.js`)
//! Converts definition syntax strings into an AST via `parse(source) -> Node`.
//! Uses `peek()` to dispatch to readers (`readType`, `readProperty`,
//! `readKeywordOrFunction`, `readGroup`, etc.). `readImplicitGroup()` is the
//! main workhorse — reads terms until a stop char, then `regroupTerms()`
//! restructures the flat list into nested Groups based on combinator precedence:
//!   - Space (juxtaposition) = 1 (tightest)
//!   - `&&` (all required, any order) = 2
//!   - `||` (at least one) = 3
//!   - `|` (exactly one) = 4 (loosest)
//!
//! ### Generator (`generate.js`)
//! Serializes AST back to definition syntax strings via
//! `generate(node, options) -> String`. Options: `forceBraces`, `compact`,
//! `decorate(str, node) -> str`.
//!
//! ### Walker (`walk.js`)
//! Pre-order traversal with enter/leave callbacks via
//! `walk(node, options, context)`. Walks Group terms, Multiplier/Boolean
//! inner terms. Leaf nodes have no children.
//!
//! ### Error (`SyntaxError.js`)
//! Custom error with `message`, `input`, `offset`, `rawMessage`.
//! Formatted with ASCII-art caret pointing to error position.
//!
//! ## AST Node Types (11 types)
//!
//! **Compound nodes:**
//! - `Group` — terms, combinator, explicit, `disallow_empty`
//! - `Multiplier` — term, min, max (0=unlimited), comma
//! - `Boolean` — term (modern `<boolean-expr[...]>`)
//!
//! **Leaf nodes:**
//! - `Type` — name, opts (optional Range)
//! - `Property` — name
//! - `Keyword` — name
//! - `AtKeyword` — name
//! - `Function` — name
//! - `StringNode` — value
//! - `Token` — value (single character)
//! - `Comma` — (no fields)

pub mod types;
pub mod error;
pub mod scanner;
pub mod parse;
pub mod generate;
pub mod walk;

pub use error::DefinitionSyntaxError;
pub use generate::{generate as generate_definition_syntax, GenerateOptions as DefSyntaxGenOptions};
pub use parse::parse as parse_definition_syntax;
pub use types::{
    Combinator, DefinitionSyntaxNode, GroupNode, MultiplierNode,
    BooleanNode, TypeNode, RangeNode, PropertyNode, KeywordNode,
    AtKeywordNode, FunctionNode, StringValueNode, TokenNode,
};
pub use walk::{walk as walk_definition_syntax, walk_enter as walk_definition_syntax_enter};
