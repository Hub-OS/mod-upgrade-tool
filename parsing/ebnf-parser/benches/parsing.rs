use criterion::{criterion_group, criterion_main, Criterion};
use ebnf_parser::*;

fn recursion(c: &mut Criterion) {
    let mut lexer = Lexer::new();
    lexer.add_token("(", "(".to_string());

    c.bench_function("left_recursion", |b| {
        let mut parser = EarleyParser::new("start");
        parser.add_rules("start", [vec![], vec!["("], vec!["start", "("]]);

        let source = "((((((((((((((((((((((((";
        let tokens = lexer.analyze(source).unwrap();

        b.iter(|| parser.parse(source, &tokens))
    });

    c.bench_function("right_recursion", |b| {
        let mut parser = EarleyParser::new("start");
        parser.add_rules("start", [vec![], vec!["("], vec!["(", "start"]]);

        let source = "((((((((((((((((((((((((";
        let tokens = lexer.analyze(source).unwrap();

        b.iter(|| parser.parse(source, &tokens))
    });

    c.bench_function("left_and_right_recursion", |b| {
        let mut parser = EarleyParser::new("start");
        parser.add_rules("start", [vec!["("], vec![], vec!["start", "start"]]);

        let source = "((((((((((((((((((((((((";
        let tokens = lexer.analyze(source).unwrap();

        b.iter(|| parser.parse(source, &tokens))
    });
}

criterion_group!(parsing, recursion);
criterion_main!(parsing);
