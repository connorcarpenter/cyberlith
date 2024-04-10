pub(crate) enum AuthServerError {
    EmailSendFailed(String),
    TokenSerdeError,
    TokenNotFound,
}