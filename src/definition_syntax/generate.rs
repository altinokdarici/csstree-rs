//! Generator for CSS Value Definition Syntax strings.

#[allow(clippy::wildcard_imports)] // All definition syntax types needed for node matching
use super::types::*;

/// Options for the definition syntax generator.
#[derive(Debug, Default)]
pub struct GenerateOptions {
    /// Always wrap groups with `[...]` even if implicit.
    pub force_braces: bool,
    /// No spaces around combinators.
    pub compact: bool,
}

/// Generate a multiplier suffix string.
fn generate_multiplier(node: &MultiplierNode) -> String {
    let (min, max, comma) = (node.min, node.max, node.comma);

    if min == 0 && max == 0 {
        return if comma { "#?".into() } else { "*".into() };
    }
    if min == 0 && max == 1 {
        return "?".into();
    }
    if min == 1 && max == 0 {
        return if comma { "#".into() } else { "+".into() };
    }
    if min == 1 && max == 1 {
        return String::new();
    }

    let prefix = if comma { "#" } else { "" };
    if min == max {
        format!("{prefix}{{{min}}}")
    } else if max == 0 {
        format!("{prefix}{{{min},}}")
    } else {
        format!("{prefix}{{{min},{max}}}")
    }
}

/// Generate range notation for a type.
fn generate_type_opts(range: &RangeNode) -> String {
    let min_str = range.min.map_or("-∞".to_string(), |v| format!("{v}"));
    let max_str = range.max.map_or("∞".to_string(), |v| format!("{v}"));
    format!(" [{min_str},{max_str}]")
}

/// Generate a group's terms joined by combinator.
fn generate_sequence(node: &GroupNode, options: &GenerateOptions) -> String {
    let combinator = if node.combinator == Combinator::Space || options.compact {
        node.combinator.as_str().to_string()
    } else {
        format!(" {} ", node.combinator.as_str())
    };

    let result: String = node
        .terms
        .iter()
        .map(|term| internal_generate(term, options))
        .collect::<Vec<_>>()
        .join(&combinator);

    if node.explicit || options.force_braces {
        let open = if options.compact || result.starts_with(',') { "[" } else { "[ " };
        let close = if options.compact { "]" } else { " ]" };
        format!("{open}{result}{close}")
    } else {
        result
    }
}

/// Internal recursive generator.
fn internal_generate(node: &DefinitionSyntaxNode, options: &GenerateOptions) -> String {
    match node {
        DefinitionSyntaxNode::Group(g) => {
            let seq = generate_sequence(g, options);
            if g.disallow_empty {
                format!("{seq}!")
            } else {
                seq
            }
        }
        DefinitionSyntaxNode::Multiplied(m) => {
            let term = internal_generate(&m.term, options);
            let mult = generate_multiplier(m);
            format!("{term}{mult}")
        }
        DefinitionSyntaxNode::Boolean(b) => {
            let inner = internal_generate(&b.term, options);
            format!("<boolean-expr[{inner}]>")
        }
        DefinitionSyntaxNode::Type(t) => {
            let opts = t.opts.as_ref().map_or(String::new(), generate_type_opts);
            format!("<{}{opts}>", t.name)
        }
        DefinitionSyntaxNode::Property(p) => format!("<'{}'>", p.name),
        DefinitionSyntaxNode::Keyword(k) => k.name.clone(),
        DefinitionSyntaxNode::AtKeyword(a) => format!("@{}", a.name),
        DefinitionSyntaxNode::Function(f) => format!("{}(", f.name),
        DefinitionSyntaxNode::StringValue(s) => s.value.clone(),
        DefinitionSyntaxNode::Token(t) => t.value.clone(),
        DefinitionSyntaxNode::Comma => ",".into(),
    }
}

/// Generate a CSS Value Definition Syntax string from an AST node.
pub fn generate(node: &DefinitionSyntaxNode, options: &GenerateOptions) -> String {
    internal_generate(node, options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_keyword() {
        let node = DefinitionSyntaxNode::Keyword(KeywordNode { name: "auto".into() });
        assert_eq!(generate(&node, &GenerateOptions::default()), "auto");
    }

    #[test]
    fn generate_type() {
        let node = DefinitionSyntaxNode::Type(TypeNode { name: "length".into(), opts: None });
        assert_eq!(generate(&node, &GenerateOptions::default()), "<length>");
    }

    #[test]
    fn generate_property() {
        let node = DefinitionSyntaxNode::Property(PropertyNode { name: "color".into() });
        assert_eq!(generate(&node, &GenerateOptions::default()), "<'color'>");
    }

    #[test]
    fn generate_multiplier_star() {
        let node = DefinitionSyntaxNode::Multiplied(MultiplierNode {
            term: Box::new(DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() })),
            min: 0,
            max: 0,
            comma: false,
        });
        assert_eq!(generate(&node, &GenerateOptions::default()), "a*");
    }

    #[test]
    fn generate_multiplier_hash() {
        let node = DefinitionSyntaxNode::Multiplied(MultiplierNode {
            term: Box::new(DefinitionSyntaxNode::Type(TypeNode { name: "color".into(), opts: None })),
            min: 1,
            max: 0,
            comma: true,
        });
        assert_eq!(generate(&node, &GenerateOptions::default()), "<color>#");
    }

    #[test]
    fn generate_group_bar() {
        let node = DefinitionSyntaxNode::Group(GroupNode {
            terms: vec![
                DefinitionSyntaxNode::Type(TypeNode { name: "length".into(), opts: None }),
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "auto".into() }),
            ],
            combinator: Combinator::Bar,
            explicit: false,
            disallow_empty: false,
        });
        assert_eq!(generate(&node, &GenerateOptions::default()), "<length> | auto");
    }

    #[test]
    fn generate_explicit_group() {
        let node = DefinitionSyntaxNode::Group(GroupNode {
            terms: vec![
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() }),
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "b".into() }),
            ],
            combinator: Combinator::Bar,
            explicit: true,
            disallow_empty: true,
        });
        assert_eq!(generate(&node, &GenerateOptions::default()), "[ a | b ]!");
    }

    #[test]
    fn generate_compact() {
        let node = DefinitionSyntaxNode::Group(GroupNode {
            terms: vec![
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() }),
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "b".into() }),
            ],
            combinator: Combinator::Bar,
            explicit: true,
            disallow_empty: false,
        });
        let opts = GenerateOptions { compact: true, force_braces: false };
        assert_eq!(generate(&node, &opts), "[a|b]");
    }
}
