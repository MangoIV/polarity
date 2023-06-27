use lalrpop_util::lexer::Token;

use miette::{Diagnostic, SourceOffset, SourceSpan};
use thiserror::Error;

fn separated<I: IntoIterator<Item = String>>(s: &str, iter: I) -> String {
    let vec: Vec<_> = iter.into_iter().collect();
    vec.join(s)
}
fn comma_separated<I: IntoIterator<Item = String>>(iter: I) -> String {
    separated(", ", iter)
}

#[derive(Error, Diagnostic, Debug)]
pub enum ParseError {
    /// Generated by the parser when it encounters a token (or EOF) it did not
    /// expect.
    #[error("Invalid token")]
    #[diagnostic(code("P-001"))]
    InvalidToken {
        #[label]
        location: SourceOffset,
    },

    /// Generated by the parser when it encounters an EOF it did not expect.
    #[error("Unexpected end of file. Expected {expected}")]
    #[diagnostic(code("P-002"))]
    UnrecognizedEOF {
        #[label]
        location: SourceOffset,
        expected: String,
    },

    /// Generated by the parser when it encounters a token it did not expect.
    #[error("Unexpected \"{token}\", expected {expected}")]
    #[diagnostic(code("P-003"))]
    UnrecognizedToken {
        token: String,
        #[label]
        span: SourceSpan,
        expected: String,
    },

    /// Generated by the parser when it encounters additional, unexpected tokens.
    #[error("Excessive \"{token}\"")]
    #[diagnostic(code("P-004"))]
    ExtraToken {
        token: String,
        #[label]
        span: SourceSpan,
    },
    #[error("{error}")]
    #[diagnostic(code("P-005"))]
    User { error: String },
}

impl From<lalrpop_util::ParseError<usize, Token<'_>, &'static str>> for ParseError {
    fn from(err: lalrpop_util::ParseError<usize, Token<'_>, &'static str>) -> Self {
        use lalrpop_util::ParseError::*;
        match err {
            InvalidToken { location } => ParseError::InvalidToken { location: location.into() },
            UnrecognizedEOF { location, expected } => ParseError::UnrecognizedEOF {
                location: location.into(),
                expected: comma_separated(expected),
            },
            UnrecognizedToken { token, expected } => ParseError::UnrecognizedToken {
                token: token.string(),
                span: token.span(),
                expected: comma_separated(expected),
            },
            ExtraToken { token } => {
                ParseError::ExtraToken { token: token.string(), span: token.span() }
            }
            User { error } => ParseError::User { error: error.to_owned() },
        }
    }
}

trait ToMietteExt {
    fn string(&self) -> String;
    fn span(&self) -> SourceSpan;
}

impl ToMietteExt for (usize, Token<'_>, usize) {
    fn span(&self) -> SourceSpan {
        (self.0, self.2 - self.0).into()
    }

    fn string(&self) -> String {
        self.1.to_string()
    }
}
