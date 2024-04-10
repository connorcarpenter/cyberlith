pub(crate) enum AuthServerError {
    EmailSendFailed(String),
    InsertedDuplicateUserId,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    UsernameOrEmailNotFound,
    PasswordIncorrect,
    TokenSerdeError,
    TokenNotFound,
    Unknown(String),
}