mod types;
pub use types::UnsupportedError;

use crate::expr::enum_::IdentParseError;
use crate::internal::alias;

#[derive(Debug, strum::Display)]
pub enum Error {
    // --- Invalid arguments ---
    #[strum(serialize = "argument must be a {expected}, not {got}")]
    ArgumentType { expected: &'static str, got: String },
    #[strum(serialize = "invalid argument value. received {got}, expected {expected}")]
    ArgumentValue { expected: &'static str, got: String },

    // --- ident ---
    #[strum(serialize = "{0}")]
    IdentParse(IdentParseError),

    // --- serializers ---
    #[strum(serialize = "failed to serialize {got} type, acceptable types are {expected} (serializer: {name})")]
    SerializeType {
        name: &'static str,
        expected: &'static str,
        got: String,
    },
    #[strum(serialize = "invalid value {value} to serialize: {reason} (serializer: {name})")]
    SerializeValue {
        name: &'static str,
        reason: String,
        value: alias::PyObject,
    },
    #[strum(serialize = "function should return str, returned {got} (serializer: CustomSerializer)")]
    CustomFunctionReturnType { got: String },
    #[strum(serialize = "custom expression has {placeholders} placeholders but {arguments} arguments")]
    CustomPlaceholderCount { placeholders: usize, arguments: usize },

    // --- dialect features ---
    #[strum(serialize = "{0} is not supported by this dialect")]
    Unsupported(&'static str),

    // --- customization protocols ---
    #[strum(serialize = "{name:?} protocol returned {got}, expected {expected}")]
    ProtocolReturnType {
        name: &'static str,
        expected: &'static str,
        got: String,
    },

    // --- custom/python variant ---
    #[strum(serialize = "{0}")]
    Custom(pyo3::PyErr),
}

impl Error {
    pub fn argument_type(expected: &'static str, got: &alias::BoundObject<'_>) -> Self {
        Self::ArgumentType {
            expected,
            got: unsafe { crate::internal::utils::get_type_name(got.py(), got.as_ptr()) },
        }
    }

    /// # Safety
    /// `ptr` must be borrowed and non-null pointer.
    pub unsafe fn serialize_value(
        name: &'static str,
        reason: impl Into<String>,
        py: pyo3::Python<'_>,
        ptr: *mut pyo3::ffi::PyObject,
    ) -> Self {
        let value = pyo3::Bound::from_borrowed_ptr(py, ptr).unbind();
        Self::SerializeValue {
            name,
            reason: reason.into(),
            value,
        }
    }

    pub fn protocol_return_type(name: &'static str, expected: &'static str, got: &alias::BoundObject<'_>) -> Self {
        Self::ProtocolReturnType {
            name,
            expected,
            got: unsafe { crate::internal::utils::get_type_name(got.py(), got.as_ptr()) },
        }
    }
}

impl std::error::Error for Error {}

impl From<IdentParseError> for Error {
    fn from(err: IdentParseError) -> Self {
        Self::IdentParse(err)
    }
}

impl From<pyo3::CastIntoError<'_>> for Error {
    fn from(err: pyo3::CastIntoError<'_>) -> Self {
        pyo3::PyErr::from(err).into()
    }
}

impl From<pyo3::PyErr> for Error {
    fn from(err: pyo3::PyErr) -> Self {
        Self::Custom(err)
    }
}

impl From<Error> for pyo3::PyErr {
    fn from(value: Error) -> Self {
        match value {
            // TypeError
            Error::ArgumentType { .. }
            | Error::SerializeType { .. }
            | Error::ProtocolReturnType { .. }
            | Error::CustomFunctionReturnType { .. } => pyo3::exceptions::PyTypeError::new_err(value.to_string()),
            // ValueError
            Error::ArgumentValue { .. }
            | Error::SerializeValue { .. }
            | Error::IdentParse(..)
            | Error::CustomPlaceholderCount { .. } => pyo3::exceptions::PyValueError::new_err(value.to_string()),
            // DialectError
            Error::Unsupported(..) => UnsupportedError::new_err(value.to_string()),
            // ?
            Error::Custom(pyerr) => pyerr,
        }
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;

#[pyo3::pymodule(name = "exceptions", gil_used = false)]
pub mod exceptions_mod {
    #[pymodule_export]
    use super::UnsupportedError;
}
