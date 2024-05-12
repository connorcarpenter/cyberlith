#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CharacterWhitelist {
    Alphanumeric,
    Password,
    Email,
}

impl CharacterWhitelist {
    const CHAR_WHITELIST_ALPHANUMERIC: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const CHAR_WHITELIST_PASSWORD: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%^&*+-/=?_.{|}~";
    const CHAR_WHITELIST_EMAIL: &'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!#$%^&*+-/=?_.{|}~@";

    pub fn alphanumeric_includes_char(c: char) -> bool {
        Self::CHAR_WHITELIST_ALPHANUMERIC.contains(c)
    }

    pub fn alphanumeric_allows_text(text: &str) -> bool {
        text.chars().all(|c| Self::CHAR_WHITELIST_ALPHANUMERIC.contains(c))
    }

    pub fn password_includes_char(c: char) -> bool {
        Self::CHAR_WHITELIST_PASSWORD.contains(c)
    }

    pub fn password_allows_text(text: &str) -> bool {
        text.chars().all(|c| Self::CHAR_WHITELIST_PASSWORD.contains(c))
    }

    pub fn email_includes_char(c: char) -> bool {
        Self::CHAR_WHITELIST_EMAIL.contains(c)
    }

    pub fn email_allows_text(text: &str) -> bool {
        text.chars().all(|c| Self::CHAR_WHITELIST_EMAIL.contains(c))
    }

    pub fn includes_char(&self, c: char) -> bool {
        match self {
            Self::Alphanumeric => Self::CHAR_WHITELIST_ALPHANUMERIC.contains(c),
            Self::Password => Self::CHAR_WHITELIST_PASSWORD.contains(c),
            Self::Email => Self::CHAR_WHITELIST_EMAIL.contains(c),
        }
    }

    pub fn allows_text(&self, text: &str) -> bool {
        text.chars().all(|c| self.includes_char(c))
    }
}