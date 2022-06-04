use crate::{ASTNode, EarleyParser, Lexer, ParserError, Token};
use std::collections::HashMap;

fn ebnf_lexer() -> Lexer<&'static str> {
    let mut lexer = Lexer::new();

    let tokens = ["=", ":=", "::=", "|", "[", "]", "{", "}", "(", ")", ";"];

    for token in tokens {
        lexer.add_token(token, token.to_string());
    }

    // terms
    lexer.add_lexer(|source, start| {
        let source_substr = &source[start..];

        let first_char = source_substr.chars().next().unwrap();

        if first_char != '"' && first_char != '\'' {
            return ("term", 0);
        }

        let string_length = &source_substr[1..]
            .find(&['\r', '\n', '"', '\''])
            .unwrap_or(0);

        let last_char = source_substr.chars().nth(string_length + 1);

        if let Some(last_char) = last_char {
            if last_char != '"' && last_char != '\'' {
                return ("term", 0);
            }
        } else {
            return ("term", 0);
        }

        ("term", string_length + 2)
    });

    lexer.add_lexer(|source, start| {
        if !source.chars().nth(start).unwrap().is_alphabetic() {
            return ("non_term", 0);
        }

        (
            "non_term",
            source
                .chars()
                .skip(start + 1)
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .count()
                + 1,
        )
    });

    // comments
    lexer.add_ignorer(|source, skip| {
        if !source[skip..].starts_with("(*") {
            return 0;
        }

        source[skip + 2..]
            .find("*)")
            .map(|len| len + 4)
            .unwrap_or(source.len() - skip)
    });

    // removing whitespace
    lexer.add_ignorer(|source, start| {
        source
            .chars()
            .skip(start)
            .take_while(|c| c.is_whitespace())
            .count()
    });

    lexer
}

fn ebnf_parser() -> EarleyParser<&'static str> {
    let mut parser = EarleyParser::new("grammar");

    // https://en.wikipedia.org/wiki/Extended_Backus%E2%80%93Naur_form

    // derived from the symbol table
    parser.add_rules("definition", [["::="], [":="], ["="]]);
    parser.add_rules("optional", [["[", "rhs", "]"]]);
    parser.add_rules("repetition", [["{", "rhs", "}"]]);
    parser.add_rules("group", [["(", "rhs", ")"]]);
    parser.add_rules("alternation", [["rhs", "|", "rhs"]]);
    parser.add_rules("concatination", [["rhs", "rhs"]]);

    parser.add_rules("grammar", [vec!["rule"], vec!["rule", "grammar"]]);
    parser.add_rules("rule", [["non_term", "definition", "rhs", ";"]]);
    parser.add_rules(
        "rhs",
        [
            ["non_term"],
            ["term"],
            ["optional"],
            ["repetition"],
            ["group"],
            ["alternation"],
            ["concatination"],
        ],
    );

    parser
}
pub struct RHSParser<'a> {
    append_rule: bool,
    rule_label: &'a str,
    children: Vec<ASTNode<'a, &'a str>>,
    first_run: bool,
}

impl<'a> RHSParser<'a> {
    pub fn new(
        append_rule: bool,
        rule_label: &'a str,
        mut children: Vec<ASTNode<'a, &'a str>>,
    ) -> Self {
        // reverse so we can pop and not need to shift the entire array
        children.reverse();

        Self {
            append_rule,
            rule_label,
            children,
            first_run: true,
        }
    }

    pub fn append_token(
        &self,
        rules_map: &mut HashMap<&'a str, Vec<Vec<&'a str>>>,
        ident: &'a str,
    ) {
        let rules = rules_map.get_mut(&self.rule_label).unwrap();

        rules.last_mut().unwrap().push(ident);
    }

