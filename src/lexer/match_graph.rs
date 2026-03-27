//! Match graph builder — converts definition syntax ASTs to matching automata.

use super::types::MatchNode;
#[allow(clippy::wildcard_imports)] // All definition syntax types needed for exhaustive matching
use crate::definition_syntax::types::*;
use std::collections::HashMap;

/// Build a match graph from a definition syntax AST node.
pub fn build_match_graph(node: &DefinitionSyntaxNode) -> MatchNode {
    build_node(node)
}

/// Build a match node from a definition syntax AST node.
fn build_node(node: &DefinitionSyntaxNode) -> MatchNode {
    match node {
        DefinitionSyntaxNode::Group(g) => build_group(g),
        DefinitionSyntaxNode::Multiplied(m) => build_multiplied(m),
        DefinitionSyntaxNode::Boolean(b) => build_node(&b.term),
        DefinitionSyntaxNode::Type(t) => {
            if is_function_type(&t.name) {
                MatchNode::Function { name: t.name[..t.name.len() - 2].to_string() }
            } else {
                MatchNode::Type { name: t.name.clone() }
            }
        }
        DefinitionSyntaxNode::Property(p) => MatchNode::Property { name: p.name.clone() },
        DefinitionSyntaxNode::Keyword(k) => MatchNode::Keyword { name: k.name.clone() },
        DefinitionSyntaxNode::AtKeyword(a) => MatchNode::AtKeyword { name: a.name.clone() },
        DefinitionSyntaxNode::Function(f) => MatchNode::Function { name: f.name.clone() },
        DefinitionSyntaxNode::StringValue(s) => MatchNode::StringMatch { value: s.value.clone() },
        DefinitionSyntaxNode::Token(t) => MatchNode::Token { value: t.value.clone() },
        DefinitionSyntaxNode::Comma => MatchNode::Comma,
    }
}

/// Check if a type name is a function type (ends with `()`).
fn is_function_type(name: &str) -> bool {
    name.len() > 2 && name.ends_with("()")
}

/// Build match graph for a group node.
fn build_group(group: &GroupNode) -> MatchNode {
    // Flatten Function+Group pairs: when a Function is followed by a Group (its body),
    // expand to [Function, ...body_terms..., Token(")")] since the definition syntax
    // parser consumes the closing ')' but the CSS tokenizer produces it.
    let mut terms: Vec<MatchNode> = Vec::new();
    let mut i = 0;
    while i < group.terms.len() {
        if matches!(&group.terms[i], DefinitionSyntaxNode::Function(_)) {
            terms.push(build_node(&group.terms[i]));
            i += 1;
            // Check if next term is a Group (function body)
            if i < group.terms.len() {
                if let DefinitionSyntaxNode::Group(body) = &group.terms[i] {
                    // Flatten the body group terms into our terms list
                    for body_term in &body.terms {
                        terms.push(build_node(body_term));
                    }
                    // Add closing paren token
                    terms.push(MatchNode::Token { value: ")".to_string() });
                    i += 1;
                }
            }
        } else {
            terms.push(build_node(&group.terms[i]));
            i += 1;
        }
    }

    if terms.is_empty() {
        return MatchNode::Match;
    }
    if terms.len() == 1 {
        let node = terms.into_iter().next().unwrap();
        return if group.disallow_empty {
            create_condition(node, MatchNode::DisallowEmpty, MatchNode::Mismatch)
        } else {
            node
        };
    }

    let result = build_group_match_graph(group.combinator, terms);

    if group.disallow_empty {
        create_condition(result, MatchNode::DisallowEmpty, MatchNode::Mismatch)
    } else {
        result
    }
}

/// Build match graph for a specific combinator type.
fn build_group_match_graph(combinator: Combinator, terms: Vec<MatchNode>) -> MatchNode {
    match combinator {
        Combinator::Space => {
            // Juxtaposition: all must match in order
            let mut result = MatchNode::Match;
            for term in terms.into_iter().rev() {
                result = create_condition(term, result, MatchNode::Mismatch);
            }
            result
        }
        Combinator::Bar => {
            // Alternation: exactly one must match
            let mut result = MatchNode::Mismatch;
            let mut map: Option<HashMap<String, MatchNode>> = None;

            for (i, term) in terms.iter().enumerate().rev() {
                if is_enum_compatible(term) {
                    if map.is_none() && i > 0 && is_enum_compatible(&terms[i - 1]) {
                        let new_map = HashMap::new();
                        map = Some(new_map);
                    }

                    if let Some(ref mut m) = map {
                        let key = enum_key(term);
                        if let std::collections::hash_map::Entry::Vacant(e) = m.entry(key) {
                            e.insert(term.clone());
                            continue;
                        }
                    }
                }

                if let Some(m) = map.take() {
                    result = create_condition(
                        MatchNode::Enum { map: m },
                        MatchNode::Match,
                        result,
                    );
                }

                result = create_condition(term.clone(), MatchNode::Match, result);
            }

            if let Some(m) = map.take() {
                result = create_condition(
                    MatchNode::Enum { map: m },
                    MatchNode::Match,
                    result,
                );
            }

            result
        }
        Combinator::DoubleAmpersand => {
            // All required, any order
            if terms.len() > 5 {
                MatchNode::MatchOnce { terms, all: true }
            } else {
                build_permutation_tree(terms, true)
            }
        }
        Combinator::DoubleBar => {
            // At least one required, any order
            if terms.len() > 5 {
                MatchNode::MatchOnce { terms, all: false }
            } else {
                build_permutation_tree(terms, false)
            }
        }
    }
}

