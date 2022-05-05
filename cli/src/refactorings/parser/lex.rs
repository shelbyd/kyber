use logos::*;
use std::collections::*;

pub type Tokens = VecDeque<Token>;

pub fn lex(contents: &str) -> Result<Tokens, String> {
    let mut tokens = Token::lexer(contents);
    std::iter::from_fn(|| {
        Some(match tokens.next()? {
            Token::Error => Err(tokens.slice().to_owned()),
            t => Ok(t),
        })
    })
    .collect()
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,

    #[token("import")]
    Import,
    #[token("from")]
    From,
    #[token("let")]
    Let,

    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    #[regex("[a-zA-Z][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[token(",")]
    Comma,
    #[token(";")]
    SemiColon,
    #[token(":")]
    Colon,

    #[token("..")]
    Concatenate,
    #[token(".")]
    Period,

    #[regex("\"[^\"]*\"", |lex| lex.slice()[1..(lex.slice().len()-1)].to_string())]
    StringLiteral(String),
    #[regex("/[^/]+/", |lex| lex.slice()[1..(lex.slice().len()-1)].to_string())]
    Regex(String),

    #[token("@")]
    Directive,
    #[token("=")]
    Equal,
}
