#[derive(PartialEq, Eq)]
pub enum ReadState {
    MatchingUrl,
    ReadingHeaders,
    ReadingBody,
    Finished,
    Error,
    Redirecting(String),
}
