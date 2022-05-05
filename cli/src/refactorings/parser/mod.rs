#![allow(dead_code)]

use super::script::*;

mod lex;
use lex::{lex, Token, Tokens};

type Result<T> = std::result::Result<T, String>;

pub fn parse(s: &str) -> Result<Script> {
    let mut tokens = lex(s)?;

    let mut top_levels = Vec::new();
    while let Some(top_level) = top_level(&mut tokens)? {
        top_levels.push(top_level);
    }
    Ok(Script::new(top_levels))
}

pub enum TopLevel {
    Import(Import),
    Directive(Directive),
    Assignment(String, Expr),
    Expr(Expr),
}

pub struct Import {
    idents: Vec<String>,
    source: String,
}

pub struct Directive {
    name: String,
    value: String,
}

pub enum Expr {
    Binding(String, Box<Expr>),
    DotAccess(Box<Expr>, String),
    FnCall(Box<Expr>, Vec<Expr>),

    Ident(String),
    StringLiteral(String),

    Concatenate(Box<Expr>, Box<Expr>),
}

fn top_level(t: &mut Tokens) -> Result<Option<TopLevel>> {
    match t.pop_front() {
        None => Ok(None),

        Some(Token::Import) => Ok(Some(TopLevel::Import(import(t)?))),
        Some(Token::Directive) => Ok(Some(TopLevel::Directive(directive(t)?))),

        Some(Token::Let) => {
            let ident = take_ident(t)?;
            take(t, Token::Equal)?;
            let value = expr(t)?;
            take(t, Token::SemiColon)?;
            Ok(Some(TopLevel::Assignment(ident, value)))
        }

        Some(unexpected) => {
            t.push_front(unexpected.clone());

            let e = expr(t)?;
            take(t, Token::SemiColon)?;
            Ok(Some(TopLevel::Expr(e)))
        }
    }
}

fn import(t: &mut Tokens) -> Result<Import> {
    take(t, Token::OpenBrace)?;

    let idents = take_until(t, Token::CloseBrace, Token::Comma, |t| take_ident(t))?;

    take(t, Token::From)?;
    let source = take_string_lit(t)?;
    take(t, Token::SemiColon)?;

    Ok(Import { idents, source })
}

fn directive(t: &mut Tokens) -> Result<Directive> {
    let name = take_ident(t)?;
    take(t, Token::Equal)?;
    let value = take_string_lit(t)?;
    take(t, Token::SemiColon)?;

    Ok(Directive { name, value })
}

fn expr(t: &mut Tokens) -> Result<Expr> {
    concatenate_expr(t)
}

/// Left associative infix parsing.
macro_rules! impl_infix_parser {
    ($name:ident, $next:ident, [$($token:pat => $expr:ident,)*]) => {
        fn $name(t: &mut Tokens) -> Result<Expr> {
            let mut e = $next(t)?;
            loop {
                match take_any(t)? {
                    $($token => {
                        let e2 = $next(t)?;
                        e = Expr::$expr(e.into(), e2.into());
                    })*
                    token => {
                        t.push_front(token);
                        return Ok(e);
                    },
                }
            }
        }
    };
}

impl_infix_parser!(concatenate_expr, binding_expr, [
    Token::Concatenate => Concatenate,
]);

fn binding_expr(t: &mut Tokens) -> Result<Expr> {
    if let (Some(Token::Ident(_)), Some(Token::Colon)) = (t.get(0), t.get(1)) {
        let binding = take_ident(t)?;
        take(t, Token::Colon)?;

        let expr = expr(t)?;
        return Ok(Expr::Binding(binding, expr.into()));
    }

    fn_expr(t)
}

fn fn_expr(t: &mut Tokens) -> Result<Expr> {
    let on = dot_access_expr(t)?;

    if !try_take(t, &Token::OpenParen) {
        return Ok(on);
    }

    let exprs = take_until(t, Token::CloseParen, Token::Comma, |t| expr(t))?;

    Ok(Expr::FnCall(on.into(), exprs))
}

fn dot_access_expr(t: &mut Tokens) -> Result<Expr> {
    let obj = paren_expr(t)?;

    if !try_take(t, &Token::Period) {
        return Ok(obj);
    }

    let prop = take_ident(t)?;
    Ok(Expr::DotAccess(obj.into(), prop))
}

fn paren_expr(t: &mut Tokens) -> Result<Expr> {
    if !try_take(t, &Token::OpenParen) {
        return leaf_expr(t);
    }

    let expr = expr(t)?;
    take(t, Token::CloseParen)?;
    Ok(expr)
}

fn leaf_expr(t: &mut Tokens) -> Result<Expr> {
    match t.pop_front() {
        Some(Token::StringLiteral(s)) => Ok(Expr::StringLiteral(s)),
        Some(Token::Ident(i)) => Ok(Expr::Ident(i)),

        None => Err(format!("Expected expr, found EOF")),
        Some(unexpected) => Err(format!("Expected expr, found {:?}", unexpected)),
    }
}

fn take(t: &mut Tokens, expected: Token) -> Result<()> {
    let front = t
        .pop_front()
        .ok_or_else(|| format!("Expected {:?}, found EOF", expected))?;
    if front == expected {
        Ok(())
    } else {
        Err(format!("Expected {:?}, found {:?}", expected, front))
    }
}

fn take_any(t: &mut Tokens) -> Result<Token> {
    t.pop_front()
        .ok_or_else(|| format!("Expected a token, found EOF"))
}

fn take_one(t: &mut Tokens, expected: &[Token]) -> Result<Token> {
    let tokens_message = || {
        expected
            .iter()
            .map(|t| format!("{:?}", t))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let front = t
        .pop_front()
        .ok_or_else(|| format!("Expected one of {}, found EOF", tokens_message()))?;

    if expected.contains(&front) {
        Ok(front)
    } else {
        Err(format!(
            "Expected one of {}, found {:?}",
            tokens_message(),
            front
        ))
    }
}

fn try_take(tokens: &mut Tokens, expected: &Token) -> bool {
    match tokens.get(0) {
        Some(t) if t == expected => {
            tokens.pop_front();
            true
        }
        _ => false,
    }
}

fn take_ident(t: &mut Tokens) -> Result<String> {
    match t.pop_front() {
        None => Err(format!("Expected ident, found EOF")),
        Some(Token::Ident(s)) => Ok(s),
        Some(t) => Err(format!("Expected ident, found {:?}", t)),
    }
}

fn take_string_lit(t: &mut Tokens) -> Result<String> {
    match t.pop_front() {
        None => Err(format!("Expected string, found EOF")),
        Some(Token::StringLiteral(s)) => Ok(s),
        Some(t) => Err(format!("Expected string, found {:?}", t)),
    }
}

fn take_until<T>(
    t: &mut Tokens,
    terminal: Token,
    separator: Token,
    mut f: impl FnMut(&mut Tokens) -> Result<T>,
) -> Result<Vec<T>> {
    let mut result = Vec::new();
    while !try_take(t, &terminal) {
        result.push(f(t)?);

        if try_take(t, &separator) {
            continue;
        }
        take(t, terminal)?;
        break;
    }
    Ok(result)
}