    // empty vec = done
    pub fn parse(
        &mut self,
        ebnf: &'a str,
        rules_map: &mut HashMap<&'a str, Vec<Vec<&'a str>>>,
    ) -> Vec<Self> {
        if self.first_run {
            self.first_run = false;

            if self.append_rule {
                rules_map
                    .get_mut(&self.rule_label)
                    .unwrap_or_else(|| {
                        panic!(
                            "Rule {:?} has not been defined, yet the parser for it has?",
                            &self.rule_label
                        )
                    })
                    .push(Vec::new());
            }
        }

        loop {
            let mut node = match self.children.pop() {
                Some(node) => node,
                None => return Vec::new(),
            };

            let node_start = node.start();
            let node_end = node.end();

            match &mut node {
                ASTNode::Leaf { token } => match token.label {
                    "term" => {
                        // strip quotes
                        let ident = &token.content[1..token.content.len() - 1];

                        self.append_token(rules_map, ident);
                    }
                    "non_term" => {
                        // no modification
                        let ident = token.content;

                        self.append_token(rules_map, ident);
                    }
                    _ => unreachable!(),
                },
                ASTNode::Branch { label, children } => match *label {
                    "optional" => {
                        // create a new rule, prefix with empty rule to always pass
                        // generating the rule name based on the entire string
                        let new_rule_name = &ebnf[node_start..node_end];

                        // add this new rule as a requirement for the current rule
                        self.append_token(rules_map, new_rule_name);

                        // create the new rule
                        rules_map.insert(new_rule_name, vec![vec![]]);

                        return vec![RHSParser::new(
                            true,
                            new_rule_name,
                            children.swap_remove(1).into_children().unwrap(),
                        )];
                    }
                    "repetition" => {
                        // create a rule for the repetition rule
                        let repetition_name = &ebnf[node_start..node_end];
                        // create a rule for the rhs
                        let new_rule_name = &ebnf[node_start..node_end - 1];

                        // add the repetition rule as a requirement for the current rule
                        self.append_token(rules_map, repetition_name);

                        // create the repetition rule
                        rules_map.insert(
                            repetition_name,
                            vec![vec![], vec![repetition_name, new_rule_name]],
                        );

                        // create the rule for the rhs
                        rules_map.insert(new_rule_name, vec![]);

                        return vec![RHSParser::new(
                            true,
                            new_rule_name,
                            children.swap_remove(1).into_children().unwrap(),
                        )];
                    }
                    "group" => {
                        // create a new rule, so alternation works on this rule and not the parent one
                        // generating the rule name based on the entire string
                        let new_rule_name = &ebnf[node_start..node_end];

                        // add this new rule as a requirement for the current rule
                        self.append_token(rules_map, new_rule_name);

                        // create the new rule
                        rules_map.insert(new_rule_name, vec![]);

                        return vec![RHSParser::new(
                            true,
                            new_rule_name,
                            children.swap_remove(1).into_children().unwrap(),
                        )];
                    }
                    "alternation" => {
                        // append a new alternative rule
                        let label = self.rule_label;

                        let right_children = children.swap_remove(2).into_children().unwrap();
                        let left_children = children.swap_remove(0).into_children().unwrap();

                        return vec![
                            // parse the left side with the existing rule
                            RHSParser::new(false, label, left_children),
                            // create a new rule for the right side
                            RHSParser::new(true, label, right_children),
                        ];
                    }
                    "concatination" => {
                        let label = self.rule_label;

                        let right_children = children.pop().unwrap().into_children().unwrap();
                        let left_children = children.pop().unwrap().into_children().unwrap();

                        // create a parser for both sides, reusing the same rule
                        return vec![
                            RHSParser::new(false, label, left_children),
                            RHSParser::new(false, label, right_children),
                        ];
                    }
                    _ => unreachable!(),
                },
            }
        }
    }
}

fn parse_ebnf(ebnf: &str) -> Result<HashMap<&str, Vec<Vec<&str>>>, ParserError<&str>> {
    let tokens = ebnf_lexer().analyze(ebnf)?;
    let ast = ebnf_parser().parse(ebnf, &tokens)?;
    let mut next_grammar_node = Some(ast);

    let mut rules_map = HashMap::new();
    let mut rhs_parsers = Vec::new();

    while let Some(grammar_node) = next_grammar_node {
        next_grammar_node = None;

        for node in grammar_node.into_children().unwrap() {
            match node.label() {
                "rule" => {
                    let mut nodes = node.into_children().unwrap();
                    let rhs_node = nodes.swap_remove(2);
                    let non_term_node = nodes.swap_remove(0);

                    let rule_label = non_term_node.token().unwrap().content;

                    rules_map.insert(rule_label, Vec::new());

                    rhs_parsers.push(RHSParser::new(
                        true,
                        rule_label,
                        rhs_node.into_children().unwrap(),
                    ));
                }
                "grammar" => next_grammar_node = Some(node),
                _ => unreachable!(),
            }
        }
    }

    while let Some(rhs_parser) = rhs_parsers.last_mut() {
        let new_parsers = rhs_parser.parse(ebnf, &mut rules_map);

        if new_parsers.is_empty() {
            rhs_parsers.pop();
        } else {
            rhs_parsers.extend(new_parsers.into_iter().rev());
        }
    }

    Ok(rules_map)
}

