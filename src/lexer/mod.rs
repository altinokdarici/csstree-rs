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
//! - **Matching**: State machine executor with backtracking stacks
//! - **Generic types**: 65+ built-in type matchers (`<length>`, `<color>`, etc.)
//! - **Token prep**: Converts CSS values → token arrays for matching
//! - **Lexer struct**: Main API managing syntax definitions and validation

pub mod types;
pub mod error;
pub mod units;

pub use error::{SyntaxMatchError, SyntaxReferenceError};
pub use types::{MatchNode, MatchResult, MatchToken, LexerConfig};
pub use units::UNITS;
