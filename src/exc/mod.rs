//! The only error type returned to Python.

mod types;

pub use types::DialectError;
pub use types::SerializeError;

use crate::expr::enum_::IdentParseError;
use crate::internal::alias;
use crate::internal::template::TemplateParseError;

/// Failure returned across the Python boundary.
///
/// ```ignore
/// use rapidquery::exc::Error;
///
/// let err = Error::dialect("CREATE SCHEMA");
/// assert_eq!(
///     err.to_string(),
///     "CREATE SCHEMA is not supported by this dialect"
/// );
/// ```
#[derive(Debug, strum::Display)]
pub enum Error {
    /// Wrong Python type at an API boundary. Becomes `TypeError`.
    ///
    /// Example: `function must be Callable, not int`.
    #[strum(to_string = "{what} must be {expected}, not {got}")]
    Type {
        /// Parameter, serializer, or protocol that rejected the object.
        what: &'static str,
        /// Types the caller may pass.
        expected: &'static str,
        /// Fully qualified name of the object that was passed.
        got: String,
    },
    /// Right type, illegal on every dialect. Becomes `ValueError`.
    ///
    /// Example: `behavior: expected 'CASCADE' or 'RESTRICT'`.
    #[strum(to_string = "{what}: {detail}")]
    Value {
        /// Parameter or constructor that rejected the value.
        what: &'static str,
        /// What is wrong with the value.
        detail: String,
    },
    /// Identifier text that [`IdentParseError`] already explains. Becomes `ValueError`.
    ///
    /// Example: `identifier is empty`.
    #[strum(to_string = "{0}")]
    Ident(IdentParseError),
    /// Dialect template text that [`TemplateParseError`] already explains. Becomes `ValueError`.
    ///
    /// Example: `invalid template for 'bytes': template is too long (maximum is 256 characters)`.
    #[strum(to_string = "invalid template for '{field}': {source}")]
    Template {
        /// Dialect field whose template failed to parse.
        field: &'static str,
        /// Why the template was rejected.
        source: TemplateParseError,
    },
    /// A Python object could not be written as a SQL literal. Becomes [`SerializeError`].
    ///
    /// Example: `IntegerSerializer: must be int, not str`.
    #[strum(to_string = "{what}: {detail}")]
    Serialize {
        /// Serializer or literal kind that rejected the object.
        what: &'static str,
        /// What is wrong with the type or the value.
        detail: String,
    },
    /// Legal RapidQuery value that this dialect cannot render. Becomes [`DialectError`].
    ///
    /// Example: `CREATE INDEX ... UNIQUE is not supported by this dialect`.
    #[strum(to_string = "{feature} is not supported by this dialect")]
    Dialect {
        /// SQL feature the dialect refuses.
        feature: &'static str,
    },
    /// `ident[i]` landed outside the parts. Becomes `IndexError`.
    ///
    /// Example: `identifier index 3 is out of range (length 1)`.
    #[strum(to_string = "{detail}")]
    Index {
        /// Index and length, already formatted.
        detail: String,
    },
    /// Exception Python already raised, or one fetched from the interpreter.
    ///
    /// The original exception is re-raised unchanged.
    #[strum(to_string = "{0}")]
    Python(pyo3::PyErr),
}

impl Error {
    /// `TypeError`: `got` is not one of `expected`.
    #[cold]
    pub fn type_mismatch(what: &'static str, expected: &'static str, got: &alias::BoundObject<'_>) -> Self {
        Self::Type {
            what,
            expected,
            got: unsafe { crate::internal::utils::get_type_name(got.py(), got.as_ptr()) },
        }
    }

    /// `ValueError`: `what` received a value no dialect accepts.
    #[cold]
    pub fn invalid(what: &'static str, detail: impl Into<String>) -> Self {
        Self::Value {
            what,
            detail: detail.into(),
        }
    }

    /// [`SerializeError`]: `what` could not write this object as a SQL literal.
    #[cold]
    pub fn serialize(what: &'static str, detail: impl Into<String>) -> Self {
        Self::Serialize {
            what,
            detail: detail.into(),
        }
    }

    /// [`SerializeError`]: `got` is not one of the types `what` can write.
    #[cold]
    pub fn serialize_type(what: &'static str, expected: &'static str, got: &alias::BoundObject<'_>) -> Self {
        let got = unsafe { crate::internal::utils::get_type_name(got.py(), got.as_ptr()) };
        Self::serialize(what, format!("must be {expected}, not {got}"))
    }

    /// [`DialectError`]: `feature` cannot be rendered by this dialect.
    #[cold]
    pub fn dialect(feature: &'static str) -> Self {
        Self::Dialect { feature }
    }

    /// `IndexError` for an identifier part index.
    #[cold]
    pub fn index(detail: impl Into<String>) -> Self {
        Self::Index { detail: detail.into() }
    }

    /// Pass `err` through to Python unchanged.
    #[cold]
    pub fn python(err: pyo3::PyErr) -> Self {
        Self::Python(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ident(err) => Some(err),
            Self::Template { source, .. } => Some(source),
            Self::Python(err) => Some(err),
            _ => None,
        }
    }
}

impl From<IdentParseError> for Error {
    fn from(err: IdentParseError) -> Self {
        Self::Ident(err)
    }
}

impl From<pyo3::CastIntoError<'_>> for Error {
    fn from(err: pyo3::CastIntoError<'_>) -> Self {
        pyo3::PyErr::from(err).into()
    }
}

impl From<pyo3::PyErr> for Error {
    fn from(err: pyo3::PyErr) -> Self {
        Self::Python(err)
    }
}

impl From<Error> for pyo3::PyErr {
    fn from(value: Error) -> Self {
        let message = value.to_string();
        match value {
            Error::Type { .. } => pyo3::exceptions::PyTypeError::new_err(message),
            Error::Value { .. } | Error::Ident(_) | Error::Template { .. } => {
                pyo3::exceptions::PyValueError::new_err(message)
            }
            Error::Dialect { .. } => DialectError::new_err(message),
            Error::Serialize { .. } => SerializeError::new_err(message),
            Error::Index { .. } => pyo3::exceptions::PyIndexError::new_err(message),
            Error::Python(err) => err,
        }
    }
}

/// [`Result`] whose error becomes a Python exception.
pub type Result<T> = ::core::result::Result<T, Error>;

#[pyo3::pymodule(name = "exc", gil_used = false)]
pub mod exc_mod {
    #[pymodule_export]
    use super::types::DialectError;
    #[pymodule_export]
    use super::types::Error;
    #[pymodule_export]
    use super::types::SerializeError;
}
