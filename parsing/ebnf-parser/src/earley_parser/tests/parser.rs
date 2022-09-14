use crate::*;

const MATH_TOKENS: [&'static str; 7] = ["(", ")", "+", "-", "/", "*", "="];

fn create_math_lexer() -> Lexer<&'static str> {
    let mut lexer = Lexer::new();

    for token in MATH_TOKENS {
        lexer.add_token(token, token.to_string());
    }

    // numbers
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
fn addition() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("addition");
    parser.add_rule("addition", ["number", "+", "number"]);

    let source = "5 + 3";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "addition",
            children: vec![
                ASTNode::new_leaf(tokens[0]),
                ASTNode::new_leaf(tokens[1]),
                ASTNode::new_leaf(tokens[2]),
            ]
        })
    );
}

#[test]
fn excess_tokens() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("start");
    parser.add_rules("start", [["number"]]);

    let source = "5 + 3";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Err(ParserError::UnexpectedToken {
            token: tokens[1],
            line: 1,
            col: 3
        })
    );
}

#[test]
fn unexpected_eof() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("start");
    parser.add_rules("start", [["number", "number"]]);

    let source = "5";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Err(ParserError::UnexpectedEOF)
    );
}

#[test]
fn recursive() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("expression");
    parser.add_rules("binary_op", [["+"], ["-"], ["*"], ["/"]]);
    parser.add_rules(
        "expression",
        [
            vec!["expression", "binary_op", "expression"],
            vec!["number"],
        ],
    );

    let source = "12 / 2 + 3";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "expression",
            children: vec![
                ASTNode::Branch {
                    label: "expression",
                    children: vec![
                        ASTNode::Branch {
                            label: "expression",
                            children: vec![
                                // number
                                ASTNode::new_leaf(tokens[0])
                            ]
                        },
                        ASTNode::Branch {
                            label: "binary_op",
                            children: vec![
                                // '/'
                                ASTNode::new_leaf(tokens[1])
                            ]
                        },
                        ASTNode::Branch {
                            label: "expression",
                            children: vec![
                                // number
                                ASTNode::new_leaf(tokens[2])
                            ]
                        },
                    ]
                },
                ASTNode::Branch {
                    label: "binary_op",
                    children: vec![
                        // '+'
                        ASTNode::new_leaf(tokens[3])
                    ]
                },
                ASTNode::Branch {
                    label: "expression",
                    children: vec![
                        // number
                        ASTNode::new_leaf(tokens[4])
                    ]
                },
            ]
        })
    )
}

#[test]
fn left_recursion() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("lr");
    parser.add_rules("lr", [vec!["+"], vec!["lr", "+"]]);
    parser.hide_rule("lr");

    let source = "+++";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "lr",
            children: vec![
                ASTNode::new_leaf(tokens[0]),
                ASTNode::new_leaf(tokens[1]),
                ASTNode::new_leaf(tokens[2])
            ]
        })
    )
}

#[test]
fn right_recursion() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("rr");
    parser.add_rules("rr", [vec![], vec!["+", "rr"]]);
    parser.hide_rule("rr");

    let source = "+++";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "rr",
            children: vec![
                ASTNode::new_leaf(tokens[0]),
                ASTNode::new_leaf(tokens[1]),
                ASTNode::new_leaf(tokens[2])
            ]
        })
    )
}

#[test]
fn left_and_right_recursion() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("lr");
    parser.add_rules("lr", [vec![""], vec!["+"], vec!["lr", "lr"]]);
    parser.hide_rule("lr");

    let source = "+++";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "lr",
            children: vec![
                ASTNode::new_leaf(tokens[0]),
                ASTNode::new_leaf(tokens[1]),
                ASTNode::new_leaf(tokens[2])
            ]
        })
    )
}

#[test]
fn optional() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("start");
    parser.add_rules("start", [["optional", "-"]]);
    parser.add_rules("optional", [vec!["number"], vec![]]);
    parser.hide_rule("optional");

    let source = "3 -";
    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "start",
            children: vec![ASTNode::new_leaf(tokens[0]), ASTNode::new_leaf(tokens[1])]
        })
    );

    let source = "-";
    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "start",
            children: vec![ASTNode::new_leaf(tokens[0])]
        })
    );
}

