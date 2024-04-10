use crypto::U32Token;

#[derive(Eq, PartialEq, Hash)]
pub struct RegisterToken {
    pub(crate) value: U32Token,
}

impl From<U32Token> for RegisterToken {
    fn from(value: U32Token) -> Self {
        Self { value }
    }
}

impl RegisterToken {
    pub fn gen_random() -> Self {
        Self {
            value: U32Token::gen_random(),
        }
    }
}