pub(crate) enum AuthServerError {
    EmailSendFailed(String),
    RegisterTokenSerdeError,
    RegisterTokenNotFound,
    InsertedDuplicateUserId,
    UsernameAlreadyExists,
    EmailAlreadyExists,
    UsernameOrEmailNotFound,
    PasswordIncorrect,
    AccessTokenSerdeError,
    AccessTokenNotFound,
    Unknown(String),
}