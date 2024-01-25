pub enum RequestError {
    None,
    SerdeError,
}

pub enum ResponseError {
    None,
    HttpError(String),
    SerdeError,
}
