//! CSS AST node types — 49 types matching `external/csstree/lib/syntax/node/`.
//!
//! Each node is a struct with typed fields. The `Node` enum wraps all types
//! for type-safe dispatch. Children are `Vec<Node>`, locations are optional.

// ── Location types ──

/// Source location point (offset + line + column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loc {
    /// Source filename.
    pub source: Option<String>,
    /// Start position.
    pub start: Position,
    /// End position.
    pub end: Position,
}

/// A position in source (offset, line, column).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    /// Byte offset into the source string.
    pub offset: usize,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
}

// ── The Node enum ──

/// Top-level AST node wrapping all 49 CSS node types.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    AnPlusB(AnPlusB),
    Atrule(Atrule),
    AtrulePrelude(AtrulePrelude),
    AttributeSelector(AttributeSelector),
    Block(Block),
    Brackets(Brackets),
    Cdc(Cdc),
    Cdo(Cdo),
    ClassSelector(ClassSelector),
    Combinator(Combinator),
    Comment(Comment),
    Condition(Condition),
    Declaration(Declaration),
    DeclarationList(DeclarationList),
    Dimension(Dimension),
    Feature(Feature),
    FeatureFunction(FeatureFunction),
    FeatureRange(FeatureRange),
    Function(Function),
    GeneralEnclosed(GeneralEnclosed),
    Hash(Hash),
    Identifier(Identifier),
    IdSelector(IdSelector),
    Layer(Layer),
    LayerList(LayerList),
    MediaQuery(MediaQuery),
    MediaQueryList(MediaQueryList),
    NestingSelector(NestingSelector),
    Nth(Nth),
    Number(Number),
    Operator(Operator),
    Parentheses(Parentheses),
    Percentage(Percentage),
    PseudoClassSelector(PseudoClassSelector),
    PseudoElementSelector(PseudoElementSelector),
    Ratio(Ratio),
    Raw(Raw),
    Rule(Rule),
    Scope(Scope),
    Selector(Selector),
    SelectorList(SelectorList),
    StringNode(StringNode),
    StyleSheet(StyleSheet),
    SupportsDeclaration(SupportsDeclaration),
    TypeSelector(TypeSelector),
    UnicodeRange(UnicodeRange),
    Url(Url),
    Value(Value),
    WhiteSpace(WhiteSpace),
}

