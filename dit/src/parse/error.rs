pub enum Error {
    Test,
}
pub enum ParseError {}
impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::Test
    }
}
pub enum ReadError {
    UnexpectedEof,
    OutOfBounds,
}
impl From<ReadError> for Error {
    fn from(value: ReadError) -> Self {
        Error::Test
    }
}
