use anyhow::{bail, Result};
use itertools::Itertools;
use std::mem::take;

use super::token::Token;

#[derive(Debug, Clone)]
pub struct Pattern<'a> {
    original: &'a str,
    tokens: Vec<Token<'a>>,
}

#[derive(PartialEq)]
enum State<'a> {
    Plain,
    InSet(Vec<&'a str>),
    InRange(&'a str),
}

impl<'a> Pattern<'a> {
    pub fn parse(s: &str) -> Result<Pattern> {
        let mut pattern = Pattern {
            original: s,
            tokens: Vec::new(),
        };
        let mut state = State::Plain;
        let (mut i, mut j) = (0, 0); // segment start & end index in s
        for (idx, char) in s.char_indices() {
            let next_idx = idx + char.len_utf8();
            match char {
                '{' => match &mut state {
                    State::Plain => {
                        pattern.tokens.push(Token::new_plain(&s[i..j]));
                        (i, j, state) = (next_idx, next_idx, State::InSet(Vec::new()));
                    }
                    _ => bail!("unexpected character '{{' at pos {}", idx),
                },
                '[' => match &mut state {
                    State::Plain => {
                        pattern.tokens.push(Token::new_plain(&s[i..j]));
                        (i, j, state) = (next_idx, next_idx, State::InRange(""));
                    }
                    _ => bail!("unexpected character '[' at pos {}", idx),
                },
                '}' => match &mut state {
                    State::InSet(v) => {
                        let mut set = take(v);
                        set.push((&s[i..j]).trim());
                        pattern.tokens.push(Token::new_set(set));
                        (i, j, state) = (next_idx, next_idx, State::Plain);
                    }
                    _ => bail!("unexpected character '}}' at pos {}", idx),
                },
                ']' => match &mut state {
                    State::InRange(start) => {
                        let end = &s[i..j].trim();
                        let token = match (start.chars().next(), end.chars().next()) {
                            (Some('A'..='Z' | 'a'..='z'), Some('A'..='Z' | 'a'..='z')) => {
                                Token::new_str_range(start, end)?
                            }
                            (Some('0'..='9'), Some('0'..='9')) => {
                                let padding = start.len().min(end.len());
                                let (start, end) = (start.parse()?, end.parse()?);
                                Token::new_num_range(start, end, padding)
                            }
                            _ => bail!("invalid characters in range token before pos {}", idx),
                        };
                        pattern.tokens.push(token);
                        (i, j, state) = (next_idx, next_idx, State::Plain);
                    }
                    _ => bail!("unexpected character ']' at pos {}", idx),
                },
                ',' => match &mut state {
                    State::Plain => j = next_idx,
                    State::InSet(set) => {
                        set.push((&s[i..j]).trim());
                        (i, j) = (next_idx, next_idx);
                    }
                    _ => bail!("unexpected character ',' at pos {}", idx),
                },
                '-' => match &mut state {
                    State::Plain | State::InSet(_) => j = next_idx,
                    State::InRange(start) => {
                        *start = s[i..j].trim();
                        (i, j) = (next_idx, next_idx);
                    }
                },
                _ => j = next_idx,
            }
        }
        if j > i {
            pattern.tokens.push(Token::Plain((&s[i..j]).trim()));
        }

        Ok(pattern)
    }

    pub fn as_str(&self) -> &str {
        self.original
    }

    pub fn iter(&self) -> impl Iterator<Item = String> + '_ {
        self.tokens
            .iter()
            .map(|v| v.iter())
            .multi_cartesian_product()
            .map(|v| v.join(""))
    }
}

impl<'a> TryFrom<&'a str> for Pattern<'a> {
    type Error = anyhow::Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
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
                    Token::new_set(vec!["a", "b", "c"]),
                    Token::new_plain("/file"),
                ],
            ),
            (
                "two set",
                "https://example.com/{a,b,c}/file/{x,y,z}",
                vec![
                    Token::new_plain("https://example.com/"),
                    Token::new_set(vec!["a", "b", "c"]),
                    Token::new_plain("/file/"),
                    Token::new_set(vec!["x", "y", "z"]),
                ],
            ),
            (
                "two set with spaces",
                "https://example.com/{a, b , c }/file/{foo bar, 你好, fizzbuzz, 世界}",
                vec![
                    Token::new_plain("https://example.com/"),
                    Token::new_set(vec!["a", "b", "c"]),
                    Token::new_plain("/file/"),
                    Token::new_set(vec!["foo bar", "你好", "fizzbuzz", "世界"]),
                ],
            ),
            (
                "one number range",
                "https://example.com/[080-120]/file",
                vec![
                    Token::new_plain("https://example.com/"),
                    Token::new_num_range(80, 120, 3),
                    Token::new_plain("/file"),
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

    #[test]
    fn test_parse_error() -> Result<()> {
        let cases = vec![
            (
                "bad pattern 1",
                "/{{a, b}",
                "unexpected character '{' at pos 2",
            ),
            (
                "bad pattern 2",
                "/{a}}",
                "unexpected character '}' at pos 4",
            ),
        ];

        for (name, input, expected) in cases {
            assert_eq!(
                Pattern::parse(input).unwrap_err().to_string(),
                expected,
                "case {name}"
            )
        }

        Ok(())
    }

    #[test]
    fn test_iter() -> Result<()> {
        let cases = vec![
            ("normal", "Hello, World!", vec!["Hello, World!"]),
            (
                "one set",
                "https://example.com/{a,b,c}/file",
                vec![
                    "https://example.com/a/file",
                    "https://example.com/b/file",
                    "https://example.com/c/file",
                ],
            ),
            (
                "two set",
                "https://example.com/{a,b,c}/file/{x,y,z}",
                vec![
                    "https://example.com/a/file/x",
                    "https://example.com/a/file/y",
                    "https://example.com/a/file/z",
                    "https://example.com/b/file/x",
                    "https://example.com/b/file/y",
                    "https://example.com/b/file/z",
                    "https://example.com/c/file/x",
                    "https://example.com/c/file/y",
                    "https://example.com/c/file/z",
                ],
            ),
            (
                "one number range",
                "https://example.com/[1-3]/file",
                vec![
                    "https://example.com/1/file",
                    "https://example.com/2/file",
                    "https://example.com/3/file",
                ],
            ),
            (
                "two number range with padding zero",
                "https://example.com/[1-2]/file/[099-101]",
                vec![
                    "https://example.com/1/file/099",
                    "https://example.com/1/file/100",
                    "https://example.com/1/file/101",
                    "https://example.com/2/file/099",
                    "https://example.com/2/file/100",
                    "https://example.com/2/file/101",
                ],
            ),
            (
                "single letter range",
                "https://example.com/[A-C]/file",
                vec![
                    "https://example.com/A/file",
                    "https://example.com/B/file",
                    "https://example.com/C/file",
                ],
            ),
            (
                "multi letters range",
                "https://example.com/[ay-bc]/file",
                vec![
                    "https://example.com/ay/file",
                    "https://example.com/az/file",
                    "https://example.com/ba/file",
                    "https://example.com/bb/file",
                    "https://example.com/bc/file",
                ],
            ),
        ];

        for (name, input, expected) in cases {
            let p = Pattern::parse(input)?;

            assert_eq!(p.iter().collect::<Vec<_>>(), expected, "case {name}");
        }

        Ok(())
    }
}