impl Node {
    /// Returns the CSS type name (e.g. `"Rule"`, `"Declaration"`).
    pub fn node_type(&self) -> &'static str {
        match self {
            Self::AnPlusB(_) => "AnPlusB",
            Self::Atrule(_) => "Atrule",
            Self::AtrulePrelude(_) => "AtrulePrelude",
            Self::AttributeSelector(_) => "AttributeSelector",
            Self::Block(_) => "Block",
            Self::Brackets(_) => "Brackets",
            Self::Cdc(_) => "CDC",
            Self::Cdo(_) => "CDO",
            Self::ClassSelector(_) => "ClassSelector",
            Self::Combinator(_) => "Combinator",
            Self::Comment(_) => "Comment",
            Self::Condition(_) => "Condition",
            Self::Declaration(_) => "Declaration",
            Self::DeclarationList(_) => "DeclarationList",
            Self::Dimension(_) => "Dimension",
            Self::Feature(_) => "Feature",
            Self::FeatureFunction(_) => "FeatureFunction",
            Self::FeatureRange(_) => "FeatureRange",
            Self::Function(_) => "Function",
            Self::GeneralEnclosed(_) => "GeneralEnclosed",
            Self::Hash(_) => "Hash",
            Self::Identifier(_) => "Identifier",
            Self::IdSelector(_) => "IdSelector",
            Self::Layer(_) => "Layer",
            Self::LayerList(_) => "LayerList",
            Self::MediaQuery(_) => "MediaQuery",
            Self::MediaQueryList(_) => "MediaQueryList",
            Self::NestingSelector(_) => "NestingSelector",
            Self::Nth(_) => "Nth",
            Self::Number(_) => "Number",
            Self::Operator(_) => "Operator",
            Self::Parentheses(_) => "Parentheses",
            Self::Percentage(_) => "Percentage",
            Self::PseudoClassSelector(_) => "PseudoClassSelector",
            Self::PseudoElementSelector(_) => "PseudoElementSelector",
            Self::Ratio(_) => "Ratio",
            Self::Raw(_) => "Raw",
            Self::Rule(_) => "Rule",
            Self::Scope(_) => "Scope",
            Self::Selector(_) => "Selector",
            Self::SelectorList(_) => "SelectorList",
            Self::StringNode(_) => "String",
            Self::StyleSheet(_) => "StyleSheet",
            Self::SupportsDeclaration(_) => "SupportsDeclaration",
            Self::TypeSelector(_) => "TypeSelector",
            Self::UnicodeRange(_) => "UnicodeRange",
            Self::Url(_) => "Url",
            Self::Value(_) => "Value",
            Self::WhiteSpace(_) => "WhiteSpace",
        }
    }

    /// Returns the location if present.
    pub fn loc(&self) -> Option<&Loc> {
        match self {
            Self::AnPlusB(n) => n.loc.as_ref(),
            Self::Atrule(n) => n.loc.as_ref(),
            Self::AtrulePrelude(n) => n.loc.as_ref(),
            Self::AttributeSelector(n) => n.loc.as_ref(),
            Self::Block(n) => n.loc.as_ref(),
            Self::Brackets(n) => n.loc.as_ref(),
            Self::Cdc(n) => n.loc.as_ref(),
            Self::Cdo(n) => n.loc.as_ref(),
            Self::ClassSelector(n) => n.loc.as_ref(),
            Self::Combinator(n) => n.loc.as_ref(),
            Self::Comment(n) => n.loc.as_ref(),
            Self::Condition(n) => n.loc.as_ref(),
            Self::Declaration(n) => n.loc.as_ref(),
            Self::DeclarationList(n) => n.loc.as_ref(),
            Self::Dimension(n) => n.loc.as_ref(),
            Self::Feature(n) => n.loc.as_ref(),
            Self::FeatureFunction(n) => n.loc.as_ref(),
            Self::FeatureRange(n) => n.loc.as_ref(),
            Self::Function(n) => n.loc.as_ref(),
            Self::GeneralEnclosed(n) => n.loc.as_ref(),
            Self::Hash(n) => n.loc.as_ref(),
            Self::Identifier(n) => n.loc.as_ref(),
            Self::IdSelector(n) => n.loc.as_ref(),
            Self::Layer(n) => n.loc.as_ref(),
            Self::LayerList(n) => n.loc.as_ref(),
            Self::MediaQuery(n) => n.loc.as_ref(),
            Self::MediaQueryList(n) => n.loc.as_ref(),
            Self::NestingSelector(n) => n.loc.as_ref(),
            Self::Nth(n) => n.loc.as_ref(),
            Self::Number(n) => n.loc.as_ref(),
            Self::Operator(n) => n.loc.as_ref(),
            Self::Parentheses(n) => n.loc.as_ref(),
            Self::Percentage(n) => n.loc.as_ref(),
            Self::PseudoClassSelector(n) => n.loc.as_ref(),
            Self::PseudoElementSelector(n) => n.loc.as_ref(),
            Self::Ratio(n) => n.loc.as_ref(),
            Self::Raw(n) => n.loc.as_ref(),
            Self::Rule(n) => n.loc.as_ref(),
            Self::Scope(n) => n.loc.as_ref(),
            Self::Selector(n) => n.loc.as_ref(),
            Self::SelectorList(n) => n.loc.as_ref(),
            Self::StringNode(n) => n.loc.as_ref(),
            Self::StyleSheet(n) => n.loc.as_ref(),
            Self::SupportsDeclaration(n) => n.loc.as_ref(),
            Self::TypeSelector(n) => n.loc.as_ref(),
            Self::UnicodeRange(n) => n.loc.as_ref(),
            Self::Url(n) => n.loc.as_ref(),
            Self::Value(n) => n.loc.as_ref(),
            Self::WhiteSpace(n) => n.loc.as_ref(),
        }
    }
}