/// Check if a node can be part of an Enum optimization.
fn is_enum_compatible(node: &MatchNode) -> bool {
    matches!(
        node,
        MatchNode::Keyword { .. }
            | MatchNode::AtKeyword { .. }
            | MatchNode::Function { .. }
    )
}

/// Get the enum key for a node.
fn enum_key(node: &MatchNode) -> String {
    match node {
        MatchNode::Keyword { name }
        | MatchNode::AtKeyword { name }
        | MatchNode::Function { name } => name.to_lowercase(),
        _ => String::new(),
    }
}

/// Build a permutation tree for && or || combinators.
fn build_permutation_tree(terms: Vec<MatchNode>, all_required: bool) -> MatchNode {
    if terms.len() == 1 {
        return terms.into_iter().next().unwrap();
    }

    // For small term counts, build a combination tree
    let mut result = MatchNode::Mismatch;
    for (i, term) in terms.iter().enumerate().rev() {
        let remaining: Vec<MatchNode> = terms
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .map(|(_, t)| t.clone())
            .collect();

        let then_branch = if remaining.is_empty() {
            MatchNode::Match
        } else {
            build_permutation_tree(remaining, all_required)
        };

        result = create_condition(term.clone(), then_branch, result);
    }

    if !all_required {
        // For ||, also allow matching without consuming all terms
        result = create_condition(result, MatchNode::Match, MatchNode::Match);
    }

    result
}

/// Build match graph for a multiplied term.
fn build_multiplied(mult: &MultiplierNode) -> MatchNode {
    let term = build_node(&mult.term);

    if mult.min == 0 && mult.max == 1 {
        // ? = optional
        return create_condition(term, MatchNode::Match, MatchNode::Match);
    }

    if mult.min == 1 && mult.max == 1 {
        // Exactly once (no multiplier effect)
        return term;
    }

    // For other multipliers, build a loop structure
    // This is simplified — the full JS implementation is more complex
    let comma_sep = mult.comma.then_some(MatchNode::Comma);

    // Build: match term, then optionally match (comma + term) repeatedly
    let repeat = if let Some(comma) = comma_sep {
        create_condition(
            comma,
            create_condition(term.clone(), MatchNode::DisallowEmpty, MatchNode::Mismatch),
            MatchNode::Match,
        )
    } else {
        create_condition(term.clone(), MatchNode::DisallowEmpty, MatchNode::Match)
    };

    if mult.min == 0 {
        // * or #? = zero or more
        create_condition(term, repeat, MatchNode::Match)
    } else {
        // + or # = one or more
        create_condition(term, repeat, MatchNode::Mismatch)
    }
}

/// Create a conditional branch, with optimization to reduce node count.
fn create_condition(
    condition: MatchNode,
    then_branch: MatchNode,
    else_branch: MatchNode,
) -> MatchNode {
    // Optimization: if then=MATCH and else=MISMATCH, just return the condition
    if matches!(then_branch, MatchNode::Match) && matches!(else_branch, MatchNode::Mismatch) {
        return condition;
    }

    // Optimization: if all branches are MATCH, just return MATCH
    if matches!(condition, MatchNode::Match)
        && matches!(then_branch, MatchNode::Match)
        && matches!(else_branch, MatchNode::Match)
    {
        return MatchNode::Match;
    }

    MatchNode::If {
        condition: Box::new(condition),
        then_branch: Box::new(then_branch),
        else_branch: Box::new(else_branch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition_syntax::parse::parse;

    #[test]
    fn build_single_keyword() {
        let ast = parse("auto").unwrap();
        let graph = build_match_graph(&ast);
        assert!(matches!(graph, MatchNode::Keyword { .. }));
    }

    #[test]
    fn build_bar_combinator() {
        let ast = parse("auto | none").unwrap();
        let graph = build_match_graph(&ast);
        // Should create an If or Enum node
        assert!(!matches!(graph, MatchNode::Match | MatchNode::Mismatch));
    }

    #[test]
    fn build_type_reference() {
        let ast = parse("<length>").unwrap();
        let graph = build_match_graph(&ast);
        assert!(matches!(graph, MatchNode::Type { .. }));
    }

    #[test]
    fn build_property_reference() {
        let ast = parse("<'color'>").unwrap();
        let graph = build_match_graph(&ast);
        assert!(matches!(graph, MatchNode::Property { .. }));
    }

    #[test]
    fn build_multiplier_optional() {
        let ast = parse("auto?").unwrap();
        let graph = build_match_graph(&ast);
        // ? creates an If with both branches going to Match
        assert!(matches!(graph, MatchNode::If { .. }));
    }

    #[test]
    fn build_space_combinator() {
        let ast = parse("a b c").unwrap();
        let graph = build_match_graph(&ast);
        // Sequential: If { a, If { b, If { c, Match, Mismatch }, Mismatch }, Mismatch }
        assert!(matches!(graph, MatchNode::If { .. }));
    }

    #[test]
    fn build_enum_optimization() {
        let ast = parse("auto | none | inherit").unwrap();
        let graph = build_match_graph(&ast);
        // Multiple keywords should get optimized into an Enum
        // Check the graph contains an Enum somewhere
        fn has_enum(node: &MatchNode) -> bool {
            match node {
                MatchNode::Enum { .. } => true,
                MatchNode::If { condition, then_branch, else_branch } => {
                    has_enum(condition) || has_enum(then_branch) || has_enum(else_branch)
                }
                _ => false,
            }
        }
        assert!(has_enum(&graph));
    }
}
