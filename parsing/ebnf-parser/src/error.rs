use crate::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError<'a, Label> {
    // lexer
    UnexpectedCharacter {
        offset: usize,
        line: usize,
        col: usize,
    },
    BadLexer {
        label: Label,
        offset: usize,
        line: usize,
        col: usize,
        final_offset: usize,
    },
    BadIgnorer {
        offset: usize,
        line: usize,
        col: usize,
        final_offset: usize,
    },
    // parser
    UndefinedRule {
        label: Label,
    },
    UnexpectedToken {
        token: Token<'a, Label>,
        line: usize,
        col: usize,
    },
    UnexpectedEOF,
}

impl<'a, Label: std::fmt::Debug> std::fmt::Display for ParserError<'a, Label> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParserError::UnexpectedCharacter { line, col, .. } => write!(f, "Lexing Error {}:{}: unexpected character", line, col),
            ParserError::BadLexer { label, line, col, .. } => write!(f, "Lexing Error {}:{}: a lexer creating {:?} tokens returned a length that would include characters past end", line, col, label),
            ParserError::BadIgnorer { line, col, .. } => write!(f, "Lexing Error {}:{}: an ignorer returned a length that would include characters past end", line, col),
            ParserError::UndefinedRule {
                label,
            } => write!(f, "Parsing Error: {:?} has no rule defined", label),
            ParserError::UnexpectedToken {
                token,
                line,
                col,
            } => write!(f, "Parsing Error {}:{}: Unexpected {:?}", line, col, token.label),
            ParserError::UnexpectedEOF=> write!(f, "Parsing Error Unexpecteed EOF"),
        }
    }
}