// ── Node structs (49 types) ──

/// `An+B` microsyntax (e.g. `2n+1` in `:nth-child(2n+1)`).
#[derive(Debug, Clone, PartialEq)]
pub struct AnPlusB {
    pub loc: Option<Loc>,
    pub a: Option<String>,
    pub b: Option<String>,
}

/// An at-rule (e.g. `@media`, `@import`).
#[derive(Debug, Clone, PartialEq)]
pub struct Atrule {
    pub loc: Option<Loc>,
    pub name: String,
    pub prelude: Option<Box<Node>>,
    pub block: Option<Box<Node>>,
}

/// The prelude of an at-rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AtrulePrelude {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// Attribute selector (e.g. `[href^="https"]`).
#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
    pub loc: Option<Loc>,
    pub name: Box<Node>,
    pub matcher: Option<String>,
    pub value: Option<Box<Node>>,
    pub flags: Option<String>,
}

/// A `{ ... }` block containing declarations or rules.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// Square brackets `[ ... ]` in a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Brackets {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// `-->` (CDC token in stylesheet context).
#[derive(Debug, Clone, PartialEq)]
pub struct Cdc {
    pub loc: Option<Loc>,
}

/// `<!--` (CDO token in stylesheet context).
#[derive(Debug, Clone, PartialEq)]
pub struct Cdo {
    pub loc: Option<Loc>,
}

/// Class selector (e.g. `.foo`).
#[derive(Debug, Clone, PartialEq)]
pub struct ClassSelector {
    pub loc: Option<Loc>,
    pub name: String,
}

/// Selector combinator (e.g. `>`, `+`, `~`, ` `).
#[derive(Debug, Clone, PartialEq)]
pub struct Combinator {
    pub loc: Option<Loc>,
    pub name: String,
}

/// CSS comment `/* ... */`.
#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    pub loc: Option<Loc>,
    pub value: String,
}

/// Media/supports condition (e.g. `(min-width: 768px) and (color)`).
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub loc: Option<Loc>,
    pub kind: String,
    pub children: Vec<Node>,
}

/// A CSS declaration (e.g. `color: red !important`).
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub loc: Option<Loc>,
    pub important: bool,
    pub property: String,
    pub value: Box<Node>,
}

/// A list of declarations (e.g. inside a style attribute).
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationList {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// A dimension value (e.g. `10px`, `2em`).
#[derive(Debug, Clone, PartialEq)]
pub struct Dimension {
    pub loc: Option<Loc>,
    pub value: String,
    pub unit: String,
}

/// A media/container feature (e.g. `width: 100px`).
#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub loc: Option<Loc>,
    pub kind: String,
    pub name: String,
    pub value: Option<Box<Node>>,
}

/// A media/container feature function (e.g. `style(--x: 1)`).
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureFunction {
    pub loc: Option<Loc>,
    pub kind: String,
    pub feature: String,
    pub value: Box<Node>,
}

/// A media/container feature range (e.g. `100px < width < 200px`).
#[derive(Debug, Clone, PartialEq)]
pub struct FeatureRange {
    pub loc: Option<Loc>,
    pub kind: String,
    pub left: Box<Node>,
    pub left_comparison: String,
    pub middle: Box<Node>,
    pub right_comparison: Option<String>,
    pub right: Option<Box<Node>>,
}

/// A CSS function call (e.g. `rgb(255, 0, 0)`).
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub loc: Option<Loc>,
    pub name: String,
    pub children: Vec<Node>,
}

/// General enclosed expression in media queries.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneralEnclosed {
    pub loc: Option<Loc>,
    pub kind: String,
    pub function: Option<String>,
    pub children: Vec<Node>,
}

