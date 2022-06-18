use std::slice::Iter;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    Plain(&'a str),
    Set(Vec<&'a str>),
    // Range(String, String),
}

impl<'a> Token<'_> {
    pub fn new_plain(s: impl Into<&'a str>) -> Token<'a> {
        Token::Plain(s.into())
    }

    pub fn new_set(s: impl Into<Vec<&'a str>>) -> Token<'a> {
        Token::Set(s.into())
    }

    pub fn iter(&self) -> TokenIter<'_> {
        TokenIter::new(self)
    }
}

#[derive(Debug, Clone)]
pub enum TokenIter<'a> {
    Plain(Option<&'a str>),
    Set(Iter<'a, &'a str>),
}

impl<'a> TokenIter<'a> {
    pub fn new(t: &'a Token) -> Self {
        match t {
            Token::Plain(v) => TokenIter::Plain(Some(v)),
            Token::Set(v) => TokenIter::Set(v.iter()),
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TokenIter::Plain(v) => v.take(),
            TokenIter::Set(v) => v.next().copied(),
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
        ];

        for (name, input, expected) in cases {
            let actual = input.iter().collect::<Vec<&str>>();

            assert_eq!(actual, expected, "case {name}")
        }
    }
}
