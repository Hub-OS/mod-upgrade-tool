use crate::{EarleyRecognizer, Lexer, Rule};

fn lexer() -> Lexer<&'static str> {
    let mut lexer = Lexer::new();
    let tokens = ["(", ")"];

    for token in tokens {
        lexer.add_token(token, token.to_string());
    }

    lexer.add_char_lexer(|c| ("[0-9]", c.is_numeric()));
    lexer.add_char_lexer(|c| ("[+-]", ['+', '-'].contains(&c)));
    lexer.add_char_lexer(|c| ("[*/]", ['*', '/'].contains(&c)));

    lexer
}

#[test]
fn test() {
    // comparing results to https://loup-vaillant.fr/tutorials/earley-parsing/recogniser

    let source = "1+(2*3-4)";

    let rules = vec![
        Rule::new(0, "Sum", vec!["Sum", "[+-]", "Product"]),
        Rule::new(1, "Sum", vec!["Product"]),
        Rule::new(2, "Product", vec!["Product", "[*/]", "Factor"]),
        Rule::new(3, "Product", vec!["Factor"]),
        Rule::new(4, "Factor", vec!["(", "Sum", ")"]),
        Rule::new(5, "Factor", vec!["Number"]),
        Rule::new(6, "Number", vec!["[0-9]", "Number"]),
        Rule::new(7, "Number", vec!["[0-9]"]),
    ];

    let nullables = crate::find_nullables(&rules);
    let tokens = lexer().analyze(source).unwrap();

    let recognizer = EarleyRecognizer::new(&nullables, &rules);
    let sets = recognizer.recognize("Sum", &tokens);

    let mut output = Vec::new();

    for (index, set) in sets.iter().enumerate() {
        output.push(format!("=== {index} ==="));

        for item in set {
            output.push(format!("{:?}", item));
        }

        output.push(String::from(""));
    }

    assert_eq!(output.join("\n"), include_str!("./recognizer_expected.txt"));
}
