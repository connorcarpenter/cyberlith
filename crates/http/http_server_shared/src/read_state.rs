#[derive(PartialEq, Eq, Debug)]
pub enum ReadState {
    MatchingUrl,
    ReadingHeaders,
    ReadingBody,
    Finished,
    Error,
    Redirecting(String),
}
