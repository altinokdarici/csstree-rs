//! AST node types for CSS Value Definition Syntax.

// ── Combinator enum ──

/// Combinator type in a Group node.
///
/// Ordered by precedence (tightest to loosest):
/// - Space (juxtaposition): all must appear in order
/// - `DoubleAmpersand`: all must appear, any order
/// - `DoubleBar`: at least one must appear
/// - Bar: exactly one must appear
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combinator {
    /// `a b` — juxtaposition (all in order). Precedence 1 (tightest).
    Space,
    /// `a && b` — all required, any order. Precedence 2.
    DoubleAmpersand,
    /// `a || b` — at least one required. Precedence 3.
    DoubleBar,
    /// `a | b` — exactly one. Precedence 4 (loosest).
    Bar,
}

impl Combinator {
    /// Precedence value (lower = tighter binding).
    pub fn precedence(self) -> u8 {
        match self {
            Self::Space => 1,
            Self::DoubleAmpersand => 2,
            Self::DoubleBar => 3,
            Self::Bar => 4,
        }
    }

    /// String representation used in definition syntax.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Space => " ",
            Self::DoubleAmpersand => "&&",
            Self::DoubleBar => "||",
            Self::Bar => "|",
        }
    }
}

// ── Node enum ──

/// A node in the CSS Value Definition Syntax AST.
#[derive(Debug, Clone, PartialEq)]
pub enum DefinitionSyntaxNode {
    /// A group of terms with a combinator.
    Group(GroupNode),
    /// A term with a repetition multiplier.
    Multiplied(MultiplierNode),
    /// A boolean expression wrapper (`<boolean-expr[...]>`).
    Boolean(BooleanNode),
    /// A CSS value type (e.g. `<length>`).
    Type(TypeNode),
    /// A CSS property reference (e.g. `<'color'>`).
    Property(PropertyNode),
    /// A literal keyword (e.g. `auto`).
    Keyword(KeywordNode),
    /// An at-keyword (e.g. `@media`).
    AtKeyword(AtKeywordNode),
    /// A function name (e.g. `rgb(`).
    Function(FunctionNode),
    /// A string literal.
    StringValue(StringValueNode),
    /// A single-character token (e.g. `/`, `:`).
    Token(TokenNode),
    /// A comma separator.
    Comma,
}

impl DefinitionSyntaxNode {
    /// Returns the node type name (matches JS `node.type`).
    pub fn node_type(&self) -> &'static str {
        match self {
            Self::Group(_) => "Group",
            Self::Multiplied(_) => "Multiplier",
            Self::Boolean(_) => "Boolean",
            Self::Type(_) => "Type",
            Self::Property(_) => "Property",
            Self::Keyword(_) => "Keyword",
            Self::AtKeyword(_) => "AtKeyword",
            Self::Function(_) => "Function",
            Self::StringValue(_) => "String",
            Self::Token(_) => "Token",
            Self::Comma => "Comma",
        }
    }
}

// ── Compound node structs ──

/// A group of terms with a shared combinator.
#[derive(Debug, Clone, PartialEq)]
pub struct GroupNode {
    /// Child terms in this group.
    pub terms: Vec<DefinitionSyntaxNode>,
    /// The combinator between terms.
    pub combinator: Combinator,
    /// True if this group was explicitly bracketed `[...]`.
    pub explicit: bool,
    /// True if this group has `!` suffix (disallow empty match).
    pub disallow_empty: bool,
}

/// A multiplier wrapping a single term.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiplierNode {
    /// The term being multiplied.
    pub term: Box<DefinitionSyntaxNode>,
    /// Minimum number of repetitions.
    pub min: u32,
    /// Maximum number of repetitions (0 = unlimited).
    pub max: u32,
    /// True if repetitions are comma-separated (`#` multiplier).
    pub comma: bool,
}

/// A boolean expression wrapper (`<boolean-expr[...]>`).
#[derive(Debug, Clone, PartialEq)]
pub struct BooleanNode {
    /// The inner definition.
    pub term: Box<DefinitionSyntaxNode>,
}

// ── Leaf node structs ──

/// A CSS value type reference (e.g. `<length>`, `<integer[-10,10]>`).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeNode {
    /// The type name (e.g. `length`, `color`, `calc()`).
    pub name: String,
    /// Optional numeric range constraint.
    pub opts: Option<RangeNode>,
}

/// Numeric range constraint for a type (e.g. `[-10, 10]`).
#[derive(Debug, Clone, PartialEq)]
pub struct RangeNode {
    /// Minimum bound (`None` = -infinity).
    pub min: Option<f64>,
    /// Maximum bound (`None` = +infinity).
    pub max: Option<f64>,
}

/// A CSS property reference (e.g. `<'color'>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyNode {
    /// The property name.
    pub name: String,
}

