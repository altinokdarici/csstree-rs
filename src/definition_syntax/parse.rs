//! Parser for CSS Value Definition Syntax strings.

use super::error::DefinitionSyntaxError;
use super::scanner::Scanner;
#[allow(clippy::wildcard_imports)] // All definition syntax types needed for node construction
use super::types::*;

type Result<T> = std::result::Result<T, DefinitionSyntaxError>;

// Character constants
const EXCLAMATIONMARK: u8 = b'!';
const NUMBERSIGN: u8 = b'#';
const AMPERSAND: u8 = b'&';
const APOSTROPHE: u8 = b'\'';
const LEFTPARENTHESIS: u8 = b'(';
const RIGHTPARENTHESIS: u8 = b')';
const ASTERISK: u8 = b'*';
const PLUSSIGN: u8 = b'+';
const COMMA: u8 = b',';
const HYPERMINUS: u8 = b'-';
const LESSTHANSIGN: u8 = b'<';
const GREATERTHANSIGN: u8 = b'>';
const QUESTIONMARK: u8 = b'?';
const COMMERCIALAT: u8 = b'@';
const LEFTSQUAREBRACKET: u8 = b'[';
const RIGHTSQUAREBRACKET: u8 = b']';
const LEFTCURLYBRACKET: u8 = b'{';
const VERTICALLINE: u8 = b'|';
const RIGHTCURLYBRACKET: u8 = b'}';


/// Internal combinator token used during parsing (before regrouping).
#[derive(Debug, Clone)]
enum ParseToken {
    Node(DefinitionSyntaxNode),
    Combinator(String),
    Spaces,
}

/// Read `{min,max}` multiplier range.
fn read_multiplier_range(scanner: &mut Scanner) -> Result<(u32, u32)> {
    scanner.eat(LEFTCURLYBRACKET)?;
    scanner.skip_ws();

    let min_str = scanner.scan_number()?;
    let min: u32 = min_str.parse().unwrap_or(0);
    scanner.skip_ws();

    let max = if scanner.char_code() == COMMA {
        scanner.pos += 1;
        scanner.skip_ws();

        if scanner.char_code() == RIGHTCURLYBRACKET {
            0 // no max = unlimited
        } else {
            let max_str = scanner.scan_number()?;
            scanner.skip_ws();
            max_str.parse().unwrap_or(0)
        }
    } else {
        min // no comma = exact count
    };

    scanner.eat(RIGHTCURLYBRACKET)?;
    Ok((min, max))
}

/// Read a multiplier suffix (*, +, ?, #, {n,m}).
fn read_multiplier(scanner: &mut Scanner) -> Result<Option<MultiplierNode>> {
    let mut comma = false;

    let range = match scanner.char_code() {
        ASTERISK => {
            scanner.pos += 1;
            Some((0, 0))
        }
        PLUSSIGN => {
            scanner.pos += 1;
            Some((1, 0))
        }
        QUESTIONMARK => {
            scanner.pos += 1;
            Some((0, 1))
        }
        NUMBERSIGN => {
            scanner.pos += 1;
            comma = true;

            if scanner.char_code() == LEFTCURLYBRACKET {
                Some(read_multiplier_range(scanner)?)
            } else if scanner.char_code() == QUESTIONMARK {
                // #? = zero or more, comma-separated
                scanner.pos += 1;
                Some((0, 0))
            } else {
                Some((1, 0))
            }
        }
        LEFTCURLYBRACKET => Some(read_multiplier_range(scanner)?),
        _ => None,
    };

    match range {
        Some((min, max)) => Ok(Some(MultiplierNode {
            term: Box::new(DefinitionSyntaxNode::Comma), // placeholder, set by caller
            min,
            max,
            comma,
        })),
        None => Ok(None),
    }
}

