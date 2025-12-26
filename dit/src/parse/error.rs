#[derive(Debug)]
pub enum Error {
    Test,
    OffsetNotFound,
    BlockParseFailed,
    TableNotFound,
    FileNotRead,
    MalformedGlyphPoints,
    WrongFormat,
    MalformedCmap,
}
#[derive(Debug)]
pub enum ParseError {}
impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::Test
    }
}
#[derive(Debug)]
pub enum ReadError {
    UnexpectedEof,
    OutOfBounds,
}
impl From<ReadError> for Error {
    fn from(value: ReadError) -> Self {
        Error::Test
    }
}