/// Adds rules to an existing parser.
pub fn apply_ebnf<'a>(
    parser: &mut EarleyParser<&'a str>,
    ebnf: &'a str,
) -> Result<(), ParserError<'a, &'a str>> {
    let rules_map = parse_ebnf(ebnf)?;

    for (label, rules) in rules_map {
        parser.add_rules(label, rules);

        // ebnf non_terms can't contain these special characters
        let assume_generated =
            // {.*}
            label.starts_with('{')
            // [.*]
            || label.starts_with('[')
            // (.*)
            || label.starts_with('(');

        if assume_generated {
            parser.hide_rule(label);
        }
    }

    Ok(())
}

/// Creates a parser using rules defined in EBNF.
pub struct EBNFParser<'a> {
    parser: EarleyParser<&'a str>,
}

impl<'a> EBNFParser<'a> {
    pub fn new(ebnf: &'a str, entry: &'a str) -> Self {
        let mut parser = EarleyParser::new(entry);
        apply_ebnf(&mut parser, ebnf).unwrap();

        Self { parser }
    }

    pub fn parse<'b>(
        &self,
        source: &'b str,
        tokens: &[Token<'b, &'a str>],
    ) -> Result<ASTNode<'b, &'a str>, ParserError<'b, &'a str>> {
        self.parser.parse(source, tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESERVED_WORDS: [&'static str; 12] = [
        "const",
        "var",
        "procedure",
        "block",
        "call",
        "begin",
        "end",
        "if",
        "then",
        "while",
        "do",
        "let",
    ];

    fn pl0_lexer<'a>() -> Lexer<&'a str> {
        let mut lexer = Lexer::new();

        let tokens = [
            "(", ")", "*", "/", "+", "-", // math
            "=", "#", "<", "<=", ">", ">=", // comparison
            "?", "!", ":=", ",", ";", ".", // structure
        ];

        for token in tokens {
            lexer.add_token(token, token.to_string());
        }

        lexer.add_lexer(|source, start| {
            RESERVED_WORDS
                .iter()
                .filter(|word| source[start..].len() >= word.len())
                .find(|word| &source[start..start + word.len()].to_lowercase() == *word)
                .map(|word| (*word, word.len()))
                .unwrap_or(("", 0))
        });

        lexer.add_lexer(|source, start| {
            if !source.chars().nth(start).unwrap().is_alphabetic() {
                return ("ident", 0);
            }

            (
                "ident",
                source
                    .chars()
                    .skip(start + 1)
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .count()
                    + 1,
            )
        });

        lexer.add_lexer(|source, start| {
            (
                "number",
                source
                    .chars()
                    .skip(start)
                    .take_while(|c| c.is_numeric())
                    .count(),
            )
        });

        // whitespace
        lexer.add_ignorer(|source, start| {
            source
                .chars()
                .skip(start)
                .take_while(|c| c.is_whitespace())
                .count()
        });

        lexer
    }

    #[test]
    fn pl0() {
        let pl0_ebnf = include_str!("../tests/pl0/pl0.ebnf");
        let pl0_source = include_str!("../tests/pl0/pl0-sample.pl0");
        let pl0_expected = include_str!("../tests/pl0/pl0-expected.txt");

        let parser = EBNFParser::new(pl0_ebnf, "program");

        let tokens = pl0_lexer().analyze(pl0_source).unwrap();

        let ast = parser.parse(pl0_source, &tokens).unwrap();

        let mut labels = Vec::new();

        ast.walk(|node, path| {
            let padding = path.len() * 2;
            labels.push(" ".repeat(padding) + node.label());
        });

        let output: String = labels.join("\n");
        assert_eq!(&output, &pl0_expected);
    }
}
