//! CSS Lexer — validates CSS values against W3C definition syntax.
//!
//! Reference: `external/csstree/lib/lexer/`
//!
//! ## Architecture
//!
//! The lexer validates CSS property values against their definition syntax by:
//! 1. Converting definition syntax → match graph (automaton)
//! 2. Converting CSS value → token sequence
//! 3. Running automaton on tokens to determine validity
//!
//! ### Module Structure (14 JS files)
//!
//! **Generic types** (`generic.js`, `generic-const.js`, `generic-an-plus-b.js`,
//! `generic-urange.js`, `units.js`):
//! 65+ built-in type matchers (`<length>`, `<color>`, `<integer>`, etc.),
//! CSS unit groups, CSS-wide keywords, An+B and Unicode range parsers.
//!
//! **Match graph** (`match-graph.js`):
//! Converts definition syntax ASTs → match graphs (DAGs). Node types:
//! - `If` — conditional branch (match/then/else)
//! - `Enum` — optimized keyword dispatch table
//! - `MatchOnce` — bitmask-based permutation matcher (for `&&`/`||` > 5 terms)
//! - `Generic` — wraps a validation function
//! - `Type`/`Property` — references another definition
//! - `Keyword`/`AtKeyword`/`Function`/`Token`/`Comma`/`String` — literal matchers
//! - `MATCH`/`MISMATCH`/`DISALLOW_EMPTY` — terminal states
//!
//! **Matching** (`match.js`):
//! State machine executor with explicit stacks for backtracking:
//! - `thenStack` — pending success branches
//! - `elseStack` — pending failure branches (with full state snapshot)
//! - `matchStack` — linked list of matched items
//! - `syntaxStack` — open syntax scopes
//! - `syntaxStash` — low-priority matches (e.g. `<custom-ident>`)
//! Iteration limit: 150,000 to prevent infinite loops.
//!
//! **Token preparation** (`prepare-tokens.js`):
//! Converts CSS values (string or AST) → token arrays for matching.
//!
//! **Lexer class** (`Lexer.js`):
//! Main API — manages syntax definitions, provides validation methods:
//! - `matchProperty(name, value)` — validate property value
//! - `matchType(name, value)` — validate against type definition
//! - `matchDeclaration(node)` — validate Declaration AST node
//! - `checkStructure(ast)` — verify AST node structure
//! - Lazy parsing and graph building for efficiency.
//!
//! **Error handling** (`error.js`):
//! `SyntaxMatchError` with mismatch location, expected syntax, CSS context.
//!
//! **Search/Trace** (`search.js`, `trace.js`):
//! Find fragments matching types within match trees, trace paths to nodes.
//!
//! **Structure** (`structure.js`):
//! Schema-based AST node validation.
