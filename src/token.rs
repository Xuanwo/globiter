use std::slice::Iter;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Plain(String),
    Set(Vec<String>),
    // Range(String, String),
}

impl Token {
    pub fn new_plain(s: impl Into<String>) -> Token {
        Token::Plain(s.into())
    }

    pub fn new_set(s: impl Into<Vec<String>>) -> Token {
        Token::Set(s.into())
    }

    pub fn iter(&self) -> TokenIter<'_> {
        TokenIter::new(self)
    }
}

pub enum TokenIter<'a> {
    Plain(&'a String, bool),
    Set(Iter<'a, String>),
}

impl<'a> TokenIter<'a> {
    pub fn new(t: &'a Token) -> Self {
        match t {
            Token::Plain(v) => TokenIter::Plain(v, false),
            Token::Set(v) => TokenIter::Set(v.iter()),
        }
    }
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TokenIter::Plain(v, used) => {
                if *used {
                    None
                } else {
                    *used = true;
                    Some(v)
                }
            }
            TokenIter::Set(v) => v.next().map(|v| v.as_str()),
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
                Token::new_set(["a", "b", "c", "d", "e"].map(|v| v.to_string()).to_vec()),
                vec!["a", "b", "c", "d", "e"],
            ),
        ];

        for (name, input, expected) in cases {
            let actual = input.iter().collect::<Vec<&str>>();

            assert_eq!(actual, expected, "case {name}")
        }
    }
}
