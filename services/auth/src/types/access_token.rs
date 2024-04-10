use crypto::U32Token;

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct AccessToken {
    pub(crate) value: U32Token,
}

impl From<U32Token> for AccessToken {
    fn from(value: U32Token) -> Self {
        Self { value }
    }
}

impl AccessToken {
    pub fn gen_random() -> Self {
        Self {
            value: U32Token::gen_random(),
        }
    }

    pub fn to_string(&self) -> String {
        self.value.as_string()
    }
}