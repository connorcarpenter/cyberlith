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
    Unknown(String),
}
