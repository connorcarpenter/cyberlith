use std::fmt::Debug;

use rand::{Rng, thread_rng};

#[derive(Eq, PartialEq, Hash, Clone, Copy, Ord, PartialOrd)]
pub struct U32Token {
    value: u32,
}

impl Debug for U32Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "U32Token(`{}` or `{}`)", self.as_string(), self.as_u32())
    }
}

impl U32Token {

    const CHARSET: [char; Self::MAX_CHARS as usize] = ['x', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'y', 'z'];
    const MAX_CHARS: u32 = 32;
    const MAX_LENGTH: u32 = 6;
    const MAX_VALUE: u32 = Self::MAX_CHARS.pow(Self::MAX_LENGTH);

    pub fn get_random() -> Self {
        let value = thread_rng().gen_range(0..Self::MAX_VALUE);
        Self { value }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        if value.len() != Self::MAX_LENGTH as usize {
            return None;
        }

        let mut result = 0;
        for c in value.chars() {
            let idx = Self::CHARSET.iter().position(|&x| x == c)?;
            result *= Self::MAX_CHARS;
            result += idx as u32;
        }
        Some(Self { value: result })
    }

    pub fn as_string(&self) -> String {
        let mut value = self.value;
        let mut result = String::new();
        for _ in 0..Self::MAX_LENGTH {
            let idx = value % Self::MAX_CHARS;
            result.insert(0, Self::CHARSET[idx as usize]);
            value /= Self::MAX_CHARS;
        }
        result
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        if value >= Self::MAX_VALUE {
            return None;
        }
        Some(Self { value })
    }

    pub fn as_u32(&self) -> u32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use crate::U32Token;

    #[test]
    fn monte_carlo_serde() {
        for _ in 0..1000 {

            // to/from string
            {
                let token = U32Token::get_random();
                let token_str = token.as_string();
                let token2 = U32Token::from_str(&token_str).unwrap();
                println!("{:?} == {:?}", token, token2);
                assert_eq!(token, token2);

            }

            // to/from u32
            {
                let token = U32Token::get_random();
                let token_u32 = token.as_u32();
                let token2 = U32Token::from_u32(token_u32).unwrap();
                assert_eq!(token, token2);
            }
        }
    }

    #[test]
    fn invalid_strings() {

        // too short
        let result = U32Token::from_str("12345");
        assert_eq!(result, None);

        // too long
        let result = U32Token::from_str("1234567");
        assert_eq!(result, None);

        // invalid characters
        let result = U32Token::from_str("!23456");
        assert_eq!(result, None);
    }

    #[test]
    fn invalid_u32s() {
        // too large
        let result = U32Token::from_u32(U32Token::MAX_VALUE);
        assert_eq!(result, None);
    }

    #[test]
    fn valid_strings() {
        let token = U32Token::from_u32(14192).unwrap();
        println!("{:?}", token);
    }
}