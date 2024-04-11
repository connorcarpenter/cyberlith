use crypto::U32Token;

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct RefreshToken {
    value: U32Token,
}

impl From<U32Token> for RefreshToken {
    fn from(value: U32Token) -> Self {
        Self { value }
    }
}

impl RefreshToken {
    pub fn gen_random() -> Self {
        Self {
            value: U32Token::gen_random(),
        }
    }

    pub fn to_string(&self) -> String {
        self.value.as_string()
    }

    pub fn from_str(value: &str) -> Option<Self> {
        U32Token::from_str(value).map(|value| Self { value })
    }
}
