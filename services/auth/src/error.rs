pub(crate) enum AuthServerError {
    EmailSendFailed(String),
    RegisterTokenSerdeError,
    RegisterTokenNotFound,
    InsertedDuplicateUserId,
}