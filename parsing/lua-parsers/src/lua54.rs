use ebnf_parser::{ASTNode, EBNFParser, Lexer, ParserError};

pub struct Lua54Parser {
    lexer: Lexer<&'static str>,
    parser: EBNFParser<'static>,
}

impl Lua54Parser {
    pub fn new() -> Self {
        let ebnf = include_str!("lua54.ebnf");

        let reserved_words = [
            "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto",
            "if", "in", "local", "nil", "not", "or", "repeat", "return", "then", "true", "until",
            "while",
        ];

        let other_tokens = [
            "+", "-", "*", "/", "%", "^", "#", "&", "~", "|", "<<", ">>", "//", "==", "~=", "<=",
            ">=", "<", ">", "=", "(", ")", "{", "}", "[", "]", "::", ";", ":", ",", ".", "..",
            "...",
        ];

        let mut lexer = Lexer::new();

        for token in other_tokens {
            lexer.add_token(token, token.to_string());
        }

        let number_regex = regex::RegexBuilder::new(
            // two expressions in one split with |
            // hexadecimal regex, starts with 0x, [a-f0-9]+ with optional fractional part, and optional binary exponent starting with p
            // decimal regex, starts with [0-9]* and optional fractional part, and optional decimal exponent starting with e
            r#"^0x(?:[a-f\d]+)(?:\.[a-f\d]*)?(?:p[+-]?\d+)?|^\d*(?:\.\d+)?(?:e[+-]?\d+)?"#,
        )
        .case_insensitive(true)
        .build()
        .unwrap();

        lexer.add_lexer(move |source, start| {
            (
                "Numeral",
                if let Some(m) = number_regex.find(&source[start..]) {
                    m.end()
                } else {
                    0
                },
            )
        });

        // reserved words and names
        lexer.add_lexer(move |source, start| {
            let first_char = source.chars().nth(start).unwrap();

            if !first_char.is_alphabetic() && first_char != '_' {
                return ("Name", 0);
            }

            let word_len = source
                .chars()
                .skip(start + 1)
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .count()
                + 1;

            let word = &source[start..start + word_len];

            if let Some(index) = reserved_words
                .iter()
                .position(|reserved_word| *reserved_word == word)
            {
                // use the reserved word as the name of the token
                (reserved_words[index], word_len)
            } else {
                ("Name", word_len)
            }
        });

        // strings
        lexer.add_lexer(|source, start| {
            let source_substr = &source[start..];

            let first_char = source_substr.chars().next().unwrap();

            if !(first_char == '"' || first_char == '\'') {
                return ("LiteralString", 0);
            }

            let mut previous_char = '"';

            let string_length = &source_substr[1..]
                .chars()
                .take_while(|c| {
                    let c = *c;
                    let is_end = (c == '\n' || c == first_char) && previous_char != '\\';

                    previous_char = c;

                    !is_end
                })
                .count();

            let last_char = source_substr.chars().nth(string_length + 1);

            if let Some(last_char) = last_char {
                if last_char != first_char {
                    return ("LiteralString", 0);
                }
            } else {
                return ("LiteralString", 0);
            }

            ("LiteralString", string_length + 2)
        });

        // regex for the start of a multiline string
        let multiline_string_start_regex = regex::Regex::new(r#"^\[=*\["#).unwrap();
        let multiline_string_end_regex = regex::Regex::new(r#"\]=*\]"#).unwrap();

        // multiline string
        lexer.add_lexer(move |source, start| {
            let start_match_len =
                if let Some(start_match) = multiline_string_start_regex.find(&source[start..]) {
                    start_match.end() - start_match.start()
                } else {
                    return ("LiteralString", 0);
                };

            let end_match = multiline_string_end_regex
                .find_iter(&source[start..])
                .find(|end_match| end_match.end() - end_match.start() == start_match_len);

            if let Some(end_match) = end_match {
                ("LiteralString", end_match.end())
            } else {
                ("LiteralString", 0)
            }
        });

        // whitespace
        lexer.add_ignorer(|source, start| {
            source
                .chars()
                .skip(start)
                .take_while(|c| c.is_whitespace())
                .count()
        });

        // comments
        lexer.add_ignorer(|source, start| {
            let source_substr = &source[start..];

            if source_substr.starts_with("--[[") {
                source_substr
                    .find("]]--")
                    .map(|index| index + 4)
                    .unwrap_or(source.len() - start)
            } else if source_substr.starts_with("--") {
                source_substr
                    .find(&['\r', '\n'])
                    .unwrap_or(source.len() - start)
            } else {
                0
            }
        });

        Self {
            lexer,
            parser: EBNFParser::new(ebnf, "chunk"),
        }
    }

    pub fn parse<'a>(
        &self,
        source: &'a str,
    ) -> Result<ASTNode<'a, &'static str>, ParserError<'a, &'static str>> {
        let tokens = self.lexer.analyze(source)?;

        self.parser.parse(source, &tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_calls_in_function() {
        let source = "function a() print() print() end";

        let parser = Lua54Parser::new();

        parser.parse(source).unwrap();
    }

    #[test]
    fn function_as_exp() {
        let source = "local a = a()";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }

    #[test]
    fn function_as_prefixexp() {
        let source = "a()()";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }

    #[test]
    fn function_as_prefixexp2() {
        let source = "a:a():a():a()";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }

    #[test]
    fn for_loop() {
        let source = "for i = 0,10 do print('hi') end";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }

    #[test]
    fn multiline_string() {
        let source = "local a = [[multiline\nstring]]";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }

    #[test]
    fn confusing_multiline_string() {
        let source = "local a = [=[multiline\nstring]]=]";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }

    #[test]
    fn indexing() {
        let source = "a[1] = 3";

        let parser = Lua54Parser::new();
        parser.parse(source).unwrap();
    }
}
