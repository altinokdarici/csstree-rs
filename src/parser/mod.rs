//! CSS parser — recursive descent with error recovery.
//!
//! Reference: `external/csstree/lib/parser/` + `lib/syntax/scope/` + `lib/syntax/atrule/`
//!
//! ## Architecture (from JS source analysis)
//!
//! The parser is a **factory pattern** that merges `TokenStream` navigation with
//! node-specific parse functions and scope-based recognizers.
//!
//! ### Entry points (contexts)
//!
//! `parse(source, options)` dispatches to a context function:
//! - `stylesheet` → `StyleSheet` (default)
//! - `selector` → `Selector`
//! - `selectorList` → `SelectorList`
//! - `value` → `Value`
//! - `declaration` → `Declaration`
//! - `declarationList` → `DeclarationList`
//! - `block` → `Block`
//! - `atrule` → `Atrule`
//! - `atrulePrelude` → (function)
//! - `mediaQueryList` → `MediaQueryList`
//! - `mediaQuery` → `MediaQuery`
//!
//! ### Scopes (recognizers)
//!
//! Each context uses a "scope" that defines how to recognize tokens:
//! - **Selector scope:** maps tokens to `TypeSelector`, `ClassSelector`, `IdSelector`,
//!   `AttributeSelector`, `PseudoClassSelector`, `PseudoElementSelector`, `Combinator`,
//!   `NestingSelector`. Implicit space combinators inserted via `on_whitespace`.
//! - **Value scope:** maps tokens to `Number`, `Dimension`, `Percentage`, `Hash`,
//!   `String`, `Identifier`, `Function`, `Url`, `Operator`, `Parentheses`, `Brackets`,
//!   `UnicodeRange`. Special handling for `calc()` whitespace around +/-.
//! - **Default scope:** base for Value, also used by `atrulePrelude`.
//!
//! ### Core loop: `read_sequence(recognizer)`
//!
//! The main parsing loop: skips whitespace/comments, calls `recognizer.get_node()`,
//! handles whitespace via `recognizer.on_whitespace()`, builds a children list.
//!
//! ### Error recovery
//!
//! `parse_with_fallback(consumer, fallback)` tries the consumer; on error, rewinds
//! the token stream and calls the fallback (usually `Raw`) to consume the bad content.
//! This is how the parser produces `Raw` nodes for malformed CSS instead of failing.
//!
//! ### Key parse functions
//!
//! - `StyleSheet`: loops over top-level tokens (rules, at-rules, comments)
//! - `Rule`: parses prelude (SelectorList or Raw) + Block
//! - `Declaration`: property + colon + value (with `!important`) + error recovery
//! - `Selector`: `read_sequence(selector_scope)`
//! - `Value`: `read_sequence(value_scope)`
//! - `Function`: name + children via scope + closing `)`
//! - `Block`: `{` + declarations/rules + `}`
//!
//! ### Parse options
//!
//! - `context` — entry point name
//! - `positions` — include source locations
//! - `filename` — for error messages
//! - `parse_atrule_prelude` / `parse_rule_prelude` / `parse_value` — toggle sub-parsing
//! - `on_parse_error` — callback for recovered errors
//! - `on_comment` / `on_token` — callbacks
//!
//! ## Rust design
//!
//! - Parser struct holds `TokenStream` + config + scope functions
//! - Scopes are trait objects or function pointers
//! - Error recovery via `Result` + `Raw` fallback pattern
//! - `CssSyntaxError` struct with source fragment display
//! - Parse options as a builder struct
