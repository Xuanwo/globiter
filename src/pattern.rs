use crate::token::Token;
use anyhow::Result;
use std::mem::take;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Pattern {
    original: String,
    tokens: Vec<Token>,
}

#[derive(PartialEq)]
enum State {
    Plain,
    InSet(Vec<String>),
}

impl Pattern {
    pub fn parse(s: &str) -> Result<Pattern> {
        let mut pattern = Pattern {
            original: s.to_string(),
            tokens: Vec::new(),
        };
        let mut state = State::Plain;
        let mut buf = String::new();
        for char in s.chars() {
            match char {
                '{' => {
                    debug_assert!(state == State::Plain);
                    state = State::InSet(Vec::new());
                    let s = take(&mut buf);
                    pattern.tokens.push(Token::new_plain(s))
                }
                '}' => {
                    let mut set = match &mut state {
                        State::Plain => panic!("unreachable"),
                        State::InSet(v) => take(v),
                    };
                    state = State::Plain;
                    let s = take(&mut buf);
                    set.push(s);
                    pattern.tokens.push(Token::new_set(set))
                }
                ',' => match &mut state {
                    State::Plain => buf.push(','),
                    State::InSet(v) => {
                        let s = take(&mut buf);
                        v.push(s);
                    }
                },
                v => buf.push(v),
            }
        }
        if !buf.is_empty() {
            let s = take(&mut buf);
            pattern.tokens.push(Token::Plain(s))
        }

        Ok(pattern)
    }
}

impl FromStr for Pattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Pattern::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> Result<()> {
        let cases = vec![
            (
                "normal",
                "Hello, World!",
                vec![Token::new_plain("Hello, World!")],
            ),
            (
                "one set",
                "https://example.com/{a,b,c}/file",
                vec![
                    Token::new_plain("https://example.com/"),
                    Token::new_set(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
                    Token::new_plain("/file"),
                ],
            ),
            (
                "two set",
                "https://example.com/{a,b,c}/file/{x,y,z}",
                vec![
                    Token::new_plain("https://example.com/"),
                    Token::new_set(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
                    Token::new_plain("/file/"),
                    Token::new_set(vec!["x".to_string(), "y".to_string(), "z".to_string()]),
                ],
            ),
        ];

        for (name, input, expected) in cases {
            let p = Pattern::parse(input)?;

            assert_eq!(p.original, input, "case {name}");
            assert_eq!(p.tokens, expected, "case {name}");
        }

        Ok(())
    }
}