/// Wrap a node in a multiplier if one follows.
fn maybe_multiplied(scanner: &mut Scanner, node: DefinitionSyntaxNode) -> Result<DefinitionSyntaxNode> {
    if let Some(mut mult) = read_multiplier(scanner)? {
        mult.term = Box::new(node);

        // +# stacking
        if scanner.char_code() == NUMBERSIGN
            && scanner.char_code_at(scanner.pos - 1) == PLUSSIGN
        {
            return maybe_multiplied(scanner, DefinitionSyntaxNode::Multiplied(mult));
        }

        // {}? stacking
        if scanner.char_code() == QUESTIONMARK
            && scanner.char_code_at(scanner.pos - 1) == RIGHTCURLYBRACKET
        {
            return maybe_multiplied(scanner, DefinitionSyntaxNode::Multiplied(mult));
        }

        Ok(DefinitionSyntaxNode::Multiplied(mult))
    } else {
        Ok(node)
    }
}

/// Read a single-character token, possibly with multiplier.
fn maybe_token(scanner: &mut Scanner) -> Result<Option<DefinitionSyntaxNode>> {
    match scanner.peek_char() {
        Some(ch) => {
            let node = DefinitionSyntaxNode::Token(TokenNode {
                value: String::from(ch as char),
            });
            Ok(Some(maybe_multiplied(scanner, node)?))
        }
        None => Ok(None),
    }
}

/// Read `<'property-name'>`.
fn read_property(scanner: &mut Scanner) -> Result<DefinitionSyntaxNode> {
    scanner.eat(LESSTHANSIGN)?;
    scanner.eat(APOSTROPHE)?;
    let name = scanner.scan_word()?;
    scanner.eat(APOSTROPHE)?;
    scanner.eat(GREATERTHANSIGN)?;

    let node = DefinitionSyntaxNode::Property(PropertyNode { name });
    maybe_multiplied(scanner, node)
}

/// Check if the byte sequence at scanner pos is the infinity symbol (∞ = U+221E = E2 88 9E).
fn is_infinity(scanner: &Scanner) -> bool {
    let p = scanner.pos;
    scanner.char_code_at(p) == 0xE2
        && scanner.char_code_at(p + 1) == 0x88
        && scanner.char_code_at(p + 2) == 0x9E
}

/// Read `[min, max]` range notation inside a type.
fn read_type_range(scanner: &mut Scanner) -> Result<RangeNode> {
    let mut min: Option<f64> = None;
    let mut max: Option<f64> = None;

    scanner.eat(LEFTSQUAREBRACKET)?;

    // Read min
    let mut sign: f64 = 1.0;
    if scanner.char_code() == HYPERMINUS {
        scanner.pos += 1;
        sign = -1.0;
    }

    if sign < 0.0 && is_infinity(scanner) {
        scanner.pos += 3; // skip ∞ (3 bytes UTF-8)
    } else {
        let num_str = scanner.scan_number()?;
        let val = sign * num_str.parse::<f64>().unwrap_or(0.0);
        // Skip optional unit suffix (in JS, number + unit are concatenated as string)
        if scanner.is_name_char_code(scanner.char_code()) {
            let _unit = scanner.scan_word()?;
        }
        min = Some(val);
    }

    scanner.skip_ws();
    scanner.eat(COMMA)?;
    scanner.skip_ws();

    // Read max
    if is_infinity(scanner) {
        scanner.pos += 3;
    } else {
        sign = 1.0;
        if scanner.char_code() == HYPERMINUS {
            scanner.pos += 1;
            sign = -1.0;
        }

        let num_str = scanner.scan_number()?;
        let val = sign * num_str.parse::<f64>().unwrap_or(0.0);
        if scanner.is_name_char_code(scanner.char_code()) {
            let _unit = scanner.scan_word()?;
        }
        max = Some(val);
    }

    scanner.eat(RIGHTSQUAREBRACKET)?;

    Ok(RangeNode { min, max })
}

