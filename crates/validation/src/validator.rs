
pub trait Validator {
    fn min_length() -> usize;
    fn max_length() -> usize;
    fn validation_type() -> ValidationType;

    fn allows_text(text: &str) -> bool {
        text.len() >= Self::min_length() && text.len() <= Self::max_length() && Self::validation_type().allows_text(text)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ValidationType {
    Username,
    Password,
    Email,
}

impl ValidationType {
    const CHAR_WHITELIST_ALPHANUMERIC: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789";
    const CHAR_WHITELIST_PASSWORD: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%^&*+-/=?_.{|}~";
    const CHAR_WHITELIST_EMAIL: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%^&*+-/=?_.{|}~@";

    pub fn includes_char(&self, c: char) -> bool {
        match self {
            Self::Username => Self::CHAR_WHITELIST_ALPHANUMERIC.contains(c),
            Self::Password => Self::CHAR_WHITELIST_PASSWORD.contains(c),
            Self::Email => Self::CHAR_WHITELIST_EMAIL.contains(c),
        }
    }

    pub fn allows_text(&self, text: &str) -> bool {
        text.chars().all(|c| self.includes_char(c))
    }
}