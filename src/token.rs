use std::{borrow::Cow, ops::RangeInclusive, slice::Iter};

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Plain(&'a str),
    Set(Vec<&'a str>),
    NumRange(usize, usize, usize /* padding width */),
    StrRange(usize, usize, bool /* uppercase */),
}

impl<'a> Token<'_> {
    pub fn new_plain(s: impl Into<&'a str>) -> Token<'a> {
        Token::Plain(s.into())
    }

    pub fn new_set(s: impl Into<Vec<&'a str>>) -> Token<'a> {
        Token::Set(s.into())
    }

    pub fn new_num_range(start: usize, end: usize, padding: usize) -> Token<'a> {
        Token::NumRange(start, end, padding)
    }

    pub fn new_str_range(start: &'a str, end: &'a str) -> Result<Token<'a>> {
        match (start.chars().next(), end.chars().next()) {
            (Some(c1), Some(c2)) => {
                let (uppercase, radix) = match (c1.is_ascii_uppercase(), c2.is_ascii_uppercase()) {
                    (false, false) => (false, 'a'..='z'),
                    (true, true) => (true, 'A'..='Z'),
                    _ => bail!("mixed uppercase with lowercase in alphabetic range"),
                };
                Ok(Token::StrRange(
                    parse_alphabetic_radix(start, radix.clone())?,
                    parse_alphabetic_radix(end, radix)?,
                    uppercase,
                ))
            }
            (None, _) => bail!("range start cannot be empty"),
            (_, None) => bail!("range end cannot be empty"),
        }
    }

    pub fn iter(&self) -> TokenIter<'_> {
        TokenIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub enum TokenIter<'a> {
    Plain(Option<&'a str>),
    Set(Iter<'a, &'a str>),
    NumRange(RangeInclusive<usize>, usize /* padding width */),
    StrRange(RangeInclusive<usize>, bool /* uppercase */),
}

impl<'a> TokenIter<'a> {
    pub fn new(t: &'a Token) -> Self {
        match t {
            Token::Plain(v) => TokenIter::Plain(Some(v)),
            Token::Set(v) => TokenIter::Set(v.iter()),
            &Token::NumRange(start, end, padding) => TokenIter::NumRange(start..=end, padding),
            &Token::StrRange(start, end, uppercase) => TokenIter::StrRange(start..=end, uppercase),
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Cow<'a, str>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TokenIter::Plain(v) => v.take().map(|v| v.into()),
            TokenIter::Set(v) => v.next().map(|&v| v.into()),
            TokenIter::NumRange(range, padding) => {
                range.next().map(|x| Cow::Owned(format!("{:0padding$}", x)))
            }
            TokenIter::StrRange(range, uppercase) => range.next().map(|x| {
                let radix = if *uppercase { 'A'..='Z' } else { 'a'..='z' };
                to_alphabetic_radix(x, radix).into()
            }),
        }
    }
}

/// Convert the usize into an alphabetic radix string
fn to_alphabetic_radix(mut x: usize, radix: RangeInclusive<char>) -> String {
    let (start, end) = (*radix.start(), *radix.end());
    let n = end as usize - start as usize + 1;
    let mut digits = Vec::new();
    while x > 0 {
        let d = ((x - 1) % n) as u8;
        x = (x - 1) / n;
        digits.push((d + start as u8) as char);
    }
    String::from_iter(digits.into_iter().rev())
}

/// Parse the alphabetic radix string into an usize
fn parse_alphabetic_radix(s: &str, radix: RangeInclusive<char>) -> Result<usize> {
    let (start, end) = (*radix.start(), *radix.end());
    let n = end as usize - start as usize + 1;
    s.chars().try_fold(0, |acc, x| {
        if radix.contains(&x) {
            Ok(acc * n + (x as usize) - (start as usize) + 1)
        } else {
            bail!("char '{x}' not in range '{start}-{end}'",)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_iter() -> Result<()> {
        let cases = vec![
            (
                "plain",
                Token::new_plain("Hello, World"),
                vec!["Hello, World"],
            ),
            (
                "set",
                Token::new_set(["a", "b", "c", "d", "e"]),
                vec!["a", "b", "c", "d", "e"],
            ),
            (
                "number range",
                Token::new_num_range(1, 3, 0),
                vec!["1", "2", "3"],
            ),
            (
                "number range with padding",
                Token::new_num_range(1, 3, 3),
                vec!["001", "002", "003"],
            ),
            (
                "single letter range",
                Token::new_str_range("a", "c")?,
                vec!["a", "b", "c"],
            ),
            (
                "multi letters range",
                Token::new_str_range("y", "af")?,
                vec!["y", "z", "aa", "ab", "ac", "ad", "ae", "af"],
            ),
            (
                "multi uppercase letters range",
                Token::new_str_range("WZ", "XF")?,
                vec!["WZ", "XA", "XB", "XC", "XD", "XE", "XF"],
            ),
        ];

        for (name, input, expected) in cases {
            let actual = input.iter().collect::<Vec<_>>();

            assert_eq!(actual, expected, "case {name}")
        }

        Ok(())
    }
}