/// Read `<type>` or `<type[min,max]>` or `<boolean-expr[...]>`.
fn read_type(scanner: &mut Scanner) -> Result<DefinitionSyntaxNode> {
    scanner.eat(LESSTHANSIGN)?;
    let mut name = scanner.scan_word()?;

    // <boolean-expr[...]>
    if name == "boolean-expr" {
        scanner.eat(LEFTSQUAREBRACKET)?;
        let group = read_implicit_group(scanner, Some(RIGHTSQUAREBRACKET))?;
        scanner.eat(RIGHTSQUAREBRACKET)?;
        scanner.eat(GREATERTHANSIGN)?;

        let term = if group.terms.len() == 1 {
            group.terms.into_iter().next().unwrap()
        } else {
            DefinitionSyntaxNode::Group(group)
        };

        return maybe_multiplied(scanner, DefinitionSyntaxNode::Boolean(BooleanNode {
            term: Box::new(term),
        }));
    }

    // Parameterized type like <calc()>
    if scanner.char_code() == LEFTPARENTHESIS && scanner.next_char_code() == RIGHTPARENTHESIS {
        scanner.pos += 2;
        name.push_str("()");
    }

    // Optional range: <integer[min,max]>
    let opts = if scanner.char_code_at(scanner.find_ws_end(scanner.pos)) == LEFTSQUAREBRACKET {
        scanner.skip_ws();
        Some(read_type_range(scanner)?)
    } else {
        None
    };

    scanner.eat(GREATERTHANSIGN)?;

    let node = DefinitionSyntaxNode::Type(TypeNode { name, opts });
    maybe_multiplied(scanner, node)
}

/// Read a keyword or function call.
fn read_keyword_or_function(scanner: &mut Scanner) -> Result<DefinitionSyntaxNode> {
    let name = scanner.scan_word()?;

    if scanner.char_code() == LEFTPARENTHESIS {
        scanner.pos += 1;
        Ok(DefinitionSyntaxNode::Function(FunctionNode { name }))
    } else {
        let node = DefinitionSyntaxNode::Keyword(KeywordNode { name });
        maybe_multiplied(scanner, node)
    }
}

/// Regroup a flat list of terms and combinator tokens into nested groups.
///
/// Modifies `terms` in place, returns the final combinator.
fn regroup_terms(terms: &mut Vec<DefinitionSyntaxNode>, combinators: &mut Vec<String>) -> Option<Combinator> {
    fn combinator_from_str(s: &str) -> Combinator {
        match s {
            "&&" => Combinator::DoubleAmpersand,
            "||" => Combinator::DoubleBar,
            "|" => Combinator::Bar,
            _ => Combinator::Space,
        }
    }

    fn precedence(s: &str) -> u8 {
        combinator_from_str(s).precedence()
    }

    // Sort combinators by precedence (tightest first)
    combinators.sort_by_key(|s| precedence(s));
    combinators.dedup();

    let mut last_combinator = None;

    while let Some(combinator) = combinators.first().cloned() {
        combinators.remove(0);

        let mut i = 0;
        let mut subgroup_start: Option<usize> = None;

        while i < terms.len() {
            // Check if this term is a combinator placeholder
            let is_combinator_match = matches!(&terms[i], DefinitionSyntaxNode::Token(t) if t.value.starts_with('\x01') && t.value[1..] == combinator);

            if is_combinator_match {
                if subgroup_start.is_none() {
                    subgroup_start = Some(if i > 0 { i - 1 } else { 0 });
                }
                terms.remove(i);
                // don't increment i
            } else {
                let is_other_combinator = matches!(&terms[i], DefinitionSyntaxNode::Token(t) if t.value.starts_with('\x01'));
                if is_other_combinator {
                    if let Some(start) = subgroup_start {
                        if i - start > 1 {
                            let sub_terms: Vec<_> = terms.drain(start..i).collect();
                            let group = DefinitionSyntaxNode::Group(GroupNode {
                                terms: sub_terms,
                                combinator: combinator_from_str(&combinator),
                                explicit: false,
                                disallow_empty: false,
                            });
                            terms.insert(start, group);
                            i = start + 1;
                        }
                    }
                    subgroup_start = None;
                }
                i += 1;
            }
        }

        if let Some(start) = subgroup_start {
            if !combinators.is_empty() && i - start > 1 {
                let sub_terms: Vec<_> = terms.drain(start..i).collect();
                let group = DefinitionSyntaxNode::Group(GroupNode {
                    terms: sub_terms,
                    combinator: combinator_from_str(&combinator),
                    explicit: false,
                    disallow_empty: false,
                });
                terms.insert(start, group);
            }
        }

        last_combinator = Some(combinator_from_str(&combinator));
    }

    last_combinator
}

