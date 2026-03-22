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
//! - `Group` — terms: Vec, combinator: ` `|`&&`|`||`|`|`, explicit: bool, disallowEmpty: bool
//! - `Multiplier` — term: Node, min: u32, max: u32 (0=unlimited), comma: bool
//!   - `*` = {0,0,false}, `+` = {1,0,false}, `?` = {0,1,false}
//!   - `#` = {1,0,true}, `#?` = {0,0,true}, `{n,m}` = {n,m,false}
//! - `Boolean` — term: Node (modern `<boolean-expr[...]>`)
//!
//! **Leaf nodes:**
//! - `Type` — name: String, opts: Option<Range> (e.g. `<length>`, `<integer[-10,10]>`)
//! - `Property` — name: String (e.g. `<'color'>`)
//! - `Keyword` — name: String (e.g. `auto`, `none`)
//! - `AtKeyword` — name: String (e.g. `@media`)
//! - `Function` — name: String (e.g. `rgb(`)
//! - `StringNode` — value: String (single-quoted)
//! - `Token` — value: String (single character: `/`, `:`, etc.)
//! - `Comma` — (no fields)
//!
//! **Nested in Type:**
//! - `Range` — min: Option<f64>, max: Option<f64>