/// Hash value (e.g. `#fff`).
#[derive(Debug, Clone, PartialEq)]
pub struct Hash {
    pub loc: Option<Loc>,
    pub value: String,
}

/// An identifier (e.g. `red`, `auto`).
#[derive(Debug, Clone, PartialEq)]
pub struct Identifier {
    pub loc: Option<Loc>,
    pub name: String,
}

/// ID selector (e.g. `#main`).
#[derive(Debug, Clone, PartialEq)]
pub struct IdSelector {
    pub loc: Option<Loc>,
    pub name: String,
}

/// A layer name (e.g. `utilities` in `@layer utilities`).
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub loc: Option<Loc>,
    pub name: String,
}

/// A layer list (e.g. `@layer a, b, c`).
#[derive(Debug, Clone, PartialEq)]
pub struct LayerList {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// A single media query.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaQuery {
    pub loc: Option<Loc>,
    pub modifier: Option<String>,
    pub media_type: Option<String>,
    pub condition: Option<Box<Node>>,
}

/// A comma-separated list of media queries.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaQueryList {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// The `&` nesting selector.
#[derive(Debug, Clone, PartialEq)]
pub struct NestingSelector {
    pub loc: Option<Loc>,
}

/// Nth pseudo-class argument (e.g. `:nth-child(2n+1 of .foo)`).
#[derive(Debug, Clone, PartialEq)]
pub struct Nth {
    pub loc: Option<Loc>,
    pub nth: Box<Node>,
    pub selector: Option<Box<Node>>,
}

/// A numeric value (e.g. `42`, `3.14`).
#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub loc: Option<Loc>,
    pub value: String,
}

/// An operator in a value (e.g. `/`, `,`).
#[derive(Debug, Clone, PartialEq)]
pub struct Operator {
    pub loc: Option<Loc>,
    pub value: String,
}

/// Parentheses `( ... )` in a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Parentheses {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// A percentage value (e.g. `50%`).
#[derive(Debug, Clone, PartialEq)]
pub struct Percentage {
    pub loc: Option<Loc>,
    pub value: String,
}

/// Pseudo-class selector (e.g. `:hover`, `:nth-child(...)`).
#[derive(Debug, Clone, PartialEq)]
pub struct PseudoClassSelector {
    pub loc: Option<Loc>,
    pub name: String,
    pub children: Option<Vec<Node>>,
}

/// Pseudo-element selector (e.g. `::before`).
#[derive(Debug, Clone, PartialEq)]
pub struct PseudoElementSelector {
    pub loc: Option<Loc>,
    pub name: String,
    pub children: Option<Vec<Node>>,
}

/// A ratio value (e.g. `16/9`).
#[derive(Debug, Clone, PartialEq)]
pub struct Ratio {
    pub loc: Option<Loc>,
    pub left: Box<Node>,
    pub right: Option<Box<Node>>,
}

/// Raw unparsed content (error recovery or unparsed values).
#[derive(Debug, Clone, PartialEq)]
pub struct Raw {
    pub loc: Option<Loc>,
    pub value: String,
}

/// A CSS rule (selector + block).
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub loc: Option<Loc>,
    pub prelude: Box<Node>,
    pub block: Box<Node>,
}

/// `@scope` rule parts.
#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub loc: Option<Loc>,
    pub root: Option<Box<Node>>,
    pub limit: Option<Box<Node>>,
}

/// A compound selector (sequence of simple selectors).
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// A comma-separated list of selectors.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectorList {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// A CSS string value (e.g. `"hello"`).
///
/// Named `StringNode` to avoid conflict with `std::string::String`.
#[derive(Debug, Clone, PartialEq)]
pub struct StringNode {
    pub loc: Option<Loc>,
    pub value: String,
}