/// Read an implicit group of terms until stop char.
fn read_implicit_group(scanner: &mut Scanner, stop: Option<u8>) -> Result<GroupNode> {
    let stop_code = stop.unwrap_or(0);
    let mut combinators_set: Vec<String> = Vec::new();
    let mut terms: Vec<DefinitionSyntaxNode> = Vec::new();
    // Use internal combinator markers: \x01 prefix
    let mut prev_token_type: Option<String> = None;
    let mut prev_token_pos = scanner.pos;
    let mut prev_is_function = false;

    while !scanner.is_eof() && (stop.is_none() || scanner.char_code() != stop_code) {
        let token = if prev_is_function {
            let group = read_implicit_group(scanner, Some(RIGHTPARENTHESIS))?;
            if scanner.char_code() == RIGHTPARENTHESIS {
                scanner.pos += 1;
            }
            Some(ParseToken::Node(DefinitionSyntaxNode::Group(group)))
        } else {
            peek_token(scanner)?
        };

        let Some(token) = token else {
            break;
        };

        if matches!(&token, ParseToken::Spaces) {
            prev_is_function = false;
            continue;
        }

        if prev_is_function {
            if let ParseToken::Node(DefinitionSyntaxNode::Group(ref group)) = token {
                if group.terms.is_empty() {
                    prev_is_function = false;
                    continue;
                }

                // Flatten space-combinator groups from function args
                if group.combinator == Combinator::Space && group.terms.len() > 1 {
                    let ParseToken::Node(DefinitionSyntaxNode::Group(mut group)) = token else {
                        unreachable!()
                    };
                    if !combinators_set.contains(&" ".to_string()) {
                        combinators_set.push(" ".to_string());
                    }
                    while group.terms.len() > 1 {
                        // Insert space combinator marker + first term
                        terms.push(DefinitionSyntaxNode::Token(TokenNode {
                            value: "\x01 ".to_string(),
                        }));
                        terms.push(group.terms.remove(0));
                    }
                    let last = group.terms.remove(0);
                    prev_token_type = Some(last.node_type().to_string());
                    prev_token_pos = scanner.pos;
                    prev_is_function = last.node_type() == "Function";
                    terms.push(last);
                    continue;
                }
            }
        }

        match token {
            ParseToken::Combinator(ref value) => {
                if prev_token_type.is_none() || prev_token_type.as_deref() == Some("Combinator") {
                    scanner.pos = prev_token_pos;
                    return Err(scanner.error("Unexpected combinator"));
                }
                if !combinators_set.contains(value) {
                    combinators_set.push(value.clone());
                }
                terms.push(DefinitionSyntaxNode::Token(TokenNode {
                    value: format!("\x01{value}"),
                }));
                prev_token_type = Some("Combinator".to_string());
            }
            ParseToken::Node(node) => {
                if prev_token_type.is_some() && prev_token_type.as_deref() != Some("Combinator") {
                    // Implicit space combinator
                    if !combinators_set.contains(&" ".to_string()) {
                        combinators_set.push(" ".to_string());
                    }
                    terms.push(DefinitionSyntaxNode::Token(TokenNode {
                        value: "\x01 ".to_string(),
                    }));
                }
                prev_is_function = node.node_type() == "Function";
                prev_token_type = Some(node.node_type().to_string());
                terms.push(node);
            }
            ParseToken::Spaces => unreachable!(),
        }
        prev_token_pos = scanner.pos;
    }

    // Check for trailing combinator
    if prev_token_type.as_deref() == Some("Combinator") {
        return Err(scanner.error("Unexpected combinator"));
    }

    let combinator = regroup_terms(&mut terms, &mut combinators_set).unwrap_or(Combinator::Space);

    Ok(GroupNode {
        terms,
        combinator,
        explicit: false,
        disallow_empty: false,
    })
}

