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