/// A CSS stylesheet (top-level node).
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// A `@supports` declaration test.
#[derive(Debug, Clone, PartialEq)]
pub struct SupportsDeclaration {
    pub loc: Option<Loc>,
    pub declaration: Box<Node>,
}

/// Type selector (e.g. `div`, `*`).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSelector {
    pub loc: Option<Loc>,
    pub name: String,
}

/// Unicode range (e.g. `U+0-7F`).
#[derive(Debug, Clone, PartialEq)]
pub struct UnicodeRange {
    pub loc: Option<Loc>,
    pub value: String,
}

/// A URL value (e.g. `url(foo.png)`).
#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    pub loc: Option<Loc>,
    pub value: String,
}

/// Value of a declaration (sequence of component values).
#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub loc: Option<Loc>,
    pub children: Vec<Node>,
}

/// Explicit whitespace node.
#[derive(Debug, Clone, PartialEq)]
pub struct WhiteSpace {
    pub loc: Option<Loc>,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_type_names() {
        let node = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert_eq!(node.node_type(), "Identifier");

        let node = Node::StringNode(StringNode { loc: None, value: "hello".into() });
        assert_eq!(node.node_type(), "String");

        let node = Node::Cdc(Cdc { loc: None });
        assert_eq!(node.node_type(), "CDC");
    }

    #[test]
    fn node_loc_access() {
        let node = Node::Number(Number { loc: None, value: "42".into() });
        assert!(node.loc().is_none());

        let node = Node::Number(Number {
            loc: Some(Loc {
                source: Some("test.css".into()),
                start: Position { offset: 0, line: 1, column: 1 },
                end: Position { offset: 2, line: 1, column: 3 },
            }),
            value: "42".into(),
        });
        assert!(node.loc().is_some());
        assert_eq!(node.loc().unwrap().start.line, 1);
    }

    #[test]
    fn construct_rule() {
        let rule = Node::Rule(Rule {
            loc: None,
            prelude: Box::new(Node::SelectorList(SelectorList {
                loc: None,
                children: vec![Node::Selector(Selector {
                    loc: None,
                    children: vec![Node::TypeSelector(TypeSelector {
                        loc: None,
                        name: "div".into(),
                    })],
                })],
            })),
            block: Box::new(Node::Block(Block {
                loc: None,
                children: vec![Node::Declaration(Declaration {
                    loc: None,
                    important: false,
                    property: "color".into(),
                    value: Box::new(Node::Value(Value {
                        loc: None,
                        children: vec![Node::Identifier(Identifier {
                            loc: None,
                            name: "red".into(),
                        })],
                    })),
                })],
            })),
        });
        assert_eq!(rule.node_type(), "Rule");
    }

