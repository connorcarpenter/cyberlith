pub enum RequestError {
    None,
    SerdeError,
}

pub enum ResponseError {
    None,
    EhttpError(String),
    SerdeError,
}
