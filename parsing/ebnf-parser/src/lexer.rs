use crate::error::ParserError;
use crate::ASTNodeLabel;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a, Label> {
    pub label: Label,
    pub content: &'a str,
    pub offset: usize,
}

type SubLexer<Label> = Rc<dyn Fn(&str, usize) -> (Label, usize)>;
type Ignorer = Rc<dyn Fn(&str, usize) -> usize>;

#[derive(Default)]
pub struct Lexer<Label: ASTNodeLabel> {
    tokens: Vec<(Label, String)>,
    lexers: Vec<SubLexer<Label>>,
    ignorers: Vec<Ignorer>,
}

impl<Label: ASTNodeLabel> Lexer<Label> {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            lexers: Vec::new(),
            ignorers: Vec::new(),
        }
    }

    /// ignorer takes source str, and start index, returns the length to skip
    pub fn add_ignorer<F: 'static>(&mut self, lexer: F)
    where
        F: Fn(&str, usize) -> usize,
    {
        self.ignorers.push(Rc::new(lexer))
    }

    /// takes source str, and start index, returns the length of the token
    pub fn add_lexer<F: 'static>(&mut self, lexer: F)
    where
        F: Fn(&str, usize) -> (Label, usize),
    {
        self.lexers.push(Rc::new(lexer));
    }

    pub fn add_char_lexer<F: 'static>(&mut self, lexer: F)
    where
        F: Fn(char) -> (Label, bool),
    {
        self.add_lexer(move |source, start| {
            let char = source[start..].chars().next().unwrap();
            let (label, pass) = lexer(char);

            if pass {
                (label, 1)
            } else {
                (label, 0)
            }
        });
    }

    pub fn add_token(&mut self, label: Label, value: String) {
        let insert_index = self
            .tokens
            .binary_search_by(|(_, existing_value)| {
                value.len().partial_cmp(&existing_value.len()).unwrap()
            })
            .unwrap_or_else(|x| x);

        self.tokens.insert(insert_index, (label, value));
    }

    pub fn analyze<'a>(
        &self,
        source: &'a str,
    ) -> Result<Vec<Token<'a, Label>>, ParserError<'a, Label>> {
        #[allow(clippy::type_complexity)]
        let mut lexers: Vec<Rc<dyn Fn(&str, usize) -> (Label, usize)>> = Vec::new();

        lexers.extend(self.lexers.iter().cloned());

        if !self.tokens.is_empty() {
            lexers.push(Rc::new(|source, start| {
                self.tokens
                    .iter()
                    .find(|(_, value)| {
                        let can_fit_lexeme = source.len() - start >= value.len();

                        can_fit_lexeme && value.as_str() == &source[start..start + value.len()]
                    })
                    .map(|(label, value)| (*label, value.len()))
                    .unwrap_or((self.tokens.first().unwrap().0, 0))
            }));
        }

        let mut skip = 0;
        let mut lexemes = Vec::new();

        while skip < source.len() {
            let length = self
                .ignorers
                .iter()
                .map(|ignorer| ignorer(source, skip))
                .find(|length| *length > 0);

            if let Some(length) = length {
                if length + skip > source.len() {
                    let (line, col) = crate::get_line_and_col(source, skip);

                    return Err(ParserError::BadIgnorer {
                        offset: skip,
                        line,
                        col,
                        final_offset: length + skip,
                    });
                }

                skip += length;
                continue;
            }

            let lexer_result = lexers
                .iter()
                .map(|sub_lexer| sub_lexer(source, skip))
                .find(|(_, length)| *length > 0);

            if let Some((label, length)) = lexer_result {
                if length + skip > source.len() {
                    let (line, col) = crate::get_line_and_col(source, skip);

                    return Err(ParserError::BadLexer {
                        label,
                        offset: skip,
                        line,
                        col,
                        final_offset: length + skip,
                    });
                }

                lexemes.push(Token {
                    label,
                    content: &source[skip..skip + length],
                    offset: skip,
                });

                skip += length;
                continue;
            }

            let (line, col) = crate::get_line_and_col(source, skip);

            return Err(ParserError::UnexpectedCharacter {
                offset: skip,
                line,
                col,
            });
        }

        Ok(lexemes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer() {
        let mut lexer = Lexer::new();

        let lexemes = ["<", "<=", ">", ">=", "=="];

        for lexeme in lexemes {
            lexer.add_token(lexeme, lexeme.to_string());
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

        assert_eq!(
            &lexer.analyze("12 >= 3").unwrap(),
            &[
                Token {
                    label: "number",
                    content: "12",
                    offset: 0
                },
                Token {
                    label: ">=",
                    content: ">=",
                    offset: 3
                },
                Token {
                    label: "number",
                    content: "3",
                    offset: 6
                }
            ]
        );
    }

    #[test]
    fn bad_lexer() {
        let mut lexer = Lexer::new();

        lexer.add_lexer(|_source, _start| ("faulty", 1000));

        assert_eq!(
            lexer.analyze("12 >= 3").unwrap_err(),
            super::ParserError::BadLexer {
                label: "faulty",
                offset: 0,
                line: 1,
                col: 1,
                final_offset: 1000
            }
        );
    }
}