#[test]
fn repeating() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("start");
    parser.add_rules("start", [["numbers", "-"]]);
    parser.add_rules("numbers", [["repeating"]]);
    parser.add_rules(
        "repeating",
        [vec![], vec!["number", "repeating"], vec!["number"]],
    );
    parser.hide_rule("repeating");

    let source = "1 2 3 4 5 -";
    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "start",
            children: vec![
                ASTNode::Branch {
                    label: "numbers",
                    children: vec![
                        ASTNode::new_leaf(tokens[0]), // number
                        ASTNode::new_leaf(tokens[1]), // number
                        ASTNode::new_leaf(tokens[2]), // number
                        ASTNode::new_leaf(tokens[3]), // number
                        ASTNode::new_leaf(tokens[4]), // number
                    ]
                },
                ASTNode::new_leaf(tokens[5]), // -
            ],
        })
    );
}

#[test]
fn neighboring_zero_length_rules() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("start");
    parser.add_rules("start", [["optional", "optional", "optional"]]);
    parser.add_rules("optional", [vec![], vec!["-"]]);
    parser.hide_rule("optional");

    let source = "";
    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens),
        Ok(ASTNode::Branch {
            label: "start",
            children: vec![],
        })
    );
}

#[test]
fn operator_precedence() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("expression");
    parser.add_rules(
        "expression",
        [
            vec!["add_subtract"],
            vec!["multiply_divide"],
            vec!["number"],
        ],
    );
    parser.add_rules(
        "add_subtract",
        [
            vec!["expression", "+", "expression"],
            vec!["expression", "-", "expression"],
        ],
    );
    parser.add_rules(
        "multiply_divide",
        [
            vec!["expression", "*", "expression"],
            vec!["expression", "/", "expression"],
        ],
    );

    parser.hide_rule("add_subtract");
    parser.hide_rule("multiply_divide");

    let source = "2 + 4 * 3";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens).unwrap(),
        ASTNode::Branch {
            label: "expression",
            children: vec![
                ASTNode::Branch {
                    label: "expression",
                    children: vec![ASTNode::new_leaf(tokens[0])], // 2
                },
                ASTNode::new_leaf(tokens[1]), // +
                ASTNode::Branch {
                    label: "expression",
                    children: vec![
                        ASTNode::Branch {
                            label: "expression",
                            children: vec![ASTNode::new_leaf(tokens[2])], // 4
                        },
                        ASTNode::new_leaf(tokens[3]), // *
                        ASTNode::Branch {
                            label: "expression",
                            children: vec![ASTNode::new_leaf(tokens[4])], // 3
                        },
                    ],
                },
            ],
        }
    );
}

#[test]
fn operator_precedence_reversed() {
    let lexer = create_math_lexer();

    let mut parser = EarleyParser::new("expression");
    parser.add_rules(
        "expression",
        [
            vec!["multiply_divide"],
            vec!["add_subtract"],
            vec!["number"],
        ],
    );
    parser.add_rules(
        "multiply_divide",
        [
            vec!["expression", "*", "expression"],
            vec!["expression", "/", "expression"],
        ],
    );
    parser.add_rules(
        "add_subtract",
        [
            vec!["expression", "+", "expression"],
            vec!["expression", "-", "expression"],
        ],
    );

    parser.hide_rule("add_subtract");
    parser.hide_rule("multiply_divide");

    let source = "2 + 4 * 3";

    let tokens = lexer.analyze(source).unwrap();

    assert_eq!(
        parser.parse(source, &tokens).unwrap(),
        ASTNode::Branch {
            label: "expression",
            children: vec![
                ASTNode::Branch {
                    label: "expression",
                    children: vec![
                        ASTNode::Branch {
                            label: "expression",
                            children: vec![ASTNode::new_leaf(tokens[0])], // 2
                        },
                        ASTNode::new_leaf(tokens[1]), // +
                        ASTNode::Branch {
                            label: "expression",
                            children: vec![ASTNode::new_leaf(tokens[2])], // 4
                        },
                    ],
                },
                ASTNode::new_leaf(tokens[3]), // *
                ASTNode::Branch {
                    label: "expression",
                    children: vec![ASTNode::new_leaf(tokens[4])], // 3
                },
            ],
        }
    );
}