/// A literal keyword (e.g. `auto`, `none`, `inherit`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordNode {
    /// The keyword text.
    pub name: String,
}

/// An at-keyword (e.g. `@media`, `@supports`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtKeywordNode {
    /// The at-keyword name (without `@`).
    pub name: String,
}

/// A CSS function name (e.g. `rgb(`, `calc(`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionNode {
    /// The function name (without opening paren).
    pub name: String,
}

/// A string literal in definition syntax.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StringValueNode {
    /// The string content (without quotes).
    pub value: String,
}

/// A single-character token (e.g. `/`, `:`, `;`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenNode {
    /// The token character.
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinator_precedence_order() {
        assert!(Combinator::Space.precedence() < Combinator::DoubleAmpersand.precedence());
        assert!(Combinator::DoubleAmpersand.precedence() < Combinator::DoubleBar.precedence());
        assert!(Combinator::DoubleBar.precedence() < Combinator::Bar.precedence());
    }

    #[test]
    fn combinator_as_str() {
        assert_eq!(Combinator::Space.as_str(), " ");
        assert_eq!(Combinator::DoubleAmpersand.as_str(), "&&");
        assert_eq!(Combinator::DoubleBar.as_str(), "||");
        assert_eq!(Combinator::Bar.as_str(), "|");
    }

    #[test]
    fn node_type_names() {
        let group = DefinitionSyntaxNode::Group(GroupNode {
            terms: vec![],
            combinator: Combinator::Space,
            explicit: false,
            disallow_empty: false,
        });
        assert_eq!(group.node_type(), "Group");

        let kw = DefinitionSyntaxNode::Keyword(KeywordNode { name: "auto".into() });
        assert_eq!(kw.node_type(), "Keyword");

        assert_eq!(DefinitionSyntaxNode::Comma.node_type(), "Comma");
    }

    #[test]
    fn multiplier_star() {
        // * = zero or more
        let node = MultiplierNode {
            term: Box::new(DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() })),
            min: 0,
            max: 0,
            comma: false,
        };
        assert_eq!(node.min, 0);
        assert_eq!(node.max, 0);
        assert!(!node.comma);
    }

    #[test]
    fn multiplier_hash() {
        // # = one or more, comma-separated
        let node = MultiplierNode {
            term: Box::new(DefinitionSyntaxNode::Type(TypeNode {
                name: "color".into(),
                opts: None,
            })),
            min: 1,
            max: 0,
            comma: true,
        };
        assert_eq!(node.min, 1);
        assert!(node.comma);
    }

    #[test]
    fn type_with_range() {
        let ty = TypeNode {
            name: "integer".into(),
            opts: Some(RangeNode { min: Some(-10.0), max: Some(10.0) }),
        };
        assert_eq!(ty.name, "integer");
        let range = ty.opts.unwrap();
        assert_eq!(range.min, Some(-10.0));
        assert_eq!(range.max, Some(10.0));
    }

    #[test]
    fn type_without_range() {
        let ty = TypeNode { name: "length".into(), opts: None };
        assert!(ty.opts.is_none());
    }

    #[test]
    fn group_explicit_with_bang() {
        let group = GroupNode {
            terms: vec![
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() }),
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "b".into() }),
            ],
            combinator: Combinator::Space,
            explicit: true,
            disallow_empty: true,
        };
        assert!(group.explicit);
        assert!(group.disallow_empty);
        assert_eq!(group.terms.len(), 2);
    }

    #[test]
    fn boolean_node() {
        let node = BooleanNode {
            term: Box::new(DefinitionSyntaxNode::Group(GroupNode {
                terms: vec![],
                combinator: Combinator::Bar,
                explicit: false,
                disallow_empty: false,
            })),
        };
        let wrapped = DefinitionSyntaxNode::Boolean(node);
        assert_eq!(wrapped.node_type(), "Boolean");
    }

    #[test]
    fn property_node() {
        let prop = PropertyNode { name: "color".into() };
        let node = DefinitionSyntaxNode::Property(prop);
        assert_eq!(node.node_type(), "Property");
    }

    #[test]
    fn all_leaf_types_constructable() {
        let nodes = vec![
            DefinitionSyntaxNode::Type(TypeNode { name: "length".into(), opts: None }),
            DefinitionSyntaxNode::Property(PropertyNode { name: "color".into() }),
            DefinitionSyntaxNode::Keyword(KeywordNode { name: "auto".into() }),
            DefinitionSyntaxNode::AtKeyword(AtKeywordNode { name: "media".into() }),
            DefinitionSyntaxNode::Function(FunctionNode { name: "rgb".into() }),
            DefinitionSyntaxNode::StringValue(StringValueNode { value: "test".into() }),
            DefinitionSyntaxNode::Token(TokenNode { value: "/".into() }),
            DefinitionSyntaxNode::Comma,
        ];
        assert_eq!(nodes.len(), 8);
    }
}
