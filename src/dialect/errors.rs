use std::fmt;

// FIXME: Implement a unique error enum for project

/// Formatting / serializing errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatError {
    /// Means a `NUL` character has found in string during escaping/serializing a string value.
    NulInString,
    /// Means something is not supported by a dialect
    Unsupported(&'static str),
}

impl From<FormatError> for pyo3::PyErr {
    fn from(value: FormatError) -> Self {
        match value {
            FormatError::NulInString => {
                pyo3::exceptions::PyValueError::new_err("NUL is not allowed in a string literal")
            }
            FormatError::Unsupported(name) => {
                pyo3::exceptions::PyRuntimeError::new_err(format!("this dialect does not support {name}"))
            }
        }
    }
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NulInString => f.write_str("NUL is not allowed in a string literal"),
            Self::Unsupported(name) => write!(f, "this dialect does not support {name}"),
        }
    }
}

impl std::error::Error for FormatError {}
