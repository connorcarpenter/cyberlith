pub(crate) enum AuthServerError {
    EmailSendFailed(String),
    InsertedDuplicateUserId,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    UsernameOrEmailNotFound,
    EmailNotFound,
    PasswordIncorrect,
    TokenNotFound,
    PasswordHashError,
    Unknown(String),
}
