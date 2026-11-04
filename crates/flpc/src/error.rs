#[derive(Debug)]
pub enum ProtocolError {
    UsernameTooLong,
    LengthMismatch,
    MessageTooLong,
    InvalidHeader,
    IncompleteMessage,
}
