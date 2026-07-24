//! Minimal, dependency-free S-expression reader for PDDL.
//!
//! PDDL is case-insensitive and its concrete syntax is an S-expression grammar.
//! This module performs only lexical and balanced-tree admission. Semantic
//! lowering lives in `parse.rs`.

use crate::error::Pddl8Error;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

impl SExpr {
    pub(crate) fn atom(&self) -> Result<&str, Pddl8Error> {
        match self {
            Self::Atom(value) => Ok(value),
            Self::List(_) => Err(Pddl8Error::ParseError("expected atom, found list".into())),
        }
    }

    pub(crate) fn list(&self) -> Result<&[SExpr], Pddl8Error> {
        match self {
            Self::List(items) => Ok(items),
            Self::Atom(value) => Err(Pddl8Error::ParseError(format!(
                "expected list, found atom {value:?}"
            ))),
        }
    }

    pub(crate) fn head(&self) -> Option<&str> {
        match self {
            Self::List(items) => items.first().and_then(|item| match item {
                Self::Atom(value) => Some(value.as_str()),
                Self::List(_) => None,
            }),
            Self::Atom(_) => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Open,
    Close,
    Atom(String),
}

pub(crate) fn parse_one(text: &str) -> Result<SExpr, Pddl8Error> {
    let tokens = tokenize(text)?;
    if tokens.is_empty() {
        return Err(Pddl8Error::ParseError("empty PDDL document".into()));
    }
    let mut cursor = 0usize;
    let expr = parse_expr(&tokens, &mut cursor)?;
    if cursor != tokens.len() {
        return Err(Pddl8Error::ParseError(format!(
            "trailing token at index {cursor}"
        )));
    }
    Ok(expr)
}

fn tokenize(text: &str) -> Result<Vec<Token>, Pddl8Error> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_comment = false;

    let flush = |current: &mut String, out: &mut Vec<Token>| {
        if !current.is_empty() {
            out.push(Token::Atom(current.to_ascii_lowercase()));
            current.clear();
        }
    };

    for ch in text.chars() {
        if in_comment {
            if ch == '\n' || ch == '\r' {
                in_comment = false;
            }
            continue;
        }
        match ch {
            ';' => {
                flush(&mut current, &mut out);
                in_comment = true;
            }
            '(' => {
                flush(&mut current, &mut out);
                out.push(Token::Open);
            }
            ')' => {
                flush(&mut current, &mut out);
                out.push(Token::Close);
            }
            c if c.is_whitespace() => flush(&mut current, &mut out),
            c if c.is_control() => {
                return Err(Pddl8Error::ParseError(format!(
                    "unsupported control character U+{:04X}",
                    c as u32
                )));
            }
            c => current.push(c),
        }
    }
    flush(&mut current, &mut out);
    Ok(out)
}

fn parse_expr(tokens: &[Token], cursor: &mut usize) -> Result<SExpr, Pddl8Error> {
    let token = tokens
        .get(*cursor)
        .ok_or_else(|| Pddl8Error::ParseError("unexpected end of document".into()))?;
    *cursor += 1;
    match token {
        Token::Atom(value) => Ok(SExpr::Atom(value.clone())),
        Token::Close => Err(Pddl8Error::ParseError(format!(
            "unexpected ')' at token {}",
            cursor.saturating_sub(1)
        ))),
        Token::Open => {
            let mut items = Vec::new();
            loop {
                match tokens.get(*cursor) {
                    Some(Token::Close) => {
                        *cursor += 1;
                        return Ok(SExpr::List(items));
                    }
                    Some(_) => items.push(parse_expr(tokens, cursor)?),
                    None => {
                        return Err(Pddl8Error::ParseError(
                            "unclosed '(' at end of document".into(),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_and_case_are_canonicalized() {
        let parsed = parse_one("(DEFINE ; ignored\n (DOMAIN D))").unwrap();
        assert_eq!(parsed.head(), Some("define"));
        let root = parsed.list().unwrap();
        assert_eq!(root[1].list().unwrap()[1].atom().unwrap(), "d");
    }

    #[test]
    fn rejects_unbalanced_input() {
        let error = parse_one("(define (domain d)").unwrap_err();
        assert!(matches!(error, Pddl8Error::ParseError(_)));
    }
}