    #[test]
    fn all_49_node_types_constructable() {
        // Verify all 49 variants can be constructed (compile-time check mostly)
        let nodes: Vec<Node> = vec![
            Node::AnPlusB(AnPlusB { loc: None, a: Some("2".into()), b: Some("1".into()) }),
            Node::Atrule(Atrule { loc: None, name: "media".into(), prelude: None, block: None }),
            Node::AtrulePrelude(AtrulePrelude { loc: None, children: vec![] }),
            Node::AttributeSelector(AttributeSelector { loc: None, name: Box::new(Node::Identifier(Identifier { loc: None, name: "href".into() })), matcher: None, value: None, flags: None }),
            Node::Block(Block { loc: None, children: vec![] }),
            Node::Brackets(Brackets { loc: None, children: vec![] }),
            Node::Cdc(Cdc { loc: None }),
            Node::Cdo(Cdo { loc: None }),
            Node::ClassSelector(ClassSelector { loc: None, name: "foo".into() }),
            Node::Combinator(Combinator { loc: None, name: ">".into() }),
            Node::Comment(Comment { loc: None, value: "test".into() }),
            Node::Condition(Condition { loc: None, kind: "media".into(), children: vec![] }),
            Node::Declaration(Declaration { loc: None, important: false, property: "color".into(), value: Box::new(Node::Raw(Raw { loc: None, value: "red".into() })) }),
            Node::DeclarationList(DeclarationList { loc: None, children: vec![] }),
            Node::Dimension(Dimension { loc: None, value: "10".into(), unit: "px".into() }),
            Node::Feature(Feature { loc: None, kind: "media".into(), name: "width".into(), value: None }),
            Node::FeatureFunction(FeatureFunction { loc: None, kind: "container".into(), feature: "style".into(), value: Box::new(Node::Raw(Raw { loc: None, value: "".into() })) }),
            Node::FeatureRange(FeatureRange { loc: None, kind: "media".into(), left: Box::new(Node::Number(Number { loc: None, value: "0".into() })), left_comparison: "<".into(), middle: Box::new(Node::Identifier(Identifier { loc: None, name: "width".into() })), right_comparison: None, right: None }),
            Node::Function(Function { loc: None, name: "rgb".into(), children: vec![] }),
            Node::GeneralEnclosed(GeneralEnclosed { loc: None, kind: "general".into(), function: None, children: vec![] }),
            Node::Hash(Hash { loc: None, value: "fff".into() }),
            Node::Identifier(Identifier { loc: None, name: "red".into() }),
            Node::IdSelector(IdSelector { loc: None, name: "main".into() }),
            Node::Layer(Layer { loc: None, name: "utilities".into() }),
            Node::LayerList(LayerList { loc: None, children: vec![] }),
            Node::MediaQuery(MediaQuery { loc: None, modifier: None, media_type: None, condition: None }),
            Node::MediaQueryList(MediaQueryList { loc: None, children: vec![] }),
            Node::NestingSelector(NestingSelector { loc: None }),
            Node::Nth(Nth { loc: None, nth: Box::new(Node::AnPlusB(AnPlusB { loc: None, a: Some("2".into()), b: None })), selector: None }),
            Node::Number(Number { loc: None, value: "42".into() }),
            Node::Operator(Operator { loc: None, value: "/".into() }),
            Node::Parentheses(Parentheses { loc: None, children: vec![] }),
            Node::Percentage(Percentage { loc: None, value: "50".into() }),
            Node::PseudoClassSelector(PseudoClassSelector { loc: None, name: "hover".into(), children: None }),
            Node::PseudoElementSelector(PseudoElementSelector { loc: None, name: "before".into(), children: None }),
            Node::Ratio(Ratio { loc: None, left: Box::new(Node::Number(Number { loc: None, value: "16".into() })), right: Some(Box::new(Node::Number(Number { loc: None, value: "9".into() }))) }),
            Node::Raw(Raw { loc: None, value: "raw content".into() }),
            Node::Rule(Rule { loc: None, prelude: Box::new(Node::Raw(Raw { loc: None, value: "div".into() })), block: Box::new(Node::Block(Block { loc: None, children: vec![] })) }),
            Node::Scope(Scope { loc: None, root: None, limit: None }),
            Node::Selector(Selector { loc: None, children: vec![] }),
            Node::SelectorList(SelectorList { loc: None, children: vec![] }),
            Node::StringNode(StringNode { loc: None, value: "hello".into() }),
            Node::StyleSheet(StyleSheet { loc: None, children: vec![] }),
            Node::SupportsDeclaration(SupportsDeclaration { loc: None, declaration: Box::new(Node::Declaration(Declaration { loc: None, important: false, property: "display".into(), value: Box::new(Node::Raw(Raw { loc: None, value: "grid".into() })) })) }),
            Node::TypeSelector(TypeSelector { loc: None, name: "div".into() }),
            Node::UnicodeRange(UnicodeRange { loc: None, value: "U+0-7F".into() }),
            Node::Url(Url { loc: None, value: "foo.png".into() }),
            Node::Value(Value { loc: None, children: vec![] }),
            Node::WhiteSpace(WhiteSpace { loc: None, value: " ".into() }),
        ];
        assert_eq!(nodes.len(), 49);
    }
}
