//! CSS AST node types — 49 types matching `external/csstree/lib/syntax/node/`.
//!
//! ## Architecture (from JS source analysis)
//!
//! Each JS node file exports: `name`, `structure`, `parse()`, `generate()`.
//! The `structure` field defines the node shape (fields + allowed types).
//!
//! ## Node categories
//!
//! **Structure:** `StyleSheet`, `Rule`, `Block`, `DeclarationList`, `SelectorList`,
//! `MediaQueryList`, `LayerList`
//!
//! **Selectors:** `Selector`, `TypeSelector`, `ClassSelector`, `IdSelector`,
//! `AttributeSelector`, `PseudoClassSelector`, `PseudoElementSelector`,
//! `Combinator`, `NestingSelector`
//!
//! **Values:** `Value`, `Function`, `Identifier`, `Number`, `Dimension`,
//! `Percentage`, `Hash`, `String`, `Url`, `Brackets`, `Parentheses`,
//! `Operator`, `Ratio`, `UnicodeRange`
//!
//! **At-rules:** `Atrule`, `AtrulePrelude`
//!
//! **Declarations:** `Declaration`
//!
//! **Media:** `MediaQuery`, `Condition`, `Feature`, `FeatureFunction`,
//! `FeatureRange`, `GeneralEnclosed`, `SupportsDeclaration`
//!
//! **Selectors (advanced):** `Nth`, `AnPlusB`, `Layer`, `Scope`
//!
//! **Special:** `Comment`, `CDO`, `CDC`, `Raw`, `WhiteSpace`
//!
//! ## Rust design
//!
//! - Each node → a struct with typed fields
//! - `Node` enum wraps all 49 types for type-safe dispatch
//! - Children stored as `Vec<Node>` (not linked list)
//! - Location is `Option<Location>` on each node
//! - String fields use `String` (owned) since AST outlives parse
//! - Optional fields use `Option<T>`
//! - Union fields use inner enums (e.g. `AtrulePreludeValue`)
