#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    ParseError(ParseError),
    MissingTable(&'static str),
    InvalidFormat(&'static str),
    ReadError(ReadError),
    Unknown,
}
#[derive(Debug)]
pub enum ParseError {
    InvalidCmap,
    MagicNumber,
}
impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::ParseError(value)
    }
}
#[derive(Debug)]
pub enum ReadError {
    UnexpectedEof,
    OutOfBounds,
}
impl From<ReadError> for Error {
    fn from(value: ReadError) -> Self {
        Error::ReadError(value)
    }
}
