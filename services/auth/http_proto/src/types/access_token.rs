use naia_serde::{BitReader, BitWrite, Serde, SerdeErr};

use crypto::U32Token;

use crate::types::get_set_cookie_value;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct AccessToken {
    value: U32Token,
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

    pub fn from_str(value: &str) -> Option<Self> {
        U32Token::from_str(value).map(|value| Self { value })
    }

    pub fn get_new_cookie_value(domain: &str, secure: bool, access_token: &str) -> String {
        const ONE_DAY_IN_SECONDS: u32 = 60 * 60 * 24;
        get_set_cookie_value("access_token", &access_token.to_string(), domain, ONE_DAY_IN_SECONDS, secure)
    }

    pub fn get_expire_cookie_value(domain: &str, secure: bool) -> String {
        get_set_cookie_value("access_token", "", domain, 0, secure)
    }
}

impl Serde for AccessToken {
    fn ser(&self, writer: &mut dyn BitWrite) {
        self.value.as_u32().ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let Some(value) = U32Token::from_u32(u32::de(reader)?) else {
            return Err(SerdeErr);
        };
        Ok(Self {
            value,
        })
    }

    fn bit_length(&self) -> u32 {
        self.value.as_u32().bit_length()
    }
}