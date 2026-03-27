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
//! ### Key Components
//!
//! - **Match graph**: DAG of `MatchNode` variants (If, Enum, Generic, etc.)
//! - **Matching**: Recursive matcher with backtracking
//! - **Generic types**: 23+ built-in type matchers (`<length>`, `<color>`, etc.)
//! - **Token prep**: Converts CSS values → token arrays for matching
//! - **Lexer struct**: Main API managing syntax definitions and validation

pub mod types;
pub mod error;
pub mod units;
pub mod match_graph;
pub mod generic;
pub mod prepare_tokens;
pub mod css_lexer;

pub use css_lexer::Lexer;
pub use error::{SyntaxMatchError, SyntaxReferenceError};
pub use generic::get_generic_matcher;
pub use match_graph::build_match_graph;
pub use types::{AtruleConfig, MatchNode, MatchResult, MatchToken, LexerConfig, ValidationResult, SyntaxKind, TraceEntry};
pub use units::UNITS;
