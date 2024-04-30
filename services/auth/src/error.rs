pub(crate) enum AuthServerError {
    EmailSendFailed(String),
    InsertedDuplicateUserId,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    UsernameOrEmailNotFound,
    EmailNotFound,
    PasswordIncorrect,
    TokenSerdeError,
    TokenNotFound,
    PasswordHashError,
    Unknown(String),
}