/// Read an explicit group `[...]`, optionally with `!`.
fn read_group(scanner: &mut Scanner) -> Result<DefinitionSyntaxNode> {
    scanner.eat(LEFTSQUAREBRACKET)?;
    let mut group = read_implicit_group(scanner, Some(RIGHTSQUAREBRACKET))?;
    scanner.eat(RIGHTSQUAREBRACKET)?;

    group.explicit = true;

    if scanner.char_code() == EXCLAMATIONMARK {
        scanner.pos += 1;
        group.disallow_empty = true;
    }

    let node = DefinitionSyntaxNode::Group(group);
    maybe_multiplied(scanner, node)
}

/// Peek at the next token from the scanner.
fn peek_token(scanner: &mut Scanner) -> Result<Option<ParseToken>> {
    let code = scanner.char_code();

    match code {
        RIGHTSQUAREBRACKET => Ok(None),

        LEFTSQUAREBRACKET => {
            let node = read_group(scanner)?;
            Ok(Some(ParseToken::Node(node)))
        }

        LESSTHANSIGN => {
            let node = if scanner.next_char_code() == APOSTROPHE {
                read_property(scanner)?
            } else {
                read_type(scanner)?
            };
            Ok(Some(ParseToken::Node(node)))
        }

        VERTICALLINE => {
            let end = scanner.pos + if scanner.next_char_code() == VERTICALLINE { 2 } else { 1 };
            let value = scanner.substring_to_pos(end);
            Ok(Some(ParseToken::Combinator(value)))
        }

        AMPERSAND => {
            scanner.pos += 1;
            scanner.eat(AMPERSAND)?;
            Ok(Some(ParseToken::Combinator("&&".to_string())))
        }

        COMMA => {
            scanner.pos += 1;
            Ok(Some(ParseToken::Node(DefinitionSyntaxNode::Comma)))
        }

        APOSTROPHE => {
            let value = scanner.scan_string()?;
            let node = DefinitionSyntaxNode::StringValue(StringValueNode { value });
            Ok(Some(ParseToken::Node(maybe_multiplied(scanner, node)?)))
        }

        b' ' | b'\t' | b'\n' | b'\r' | 12 => {
            scanner.skip_ws();
            Ok(Some(ParseToken::Spaces))
        }

        COMMERCIALAT => {
            let next = scanner.next_char_code();
            if scanner.is_name_char_code(next) {
                scanner.pos += 1;
                let name = scanner.scan_word()?;
                Ok(Some(ParseToken::Node(DefinitionSyntaxNode::AtKeyword(AtKeywordNode { name }))))
            } else {
                Ok(maybe_token(scanner)?.map(ParseToken::Node))
            }
        }

        ASTERISK | PLUSSIGN | QUESTIONMARK | NUMBERSIGN | EXCLAMATIONMARK => {
            // Prohibited tokens (used as multiplier starts)
            Ok(None)
        }

        LEFTCURLYBRACKET => {
            let next = scanner.next_char_code();
            if next.is_ascii_digit() {
                Ok(None)
            } else {
                Ok(maybe_token(scanner)?.map(ParseToken::Node))
            }
        }

        _ => {
            if scanner.is_name_char_code(code) {
                let node = read_keyword_or_function(scanner)?;
                Ok(Some(ParseToken::Node(node)))
            } else {
                Ok(maybe_token(scanner)?.map(ParseToken::Node))
            }
        }
    }
}

