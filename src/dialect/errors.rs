use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    Write,
    NulInString,
    Unsupported(&'static str),
}

impl From<fmt::Error> for FormatError {
    #[inline]
    fn from(_: fmt::Error) -> Self {
        Self::Write
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Write => f.write_str("failed to write SQL"),
            Self::NulInString => f.write_str("NUL is not allowed in a string literal"),
            Self::Unsupported(name) => write!(f, "this dialect does not support {name}"),
        }
    }
}

impl std::error::Error for FormatError {}
