#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedEof,
    InvalidTag,
    InvalidHeader,
    MissingRequiredTable(&'static str),
    OutOfBounds { pos: usize, len: usize },
}
pub enum CursorError {
    OutOfBounds { pos: usize, len: usize },
}