/// Parse a CSS Value Definition Syntax string into an AST.
pub fn parse(source: &str) -> Result<DefinitionSyntaxNode> {
    let mut scanner = Scanner::new(source);
    let result = read_implicit_group(&mut scanner, None)?;

    if scanner.pos != scanner.len() {
        return Err(scanner.error("Unexpected input"));
    }

    // Reduce redundant groups with single group term
    if result.terms.len() == 1 {
        if let DefinitionSyntaxNode::Group(_) = &result.terms[0] {
            return Ok(result.terms.into_iter().next().unwrap());
        }
    }

    Ok(DefinitionSyntaxNode::Group(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_keyword() {
        let node = parse("auto").unwrap();
        assert_eq!(node.node_type(), "Group");
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.terms.len(), 1);
            assert_eq!(g.terms[0].node_type(), "Keyword");
        }
    }

    #[test]
    fn parse_type() {
        let node = parse("<length>").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.terms.len(), 1);
            if let DefinitionSyntaxNode::Type(t) = &g.terms[0] {
                assert_eq!(t.name, "length");
                assert!(t.opts.is_none());
            } else {
                panic!("Expected Type node");
            }
        }
    }

    #[test]
    fn parse_bar_combinator() {
        let node = parse("<length> | auto").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.combinator, Combinator::Bar);
            assert_eq!(g.terms.len(), 2);
        } else {
            panic!("Expected Group node");
        }
    }

    #[test]
    fn parse_double_ampersand() {
        let node = parse("a && b").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.combinator, Combinator::DoubleAmpersand);
            assert_eq!(g.terms.len(), 2);
        } else {
            panic!("Expected Group node");
        }
    }

    #[test]
    fn parse_double_bar() {
        let node = parse("a || b").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.combinator, Combinator::DoubleBar);
            assert_eq!(g.terms.len(), 2);
        } else {
            panic!("Expected Group node");
        }
    }

    #[test]
    fn parse_space_combinator() {
        let node = parse("a b").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.combinator, Combinator::Space);
            assert_eq!(g.terms.len(), 2);
        } else {
            panic!("Expected Group node");
        }
    }

    #[test]
    fn parse_multiplier_star() {
        let node = parse("a*").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert_eq!(g.terms.len(), 1);
            if let DefinitionSyntaxNode::Multiplied(m) = &g.terms[0] {
                assert_eq!(m.min, 0);
                assert_eq!(m.max, 0);
                assert!(!m.comma);
            }
        }
    }

    #[test]
    fn parse_multiplier_plus() {
        let node = parse("a+").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            if let DefinitionSyntaxNode::Multiplied(m) = &g.terms[0] {
                assert_eq!(m.min, 1);
                assert_eq!(m.max, 0);
            }
        }
    }

    #[test]
    fn parse_multiplier_question() {
        let node = parse("a?").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            if let DefinitionSyntaxNode::Multiplied(m) = &g.terms[0] {
                assert_eq!(m.min, 0);
                assert_eq!(m.max, 1);
            }
        }
    }

    #[test]
    fn parse_multiplier_hash() {
        let node = parse("a#").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            if let DefinitionSyntaxNode::Multiplied(m) = &g.terms[0] {
                assert_eq!(m.min, 1);
                assert_eq!(m.max, 0);
                assert!(m.comma);
            }
        }
    }

    #[test]
    fn parse_multiplier_range() {
        let node = parse("a{2,5}").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            if let DefinitionSyntaxNode::Multiplied(m) = &g.terms[0] {
                assert_eq!(m.min, 2);
                assert_eq!(m.max, 5);
            }
        }
    }

    #[test]
    fn parse_property_reference() {
        let node = parse("<'color'>").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            if let DefinitionSyntaxNode::Property(p) = &g.terms[0] {
                assert_eq!(p.name, "color");
            } else {
                panic!("Expected Property node");
            }
        }
    }

    #[test]
    fn parse_explicit_group() {
        let node = parse("[ a | b ]").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert!(g.explicit);
            assert_eq!(g.combinator, Combinator::Bar);
        }
    }

    #[test]
    fn parse_explicit_group_with_bang() {
        let node = parse("[ a b ]!").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            assert!(g.explicit);
            assert!(g.disallow_empty);
        }
    }

    #[test]
    fn parse_comma_node() {
        let node = parse("a , b").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            // a, comma, b with space combinators
            assert!(g.terms.len() >= 2);
        }
    }

    #[test]
    fn parse_error_double_combinator() {
        let result = parse("a | | b");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_leading_combinator() {
        let result = parse("| a");
        assert!(result.is_err());
    }

    #[test]
    fn parse_parameterized_type() {
        let node = parse("<calc()>").unwrap();
        if let DefinitionSyntaxNode::Group(g) = &node {
            if let DefinitionSyntaxNode::Type(t) = &g.terms[0] {
                assert_eq!(t.name, "calc()");
            }
        }
    }
}
