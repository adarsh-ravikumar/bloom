#[derive(Debug)]
pub enum ParserError {
    UnexpectedEOF,
    UnexpectedCharacter,
    InvalidIdentifier,
    InvalidTagClose,
}
