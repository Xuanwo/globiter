use std::{borrow::Cow, ops::RangeInclusive, slice::Iter};

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Plain(&'a str),
    Set(Vec<&'a str>),
    NumRange(usize, usize, usize),
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

    pub fn iter(&self) -> TokenIter<'_> {
        TokenIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub enum TokenIter<'a> {
    Plain(Option<&'a str>),
    Set(Iter<'a, &'a str>),
    NumRange(RangeInclusive<usize>, usize /* padding width */),
}

impl<'a> TokenIter<'a> {
    pub fn new(t: &'a Token) -> Self {
        match t {
            Token::Plain(v) => TokenIter::Plain(Some(v)),
            Token::Set(v) => TokenIter::Set(v.iter()),
            &Token::NumRange(start, end, padding) => TokenIter::NumRange(start..=end, padding),
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_iter() {
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
        ];

        for (name, input, expected) in cases {
            let actual = input.iter().collect::<Vec<_>>();

            assert_eq!(actual, expected, "case {name}")
        }
    }
}
