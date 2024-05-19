use crypto::U32Token;
use naia_serde::{BitReader, BitWrite, Serde, SerdeErr};

#[derive(Eq, PartialEq, Hash, Clone, Debug, Copy)]
pub struct RegisterToken {
    value: U32Token,
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

    pub fn to_string(&self) -> String {
        self.value.as_string()
    }

    pub fn from_str(value: &str) -> Option<Self> {
        U32Token::from_str(value).map(|value| Self { value })
    }
}

impl Serde for RegisterToken {
    fn ser(&self, writer: &mut dyn BitWrite) {
        self.value.as_u32().ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let Some(value) = U32Token::from_u32(u32::de(reader)?) else {
            return Err(SerdeErr);
        };
        Ok(Self { value })
    }

    fn bit_length(&self) -> u32 {
        self.value.as_u32().bit_length()
    }
}
